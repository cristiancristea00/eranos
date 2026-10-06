//! Every failure of this crate converts through `FromMoneyError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: MoneyError,
    held: String,
    method: &'static str,
}

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
        Case {
            error: MoneyError::CurrencyMismatch { left, right },
            held: format!("{left} {right}"),
            method: "currency_mismatch",
        },
        Case {
            error: MoneyError::InvalidAmount,
            held: String::new(),
            method: "invalid_amount",
        },
        Case {
            error: MoneyError::InvalidExchangeRate,
            held: String::new(),
            method: "invalid_exchange_rate",
        },
        Case {
            error: MoneyError::InvalidParts,
            held: String::new(),
            method: "invalid_parts",
        },
        Case {
            error: MoneyError::InvalidPercentage,
            held: String::new(),
            method: "invalid_percentage",
        },
        Case {
            error: MoneyError::Overflow,
            held: String::new(),
            method: "overflow",
        },
        Case {
            error: MoneyError::TooManyDecimals { allowed, currency },
            held: format!("{allowed} {currency}"),
            method: "too_many_decimals",
        },
        Case {
            error: MoneyError::UnknownCurrency,
            held: String::new(),
            method: "unknown_currency",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedMoneyError::from_money_error(&case.error),
            ConvertedMoneyError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
