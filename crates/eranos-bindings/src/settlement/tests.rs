//! Settlements reach the apps as records, and strategies reach the library
//! from them. A strategy converted to another, or a payment with its members
//! swapped, would ask the wrong member to pay.

use eranos::{CalendarDate, Currency, Group, Member, Money, Transfer};

use super::*;

/// A strategy would reach the library, or come back from it, as another.
#[test]
fn every_strategy_converts_both_ways() {
    for strategy in [
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
    let mut group = Group::builder(Currency::EUR, "Flat").build().unwrap();
    let ana = group
        .add_member(Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(Member::builder("Bob").build().unwrap())
        .unwrap();
    let day = CalendarDate::new(1, 10, 2026).unwrap();
    // Bob paid Ana 20.00 too much, so Ana owes it back.
    let transfer = Transfer::builder(Money::from_minor_units(Currency::EUR, 2_000), day, bob, ana)
        .build()
        .unwrap();
    group.add_transfer(transfer).unwrap();
    let settlement = group
        .settlement(library::SettlementStrategy::Greedy)
        .unwrap();
    assert_eq!(
        Settlement::from_library(settlement),
        Settlement {
            payments: vec![Payment {
                amount: crate::Money {
                    currency: "EUR".to_owned(),
                    minor_units: 2_000,
                },
                from: ana.to_string(),
                to: bob.to_string(),
            }],
            strategy: SettlementStrategy::Greedy,
        }
    );
}
