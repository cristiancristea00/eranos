//! Every failure of this crate converts through `FromAccountError`, and the
//! reason of a refused IBAN through `FromIbanError`: the traits through which
//! a crate outside this one, such as the binding core, reports them. These
//! tests keep each variant reaching its own method with what it holds, and
//! each failure its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case<Failure> {
    error: Failure,
    held: String,
    method: &'static str,
}

/// An [`AccountError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedAccountError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromAccountError for ConvertedAccountError {
    fn empty_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_name",
        }
    }

    fn empty_username(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_username",
        }
    }

    fn invalid_iban(message: String, reason: IbanError) -> Self {
        Self {
            held: format!("{reason:?}"),
            message,
            method: "invalid_iban",
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
            error: AccountError::EmptyName,
            held: String::new(),
            method: "empty_name",
        },
        Case {
            error: AccountError::EmptyUsername,
            held: String::new(),
            method: "empty_username",
        },
        Case {
            error: AccountError::InvalidIban(reason),
            held: format!("{reason:?}"),
            method: "invalid_iban",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedAccountError::from_account_error(&case.error),
            ConvertedAccountError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}

/// An [`IbanError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedIbanError {
    held: String,
    method: &'static str,
}

impl FromIbanError for ConvertedIbanError {
    fn invalid_character() -> Self {
        Self {
            held: String::new(),
            method: "invalid_character",
        }
    }

    fn invalid_checksum() -> Self {
        Self {
            held: String::new(),
            method: "invalid_checksum",
        }
    }

    fn invalid_format() -> Self {
        Self {
            held: String::new(),
            method: "invalid_format",
        }
    }

    fn invalid_structure() -> Self {
        Self {
            held: String::new(),
            method: "invalid_structure",
        }
    }

    fn unknown_country() -> Self {
        Self {
            held: String::new(),
            method: "unknown_country",
        }
    }

    fn wrong_length(expected: usize, found: usize) -> Self {
        Self {
            held: format!("{expected} {found}"),
            method: "wrong_length",
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
            held: String::new(),
            method: "invalid_character",
        },
        Case {
            error: IbanError::InvalidChecksum,
            held: String::new(),
            method: "invalid_checksum",
        },
        Case {
            error: IbanError::InvalidFormat,
            held: String::new(),
            method: "invalid_format",
        },
        Case {
            error: IbanError::InvalidStructure,
            held: String::new(),
            method: "invalid_structure",
        },
        Case {
            error: IbanError::UnknownCountry,
            held: String::new(),
            method: "unknown_country",
        },
        Case {
            error: IbanError::WrongLength { expected, found },
            held: format!("{expected} {found}"),
            method: "wrong_length",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedIbanError::from_iban_error(&case.error),
            ConvertedIbanError {
                held: case.held,
                method: case.method
            }
        );
    }
}
