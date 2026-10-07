//! Amounts, costs, exchange rates, and currencies, as the bindings carry
//! them.

use eranos::error::MoneyError;
use eranos::money as library;
use eranos_money::FromCost;

use crate::EranosError;

/// An amount: a whole number of minor units of a currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    /// The amount in minor units: cents for the euro, none for the yen.
    pub minor_units: i64,
    /// The ISO 4217 code of the currency.
    pub currency: String,
}

impl Money {
    /// The record of `money`.
    pub(crate) fn from_library(money: library::Money) -> Self {
        Self {
            currency: money.currency().code().to_owned(),
            minor_units: money.minor_units(),
        }
    }

    /// The library's amount.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for a code that is not an ISO 4217 currency with minor units.
    pub(crate) fn to_library(&self) -> Result<library::Money, EranosError> {
        Ok(library::Money::from_minor_units(
            self.minor_units,
            currency(&self.currency)?,
        ))
    }
}

/// What an entry cost: an amount in the group currency, or one in another
/// currency with its exchange rate into the group currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cost {
    /// An amount in the group currency.
    Local {
        /// The amount.
        amount: Money,
    },
    /// An amount in another currency than the group's, converted at `rate`.
    Converted {
        /// The amount, in its own currency.
        amount: Money,
        /// The rate into the group currency, as a decimal: "4.97".
        rate: String,
    },
}

impl Cost {
    /// The library's cost.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for an unknown currency and
    /// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate)
    /// for a rate that is not a positive decimal.
    pub(crate) fn to_library(&self) -> Result<library::Cost, EranosError> {
        match self {
            Self::Local { amount } => Ok(library::Cost::from(amount.to_library()?)),
            Self::Converted { amount, rate } => Ok(amount.to_library()?.at(exchange_rate(rate)?)),
        }
    }
}

impl FromCost for Cost {
    fn local(amount: library::Money) -> Self {
        Self::Local {
            amount: Money::from_library(amount),
        }
    }

    fn converted(amount: library::Money, rate: library::ExchangeRate) -> Self {
        Self::Converted {
            amount: Money::from_library(amount),
            rate: rate.to_string(),
        }
    }
}

/// The rate of a currency into another: one of the rates a currency change
/// takes.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyRate {
    /// The ISO 4217 code of the currency the rate converts from.
    pub currency: String,
    /// The rate, as a decimal: "4.97".
    pub rate: String,
}

impl CurrencyRate {
    /// The library's rate.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// and
    /// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate).
    pub(crate) fn to_library(&self) -> Result<library::CurrencyRate, EranosError> {
        Ok(library::CurrencyRate::new(
            currency(&self.currency)?,
            exchange_rate(&self.rate)?,
        ))
    }
}

/// What ISO 4217 says of a currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyDetails {
    /// The three-letter code: "EUR".
    pub code: String,
    /// The English name: "Euro".
    pub name: String,
    /// The three-digit numeric code: 978 for the euro.
    pub numeric_code: i32,
    /// How many decimal places its minor unit has: 2 for the euro.
    pub decimal_places: i32,
}

impl CurrencyDetails {
    /// The details of `currency`.
    pub(crate) fn from_library(currency: library::CurrencyCode) -> Self {
        Self {
            code: currency.code().to_owned(),
            name: currency.name().to_owned(),
            numeric_code: i32::from(currency.numeric_code()),
            decimal_places: i32::from(currency.decimal_places()),
        }
    }
}

/// The currency with the ISO 4217 code `code`.
///
/// # Errors
///
/// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency) for
/// a code that names no currency with minor units.
pub(crate) fn currency(code: &str) -> Result<library::CurrencyCode, EranosError> {
    Ok(library::CurrencyCode::from_code(code)?)
}

/// The exchange rate written `text`.
///
/// # Errors
///
/// [`MoneyFailure::InvalidExchangeRate`](crate::MoneyFailure::InvalidExchangeRate)
/// for text that is not a positive decimal with at most 18 decimal places.
pub(crate) fn exchange_rate(text: &str) -> Result<library::ExchangeRate, EranosError> {
    Ok(library::ExchangeRate::parse(text)?)
}

/// The percentage of `basis_points`.
///
/// # Errors
///
/// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
/// for a negative number.
pub(crate) fn percentage(basis_points: i32) -> Result<library::Percentage, EranosError> {
    let basis_points = u32::try_from(basis_points).map_err(|_| MoneyError::InvalidPercentage)?;
    Ok(library::Percentage::from_basis_points(basis_points))
}

/// The parts of `hundredths`.
///
/// # Errors
///
/// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for a
/// number that is not positive.
pub(crate) fn parts(hundredths: i32) -> Result<library::Parts, EranosError> {
    let hundredths = u32::try_from(hundredths).map_err(|_| MoneyError::InvalidParts)?;
    Ok(library::Parts::from_hundredths(hundredths)?)
}

#[cfg(test)]
mod tests;
