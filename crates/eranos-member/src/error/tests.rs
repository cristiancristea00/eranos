//! Every failure of this crate converts through `FromMemberError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// A [`MemberError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedMemberError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromMemberError for ConvertedMemberError {
    fn empty_first_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_first_name",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_member_error_converts_through_the_method_of_its_variant() {
    let cases = [(
        MemberError::EmptyFirstName,
        "empty_first_name",
        String::new(),
    )];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedMemberError::from_member_error(error),
            ConvertedMemberError {
                held,
                message: error.to_string(),
                method,
            }
        );
    }
}
