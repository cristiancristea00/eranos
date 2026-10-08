//! A discount is taken off an item's line or off each of its units. These
//! tests keep a discount off the line unless another scope is chosen, keep
//! a percentage rounded half away from zero once for the line and once per
//! unit off each unit, keep a fixed discount taken off as entered, in the
//! item's currency, never negative and never above what it reduces, keep a
//! discount of the whole price making a free line, and keep both parts of a
//! discount through serde and through their conversion traits.

use eranos_money::CurrencyCode;

use super::*;

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn per_cent(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

/// The quantity written `text`.
fn quantity(text: &str) -> Quantity {
    Quantity::parse(text).unwrap()
}

/// What is left of `quantity` units at `unit_price` after `discount`, or
/// the error that refuses it.
fn line_total(
    unit_price: Money,
    quantity: Quantity,
    discount: Discount,
) -> Result<Money, ExpenseError> {
    LineTotals::of(unit_price, quantity, Some(discount)).map(|totals| totals.line_total)
}

/// A discount entered without choosing a scope would be taken off each
/// unit, multiplying it by the quantity.
#[test]
fn a_discount_is_taken_off_the_line_by_default() {
    // Percentages.
    let percentage = Discount::percentage(per_cent("10"));
    assert_eq!(
        percentage.value(),
        DiscountValue::Percentage(per_cent("10"))
    );
    assert_eq!(percentage.scope(), DiscountScope::Line);
    // Fixed amounts.
    let fixed = Discount::fixed(euros(100));
    assert_eq!(fixed.value(), DiscountValue::Fixed(euros(100)));
    assert_eq!(fixed.scope(), DiscountScope::Line);
    // The scope's own default.
    assert_eq!(DiscountScope::default(), DiscountScope::Line);
}

/// Choosing a scope would also change the discount's value.
#[test]
fn scoped_changes_only_the_scope() {
    // Percentages, switched back and forth.
    let percentage = Discount::percentage(per_cent("10"))
        .scoped(DiscountScope::Unit)
        .scoped(DiscountScope::Line);
    assert_eq!(percentage, Discount::percentage(per_cent("10")));
    // Fixed amounts.
    let fixed = Discount::fixed(euros(100)).scoped(DiscountScope::Unit);
    assert_eq!(fixed.value(), DiscountValue::Fixed(euros(100)));
    assert_eq!(fixed.scope(), DiscountScope::Unit);
}

/// A line without a discount would be reduced, or its total before the
/// discount would differ from its line total.
#[test]
fn a_line_without_a_discount_keeps_its_total() {
    // 3 × 4.50.
    let totals = LineTotals::of(euros(450), quantity("3"), None).unwrap();
    assert_eq!(
        totals,
        LineTotals {
            before_discount: euros(1_350),
            line_total: euros(1_350),
        }
    );
}

/// A percentage off each unit would be rounded once for the line, or a
/// percentage off the line once per unit, so the receipt and the
/// expense would disagree by a cent.
#[test]
fn a_percentage_rounds_once_per_unit_off_each_unit_and_once_off_the_line() {
    let half = Discount::percentage(per_cent("50"));
    // Off each unit: 1.665 off 3.33 rounds to 1.67, leaving 1.66 a unit,
    // 4.98 for three.
    assert_eq!(
        line_total(euros(333), quantity("3"), half.scoped(DiscountScope::Unit)),
        Ok(euros(498))
    );
    // Off the line: 4.995 off 9.99 rounds to 5.00, leaving 4.99.
    let totals = LineTotals::of(euros(333), quantity("3"), Some(half)).unwrap();
    assert_eq!(totals.before_discount, euros(999));
    assert_eq!(totals.line_total, euros(499));
    // 10 % off 0.35 kg at 42.00: 1.47 off 14.70.
    assert_eq!(
        line_total(
            euros(4_200),
            quantity("0.35"),
            Discount::percentage(per_cent("10"))
        ),
        Ok(euros(1_323))
    );
    // Off each unit of a fractional quantity, the reduced price is
    // multiplied and rounded: 37.80 × 0.35 is 13.23.
    assert_eq!(
        line_total(
            euros(4_200),
            quantity("0.35"),
            Discount::percentage(per_cent("10")).scoped(DiscountScope::Unit)
        ),
        Ok(euros(1_323))
    );
}

/// A fixed discount would be scaled or rounded instead of taken off as
/// entered: once off the line, once per unit off each unit.
#[test]
fn a_fixed_discount_is_taken_off_as_entered() {
    let one_euro = Discount::fixed(euros(100));
    // Off the line: 9.00 − 1.00.
    assert_eq!(
        line_total(euros(300), quantity("3"), one_euro),
        Ok(euros(800))
    );
    // Off each unit: 3 × (3.00 − 1.00).
    assert_eq!(
        line_total(
            euros(300),
            quantity("3"),
            one_euro.scoped(DiscountScope::Unit)
        ),
        Ok(euros(600))
    );
    // Off each of 0.35 units: 41.00 × 0.35 is 14.35.
    assert_eq!(
        line_total(
            euros(4_200),
            quantity("0.35"),
            one_euro.scoped(DiscountScope::Unit)
        ),
        Ok(euros(1_435))
    );
    // A discount of zero changes nothing.
    assert_eq!(
        line_total(euros(300), quantity("3"), Discount::fixed(euros(0))),
        Ok(euros(900))
    );
    assert_eq!(
        line_total(
            euros(300),
            quantity("3"),
            Discount::percentage(Percentage::ZERO)
        ),
        Ok(euros(900))
    );
}

/// A dish on the house would be refused, or charged a cent it does not
/// cost.
#[test]
fn a_discount_of_the_whole_price_makes_a_free_line() {
    // 100 %, off the line and off each unit.
    for scope in [DiscountScope::Line, DiscountScope::Unit] {
        let totals = LineTotals::of(
            euros(900),
            quantity("1"),
            Some(Discount::percentage(Percentage::HUNDRED).scoped(scope)),
        )
        .unwrap();
        assert_eq!(totals.before_discount, euros(900), "{scope:?}");
        assert_eq!(totals.line_total, euros(0), "{scope:?}");
    }
    // A fixed amount equal to the line, and to each unit.
    assert_eq!(
        line_total(euros(300), quantity("3"), Discount::fixed(euros(900))),
        Ok(euros(0))
    );
    assert_eq!(
        line_total(
            euros(300),
            quantity("3"),
            Discount::fixed(euros(300)).scoped(DiscountScope::Unit)
        ),
        Ok(euros(0))
    );
}

/// A discount larger than what it reduces would make a line cost less than
/// nothing and pay its members to share it.
#[test]
fn a_discount_above_what_it_reduces_is_refused() {
    // 10.00 off a line of 9.00.
    assert_eq!(
        line_total(euros(900), quantity("1"), Discount::fixed(euros(1_000))),
        Err(ExpenseError::DiscountAbovePrice)
    );
    // 3.01 off each unit at 3.00, though the line of 9.00 is larger.
    assert_eq!(
        line_total(
            euros(300),
            quantity("3"),
            Discount::fixed(euros(301)).scoped(DiscountScope::Unit)
        ),
        Err(ExpenseError::DiscountAbovePrice)
    );
    // A cent off a free line of 0.4 of a cent.
    assert_eq!(
        line_total(euros(1), quantity("0.4"), Discount::fixed(euros(1))),
        Err(ExpenseError::DiscountAbovePrice)
    );
    // A basis point above 100 %, off the line and off each unit.
    for scope in [DiscountScope::Line, DiscountScope::Unit] {
        let above = Discount::percentage(Percentage::from_basis_points(10_001)).scoped(scope);
        assert_eq!(
            line_total(euros(900), quantity("1"), above),
            Err(ExpenseError::DiscountAboveHundredPercent),
            "{scope:?}"
        );
    }
}

/// A negative discount would quietly add to a line what the receipt does
/// not charge.
#[test]
fn a_negative_fixed_discount_is_refused() {
    // One cent below zero, off the line and off each unit.
    for scope in [DiscountScope::Line, DiscountScope::Unit] {
        assert_eq!(
            line_total(
                euros(900),
                quantity("1"),
                Discount::fixed(euros(-1)).scoped(scope)
            ),
            Err(ExpenseError::NegativeDiscount),
            "{scope:?}"
        );
    }
}

/// A discount in dollars would be taken off a euro line as if it were
/// euros.
#[test]
fn a_fixed_discount_in_another_currency_is_refused() {
    let mismatch = Err(ExpenseError::CurrencyMismatch {
        expected: CurrencyCode::EUR,
        found: CurrencyCode::USD,
    });
    // A dollar off a euro line.
    let dollar = Money::from_minor_units(100, CurrencyCode::USD);
    assert_eq!(
        line_total(euros(900), quantity("1"), Discount::fixed(dollar)),
        mismatch
    );
    // The currency is checked before the sign and the size.
    for minor_units in [-100, 100_000] {
        let dollars = Money::from_minor_units(minor_units, CurrencyCode::USD);
        assert_eq!(
            line_total(
                euros(900),
                quantity("1"),
                Discount::fixed(dollars).scoped(DiscountScope::Unit)
            ),
            mismatch
        );
    }
}

/// A line too large for 64-bit minor units would wrap around before its
/// discount is taken off, and a discount at the edge of the range would be
/// refused though it fits.
#[test]
fn a_line_out_of_range_is_refused_whatever_its_discount() {
    // Two units of the largest price, even at 100 % off.
    assert_eq!(
        line_total(
            euros(i64::MAX),
            quantity("2"),
            Discount::percentage(Percentage::HUNDRED)
        ),
        Err(ExpenseError::Overflow)
    );
    // One unit of the largest price, less a cent off each unit and less
    // all of it off the line.
    assert_eq!(
        line_total(
            euros(i64::MAX),
            quantity("1"),
            Discount::fixed(euros(1)).scoped(DiscountScope::Unit)
        ),
        Ok(euros(i64::MAX - 1))
    );
    assert_eq!(
        line_total(
            euros(i64::MAX),
            quantity("1"),
            Discount::percentage(Percentage::HUNDRED)
        ),
        Ok(euros(0))
    );
}

/// Saved discounts would fail to load, or load off another scope.
#[cfg(feature = "serde")]
#[test]
fn discounts_round_trip_through_serde() {
    // Round trip, with the scope written.
    let discount = Discount::percentage(per_cent("10")).scoped(DiscountScope::Unit);
    let json = serde_json::to_string(&discount).unwrap();
    assert_eq!(json, r#"{"value":{"percentage":1000},"scope":"unit"}"#);
    assert_eq!(serde_json::from_str::<Discount>(&json).unwrap(), discount);
    // A fixed discount off the line.
    let fixed = Discount::fixed(euros(100));
    assert_eq!(
        serde_json::to_value(fixed).unwrap(),
        serde_json::json!({"value": {"fixed": {"minor_units": 100, "currency": "EUR"}}, "scope": "line"})
    );
    // A discount saved without a scope is off the line.
    let loaded: Discount = serde_json::from_str(r#"{"value":{"percentage":1000}}"#).unwrap();
    assert_eq!(loaded, Discount::percentage(per_cent("10")));
}

/// A discount's value as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedValue {
    Percentage(Percentage),
    Fixed(Money),
}

impl FromDiscountValue for ConvertedValue {
    fn percentage(percentage: Percentage) -> Self {
        Self::Percentage(percentage)
    }

    fn fixed(amount: Money) -> Self {
        Self::Fixed(amount)
    }
}

/// A discount's scope as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedScope {
    Line,
    Unit,
}

impl FromDiscountScope for ConvertedScope {
    fn line() -> Self {
        Self::Line
    }

    fn unit() -> Self {
        Self::Unit
    }
}

/// A conversion that called another variant's method would hand the
/// bindings a fixed discount as a percentage, or a discount off each unit
/// as one off the line.
#[test]
fn a_discount_converts_through_the_methods_of_its_variants() {
    assert_eq!(
        ConvertedValue::from_discount_value(DiscountValue::Percentage(per_cent("10"))),
        ConvertedValue::Percentage(per_cent("10"))
    );
    assert_eq!(
        ConvertedValue::from_discount_value(DiscountValue::Fixed(euros(100))),
        ConvertedValue::Fixed(euros(100))
    );
    assert_eq!(
        ConvertedScope::from_discount_scope(DiscountScope::Line),
        ConvertedScope::Line
    );
    assert_eq!(
        ConvertedScope::from_discount_scope(DiscountScope::Unit),
        ConvertedScope::Unit
    );
}
