//! Transfers record repayments between members and move their balances.
//! These tests keep impossible transfers out (to oneself, of nothing, or of
//! a negative amount), keep optional details absent rather than blank, and
//! keep transfers edited through `to_builder` or recorded from a
//! settlement's payment identical to ones built by hand.

use eranos_ledger::{Delta, Ledger, MemberPair};

use super::*;

use crate::money::Currency;
use crate::settlement::SettlementStrategy;

// Three members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);
const CAROL: MemberId = MemberId::from_bytes([3; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

fn september_24() -> CalendarDate {
    CalendarDate::new(24, 9, 2026).unwrap()
}

/// Bob repays Ana 30.00 on 24 September 2026.
fn repayment() -> TransferBuilder {
    Transfer::builder(euros(3_000), september_24(), BOB, ANA)
}

/// The repayment with every optional field set, converted at 1 EUR =
/// 4.9765 RON.
fn full_repayment() -> Transfer {
    repayment()
        .cost(euros(3_000).at(ExchangeRate::parse("4.9765").unwrap()))
        .time(TimeOfDay::new(9, 15).unwrap())
        .note("Dinner")
        .build()
        .unwrap()
}

/// A repayment would be recorded in the wrong direction, or gain details
/// nobody entered.
#[test]
fn a_transfer_keeps_its_fields_and_defaults() {
    let transfer = repayment().build().unwrap();
    // Required fields; a plain amount is a local cost, at a rate of 1.
    assert_eq!(transfer.from(), BOB);
    assert_eq!(transfer.to(), ANA);
    assert_eq!(transfer.cost(), Cost::from(euros(3_000)));
    assert_eq!(transfer.amount(), euros(3_000));
    assert_eq!(transfer.exchange_rate(), ExchangeRate::ONE);
    assert_eq!(transfer.date(), september_24());
    // Optional fields are absent.
    assert_eq!(transfer.time(), None);
    assert_eq!(transfer.note(), None);
}

/// A member paying themselves would be recorded as a repayment that moves
/// no money.
#[test]
fn a_transfer_to_oneself_is_rejected() {
    // Same sender and recipient.
    assert_eq!(
        Transfer::builder(euros(3_000), september_24(), ANA, ANA).build(),
        Err(TransferError::SameMember)
    );
    // The members are checked before the amount.
    assert_eq!(
        Transfer::builder(euros(0), september_24(), ANA, ANA).build(),
        Err(TransferError::SameMember)
    );
}

/// A repayment of nothing, or a negative one that reverses the debt, would
/// be accepted.
#[test]
fn a_non_positive_amount_is_rejected() {
    // Zero and negative amounts.
    for amount in [euros(0), euros(-3_000)] {
        assert_eq!(
            repayment().cost(amount).build(),
            Err(TransferError::NonPositiveAmount)
        );
    }
    // The smallest positive amount.
    assert!(repayment().cost(euros(1)).build().is_ok());
}

/// A note left blank would be stored as an empty string and shown as an
/// empty line.
#[test]
fn the_note_is_trimmed_and_blank_clears_it() {
    // Surrounding whitespace.
    let transfer = repayment().note("  Dinner \n").build().unwrap();
    assert_eq!(transfer.note(), Some("Dinner"));
    // Blank text is no note.
    let blank_note_transfer = repayment().note("   ").build().unwrap();
    assert_eq!(blank_note_transfer.note(), None);
    // Blank text clears a note set before.
    let cleared_note_transfer = full_repayment().to_builder().note("").build().unwrap();
    assert_eq!(cleared_note_transfer.note(), None);
}

/// A time could not be added, or not removed once added.
#[test]
fn the_time_is_set_and_cleared_with_none() {
    // Set.
    let transfer = full_repayment();
    assert_eq!(transfer.time(), Some(TimeOfDay::new(9, 15).unwrap()));
    // Cleared.
    let untimed_transfer = transfer.to_builder().time(None).build().unwrap();
    assert_eq!(untimed_transfer.time(), None);
}

/// Correcting a repayment's amount would keep a rate the new cost does not
/// have, or drop the rate of a corrected conversion.
#[test]
fn a_replaced_cost_brings_its_own_rate() {
    let transfer = full_repayment();
    let rate = ExchangeRate::parse("4.9765").unwrap();
    assert_eq!(transfer.cost(), euros(3_000).at(rate));
    assert_eq!(transfer.exchange_rate(), rate);
    // A plain amount makes the cost local.
    let local_transfer = transfer.to_builder().cost(euros(1_000)).build().unwrap();
    assert_eq!(local_transfer.cost(), Cost::from(euros(1_000)));
    // A corrected conversion keeps its rate.
    let corrected_transfer = transfer
        .to_builder()
        .cost(transfer.cost().with_amount(euros(1_000)))
        .build()
        .unwrap();
    assert_eq!(corrected_transfer.cost(), euros(1_000).at(rate));
}

/// Editing a repayment would silently drop fields that were not edited.
#[test]
fn to_builder_rebuilds_an_equal_transfer() {
    // Every optional field set.
    let transfer = full_repayment();
    assert_eq!(transfer.to_builder().build().unwrap(), transfer);
    // No optional field set.
    let minimal_transfer = repayment().build().unwrap();
    assert_eq!(
        minimal_transfer.to_builder().build().unwrap(),
        minimal_transfer
    );
}

/// Editing a repayment's members, amount, or date would be lost or skip
/// validation.
#[test]
fn builder_setters_replace_each_field() {
    // Members, amount, and date replaced.
    let september_25 = CalendarDate::new(25, 9, 2026).unwrap();
    let transfer = full_repayment()
        .to_builder()
        .from(CAROL)
        .to(BOB)
        .cost(euros(1_000))
        .date(september_25)
        .build()
        .unwrap();
    assert_eq!(transfer.from(), CAROL);
    assert_eq!(transfer.to(), BOB);
    assert_eq!(transfer.amount(), euros(1_000));
    assert_eq!(transfer.date(), september_25);
    // The other fields are kept.
    assert_eq!(transfer.note(), Some("Dinner"));
    // Replaced fields are validated again.
    assert_eq!(
        full_repayment().to_builder().to(BOB).build(),
        Err(TransferError::SameMember)
    );
}

/// Repayments proposed by the settlement planner would differ from the same
/// repayments entered by hand, or two recordings of one payment would be
/// one entry.
#[test]
fn a_payment_becomes_the_transfer_that_records_it() {
    // A transfer from Ana to Bob leaves Bob owing her 30.00, which the
    // settlement has him pay back.
    let mut delta = Delta::default();
    delta.add_transfer(MemberPair { from: ANA, to: BOB }, 3_000, 1);
    let mut ledger = Ledger::new(Currency::EUR);
    ledger.apply(&delta).unwrap();
    let settlement = eranos_settlement::settle(SettlementStrategy::Greedy, &ledger).unwrap();
    let payment = &settlement.payments()[0];
    // Same fields, no optional details, and an identifier of its own.
    let transfer = Transfer::from_payment(payment, september_24());
    assert_eq!(
        transfer,
        repayment().with_id(transfer.id()).build().unwrap()
    );
    assert_ne!(
        Transfer::from_payment(payment, september_24()).id(),
        transfer.id()
    );
}

/// A transfer in the group currency would be counted at another amount, or
/// a foreign one without its rate.
#[test]
fn a_transfer_converts_into_the_group_currency() {
    // In the group currency, the amount is taken as is.
    assert_eq!(
        resolve_transfer(&repayment().build().unwrap(), Currency::EUR),
        Ok(3_000)
    );
    // In a foreign currency, the rate converts it.
    let cash = repayment()
        .cost(
            Money::from_minor_units(Currency::RON, 10_000)
                .at(ExchangeRate::parse("0.2011").unwrap()),
        )
        .build()
        .unwrap();
    assert_eq!(resolve_transfer(&cash, Currency::EUR), Ok(2_011));
}

/// Two transfers would share an identifier, or an edited transfer would
/// become another entry.
#[test]
fn a_transfer_keeps_its_identifier_through_edits() {
    let transfer = full_repayment();
    assert_ne!(full_repayment().id(), transfer.id());
    let edited = transfer.to_builder().note("Lunch").build().unwrap();
    assert_eq!(edited.id(), transfer.id());
}

/// The exchange rate fixed when a group changes currency would not be
/// stored, could not be removed, or would change something else.
#[test]
fn with_exchange_rate_replaces_or_clears_the_rate() {
    let rate = ExchangeRate::parse("0.2011").unwrap();
    // Replaced, and only the cost changes.
    let transfer = full_repayment();
    let converted = transfer.clone().with_exchange_rate(Some(rate));
    assert_eq!(converted.cost(), euros(3_000).at(rate));
    assert_eq!(
        converted,
        transfer
            .to_builder()
            .cost(euros(3_000).at(rate))
            .build()
            .unwrap()
    );
    // Cleared.
    let local = converted.with_exchange_rate(None);
    assert_eq!(local.cost(), Cost::from(euros(3_000)));
}

/// Saved repayments would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn transfers_round_trip_through_serde() {
    // Every optional field set.
    let transfer = full_repayment();
    let json = serde_json::to_string(&transfer).unwrap();
    assert_eq!(serde_json::from_str::<Transfer>(&json).unwrap(), transfer);
    assert!(json.contains(r#""exchange_rate":"4.9765""#));
    // No optional field set and a local cost; absent fields are left out.
    let minimal_transfer = repayment().build().unwrap();
    let minimal_json = serde_json::to_string(&minimal_transfer).unwrap();
    for field in ["note", "time", "exchange_rate"] {
        assert!(!minimal_json.contains(field), "{field}");
    }
    assert_eq!(
        serde_json::from_str::<Transfer>(&minimal_json).unwrap(),
        minimal_transfer
    );
    // Loading validates like the builder.
    let to_oneself = minimal_json.replace(&ANA.to_string(), &BOB.to_string());
    assert_ne!(to_oneself, minimal_json);
    assert!(serde_json::from_str::<Transfer>(&to_oneself).is_err());
    // A record that is not a transfer's shape is refused before its checks.
    assert!(serde_json::from_str::<Transfer>(r#"{"note":"Dinner"}"#).is_err());
}
