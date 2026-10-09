//! A group refuses to remove a member that entries still reference, and its
//! message counts those entries. These tests keep each count in agreement
//! with its noun, and keep every failure converting through
//! `FromExpenseGroupError` and `FromLanguageError` with what it holds and its
//! message. The messages themselves are pinned through the public API in
//! `tests/errors.rs`.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

/// A count in an error message agrees with its noun: "1 expense", never
/// "1 expenses", however many of each kind of entry there are.
#[test]
fn usage_counts_agree_with_their_nouns() {
    // Only one kind of entry is named when the other is absent.
    assert_eq!(usage(1, 0), "1 expense");
    assert_eq!(usage(3, 0), "3 expenses");
    assert_eq!(usage(0, 1), "1 transfer");
    assert_eq!(usage(0, 2), "2 transfers");
    // Both kinds, each in its own number.
    assert_eq!(usage(1, 2), "1 expense and 2 transfers");
    assert_eq!(count(1, "item", "items"), "1 item");
    assert_eq!(count(0, "item", "items"), "0 items");
}

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_language_error_converts_through_the_method_of_its_variant: LanguageError => ConvertedLanguageError through FromLanguageError::from_language_error() {
        methods {
            invalid_language() => String::new();
        }
        setup {}
        cases {
            LanguageError::InvalidLanguage => invalid_language, String::new();
        }
    }
}

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_group_error_converts_through_the_method_of_its_variant: ExpenseGroupError => ConvertedExpenseGroupError through FromExpenseGroupError::from_expense_group_error(&) {
        methods {
            unsupported_format(format: u32) => format!("{format}");
            empty_name() => String::new();
            missing_exchange_rate(currency: CurrencyCode) => format!("{currency}");
            unexpected_exchange_rate() => String::new();
            duplicate_exchange_rate(currency: CurrencyCode) => format!("{currency}");
            duplicate_member(member: MemberId) => format!("{member}");
            unknown_member(member: MemberId) => format!("{member}");
            archived_member(member: MemberId) => format!("{member}");
            member_in_use(member: MemberId, expenses: &[ExpenseId], transfers: &[TransferId]) => format!("{member} {expenses:?} {transfers:?}");
            duplicate_category(category: CategoryId) => format!("{category}");
            duplicate_category_name(existing: CategoryId) => format!("{existing}");
            unknown_category(category: CategoryId) => format!("{category}");
            built_in_category() => String::new();
            duplicate_expense(expense: ExpenseId) => format!("{expense}");
            unknown_expense(expense: ExpenseId) => format!("{expense}");
            duplicate_transfer(transfer: TransferId) => format!("{transfer}");
            unknown_transfer(transfer: TransferId) => format!("{transfer}");
            busy() => String::new();
            overflow() => String::new();
        }
        setup {
            let format = 11;
            let currency = CurrencyCode::EUR;
            let member = MemberId::new("ana").unwrap();
            let expenses = vec![ExpenseId::new("dinner").unwrap()];
            let transfers = vec![TransferId::new("repayment").unwrap()];
            let category = CategoryId::new("food").unwrap();
            let existing = CategoryId::new("drinks").unwrap();
            let expense = ExpenseId::new("lunch").unwrap();
            let transfer = TransferId::new("refund").unwrap();
        }
        cases {
            ExpenseGroupError::UnsupportedFormat { format } => unsupported_format, format!("{format}");
            ExpenseGroupError::EmptyName => empty_name, String::new();
            ExpenseGroupError::MissingExchangeRate { currency } => missing_exchange_rate, format!("{currency}");
            ExpenseGroupError::UnexpectedExchangeRate => unexpected_exchange_rate, String::new();
            ExpenseGroupError::DuplicateExchangeRate { currency } => duplicate_exchange_rate, format!("{currency}");
            ExpenseGroupError::DuplicateMember(member) => duplicate_member, format!("{member}");
            ExpenseGroupError::UnknownMember(member) => unknown_member, format!("{member}");
            ExpenseGroupError::ArchivedMember(member) => archived_member, format!("{member}");
            ExpenseGroupError::MemberInUse { member, expenses: expenses.clone(), transfers: transfers.clone(), } => member_in_use, format!("{member} {expenses:?} {transfers:?}");
            ExpenseGroupError::DuplicateCategory(category) => duplicate_category, format!("{category}");
            ExpenseGroupError::DuplicateCategoryName { existing } => duplicate_category_name, format!("{existing}");
            ExpenseGroupError::UnknownCategory(category) => unknown_category, format!("{category}");
            ExpenseGroupError::BuiltInCategory => built_in_category, String::new();
            ExpenseGroupError::DuplicateExpense(expense) => duplicate_expense, format!("{expense}");
            ExpenseGroupError::UnknownExpense(expense) => unknown_expense, format!("{expense}");
            ExpenseGroupError::DuplicateTransfer(transfer) => duplicate_transfer, format!("{transfer}");
            ExpenseGroupError::UnknownTransfer(transfer) => unknown_transfer, format!("{transfer}");
            ExpenseGroupError::Busy => busy, String::new();
            ExpenseGroupError::Overflow => overflow, String::new();
        }
    }
}
