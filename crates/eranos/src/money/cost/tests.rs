//! An entry's cost says whether its amount is in the group currency or
//! converts into it at a fixed rate. These tests keep a plain amount local
//! at a rate of 1, keep a conversion's rate through a corrected amount and a
//! save, and keep a conversion at 1 apart from a local cost, because a
//! pegged currency converts at exactly 1.

use super::*;

use crate::money::Currency;

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn lei(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::RON)
}

fn rate(text: &str) -> ExchangeRate {
    ExchangeRate::parse(text).unwrap()
}

/// An amount typed in the group currency would need a rate, or would
/// convert at something other than 1.
#[test]
fn a_plain_amount_is_a_local_cost_at_a_rate_of_one() {
    let cost = Cost::from(euros(2_000));
    assert_eq!(cost, Cost::Local(euros(2_000)));
    assert_eq!(cost.amount(), euros(2_000));
    assert_eq!(cost.exchange_rate(), ExchangeRate::ONE);
    assert_eq!(cost.conversion(), None);
}

/// A foreign amount would lose the rate it was entered with.
#[test]
fn at_converts_an_amount_at_its_rate() {
    let cost = lei(40_000).at(rate("0.2"));
    assert_eq!(
        cost,
        Cost::Converted {
            amount: lei(40_000),
            rate: rate("0.2")
        }
    );
    assert_eq!(cost.amount(), lei(40_000));
    assert_eq!(cost.exchange_rate(), rate("0.2"));
    assert_eq!(cost.conversion(), Some(rate("0.2")));
}

/// An amount in a currency pegged at 1, such as the Bermudian dollar in a
/// US dollar group, would be taken for a local cost and refused.
#[test]
fn a_conversion_at_one_is_not_a_local_cost() {
    let bermudian_dollars = Money::from_minor_units(5_000, Currency::BMD);
    let pegged = bermudian_dollars.at(ExchangeRate::ONE);
    assert_ne!(pegged, Cost::from(bermudian_dollars));
    assert_eq!(pegged.exchange_rate(), ExchangeRate::ONE);
    assert_eq!(pegged.conversion(), Some(ExchangeRate::ONE));
}

/// Correcting an amount would drop its conversion, or give a local cost
/// one.
#[test]
fn with_amount_keeps_the_kind_of_cost() {
    // Local.
    assert_eq!(
        Cost::from(euros(2_000)).with_amount(euros(2_500)),
        Cost::from(euros(2_500))
    );
    // Converted.
    assert_eq!(
        lei(40_000).at(rate("0.2")).with_amount(lei(45_000)),
        lei(45_000).at(rate("0.2"))
    );
}

/// A stored entry would load as a converted cost without its rate, or as a
/// local one with it.
#[test]
fn from_parts_reads_a_missing_rate_as_a_local_cost() {
    assert_eq!(
        Cost::from_parts(euros(2_000), None),
        Cost::from(euros(2_000))
    );
    assert_eq!(
        Cost::from_parts(lei(40_000), Some(rate("0.2"))),
        lei(40_000).at(rate("0.2"))
    );
}

/// A saved cost would load as the other kind, or with another rate.
#[cfg(feature = "serde")]
#[test]
fn costs_serialise_by_kind() {
    // Local.
    let local = Cost::from(euros(2_000));
    let json = serde_json::to_string(&local).unwrap();
    assert_eq!(json, r#"{"local":{"currency":"EUR","minor_units":2000}}"#);
    assert_eq!(serde_json::from_str::<Cost>(&json).unwrap(), local);
    // Converted, with the rate as text.
    let converted = lei(40_000).at(rate("0.2"));
    let converted_json = serde_json::to_string(&converted).unwrap();
    assert_eq!(
        converted_json,
        r#"{"converted":{"amount":{"currency":"RON","minor_units":40000},"rate":"0.2"}}"#
    );
    assert_eq!(
        serde_json::from_str::<Cost>(&converted_json).unwrap(),
        converted
    );
}
