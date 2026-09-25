//! Every failure of this crate converts through `FromMoneyError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// A [`MoneyError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedMoneyError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromMoneyError for ConvertedMoneyError {
    fn currency_mismatch(left: Currency, message: String, right: Currency) -> Self {
        Self {
            held: format!("{left} {right}"),
            message,
            method: "currency_mismatch",
        }
    }

    fn invalid_amount(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_amount",
        }
    }

    fn invalid_exchange_rate(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_exchange_rate",
        }
    }

    fn invalid_parts(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_parts",
        }
    }

    fn invalid_percentage(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_percentage",
        }
    }

    fn overflow(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "overflow",
        }
    }

    fn too_many_decimals(allowed: u8, currency: Currency, message: String) -> Self {
        Self {
            held: format!("{allowed} {currency}"),
            message,
            method: "too_many_decimals",
        }
    }

    fn unknown_currency(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "unknown_currency",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_money_error_converts_through_the_method_of_its_variant() {
    let left = Currency::EUR;
    let right = Currency::RON;
    let allowed = 4;
    let currency = Currency::EUR;
    let cases = [
        (
            MoneyError::CurrencyMismatch { left, right },
            "currency_mismatch",
            format!("{left} {right}"),
        ),
        (MoneyError::InvalidAmount, "invalid_amount", String::new()),
        (
            MoneyError::InvalidExchangeRate,
            "invalid_exchange_rate",
            String::new(),
        ),
        (MoneyError::InvalidParts, "invalid_parts", String::new()),
        (
            MoneyError::InvalidPercentage,
            "invalid_percentage",
            String::new(),
        ),
        (MoneyError::Overflow, "overflow", String::new()),
        (
            MoneyError::TooManyDecimals { allowed, currency },
            "too_many_decimals",
            format!("{allowed} {currency}"),
        ),
        (
            MoneyError::UnknownCurrency,
            "unknown_currency",
            String::new(),
        ),
    ];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedMoneyError::from_money_error(&error),
            ConvertedMoneyError {
                held,
                message: error.to_string(),
                method,
            }
        );
    }
}
