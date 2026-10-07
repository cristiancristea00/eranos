//! Tips are added on top of an expense and covered by its payers. These
//! tests keep the default tip at nothing, keep percentage tips rounded to
//! the nearest minor unit, with halves going up, keep fixed tips in the
//! expense's currency and never negative, keep how a tip is shared
//! independent of how it is expressed, and keep both through their
//! conversion traits.

use eranos_money::Currency;

use super::*;

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn per_cent(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

/// An expense without a tip would be charged one, or its tip would need a
/// currency the expense has not given it yet.
#[test]
fn the_default_tip_is_none_at_all() {
    let tip = TipRule::default();
    assert_eq!(tip.value(), TipValue::Percentage(Percentage::ZERO));
    assert_eq!(tip.distribution(), TipDistribution::Proportional);
    assert_eq!(tip.amount_on(euros(9_000)), Ok(euros(0)));
}

/// Tips entered without choosing a distribution would be shared equally,
/// charging light eaters as much as heavy ones.
#[test]
fn tips_are_shared_proportionally_by_default() {
    // Percentage tips.
    let percentage = TipRule::percentage(per_cent("10"));
    assert_eq!(percentage.value(), TipValue::Percentage(per_cent("10")));
    assert_eq!(percentage.distribution(), TipDistribution::Proportional);
    // Fixed tips.
    let fixed = TipRule::fixed(euros(500));
    assert_eq!(fixed.value(), TipValue::Fixed(euros(500)));
    assert_eq!(fixed.distribution(), TipDistribution::Proportional);
    // The distribution's own default.
    assert_eq!(TipDistribution::default(), TipDistribution::Proportional);
}

/// Choosing an equal distribution would also change the tip's value.
#[test]
fn distributed_changes_only_the_distribution() {
    // Percentage tips, switched back and forth.
    let percentage = TipRule::percentage(per_cent("10"))
        .distributed(TipDistribution::Equal)
        .distributed(TipDistribution::Proportional);
    assert_eq!(percentage, TipRule::percentage(per_cent("10")));
    // Fixed tips.
    let fixed = TipRule::fixed(euros(500)).distributed(TipDistribution::Equal);
    assert_eq!(fixed.value(), TipValue::Fixed(euros(500)));
    assert_eq!(fixed.distribution(), TipDistribution::Equal);
}

/// A percentage tip would charge another amount than the percentage says.
#[test]
fn a_percentage_tip_is_that_share_of_the_amount() {
    // 10 % of 90.00 is 9.00.
    assert_eq!(
        TipRule::percentage(per_cent("10")).amount_on(euros(9_000)),
        Ok(euros(900))
    );
    // Tips above 100 % are allowed.
    assert_eq!(
        TipRule::percentage(per_cent("150")).amount_on(euros(1_000)),
        Ok(euros(1_500))
    );
    // Currencies without decimals: 10 % of 1234 JPY is 123.4, so 123.
    let yen = Money::from_minor_units(1_234, Currency::JPY);
    assert_eq!(
        TipRule::percentage(per_cent("10")).amount_on(yen),
        Ok(Money::from_minor_units(123, Currency::JPY))
    );
}

/// Percentage tips would be off by a cent, or round halves down.
#[test]
fn percentage_tips_round_to_the_nearest_minor_unit() {
    // 12.5 % of 0.99 is 12.375 cents, so 12.
    assert_eq!(
        TipRule::percentage(per_cent("12.5")).amount_on(euros(99)),
        Ok(euros(12))
    );
    // 15 % of 0.10 is 1.5 cents, so 2.
    assert_eq!(
        TipRule::percentage(per_cent("15")).amount_on(euros(10)),
        Ok(euros(2))
    );
    // 12.5 % of 1.00 is 12.5 cents, so 13.
    assert_eq!(
        TipRule::percentage(per_cent("12.5")).amount_on(euros(100)),
        Ok(euros(13))
    );
    // 0.01 % of 0.49 is 0.0049 cents, so 0.
    assert_eq!(
        TipRule::percentage(per_cent("0.01")).amount_on(euros(49)),
        Ok(euros(0))
    );
}

/// A fixed tip would be scaled or rounded instead of added as entered.
#[test]
fn a_fixed_tip_is_added_as_entered() {
    // Whatever the amount.
    assert_eq!(
        TipRule::fixed(euros(500)).amount_on(euros(9_000)),
        Ok(euros(500))
    );
    assert_eq!(
        TipRule::fixed(euros(500)).amount_on(euros(1)),
        Ok(euros(500))
    );
}

/// An expense with a tip of zero would be refused.
#[test]
fn zero_tips_are_allowed() {
    // Percentage and fixed.
    assert_eq!(
        TipRule::percentage(Percentage::ZERO).amount_on(euros(9_000)),
        Ok(euros(0))
    );
    assert_eq!(
        TipRule::fixed(euros(0)).amount_on(euros(9_000)),
        Ok(euros(0))
    );
}

/// A tip in dollars would be added to a euro expense as if it were euros.
#[test]
fn a_fixed_tip_in_another_currency_is_rejected() {
    // A dollar tip on a euro expense.
    let dollars = Money::from_minor_units(500, Currency::USD);
    assert_eq!(
        TipRule::fixed(dollars).amount_on(euros(9_000)),
        Err(ExpenseError::CurrencyMismatch {
            expected: Currency::EUR,
            found: Currency::USD
        })
    );
    // The currency is checked before the sign.
    let negative_dollars = Money::from_minor_units(-500, Currency::USD);
    assert_eq!(
        TipRule::fixed(negative_dollars).amount_on(euros(9_000)),
        Err(ExpenseError::CurrencyMismatch {
            expected: Currency::EUR,
            found: Currency::USD
        })
    );
}

/// A negative tip would quietly discount the expense for its payers.
#[test]
fn a_negative_fixed_tip_is_rejected() {
    // One cent below zero.
    assert_eq!(
        TipRule::fixed(euros(-1)).amount_on(euros(9_000)),
        Err(ExpenseError::NegativeTip)
    );
}

/// A tip too large for 64-bit minor units would wrap around to a wrong
/// amount.
#[test]
fn a_tip_out_of_range_is_rejected() {
    // 200 % of the largest amount.
    assert_eq!(
        TipRule::percentage(per_cent("200")).amount_on(euros(i64::MAX)),
        Err(ExpenseError::Overflow)
    );
    // The largest percentage of the largest amount.
    assert_eq!(
        TipRule::percentage(Percentage::from_basis_points(u32::MAX)).amount_on(euros(i64::MAX)),
        Err(ExpenseError::Overflow)
    );
    // 100 % of the largest amount fits.
    assert_eq!(
        TipRule::percentage(Percentage::HUNDRED).amount_on(euros(i64::MAX)),
        Ok(euros(i64::MAX))
    );
}

/// Saved tips would fail to load, or load with another distribution.
#[cfg(feature = "serde")]
#[test]
fn tips_round_trip_through_serde() {
    // Round trip.
    let tip = TipRule::percentage(per_cent("10")).distributed(TipDistribution::Equal);
    let json = serde_json::to_string(&tip).unwrap();
    assert_eq!(
        json,
        r#"{"value":{"percentage":1000},"distribution":"equal"}"#
    );
    assert_eq!(serde_json::from_str::<TipRule>(&json).unwrap(), tip);
    // A tip saved without a distribution is shared proportionally.
    let loaded: TipRule = serde_json::from_str(r#"{"value":{"percentage":1000}}"#).unwrap();
    assert_eq!(loaded, TipRule::percentage(per_cent("10")));
}

/// A tip's value as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedValue {
    Percentage(Percentage),
    Fixed(Money),
}

impl FromTipValue for ConvertedValue {
    fn percentage(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }

    fn fixed(amount: Money) -> Self {
        Self::Fixed(amount)
    }
}

/// A tip's distribution as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedDistribution {
    Proportional,
    Equal,
}

impl FromTipDistribution for ConvertedDistribution {
    fn proportional() -> Self {
        Self::Proportional
    }

    fn equal() -> Self {
        Self::Equal
    }
}

/// A conversion that called another variant's method would hand the
/// bindings a fixed tip as a percentage, or an equal share of a tip as a
/// proportional one.
#[test]
fn a_tip_converts_through_the_methods_of_its_variants() {
    assert_eq!(
        ConvertedValue::from_tip_value(TipValue::Percentage(per_cent("10"))),
        ConvertedValue::Percentage(per_cent("10"))
    );
    assert_eq!(
        ConvertedValue::from_tip_value(TipValue::Fixed(euros(500))),
        ConvertedValue::Fixed(euros(500))
    );
    assert_eq!(
        ConvertedDistribution::from_tip_distribution(TipDistribution::Proportional),
        ConvertedDistribution::Proportional
    );
    assert_eq!(
        ConvertedDistribution::from_tip_distribution(TipDistribution::Equal),
        ConvertedDistribution::Equal
    );
}
