//! Every failure of this crate converts through `FromContactError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: ContactError,
    held: String,
    method: &'static str,
}

/// A [`ContactError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedContactError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromContactError for ConvertedContactError {
    fn invalid_email(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_email",
        }
    }

    fn invalid_phone_number(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_phone_number",
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
            held: String::new(),
            method: "invalid_email",
        },
        Case {
            error: ContactError::InvalidPhoneNumber,
            held: String::new(),
            method: "invalid_phone_number",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedContactError::from_contact_error(case.error),
            ConvertedContactError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
