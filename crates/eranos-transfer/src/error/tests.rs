//! Every failure of this crate converts through `FromTransferError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: TransferError,
    method: &'static str,
    held: String,
}

/// A [`TransferError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedTransferError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromTransferError for ConvertedTransferError {
    fn same_member(message: String) -> Self {
        Self {
            method: "same_member",
            held: String::new(),
            message,
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self {
            method: "non_positive_amount",
            held: String::new(),
            message,
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
            error: TransferError::SameMember,
            method: "same_member",
            held: String::new(),
        },
        Case {
            error: TransferError::NonPositiveAmount,
            method: "non_positive_amount",
            held: String::new(),
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedTransferError::from_transfer_error(case.error),
            ConvertedTransferError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
