//! Every failure of this crate converts through `FromIdError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message, and each message free of the refused text.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: IdError,
    method: &'static str,
    held: String,
}

/// An [`IdError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedIdError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromIdError for ConvertedIdError {
    fn empty(message: String) -> Self {
        Self {
            method: "empty",
            held: String::new(),
            message,
        }
    }

    fn too_long(message: String) -> Self {
        Self {
            method: "too_long",
            held: String::new(),
            message,
        }
    }

    fn control_character(message: String) -> Self {
        Self {
            method: "control_character",
            held: String::new(),
            message,
        }
    }

    fn surrounding_whitespace(message: String) -> Self {
        Self {
            method: "surrounding_whitespace",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_id_error_converts_through_the_method_of_its_variant() {
    let cases = [
        Case {
            error: IdError::Empty,
            method: "empty",
            held: String::new(),
        },
        Case {
            error: IdError::TooLong,
            method: "too_long",
            held: String::new(),
        },
        Case {
            error: IdError::ControlCharacter,
            method: "control_character",
            held: String::new(),
        },
        Case {
            error: IdError::SurroundingWhitespace,
            method: "surrounding_whitespace",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedIdError::from_id_error(case.error),
            ConvertedIdError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}

/// The apps and the ports of the examples show these messages; a message
/// that changed would differ from theirs, and one that repeated the text
/// would echo whatever a file or a user supplied.
#[test]
fn every_failure_of_id_error_has_its_own_message() {
    assert_eq!(
        IdError::Empty.to_string(),
        "an identifier must not be empty"
    );
    assert_eq!(
        IdError::TooLong.to_string(),
        "an identifier can be at most 64 bytes long"
    );
    assert_eq!(
        IdError::ControlCharacter.to_string(),
        "an identifier cannot contain control characters"
    );
    assert_eq!(
        IdError::SurroundingWhitespace.to_string(),
        "an identifier cannot start or end with whitespace"
    );
}
