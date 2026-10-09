//! Every failure of this crate converts through `FromIdError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message, and each message free of the refused text.

use super::*;

include!("conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_id_error_converts_through_the_method_of_its_variant: IdError => ConvertedIdError through FromIdError::from_id_error() {
        methods {
            empty() => String::new();
            too_long() => String::new();
            control_character() => String::new();
            surrounding_whitespace() => String::new();
        }
        setup {}
        cases {
            IdError::Empty => empty, String::new();
            IdError::TooLong => too_long, String::new();
            IdError::ControlCharacter => control_character, String::new();
            IdError::SurroundingWhitespace => surrounding_whitespace, String::new();
        }
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
