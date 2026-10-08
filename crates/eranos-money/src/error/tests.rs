//! Every failure of this crate converts through `FromMoneyError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: MoneyError,
    method: &'static str,
    held: String,
}

/// A [`MoneyError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedMoneyError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromMoneyError for ConvertedMoneyError {
    fn invalid_amount(message: String) -> Self {
        Self {
            method: "invalid_amount",
            held: String::new(),
            message,
        }
    }

    fn too_many_decimals(currency: CurrencyCode, allowed: u8, message: String) -> Self {
        Self {
            method: "too_many_decimals",
            held: format!("{currency} {allowed}"),
            message,
        }
    }

    fn currency_mismatch(left: CurrencyCode, right: CurrencyCode, message: String) -> Self {
        Self {
            method: "currency_mismatch",
            held: format!("{left} {right}"),
            message,
        }
    }

    fn unknown_currency(message: String) -> Self {
        Self {
            method: "unknown_currency",
            held: String::new(),
            message,
        }
    }

    fn invalid_exchange_rate(message: String) -> Self {
        Self {
            method: "invalid_exchange_rate",
            held: String::new(),
            message,
        }
    }

    fn invalid_parts(message: String) -> Self {
        Self {
            method: "invalid_parts",
            held: String::new(),
            message,
        }
    }

    fn invalid_percentage(message: String) -> Self {
        Self {
            method: "invalid_percentage",
            held: String::new(),
            message,
        }
    }

    fn invalid_quantity(message: String) -> Self {
        Self {
            method: "invalid_quantity",
            held: String::new(),
            message,
        }
    }

    fn not_finite(message: String) -> Self {
        Self {
            method: "not_finite",
            held: String::new(),
            message,
        }
    }

    fn overflow(message: String) -> Self {
        Self {
            method: "overflow",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_money_error_converts_through_the_method_of_its_variant() {
    let currency = CurrencyCode::EUR;
    let allowed = 4;
    let left = CurrencyCode::EUR;
    let right = CurrencyCode::RON;
    let cases = [
        Case {
            error: MoneyError::InvalidAmount,
            method: "invalid_amount",
            held: String::new(),
        },
        Case {
            error: MoneyError::TooManyDecimals { currency, allowed },
            method: "too_many_decimals",
            held: format!("{currency} {allowed}"),
        },
        Case {
            error: MoneyError::CurrencyMismatch { left, right },
            method: "currency_mismatch",
            held: format!("{left} {right}"),
        },
        Case {
            error: MoneyError::UnknownCurrency,
            method: "unknown_currency",
            held: String::new(),
        },
        Case {
            error: MoneyError::InvalidExchangeRate,
            method: "invalid_exchange_rate",
            held: String::new(),
        },
        Case {
            error: MoneyError::InvalidParts,
            method: "invalid_parts",
            held: String::new(),
        },
        Case {
            error: MoneyError::InvalidPercentage,
            method: "invalid_percentage",
            held: String::new(),
        },
        Case {
            error: MoneyError::InvalidQuantity,
            method: "invalid_quantity",
            held: String::new(),
        },
        Case {
            error: MoneyError::NotFinite,
            method: "not_finite",
            held: String::new(),
        },
        Case {
            error: MoneyError::Overflow,
            method: "overflow",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedMoneyError::from_money_error(&case.error),
            ConvertedMoneyError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
