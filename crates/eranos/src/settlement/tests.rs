//! Tests for turning a ledger into a settlement. The payments are what the
//! apps ask members to send, so they must be in the group currency, sorted
//! the same way on every device, and each one convertible into the transfer
//! that records it.

use super::*;
use crate::ledger::Delta;
use crate::money::Currency;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

/// A ledger where members 3 and 4 owe members 1 and 2 through transfers
/// received and sent.
fn ledger() -> Ledger {
    let mut delta = Delta::default();
    // A transfer from 1 to 3 leaves 3 owing 1; the same for 2 and 4.
    delta.add_transfer(member(1), member(3), 500, 1);
    delta.add_transfer(member(2), member(4), 300, 1);
    let mut ledger = Ledger::new(Currency::RON);
    ledger.apply(&delta).unwrap();
    ledger
}

/// Every strategy records itself, pays in the group currency, and sorts its
/// payments by payer and recipient.
#[test]
fn every_strategy_pays_in_the_group_currency_in_order() {
    for strategy in SettlementStrategy::all() {
        let settlement = settle(strategy, &ledger()).unwrap();
        assert_eq!(settlement.strategy(), strategy);
        assert_eq!(settlement.len(), 2);
        assert!(!settlement.is_empty());
        let payments: Vec<(MemberId, MemberId, Money)> = settlement
            .payments()
            .iter()
            .map(|payment| (payment.from(), payment.to(), payment.amount()))
            .collect();
        assert_eq!(
            payments,
            vec![
                (
                    member(3),
                    member(1),
                    Money::from_minor_units(500, Currency::RON)
                ),
                (
                    member(4),
                    member(2),
                    Money::from_minor_units(300, Currency::RON)
                ),
            ]
        );
    }
}

/// A settled group has an empty settlement under every strategy.
#[test]
fn a_settled_ledger_has_no_payments() {
    for strategy in SettlementStrategy::all() {
        assert!(
            settle(strategy, &Ledger::new(Currency::EUR))
                .unwrap()
                .is_empty()
        );
    }
}

/// A payment becomes the transfer that records it, with the same members,
/// amount, and the given date.
#[test]
fn a_payment_becomes_its_transfer() {
    let settlement = settle(SettlementStrategy::Greedy, &ledger()).unwrap();
    let payment = settlement.payments()[0];
    let date = Date::new(2026, 9, 24).unwrap();
    let transfer = payment.to_transfer(date);
    assert_eq!(
        transfer,
        Transfer::builder(payment.from(), payment.to(), payment.amount(), date)
            .build()
            .unwrap()
    );
}

/// The exact solver's refusal reaches the caller, and the other strategies
/// still settle the same balances.
#[test]
fn an_exact_plan_too_large_to_compute_is_an_error() {
    // 63 members each sent the hub a different amount: 64 open balances
    // and no exact opposites among them.
    let hub = member(100);
    let mut delta = Delta::default();
    for number in 1..=63 {
        delta.add_transfer(member(number), hub, i64::from(number), 1);
    }
    let mut ledger = Ledger::new(Currency::EUR);
    ledger.apply(&delta).unwrap();
    assert_eq!(
        settle(SettlementStrategy::MinimumTransfers, &ledger),
        Err(SettlementError::OutOfMemory { balances: 64 })
    );
    for strategy in [SettlementStrategy::Greedy, SettlementStrategy::Direct] {
        assert_eq!(settle(strategy, &ledger).unwrap().len(), 63);
    }
}
