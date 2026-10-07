//! Every failure of this crate converts through `FromContactError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: ContactError,
    method: &'static str,
    held: String,
}

/// A [`ContactError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedContactError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromContactError for ConvertedContactError {
    fn invalid_email(message: String) -> Self {
        Self {
            method: "invalid_email",
            held: String::new(),
            message,
        }
    }

    fn invalid_phone_number(message: String) -> Self {
        Self {
            method: "invalid_phone_number",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_contact_error_converts_through_the_method_of_its_variant() {
    let cases = [
        Case {
            error: ContactError::InvalidEmail,
            method: "invalid_email",
            held: String::new(),
        },
        Case {
            error: ContactError::InvalidPhoneNumber,
            method: "invalid_phone_number",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedContactError::from_contact_error(case.error),
            ConvertedContactError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
