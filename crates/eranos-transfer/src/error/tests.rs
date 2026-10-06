//! Every failure of this crate converts through `FromTransferError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: TransferError,
    held: String,
    method: &'static str,
}

/// A [`TransferError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedTransferError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromTransferError for ConvertedTransferError {
    fn non_positive_amount(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "non_positive_amount",
        }
    }

    fn same_member(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "same_member",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_transfer_error_converts_through_the_method_of_its_variant() {
    let cases = [
        Case {
            error: TransferError::NonPositiveAmount,
            held: String::new(),
            method: "non_positive_amount",
        },
        Case {
            error: TransferError::SameMember,
            held: String::new(),
            method: "same_member",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedTransferError::from_transfer_error(case.error),
            ConvertedTransferError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
