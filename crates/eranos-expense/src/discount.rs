//! Discounts taken off the lines of an itemised expense.

use eranos_money::allocation::{apply_basis_points, apply_ten_thousandths};
use eranos_money::{Money, Percentage, Quantity};

use crate::error::ExpenseError;

/// A discount off an [`ExpenseItem`](crate::ExpenseItem): a percentage or a
/// fixed amount, taken off the line as a whole (the default) or off the
/// price of each unit.
///
/// Off the line, a percentage is taken of the line total before the
/// discount, the unit price times the quantity, and rounded half away from
/// zero to the minor unit. Off each unit, a percentage of the unit price is
/// rounded to the minor unit first, and the reduced price is then
/// multiplied by the quantity, so three units at 3.33 with 50 % off cost
/// 4.98 with the discount off each unit and 4.99 with it off the line. A
/// discount of the whole price, 100 % or
/// a fixed amount equal to what it reduces, makes a free line, whose total
/// is zero; a larger one is refused when the item is built.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, Discount, DiscountScope, DiscountValue, Money, Percentage};
///
/// let ten_per_cent = Discount::percentage(Percentage::parse("10")?);
/// assert_eq!(ten_per_cent.scope(), DiscountScope::Line);
///
/// let one_euro = Money::parse("1.00", CurrencyCode::EUR)?;
/// let off_each = Discount::fixed(one_euro).scoped(DiscountScope::Unit);
/// assert_eq!(off_each.value(), DiscountValue::Fixed(one_euro));
/// assert_eq!(off_each.scope(), DiscountScope::Unit);
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Discount {
    value: DiscountValue,
    #[cfg_attr(feature = "serde", serde(default))]
    scope: DiscountScope,
}

impl Discount {
    /// A discount of `rate` of the line total, rounded half away from zero
    /// to the minor unit.
    #[must_use]
    pub const fn percentage(rate: Percentage) -> Self {
        Self {
            value: DiscountValue::Percentage(rate),
            scope: DiscountScope::Line,
        }
    }

    /// A discount of a fixed amount off the line total, in the item's
    /// currency.
    #[must_use]
    pub const fn fixed(amount: Money) -> Self {
        Self {
            value: DiscountValue::Fixed(amount),
            scope: DiscountScope::Line,
        }
    }

    /// How the discount is expressed.
    #[must_use]
    pub const fn value(self) -> DiscountValue {
        self.value
    }

    /// What the discount reduces: the line total or the price of each unit.
    #[must_use]
    pub const fn scope(self) -> DiscountScope {
        self.scope
    }

    /// Returns the same discount taken off `scope`.
    #[must_use]
    pub const fn scoped(self, scope: DiscountScope) -> Self {
        Self { scope, ..self }
    }

    /// What is left of `price`, the line total before the discount or the
    /// unit price, after the discount, in minor units.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::CurrencyMismatch`] when a fixed discount is in
    ///   another currency than `price`.
    /// - [`ExpenseError::NegativeDiscount`] when a fixed discount is
    ///   negative.
    /// - [`ExpenseError::DiscountAboveHundredPercent`] when a percentage is
    ///   above 100 %.
    /// - [`ExpenseError::DiscountAbovePrice`] when a fixed discount is
    ///   larger than `price`.
    fn left_of(self, price: Money) -> Result<i64, ExpenseError> {
        let minor_units = price.minor_units();
        let discount = match self.value {
            DiscountValue::Percentage(rate) => {
                if Percentage::HUNDRED < rate {
                    return Err(ExpenseError::DiscountAboveHundredPercent);
                }
                // At most 100 % of a price is at most the price, which
                // fits, so the fallback never applies.
                apply_basis_points(minor_units, rate.basis_points()).unwrap_or(minor_units)
            }
            DiscountValue::Fixed(amount) => {
                if amount.currency() != price.currency() {
                    return Err(ExpenseError::CurrencyMismatch {
                        expected: price.currency(),
                        found: amount.currency(),
                    });
                }
                if amount.is_negative() {
                    return Err(ExpenseError::NegativeDiscount);
                }
                if minor_units < amount.minor_units() {
                    return Err(ExpenseError::DiscountAbovePrice);
                }
                amount.minor_units()
            }
        };
        // 0 ≤ discount ≤ price, so the difference is between zero and the
        // price.
        Ok(minor_units - discount)
    }
}

/// How a [`Discount`] is expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum DiscountValue {
    /// A percentage of what the discount reduces, at most 100 %.
    Percentage(Percentage),
    /// A fixed amount in the item's currency, at most what the discount
    /// reduces.
    Fixed(Money),
}

/// What a [`Discount`] is taken off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum DiscountScope {
    /// The line total, the unit price times the quantity: one discount for
    /// the whole line.
    #[default]
    Line,
    /// The price of each unit, before it is multiplied by the quantity.
    Unit,
}

/// Builds a value from a [`DiscountValue`], one method per way of
/// expressing a discount.
pub trait FromDiscountValue: Sized {
    /// The value for a `percentage` of what the discount reduces.
    fn percentage(percentage: Percentage) -> Self;

    /// The value for a fixed `amount`.
    fn fixed(amount: Money) -> Self;

    /// The value for `value`, from the method of its variant.
    #[must_use]
    fn from_discount_value(value: DiscountValue) -> Self {
        match value {
            DiscountValue::Percentage(percentage) => Self::percentage(percentage),
            DiscountValue::Fixed(amount) => Self::fixed(amount),
        }
    }
}

/// Builds a value from a [`DiscountScope`], one method per thing a discount
/// is taken off.
pub trait FromDiscountScope: Sized {
    /// The value for a discount off the line total.
    fn line() -> Self;

    /// The value for a discount off the price of each unit.
    fn unit() -> Self;

    /// The value for `scope`, from the method of its variant.
    #[must_use]
    fn from_discount_scope(scope: DiscountScope) -> Self {
        match scope {
            DiscountScope::Line => Self::line(),
            DiscountScope::Unit => Self::unit(),
        }
    }
}

/// What a line of `quantity` units costs before its discount and after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LineTotals {
    /// The unit price times the quantity, rounded half away from zero to
    /// the minor unit.
    pub(crate) before_discount: Money,
    /// What is left after the discount: between zero and the total before
    /// it.
    pub(crate) line_total: Money,
}

impl LineTotals {
    /// The totals of `quantity` units at `unit_price`, less `discount` when
    /// there is one; `unit_price` is positive.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::Overflow`] when the line total before the discount
    ///   does not fit.
    /// - The errors of a discount the line cannot take:
    ///   [`ExpenseError::CurrencyMismatch`],
    ///   [`ExpenseError::NegativeDiscount`],
    ///   [`ExpenseError::DiscountAboveHundredPercent`], and
    ///   [`ExpenseError::DiscountAbovePrice`].
    pub(crate) fn of(
        unit_price: Money,
        quantity: Quantity,
        discount: Option<Discount>,
    ) -> Result<Self, ExpenseError> {
        let currency = unit_price.currency();
        let multiplied = |price: i64| apply_ten_thousandths(price, quantity.ten_thousandths());
        let before_discount = Money::from_minor_units(
            multiplied(unit_price.minor_units()).ok_or(ExpenseError::Overflow)?,
            currency,
        );
        let Some(discount) = discount else {
            return Ok(Self {
                before_discount,
                line_total: before_discount,
            });
        };
        let minor_units = match discount.scope {
            DiscountScope::Line => discount.left_of(before_discount)?,
            // What is left of the unit price is between zero and the price,
            // so its product with the quantity is at most the total before
            // the discount, which fits, and the fallback never applies.
            DiscountScope::Unit => {
                let fallback = before_discount.minor_units();
                multiplied(discount.left_of(unit_price)?).unwrap_or(fallback)
            }
        };
        Ok(Self {
            before_discount,
            line_total: Money::from_minor_units(minor_units, currency),
        })
    }
}

#[cfg(test)]
mod tests;
