//! Tips added on top of an expense.

use crate::error::ExpenseError;
use crate::money::allocation::apply_basis_points;
use crate::money::{Money, Percentage};

/// A tip added on top of an expense's amount: a fixed amount or a percentage
/// of the amount, shared among the participants proportionally (the default)
/// or equally.
///
/// The payers cover the amount plus the tip. An expense without a tip has
/// [`Tip::default`], 0 % shared proportionally, which needs no currency.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Money, Percentage, Tip, TipDistribution};
///
/// let ten_per_cent = Tip::percentage(Percentage::parse("10")?);
/// assert_eq!(ten_per_cent.distribution(), TipDistribution::Proportional);
///
/// let round_up = Tip::fixed(Money::parse("5.00", Currency::EUR)?).distributed(TipDistribution::Equal);
/// assert_eq!(round_up.distribution(), TipDistribution::Equal);
///
/// assert_eq!(Tip::default(), Tip::percentage(Percentage::ZERO));
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tip {
    value: TipValue,
    #[cfg_attr(feature = "serde", serde(default))]
    distribution: TipDistribution,
}

impl Tip {
    /// A fixed tip in the expense's currency, shared proportionally.
    #[must_use]
    pub const fn fixed(amount: Money) -> Self {
        Self {
            value: TipValue::Fixed(amount),
            distribution: TipDistribution::Proportional,
        }
    }

    /// A tip of `rate` of the expense's amount, rounded half up to the minor
    /// unit and shared proportionally.
    #[must_use]
    pub const fn percentage(rate: Percentage) -> Self {
        Self {
            value: TipValue::Percentage(rate),
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

    /// The tip on `amount`, in the amount's currency.
    pub(crate) fn amount_on(self, amount: Money) -> Result<Money, ExpenseError> {
        match self.value {
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
            TipValue::Percentage(rate) => {
                apply_basis_points(amount.minor_units(), rate.basis_points())
                    .map(|tip| Money::from_minor_units(amount.currency(), tip))
                    .ok_or(ExpenseError::Overflow)
            }
        }
    }
}

impl Default for Tip {
    /// No tip: 0 % of the amount, shared proportionally.
    fn default() -> Self {
        Self::percentage(Percentage::ZERO)
    }
}

/// How a [`Tip`] is expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum TipValue {
    /// A fixed amount in the expense's currency.
    Fixed(Money),
    /// A percentage of the expense's amount.
    Percentage(Percentage),
}

/// How a [`Tip`] is shared among an expense's participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum TipDistribution {
    /// The same for every participant.
    Equal,
    /// In proportion to each participant's share of the amount: whoever
    /// ordered more tips more.
    #[default]
    Proportional,
}

#[cfg(test)]
mod tests;
