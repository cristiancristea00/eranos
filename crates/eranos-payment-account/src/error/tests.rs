//! Every failure of this crate converts through `FromPaymentAccountError`, and the
//! reason of a refused IBAN through `FromIbanError`: the traits through which
//! a crate outside this one, such as the binding core, reports them. These
//! tests keep each variant reaching its own method with what it holds, and
//! each failure its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case<Failure> {
    error: Failure,
    method: &'static str,
    held: String,
}

/// An [`PaymentAccountError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedPaymentAccountError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromPaymentAccountError for ConvertedPaymentAccountError {
    fn empty_name(message: String) -> Self {
        Self {
            method: "empty_name",
            held: String::new(),
            message,
        }
    }

    fn invalid_iban(reason: IbanError, message: String) -> Self {
        Self {
            method: "invalid_iban",
            held: format!("{reason:?}"),
            message,
        }
    }

    fn empty_username(message: String) -> Self {
        Self {
            method: "empty_username",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_account_error_converts_through_the_method_of_its_variant() {
    let reason = IbanError::InvalidChecksum;
    let cases = [
        Case {
            error: PaymentAccountError::EmptyName,
            method: "empty_name",
            held: String::new(),
        },
        Case {
            error: PaymentAccountError::InvalidIban(reason),
            method: "invalid_iban",
            held: format!("{reason:?}"),
        },
        Case {
            error: PaymentAccountError::EmptyUsername,
            method: "empty_username",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedPaymentAccountError::from_payment_account_error(&case.error),
            ConvertedPaymentAccountError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}

/// An [`IbanError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedIbanError {
    method: &'static str,
    held: String,
}

impl FromIbanError for ConvertedIbanError {
    fn invalid_character() -> Self {
        Self {
            method: "invalid_character",
            held: String::new(),
        }
    }

    fn invalid_format() -> Self {
        Self {
            method: "invalid_format",
            held: String::new(),
        }
    }

    fn unknown_country() -> Self {
        Self {
            method: "unknown_country",
            held: String::new(),
        }
    }

    fn wrong_length(expected: usize, found: usize) -> Self {
        Self {
            method: "wrong_length",
            held: format!("{expected} {found}"),
        }
    }

    fn invalid_structure() -> Self {
        Self {
            method: "invalid_structure",
            held: String::new(),
        }
    }

    fn invalid_checksum() -> Self {
        Self {
            method: "invalid_checksum",
            held: String::new(),
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds, would show the apps another reason than the
/// library's.
#[test]
fn every_reason_of_iban_error_converts_through_the_method_of_its_variant() {
    let expected = 20;
    let found = 21;
    let cases = [
        Case {
            error: IbanError::InvalidCharacter,
            method: "invalid_character",
            held: String::new(),
        },
        Case {
            error: IbanError::InvalidFormat,
            method: "invalid_format",
            held: String::new(),
        },
        Case {
            error: IbanError::UnknownCountry,
            method: "unknown_country",
            held: String::new(),
        },
        Case {
            error: IbanError::WrongLength { expected, found },
            method: "wrong_length",
            held: format!("{expected} {found}"),
        },
        Case {
            error: IbanError::InvalidStructure,
            method: "invalid_structure",
            held: String::new(),
        },
        Case {
            error: IbanError::InvalidChecksum,
            method: "invalid_checksum",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedIbanError::from_iban_error(&case.error),
            ConvertedIbanError {
                method: case.method,
                held: case.held,
            }
        );
    }
}
