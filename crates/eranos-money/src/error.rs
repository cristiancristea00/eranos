//! The failure of building, reading, or combining an amount, a currency, a
//! rate, a percentage, or a number of parts.

use crate::Currency;

/// An amount, a currency, an exchange rate, a percentage, or a number of
/// parts that could not be built, read, or computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum MoneyError {
    /// Arithmetic between amounts of two different currencies.
    #[error("cannot combine {left} and {right} amounts")]
    CurrencyMismatch {
        /// The currency of the left operand.
        left: Currency,
        /// The currency of the right operand.
        right: Currency,
    },
    /// An amount that is not a decimal number.
    #[error("invalid amount: expected digits with an optional `.` and sign")]
    InvalidAmount,
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
    /// A number or a result that does not fit its range.
    #[error("value out of range")]
    Overflow,
    /// An amount with more decimal places than its currency has.
    #[error("{currency} amounts have at most {allowed} decimal places")]
    TooManyDecimals {
        /// The currency's number of decimal places.
        allowed: u8,
        /// The currency of the amount.
        currency: Currency,
    },
    /// A currency code that is not an active ISO 4217 code.
    #[error("unknown currency code")]
    UnknownCurrency,
}
