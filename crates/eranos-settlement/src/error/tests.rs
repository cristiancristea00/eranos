//! Every failure of this crate converts through `FromSettlementError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// A [`SettlementError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedSettlementError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromSettlementError for ConvertedSettlementError {
    fn out_of_memory(balances: usize, message: String) -> Self {
        Self {
            held: format!("{balances}"),
            message,
            method: "out_of_memory",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_settlement_error_converts_through_the_method_of_its_variant() {
    let balances = 20;
    let cases = [(
        SettlementError::OutOfMemory { balances },
        "out_of_memory",
        format!("{balances}"),
    )];
    for (error, method, held) in cases {
        assert_eq!(
            ConvertedSettlementError::from_settlement_error(&error),
            ConvertedSettlementError {
                held,
                message: error.to_string(),
                method,
            }
        );
    }
}
