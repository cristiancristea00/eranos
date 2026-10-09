//! Every failure of this crate converts through `FromTransferError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_transfer_error_converts_through_the_method_of_its_variant: TransferError => ConvertedTransferError through FromTransferError::from_transfer_error() {
        methods {
            same_member() => String::new();
            non_positive_amount() => String::new();
        }
        setup {}
        cases {
            TransferError::SameMember => same_member, String::new();
            TransferError::NonPositiveAmount => non_positive_amount, String::new();
        }
    }
}
