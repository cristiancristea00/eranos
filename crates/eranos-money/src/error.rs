//! The failure of building, reading, or combining an amount, a currency, a
//! rate, a percentage, a number of parts, or a quantity.

use crate::CurrencyCode;

/// An amount, a currency, an exchange rate, a percentage, a number of parts,
/// or a quantity that could not be built, read, or computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum MoneyError {
    /// An amount that is not a decimal number.
    #[error("invalid amount: expected digits with an optional `.` and sign")]
    InvalidAmount,
    /// An amount with more decimal places than its currency has.
    #[error("{currency} amounts have at most {allowed} decimal places")]
    TooManyDecimals {
        /// The currency of the amount.
        currency: CurrencyCode,
        /// The currency's number of decimal places.
        allowed: u8,
    },
    /// Arithmetic between amounts of two different currencies.
    #[error("cannot combine {left} and {right} amounts")]
    CurrencyMismatch {
        /// The currency of the left operand.
        left: CurrencyCode,
        /// The currency of the right operand.
        right: CurrencyCode,
    },
    /// A currency code that is not an active ISO 4217 code.
    #[error("unknown currency code")]
    UnknownCurrency,
    /// An exchange rate that is not a positive decimal number with at most 18
    /// decimal places.
    #[error("invalid exchange rate: expected a positive number with at most 18 decimal places")]
    InvalidExchangeRate,
    /// A number of parts that is not a positive decimal number with at most
    /// two decimal places.
    #[error("invalid parts: expected a positive number with at most 2 decimal places")]
    InvalidParts,
    /// A percentage that is not a decimal number with at most two decimal
    /// places.
    #[error("invalid percentage: expected a number with at most 2 decimal places")]
    InvalidPercentage,
    /// A quantity that is not a positive decimal number with at most four
    /// decimal places, or whose ten-thousandths do not fit in an `i64`.
    #[error("invalid quantity: expected a positive number with at most 4 decimal places")]
    InvalidQuantity,
    /// A double that is NaN or infinite, which no amount, percentage, number
    /// of parts, quantity, or exchange rate is.
    #[error("the number is not finite")]
    NotFinite,
    /// A number or a result that does not fit its range.
    #[error("value out of range")]
    Overflow,
}

/// Builds a value from a [`MoneyError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromMoneyError: Sized {
    /// The value for [`MoneyError::InvalidAmount`].
    fn invalid_amount(message: String) -> Self;

    /// The value for [`MoneyError::TooManyDecimals`].
    fn too_many_decimals(currency: CurrencyCode, allowed: u8, message: String) -> Self;

    /// The value for [`MoneyError::CurrencyMismatch`].
    fn currency_mismatch(left: CurrencyCode, right: CurrencyCode, message: String) -> Self;

    /// The value for [`MoneyError::UnknownCurrency`].
    fn unknown_currency(message: String) -> Self;

    /// The value for [`MoneyError::InvalidExchangeRate`].
    fn invalid_exchange_rate(message: String) -> Self;

    /// The value for [`MoneyError::InvalidParts`].
    fn invalid_parts(message: String) -> Self;

    /// The value for [`MoneyError::InvalidPercentage`].
    fn invalid_percentage(message: String) -> Self;

    /// The value for [`MoneyError::InvalidQuantity`].
    fn invalid_quantity(message: String) -> Self;

    /// The value for [`MoneyError::NotFinite`].
    fn not_finite(message: String) -> Self;

    /// The value for [`MoneyError::Overflow`].
    fn overflow(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_money_error(error: &MoneyError) -> Self {
        let message = error.to_string();
        match error {
            MoneyError::InvalidAmount => Self::invalid_amount(message),
            MoneyError::TooManyDecimals { currency, allowed } => {
                Self::too_many_decimals(*currency, *allowed, message)
            }
            MoneyError::CurrencyMismatch { left, right } => {
                Self::currency_mismatch(*left, *right, message)
            }
            MoneyError::UnknownCurrency => Self::unknown_currency(message),
            MoneyError::InvalidExchangeRate => Self::invalid_exchange_rate(message),
            MoneyError::InvalidParts => Self::invalid_parts(message),
            MoneyError::InvalidPercentage => Self::invalid_percentage(message),
            MoneyError::InvalidQuantity => Self::invalid_quantity(message),
            MoneyError::NotFinite => Self::not_finite(message),
            MoneyError::Overflow => Self::overflow(message),
        }
    }
}

#[cfg(test)]
mod tests;
