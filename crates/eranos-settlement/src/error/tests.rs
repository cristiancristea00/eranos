//! Every failure of this crate converts through `FromSettlementError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// One variant in a table of conversions: the error, the method it converts
/// through, and what the conversion finds it holding.
#[derive(Debug)]
struct Case {
    error: SettlementError,
    method: &'static str,
    held: String,
}

/// A [`SettlementError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedSettlementError {
    method: &'static str,
    held: String,
    message: String,
}

impl FromSettlementError for ConvertedSettlementError {
    fn out_of_memory(balances: usize, message: String) -> Self {
        Self {
            method: "out_of_memory",
            held: format!("{balances}"),
            message,
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_settlement_error_converts_through_the_method_of_its_variant() {
    let balances = 20;
    let cases = [Case {
        error: SettlementError::OutOfMemory { balances },
        method: "out_of_memory",
        held: format!("{balances}"),
    }];
    for case in cases {
        assert_eq!(
            ConvertedSettlementError::from_settlement_error(&case.error),
            ConvertedSettlementError {
                method: case.method,
                held: case.held,
                message: case.error.to_string(),
            }
        );
    }
}
