//! The error in the shape a binding without enums that hold data reads. A
//! failure flattened to another kind or another failure, or without what it
//! holds, would show a C or C++ program another failure than the library's,
//! so every failure of every module is flattened here, with every reason of
//! a refused IBAN.

use std::collections::HashSet;

use super::*;

/// The message every error of the table holds.
const DETAIL: &str = "the library's message";

/// The category the failures that name one hold.
const CATEGORY: &str = "0192f1c3-0000-7000-8000-000000000003";

/// The expense the failures that name one hold.
const EXPENSE: &str = "0192f1c3-0000-7000-8000-000000000004";

/// The member the failures that name one hold.
const MEMBER: &str = "0192f1c3-0000-7000-8000-000000000001";

/// The transfer the failures that name one hold.
const TRANSFER: &str = "0192f1c3-0000-7000-8000-000000000005";

/// An error beside the flat form it must take.
struct Flattening {
    error: EranosError,
    flat: FlatError,
}

/// Every failure of every module, every reason of a refused IBAN, and the
/// binding's own cases.
#[expect(
    clippy::too_many_lines,
    reason = "one entry for each of the 65 errors, which read best as one table"
)]
fn flattenings() -> Vec<Flattening> {
    vec![
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::EmptyName,
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountEmptyName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::EmptyUsername,
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountEmptyUsername),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::InvalidCharacter,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidCharacter),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::InvalidChecksum,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidChecksum),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::InvalidFormat,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidFormat),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::InvalidStructure,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidStructure),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::UnknownCountry,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    reason: Some(IbanReasonKind::UnknownCountry),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Account {
                detail: DETAIL.to_owned(),
                failure: AccountFailure::InvalidIban {
                    reason: IbanReason::WrongLength {
                        expected: 24,
                        found: 22,
                    },
                },
            },
            flat: FlatError {
                kind: ErrorKind::Account,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::AccountInvalidIban),
                payload: Payload {
                    expected_length: Some(24),
                    found_length: Some(22),
                    reason: Some(IbanReasonKind::WrongLength),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Calendar {
                detail: DETAIL.to_owned(),
                failure: CalendarFailure::InvalidDate,
            },
            flat: FlatError {
                kind: ErrorKind::Calendar,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::CalendarInvalidDate),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Calendar {
                detail: DETAIL.to_owned(),
                failure: CalendarFailure::InvalidTime,
            },
            flat: FlatError {
                kind: ErrorKind::Calendar,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::CalendarInvalidTime),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Category {
                detail: DETAIL.to_owned(),
                failure: CategoryFailure::EmptyName,
            },
            flat: FlatError {
                kind: ErrorKind::Category,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::CategoryEmptyName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Contact {
                detail: DETAIL.to_owned(),
                failure: ContactFailure::InvalidEmail,
            },
            flat: FlatError {
                kind: ErrorKind::Contact,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ContactInvalidEmail),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Contact {
                detail: DETAIL.to_owned(),
                failure: ContactFailure::InvalidPhoneNumber,
            },
            flat: FlatError {
                kind: ErrorKind::Contact,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ContactInvalidPhoneNumber),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::AmountsDoNotAddUp {
                    actual: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 900,
                    },
                    expected: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 1000,
                    },
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseAmountsDoNotAddUp),
                payload: Payload {
                    actual: Some(Money {
                        currency: "EUR".to_owned(),
                        minor_units: 900,
                    }),
                    expected_amount: Some(Money {
                        currency: "EUR".to_owned(),
                        minor_units: 1000,
                    }),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::CurrencyMismatch {
                    expected: "EUR".to_owned(),
                    found: "RON".to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseCurrencyMismatch),
                payload: Payload {
                    expected_currency: Some("EUR".to_owned()),
                    found_currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::DuplicateMember {
                    member: MEMBER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseDuplicateMember),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::EmptyItemName,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseEmptyItemName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::EmptyName,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseEmptyName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::EmptySplit,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseEmptySplit),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::ItemsDoNotAddUp {
                    actual: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 900,
                    },
                    expected: Money {
                        currency: "EUR".to_owned(),
                        minor_units: 1000,
                    },
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseItemsDoNotAddUp),
                payload: Payload {
                    actual: Some(Money {
                        currency: "EUR".to_owned(),
                        minor_units: 900,
                    }),
                    expected_amount: Some(Money {
                        currency: "EUR".to_owned(),
                        minor_units: 1000,
                    }),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::NegativeTip,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseNegativeTip),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::NoItems,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseNoItems),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::NonPositiveAmount,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseNonPositiveAmount),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::NonPositiveShare {
                    member: MEMBER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseNonPositiveShare),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::Overflow,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseOverflow),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::PercentagesDoNotAddUp {
                    total_basis_points: 9_900,
                },
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpensePercentagesDoNotAddUp),
                payload: Payload {
                    total_basis_points: Some(9_900),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                detail: DETAIL.to_owned(),
                failure: ExpenseFailure::ZeroQuantity,
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::ExpenseZeroQuantity),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::ArchivedMember {
                    member: MEMBER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupArchivedMember),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::BuiltInCategory,
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupBuiltInCategory),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::Busy,
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupBusy),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateCategory {
                    category: CATEGORY.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateCategory),
                payload: Payload {
                    category: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateCategoryName {
                    existing: CATEGORY.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateCategoryName),
                payload: Payload {
                    existing: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateExchangeRate {
                    currency: "RON".to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateExchangeRate),
                payload: Payload {
                    currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateExpense {
                    expense: EXPENSE.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateExpense),
                payload: Payload {
                    expense: Some(EXPENSE.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateMember {
                    member: MEMBER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateMember),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::DuplicateTransfer {
                    transfer: TRANSFER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupDuplicateTransfer),
                payload: Payload {
                    transfer: Some(TRANSFER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::EmptyName,
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupEmptyName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::MemberInUse {
                    expenses: vec![EXPENSE.to_owned()],
                    member: MEMBER.to_owned(),
                    transfers: vec![TRANSFER.to_owned()],
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupMemberInUse),
                payload: Payload {
                    expenses: Some(vec![EXPENSE.to_owned()]),
                    member: Some(MEMBER.to_owned()),
                    transfers: Some(vec![TRANSFER.to_owned()]),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::MissingExchangeRate {
                    currency: "RON".to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupMissingExchangeRate),
                payload: Payload {
                    currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::Overflow,
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupOverflow),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnexpectedExchangeRate,
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnexpectedExchangeRate),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnknownCategory {
                    category: CATEGORY.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnknownCategory),
                payload: Payload {
                    category: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnknownExpense {
                    expense: EXPENSE.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnknownExpense),
                payload: Payload {
                    expense: Some(EXPENSE.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnknownMember {
                    member: MEMBER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnknownMember),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnknownTransfer {
                    transfer: TRANSFER.to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnknownTransfer),
                payload: Payload {
                    transfer: Some(TRANSFER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Group {
                detail: DETAIL.to_owned(),
                failure: GroupFailure::UnsupportedFormat { format: 3 },
            },
            flat: FlatError {
                kind: ErrorKind::Group,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::GroupUnsupportedFormat),
                payload: Payload {
                    format: Some(3),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Id {
                detail: DETAIL.to_owned(),
                failure: IdFailure::InvalidId,
            },
            flat: FlatError {
                kind: ErrorKind::Id,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::IdInvalidId),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Language {
                detail: DETAIL.to_owned(),
                failure: LanguageFailure::InvalidLanguage,
            },
            flat: FlatError {
                kind: ErrorKind::Language,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::LanguageInvalidLanguage),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Member {
                detail: DETAIL.to_owned(),
                failure: MemberFailure::EmptyFirstName,
            },
            flat: FlatError {
                kind: ErrorKind::Member,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MemberEmptyFirstName),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::CurrencyMismatch {
                    left: "EUR".to_owned(),
                    right: "RON".to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyCurrencyMismatch),
                payload: Payload {
                    left: Some("EUR".to_owned()),
                    right: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::InvalidAmount,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyInvalidAmount),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::InvalidExchangeRate,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyInvalidExchangeRate),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::InvalidParts,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyInvalidParts),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::InvalidPercentage,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyInvalidPercentage),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::Overflow,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyOverflow),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::TooManyDecimals {
                    allowed: 2,
                    currency: "EUR".to_owned(),
                },
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyTooManyDecimals),
                payload: Payload {
                    allowed: Some(2),
                    currency: Some("EUR".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Money {
                detail: DETAIL.to_owned(),
                failure: MoneyFailure::UnknownCurrency,
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::MoneyUnknownCurrency),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Settlement {
                detail: DETAIL.to_owned(),
                failure: SettlementFailure::OutOfMemory { balances: 64 },
            },
            flat: FlatError {
                kind: ErrorKind::Settlement,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::SettlementOutOfMemory),
                payload: Payload {
                    balances: Some(64),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Transfer {
                detail: DETAIL.to_owned(),
                failure: TransferFailure::NonPositiveAmount,
            },
            flat: FlatError {
                kind: ErrorKind::Transfer,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::TransferNonPositiveAmount),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Transfer {
                detail: DETAIL.to_owned(),
                failure: TransferFailure::SameMember,
            },
            flat: FlatError {
                kind: ErrorKind::Transfer,
                message: DETAIL.to_owned(),
                failure: Some(FailureKind::TransferSameMember),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Cancelled {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Cancelled,
                message: DETAIL.to_owned(),
                failure: None,
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::IdentifierGiven {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::IdentifierGiven,
                message: DETAIL.to_owned(),
                failure: None,
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::IdentifierMissing {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::IdentifierMissing,
                message: DETAIL.to_owned(),
                failure: None,
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::InvalidJson {
                column: 7,
                detail: DETAIL.to_owned(),
                line: 2,
            },
            flat: FlatError {
                kind: ErrorKind::InvalidJson,
                message: DETAIL.to_owned(),
                failure: None,
                payload: Payload {
                    column: Some(7),
                    line: Some(2),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Released {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Released,
                message: DETAIL.to_owned(),
                failure: None,
                payload: Payload::default(),
            },
        },
    ]
}

/// A failure flattened to another kind or another failure, or without a
/// field it holds, would show a C or C++ program another failure than the
/// library's; every failure takes a flat failure of its own, and every
/// reason of a refused IBAN a kind of its own.
#[test]
fn every_error_flattens_to_its_kind_its_failure_and_what_it_holds() {
    let flattenings = flattenings();
    assert_eq!(flattenings.len(), 65);
    for Flattening { error, flat } in &flattenings {
        assert_eq!(&FlatError::from(error), flat);
    }
    let kinds: HashSet<ErrorKind> = flattenings
        .iter()
        .map(|flattening| flattening.flat.kind)
        .collect();
    let failures: HashSet<FailureKind> = flattenings
        .iter()
        .filter_map(|flattening| flattening.flat.failure)
        .collect();
    let reasons: HashSet<IbanReasonKind> = flattenings
        .iter()
        .filter_map(|flattening| flattening.flat.payload.reason)
        .collect();
    assert_eq!((kinds.len(), failures.len(), reasons.len()), (17, 55, 6));
}

/// Text that is not UTF-8 would read as a failure of the library, whose
/// rules never saw it; it is a kind of its own, with no failure and nothing
/// it holds.
#[test]
fn text_that_is_not_utf8_is_a_kind_of_its_own() {
    assert_eq!(
        FlatError::invalid_text(),
        FlatError {
            kind: ErrorKind::InvalidText,
            message: "the text is not UTF-8".to_owned(),
            failure: None,
            payload: Payload::default(),
        }
    );
}
