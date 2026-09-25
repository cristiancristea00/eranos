//! Every failure of this crate converts through `FromAccountError`, and the
//! reason of a refused IBAN through `FromIbanError`: the traits through which
//! a crate outside this one, such as the binding core, reports them. These
//! tests keep each variant reaching its own method with what it holds, and
//! each failure its message.

use super::*;

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
        (AccountError::EmptyName, "empty_name", String::new()),
        (AccountError::EmptyUsername, "empty_username", String::new()),
        (
            AccountError::InvalidIban(reason),
            "invalid_iban",
            format!("{reason:?}"),
        ),
    ];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedAccountError::from_account_error(&error),
            ConvertedAccountError {
                held,
                message: error.to_string(),
                method,
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
        (
            IbanError::InvalidCharacter,
            "invalid_character",
            String::new(),
        ),
        (
            IbanError::InvalidChecksum,
            "invalid_checksum",
            String::new(),
        ),
        (IbanError::InvalidFormat, "invalid_format", String::new()),
        (
            IbanError::InvalidStructure,
            "invalid_structure",
            String::new(),
        ),
        (IbanError::UnknownCountry, "unknown_country", String::new()),
        (
            IbanError::WrongLength { expected, found },
            "wrong_length",
            format!("{expected} {found}"),
        ),
    ];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedIbanError::from_iban_error(&error),
            ConvertedIbanError { held, method }
        );
    }
}
