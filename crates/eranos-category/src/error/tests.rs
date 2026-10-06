//! Every failure of this crate converts through `FromCategoryError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: CategoryError,
    held: String,
    method: &'static str,
}

/// A [`CategoryError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedCategoryError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromCategoryError for ConvertedCategoryError {
    fn empty_name(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "empty_name",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_category_error_converts_through_the_method_of_its_variant() {
    let cases = [Case {
        error: CategoryError::EmptyName,
        held: String::new(),
        method: "empty_name",
    }];
    for case in cases {
        assert_eq!(
            ConvertedCategoryError::from_category_error(case.error),
            ConvertedCategoryError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
