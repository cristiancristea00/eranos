//! Tips added on top of an expense.

use eranos_money::allocation::apply_basis_points;
use eranos_money::{Money, Percentage};

use crate::error::ExpenseError;

/// A tip added on top of an expense's amount: a percentage of the amount or a
/// fixed amount, shared among the participants proportionally (the default)
/// or equally.
///
/// The payers cover the amount plus the tip. An expense without a tip has
/// [`TipRule::default`], 0 % shared proportionally, which needs no currency.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, Money, Percentage, TipRule, TipDistribution};
///
/// let ten_per_cent = TipRule::percentage(Percentage::parse("10")?);
/// assert_eq!(ten_per_cent.distribution(), TipDistribution::Proportional);
///
/// let round_up = TipRule::fixed(Money::parse("5.00", CurrencyCode::EUR)?).distributed(TipDistribution::Equal);
/// assert_eq!(round_up.distribution(), TipDistribution::Equal);
///
/// assert_eq!(TipRule::default(), TipRule::percentage(Percentage::ZERO));
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TipRule {
    value: TipValue,
    #[cfg_attr(feature = "serde", serde(default))]
    distribution: TipDistribution,
}

impl TipRule {
    /// A tip of `rate` of the expense's amount, rounded half up to the minor
    /// unit and shared proportionally.
    #[must_use]
    pub const fn percentage(rate: Percentage) -> Self {
        Self {
            value: TipValue::Percentage(rate),
            distribution: TipDistribution::Proportional,
        }
    }

    /// A fixed tip in the expense's currency, shared proportionally.
    #[must_use]
    pub const fn fixed(amount: Money) -> Self {
        Self {
            value: TipValue::Fixed(amount),
            distribution: TipDistribution::Proportional,
        }
    }

    /// How the tip is expressed.
    #[must_use]
    pub const fn value(self) -> TipValue {
        self.value
    }

    /// How the tip is shared among the participants.
    #[must_use]
    pub const fn distribution(self) -> TipDistribution {
        self.distribution
    }

    /// Returns the same tip shared by `distribution`.
    #[must_use]
    pub const fn distributed(self, distribution: TipDistribution) -> Self {
        Self {
            distribution,
            ..self
        }
    }

    /// The tip on `amount`, in the amount's currency: the percentage of
    /// `amount` rounded to the nearest minor unit, halves away from zero, or
    /// the fixed amount.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::CurrencyMismatch`] when a fixed tip is in another
    ///   currency than `amount`.
    /// - [`ExpenseError::NegativeTip`] when a fixed tip is negative.
    /// - [`ExpenseError::Overflow`] when the percentage of `amount` leaves
    ///   the range of an amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, Money, Percentage, TipRule};
    ///
    /// let bill = Money::parse("90.00", CurrencyCode::EUR)?;
    /// let tip = TipRule::percentage(Percentage::parse("10")?);
    /// assert_eq!(tip.amount_on(bill)?, Money::parse("9.00", CurrencyCode::EUR)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn amount_on(self, amount: Money) -> Result<Money, ExpenseError> {
        match self.value {
            TipValue::Percentage(rate) => {
                apply_basis_points(amount.minor_units(), rate.basis_points())
                    .map(|tip| Money::from_minor_units(tip, amount.currency()))
                    .ok_or(ExpenseError::Overflow)
            }
            TipValue::Fixed(tip) => {
                if tip.currency() != amount.currency() {
                    return Err(ExpenseError::CurrencyMismatch {
                        expected: amount.currency(),
                        found: tip.currency(),
                    });
                }
                if tip.is_negative() {
                    return Err(ExpenseError::NegativeTip);
                }
                Ok(tip)
            }
        }
    }
}

impl Default for TipRule {
    /// No tip: 0 % of the amount, shared proportionally.
    fn default() -> Self {
        Self::percentage(Percentage::ZERO)
    }
}

/// How a [`TipRule`] is expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum TipValue {
    /// A percentage of the expense's amount.
    Percentage(Percentage),
    /// A fixed amount in the expense's currency.
    Fixed(Money),
}

/// How a [`TipRule`] is shared among an expense's participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum TipDistribution {
    /// In proportion to each participant's share of the amount: whoever
    /// ordered more tips more.
    #[default]
    Proportional,
    /// The same for every participant.
    Equal,
}

/// Builds a value from a [`TipValue`], one method per way of expressing a
/// tip.
pub trait FromTipValue: Sized {
    /// The value for a `percentage` of the expense's amount.
    fn percentage(percentage: Percentage) -> Self;

    /// The value for a fixed `amount`.
    fn fixed(amount: Money) -> Self;

    /// The value for `value`, from the method of its variant.
    #[must_use]
    fn from_tip_value(value: TipValue) -> Self {
        match value {
            TipValue::Percentage(percentage) => Self::percentage(percentage),
            TipValue::Fixed(amount) => Self::fixed(amount),
        }
    }
}

/// Builds a value from a [`TipDistribution`], one method per way of sharing
/// a tip.
pub trait FromTipDistribution: Sized {
    /// The value for a tip shared in proportion to each participant's share.
    fn proportional() -> Self;

    /// The value for a tip shared equally.
    fn equal() -> Self;

    /// The value for `distribution`, from the method of its variant.
    #[must_use]
    fn from_tip_distribution(distribution: TipDistribution) -> Self {
        match distribution {
            TipDistribution::Proportional => Self::proportional(),
            TipDistribution::Equal => Self::equal(),
        }
    }
}

#[cfg(test)]
mod tests;
