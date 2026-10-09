//! Every failure of this crate converts through `FromPaymentAccountError`, and the
//! reason of a refused IBAN through `FromIbanError`: the traits through which
//! a crate outside this one, such as the binding core, reports them. These
//! tests keep each variant reaching its own method with what it holds, and
//! each failure its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_account_error_converts_through_the_method_of_its_variant: PaymentAccountError => ConvertedPaymentAccountError through FromPaymentAccountError::from_payment_account_error(&) {
        methods {
            empty_name() => String::new();
            invalid_iban(reason: IbanError) => format!("{reason:?}");
            empty_username() => String::new();
        }
        setup {
            let reason = IbanError::InvalidChecksum;
        }
        cases {
            PaymentAccountError::EmptyName => empty_name, String::new();
            PaymentAccountError::InvalidIban(reason) => invalid_iban, format!("{reason:?}");
            PaymentAccountError::EmptyUsername => empty_username, String::new();
        }
    }
}

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds, would show the apps another reason than the
    /// library's.
    fn every_reason_of_iban_error_converts_through_the_method_of_its_variant: IbanError => ConvertedIbanError through FromIbanError::from_iban_error(&) without message {
        methods {
            invalid_character() => String::new();
            invalid_format() => String::new();
            unknown_country() => String::new();
            wrong_length(expected: usize, found: usize) => format!("{expected} {found}");
            invalid_structure() => String::new();
            invalid_checksum() => String::new();
        }
        setup {
            let expected = 20;
            let found = 21;
        }
        cases {
            IbanError::InvalidCharacter => invalid_character, String::new();
            IbanError::InvalidFormat => invalid_format, String::new();
            IbanError::UnknownCountry => unknown_country, String::new();
            IbanError::WrongLength { expected, found } => wrong_length, format!("{expected} {found}");
            IbanError::InvalidStructure => invalid_structure, String::new();
            IbanError::InvalidChecksum => invalid_checksum, String::new();
        }
    }
}
