//! What an entry cost, and how it converts into its group's currency.

use crate::money::{ExchangeRate, Money};

/// What an entry cost: its amount, and for an amount in another currency
/// than its group's, the rate into the group currency.
///
/// A plain [`Money`] converts into a [`Cost::Local`] cost, which is in the
/// group currency and converts at [`ExchangeRate::ONE`]. [`Money::at`] makes
/// a [`Cost::Converted`] one, with the rate fixed when the entry is entered.
/// A conversion at 1 is still a conversion: the Bermudian dollar is pegged
/// to the US dollar, so a BMD expense in a USD group converts at
/// `ExchangeRate::ONE`, while a local cost in BMD there is refused. The
/// group currency is known only once the entry is added to a group, which
/// checks then that a local cost is in its currency and a converted one is
/// not.
///
/// # Examples
///
/// ```
/// use eranos::{Cost, Currency, ExchangeRate, Money};
///
/// // 20.00 EUR in a EUR group.
/// let museum = Cost::from(Money::parse("20.00", Currency::EUR)?);
/// assert_eq!(museum.exchange_rate(), ExchangeRate::ONE);
///
/// // 400.00 RON in a EUR group, at 1 RON = 0.2 EUR.
/// let rate = ExchangeRate::parse("0.2")?;
/// let fuel = Money::parse("400.00", Currency::RON)?.at(rate);
/// assert_eq!(fuel.amount(), Money::parse("400.00", Currency::RON)?);
/// assert_eq!(fuel.exchange_rate(), rate);
///
/// // A corrected amount keeps the rate.
/// let corrected = fuel.with_amount(Money::parse("450.00", Currency::RON)?);
/// assert_eq!(corrected.exchange_rate(), rate);
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum Cost {
    /// An amount in another currency, with the rate into the group
    /// currency; made by [`Money::at`].
    #[non_exhaustive]
    Converted {
        /// The amount, in the entry's own currency.
        amount: Money,
        /// How many units of the group currency one unit of the amount's
        /// currency is worth.
        rate: ExchangeRate,
    },
    /// An amount in the group currency.
    Local(Money),
}

impl Cost {
    /// The amount, in the entry's own currency.
    #[must_use]
    pub const fn amount(self) -> Money {
        match self {
            Self::Converted { amount, .. } | Self::Local(amount) => amount,
        }
    }

    /// The rate into the group currency: [`ExchangeRate::ONE`] for a local
    /// cost.
    #[must_use]
    pub const fn exchange_rate(self) -> ExchangeRate {
        match self {
            Self::Converted { rate, .. } => rate,
            Self::Local(_) => ExchangeRate::ONE,
        }
    }

    /// The same kind of cost for another amount: a converted cost keeps its
    /// rate.
    #[must_use]
    pub const fn with_amount(self, amount: Money) -> Self {
        match self {
            Self::Converted { rate, .. } => Self::Converted { amount, rate },
            Self::Local(_) => Self::Local(amount),
        }
    }

    /// The cost of an amount stored with its rate: `None` for a local cost.
    pub(crate) const fn from_parts(amount: Money, rate: Option<ExchangeRate>) -> Self {
        match rate {
            None => Self::Local(amount),
            Some(rate) => Self::Converted { amount, rate },
        }
    }

    /// The rate of a converted cost; `None` for a local one.
    pub(crate) const fn conversion(self) -> Option<ExchangeRate> {
        match self {
            Self::Converted { rate, .. } => Some(rate),
            Self::Local(_) => None,
        }
    }
}

impl From<Money> for Cost {
    fn from(amount: Money) -> Self {
        Self::Local(amount)
    }
}

#[cfg(test)]
mod tests;
