//! Every failure of this crate converts through `FromMemberError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: MemberError,
    method: &'static str,
    held: String,
}

/// A [`MemberError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedMemberError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromMemberError for ConvertedMemberError {
    fn empty_first_name(message: String) -> Self {
        Self {
            method: "empty_first_name",
            held: String::new(),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_member_error_converts_through_the_method_of_its_variant() {
    let cases = [Case {
        error: MemberError::EmptyFirstName,
        method: "empty_first_name",
        held: String::new(),
    }];
    for case in cases {
        assert_eq!(
            ConvertedMemberError::from_member_error(case.error),
            ConvertedMemberError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
