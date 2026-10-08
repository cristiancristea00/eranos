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
    /// The currency, with what ISO 4217 says of it.
    pub currency: CurrencyDetails,
}

impl Money {
    /// The record of `money`.
    pub(crate) fn from_library(money: library::Money) -> Self {
        Self {
            minor_units: money.minor_units(),
            currency: CurrencyDetails::from_library(money.currency()),
        }
    }

    /// The library's amount.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for a currency the library does not list: a code that is not an ISO
    /// 4217 currency with minor units, or details other than the library's
    /// for the code.
    pub(crate) fn to_library(&self) -> Result<library::Money, EranosError> {
        Ok(library::Money::from_minor_units(
            self.minor_units,
            self.currency.to_library()?,
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
    /// for an amount in a currency the library does not list, as for
    /// [`Money::to_library`], and
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
    pub decimal_places: u8,
}

impl CurrencyDetails {
    /// The details of `currency`.
    pub(crate) fn from_library(currency: library::CurrencyCode) -> Self {
        Self {
            code: currency.code().to_owned(),
            name: currency.name().to_owned(),
            numeric_code: i32::from(currency.numeric_code()),
            decimal_places: currency.decimal_places(),
        }
    }

    /// The library's currency of these details.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for a code that names no currency with minor units, and for details
    /// other than the library's for the code, a code in lower case included:
    /// such details name no currency the library lists.
    pub(crate) fn to_library(&self) -> Result<library::CurrencyCode, EranosError> {
        let Self {
            code,
            name,
            numeric_code,
            decimal_places,
        } = self;
        let currency = library::CurrencyCode::from_code(code)?;
        if currency.code() == code
            && currency.name() == name
            && i32::from(currency.numeric_code()) == *numeric_code
            && currency.decimal_places() == *decimal_places
        {
            Ok(currency)
        } else {
            Err(MoneyError::UnknownCurrency.into())
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

/// The quantity of `ten_thousandths`.
///
/// # Errors
///
/// [`MoneyFailure::InvalidQuantity`](crate::MoneyFailure::InvalidQuantity)
/// for a number that is not positive.
pub(crate) fn quantity(ten_thousandths: i64) -> Result<library::Quantity, EranosError> {
    let ten_thousandths =
        u64::try_from(ten_thousandths).map_err(|_| MoneyError::InvalidQuantity)?;
    Ok(library::Quantity::from_ten_thousandths(ten_thousandths)?)
}

/// The ten-thousandths of `quantity`, as the bindings' 64-bit signed
/// integers hold them.
pub(crate) fn ten_thousandths(quantity: library::Quantity) -> i64 {
    // A quantity holds at most `i64::MAX` ten-thousandths, so the fallback
    // never applies.
    i64::try_from(quantity.ten_thousandths()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests;
