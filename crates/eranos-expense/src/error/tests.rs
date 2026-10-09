//! Every failure of this crate converts through `FromExpenseError`, the trait
//! through which a crate outside this one, such as the binding core, reports
//! it. These tests keep each variant reaching its own method with what it
//! holds and its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_expense_error_converts_through_the_method_of_its_variant: ExpenseError => ConvertedExpenseError through FromExpenseError::from_expense_error(&) {
        methods {
            empty_name() => String::new();
            non_positive_amount() => String::new();
            currency_mismatch(expected: CurrencyCode, found: CurrencyCode) => format!("{expected} {found}");
            empty_split() => String::new();
            duplicate_member(member: MemberId) => format!("{member}");
            non_positive_share(member: MemberId) => format!("{member}");
            amounts_do_not_add_up(expected: Money, actual: Money) => format!("{expected} {actual}");
            percentages_do_not_add_up(total_basis_points: u64) => format!("{total_basis_points}");
            no_items() => String::new();
            empty_item_name() => String::new();
            items_do_not_add_up(expected: Money, actual: Money) => format!("{expected} {actual}");
            negative_discount() => String::new();
            discount_above_hundred_percent() => String::new();
            discount_above_price() => String::new();
            negative_tip() => String::new();
            overflow() => String::new();
        }
        setup {
            let expected_currency = CurrencyCode::USD;
            let found = CurrencyCode::EUR;
            let member = MemberId::new("ana").unwrap();
            let expected_money = Money::from_minor_units(200, CurrencyCode::EUR);
            let actual = Money::from_minor_units(100, CurrencyCode::EUR);
            let total_basis_points = 9005;
        }
        cases {
            ExpenseError::EmptyName => empty_name, String::new();
            ExpenseError::NonPositiveAmount => non_positive_amount, String::new();
            ExpenseError::CurrencyMismatch { expected: expected_currency, found, } => currency_mismatch, format!("{expected_currency} {found}");
            ExpenseError::EmptySplit => empty_split, String::new();
            ExpenseError::DuplicateMember { member } => duplicate_member, format!("{member}");
            ExpenseError::NonPositiveShare { member } => non_positive_share, format!("{member}");
            ExpenseError::AmountsDoNotAddUp { expected: expected_money, actual, } => amounts_do_not_add_up, format!("{expected_money} {actual}");
            ExpenseError::PercentagesDoNotAddUp { total_basis_points } => percentages_do_not_add_up, format!("{total_basis_points}");
            ExpenseError::NoItems => no_items, String::new();
            ExpenseError::EmptyItemName => empty_item_name, String::new();
            ExpenseError::ItemsDoNotAddUp { expected: expected_money, actual, } => items_do_not_add_up, format!("{expected_money} {actual}");
            ExpenseError::NegativeDiscount => negative_discount, String::new();
            ExpenseError::DiscountAboveHundredPercent => discount_above_hundred_percent, String::new();
            ExpenseError::DiscountAbovePrice => discount_above_price, String::new();
            ExpenseError::NegativeTip => negative_tip, String::new();
            ExpenseError::Overflow => overflow, String::new();
        }
    }
}
