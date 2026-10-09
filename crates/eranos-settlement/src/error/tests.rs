//! Every failure of this crate converts through `FromSettlementError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_settlement_error_converts_through_the_method_of_its_variant: SettlementError => ConvertedSettlementError through FromSettlementError::from_settlement_error(&) {
        methods {
            out_of_memory(balances: usize) => format!("{balances}");
        }
        setup {
            let balances = 20;
        }
        cases {
            SettlementError::OutOfMemory { balances } => out_of_memory, format!("{balances}");
        }
    }
}
