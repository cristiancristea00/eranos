//! Transfers record repayments between members and move their balances.
//! These tests keep impossible transfers out (to oneself, of nothing, or of
//! a negative amount), keep optional details absent rather than blank, and
//! keep transfers edited through `to_builder` or created by the settlement
//! planner identical to ones built by hand.

use super::*;

use crate::money::Currency;

// Three members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);
const CAROL: MemberId = MemberId::from_bytes([3; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn september_24() -> Date {
    Date::new(2026, 9, 24).unwrap()
}

/// Bob repays Ana 30.00 on 24 September 2026.
fn repayment() -> TransferBuilder {
    Transfer::builder(BOB, ANA, euros(3_000), september_24())
}

/// The repayment with every optional field set.
fn full_repayment() -> Transfer {
    repayment()
        .time(Time::new(9, 15).unwrap())
        .exchange_rate(ExchangeRate::parse("4.9765").unwrap())
        .note("Dinner")
        .build()
        .unwrap()
}

/// A repayment would be recorded in the wrong direction, or gain details
/// nobody entered.
#[test]
fn a_transfer_keeps_its_fields_and_defaults() {
    let transfer = repayment().build().unwrap();
    // Required fields.
    assert_eq!(transfer.from(), BOB);
    assert_eq!(transfer.to(), ANA);
    assert_eq!(transfer.amount(), euros(3_000));
    assert_eq!(transfer.date(), september_24());
    // Optional fields are absent.
    assert_eq!(transfer.time(), None);
    assert_eq!(transfer.exchange_rate(), None);
    assert_eq!(transfer.note(), None);
}

/// A member paying themselves would be recorded as a repayment that moves
/// no money.
#[test]
fn a_transfer_to_oneself_is_rejected() {
    // Same sender and recipient.
    assert_eq!(
        Transfer::builder(ANA, ANA, euros(3_000), september_24()).build(),
        Err(TransferError::SameMember)
    );
    // The members are checked before the amount.
    assert_eq!(
        Transfer::builder(ANA, ANA, euros(0), september_24()).build(),
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
            repayment().amount(amount).build(),
            Err(TransferError::NonPositiveAmount)
        );
    }
    // The smallest positive amount.
    assert!(repayment().amount(euros(1)).build().is_ok());
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

/// A time or exchange rate could not be added, or not removed once added.
#[test]
fn time_and_exchange_rate_are_set_and_cleared_with_none() {
    // Set.
    let transfer = full_repayment();
    assert_eq!(transfer.time(), Some(Time::new(9, 15).unwrap()));
    assert_eq!(
        transfer.exchange_rate(),
        Some(ExchangeRate::parse("4.9765").unwrap())
    );
    // Cleared.
    let transfer = transfer
        .to_builder()
        .time(None)
        .exchange_rate(None)
        .build()
        .unwrap();
    assert_eq!(transfer.time(), None);
    assert_eq!(transfer.exchange_rate(), None);
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
    let september_25 = Date::new(2026, 9, 25).unwrap();
    let transfer = full_repayment()
        .to_builder()
        .from(CAROL)
        .to(BOB)
        .amount(euros(1_000))
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
/// repayments entered by hand.
#[test]
fn new_unchecked_matches_a_built_transfer() {
    // Same fields, no optional details.
    assert_eq!(
        Transfer::new_unchecked(BOB, ANA, euros(3_000), september_24()),
        repayment().build().unwrap()
    );
}

/// The exchange rate fixed when a group changes currency would not be
/// stored, or could not be removed.
#[test]
fn set_exchange_rate_replaces_or_clears_the_rate() {
    let rate = ExchangeRate::parse("0.2011").unwrap();
    // Replaced.
    let mut transfer = full_repayment();
    transfer.set_exchange_rate(Some(rate));
    assert_eq!(transfer.exchange_rate(), Some(rate));
    // Cleared.
    transfer.set_exchange_rate(None);
    assert_eq!(transfer.exchange_rate(), None);
}

/// Saved repayments would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn transfers_round_trip_through_serde() {
    // Every optional field set.
    let transfer = full_repayment();
    let json = serde_json::to_string(&transfer).unwrap();
    assert_eq!(serde_json::from_str::<Transfer>(&json).unwrap(), transfer);
    // No optional field set; absent fields are left out.
    let minimal_transfer = repayment().build().unwrap();
    let minimal_json = serde_json::to_string(&minimal_transfer).unwrap();
    assert!(!minimal_json.contains("note") && !minimal_json.contains("time"));
    assert_eq!(
        serde_json::from_str::<Transfer>(&minimal_json).unwrap(),
        minimal_transfer
    );
    // Loading validates like the builder.
    let to_oneself = minimal_json.replace(&ANA.to_string(), &BOB.to_string());
    assert_ne!(to_oneself, minimal_json);
    assert!(serde_json::from_str::<Transfer>(&to_oneself).is_err());
}
