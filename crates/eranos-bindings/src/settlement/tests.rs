//! Settlements reach the apps as records, and strategies reach the library
//! from them. A strategy converted to another, or a payment with its members
//! swapped, would ask the wrong member to pay.

use eranos::{CalendarDate, Currency, ExpenseGroup, Member, Money, Transfer};

use super::*;

/// A strategy would reach the library, or come back from it, as another.
#[test]
fn every_strategy_converts_both_ways() {
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::Direct,
        SettlementStrategy::Greedy,
        SettlementStrategy::MinimumTransfers,
    ] {
        assert_eq!(
            SettlementStrategy::from_settlement_strategy(strategy.to_library()),
            strategy
        );
    }
}

/// A payment would reach the apps with its members swapped, its amount
/// changed, or under another strategy.
#[test]
fn a_settlement_keeps_every_payment() {
    let mut group = ExpenseGroup::builder("Flat", Currency::EUR)
        .build()
        .unwrap();
    let ana = group
        .add_member(Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(Member::builder("Bob").build().unwrap())
        .unwrap();
    let day = CalendarDate::new(2026, 10, 1).unwrap();
    // Bob paid Ana 20.00 too much, so Ana owes it back.
    let transfer = Transfer::builder(bob, ana, Money::from_minor_units(2_000, Currency::EUR), day)
        .build()
        .unwrap();
    group.add_transfer(transfer).unwrap();
    let settlement = group
        .settlement(library::SettlementStrategy::Greedy)
        .unwrap();
    assert_eq!(
        SettlementPlan::from_library(settlement),
        SettlementPlan {
            payments: vec![Payment {
                from: ana.to_string(),
                to: bob.to_string(),
                amount: crate::Money {
                    minor_units: 2_000,
                    currency: "EUR".to_owned(),
                },
            }],
            strategy: SettlementStrategy::Greedy,
        }
    );
}
