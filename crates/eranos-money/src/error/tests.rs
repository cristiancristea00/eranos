//! Every failure of this crate converts through `FromMoneyError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_money_error_converts_through_the_method_of_its_variant: MoneyError => ConvertedMoneyError through FromMoneyError::from_money_error(&) {
        methods {
            invalid_amount() => String::new();
            too_many_decimals(currency: CurrencyCode, allowed: u8) => format!("{currency} {allowed}");
            currency_mismatch(left: CurrencyCode, right: CurrencyCode) => format!("{left} {right}");
            unknown_currency() => String::new();
            invalid_exchange_rate() => String::new();
            invalid_parts() => String::new();
            invalid_percentage() => String::new();
            invalid_quantity() => String::new();
            not_finite() => String::new();
            overflow() => String::new();
        }
        setup {
            let currency = CurrencyCode::EUR;
            let allowed = 4;
            let left = CurrencyCode::EUR;
            let right = CurrencyCode::RON;
        }
        cases {
            MoneyError::InvalidAmount => invalid_amount, String::new();
            MoneyError::TooManyDecimals { currency, allowed } => too_many_decimals, format!("{currency} {allowed}");
            MoneyError::CurrencyMismatch { left, right } => currency_mismatch, format!("{left} {right}");
            MoneyError::UnknownCurrency => unknown_currency, String::new();
            MoneyError::InvalidExchangeRate => invalid_exchange_rate, String::new();
            MoneyError::InvalidParts => invalid_parts, String::new();
            MoneyError::InvalidPercentage => invalid_percentage, String::new();
            MoneyError::InvalidQuantity => invalid_quantity, String::new();
            MoneyError::NotFinite => not_finite, String::new();
            MoneyError::Overflow => overflow, String::new();
        }
    }
}
