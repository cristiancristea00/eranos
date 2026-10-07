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
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::EmptyName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountEmptyName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::EmptyUsername,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountEmptyUsername),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::InvalidCharacter,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidCharacter),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::InvalidChecksum,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidChecksum),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::InvalidFormat,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidFormat),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::InvalidStructure,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
                payload: Payload {
                    reason: Some(IbanReasonKind::InvalidStructure),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::UnknownCountry,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
                payload: Payload {
                    reason: Some(IbanReasonKind::UnknownCountry),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::PaymentAccount {
                failure: PaymentAccountFailure::InvalidIban {
                    reason: IbanReason::WrongLength {
                        expected: 24,
                        found: 22,
                    },
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::PaymentAccount,
                failure: Some(FailureKind::PaymentAccountInvalidIban),
                message: DETAIL.to_owned(),
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
                failure: CalendarFailure::InvalidDate,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Calendar,
                failure: Some(FailureKind::CalendarInvalidDate),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Calendar {
                failure: CalendarFailure::InvalidTime,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Calendar,
                failure: Some(FailureKind::CalendarInvalidTime),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Category {
                failure: CategoryFailure::EmptyName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Category,
                failure: Some(FailureKind::CategoryEmptyName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Contact {
                failure: ContactFailure::InvalidEmail,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Contact,
                failure: Some(FailureKind::ContactInvalidEmail),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Contact {
                failure: ContactFailure::InvalidPhoneNumber,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Contact,
                failure: Some(FailureKind::ContactInvalidPhoneNumber),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::AmountsDoNotAddUp {
                    expected: Money {
                        minor_units: 1000,
                        currency: "EUR".to_owned(),
                    },
                    actual: Money {
                        minor_units: 900,
                        currency: "EUR".to_owned(),
                    },
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseAmountsDoNotAddUp),
                message: DETAIL.to_owned(),
                payload: Payload {
                    actual: Some(Money {
                        minor_units: 900,
                        currency: "EUR".to_owned(),
                    }),
                    expected_amount: Some(Money {
                        minor_units: 1000,
                        currency: "EUR".to_owned(),
                    }),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::CurrencyMismatch {
                    expected: "EUR".to_owned(),
                    found: "RON".to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseCurrencyMismatch),
                message: DETAIL.to_owned(),
                payload: Payload {
                    expected_currency: Some("EUR".to_owned()),
                    found_currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::DuplicateMember {
                    member: MEMBER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseDuplicateMember),
                message: DETAIL.to_owned(),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::EmptyItemName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseEmptyItemName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::EmptyName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseEmptyName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::EmptySplit,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseEmptySplit),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::ItemsDoNotAddUp {
                    expected: Money {
                        minor_units: 1000,
                        currency: "EUR".to_owned(),
                    },
                    actual: Money {
                        minor_units: 900,
                        currency: "EUR".to_owned(),
                    },
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseItemsDoNotAddUp),
                message: DETAIL.to_owned(),
                payload: Payload {
                    actual: Some(Money {
                        minor_units: 900,
                        currency: "EUR".to_owned(),
                    }),
                    expected_amount: Some(Money {
                        minor_units: 1000,
                        currency: "EUR".to_owned(),
                    }),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::NegativeTip,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseNegativeTip),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::NoItems,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseNoItems),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::NonPositiveAmount,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseNonPositiveAmount),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::NonPositiveShare {
                    member: MEMBER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseNonPositiveShare),
                message: DETAIL.to_owned(),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::Overflow,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseOverflow),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::PercentagesDoNotAddUp {
                    total_basis_points: 9_900,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpensePercentagesDoNotAddUp),
                message: DETAIL.to_owned(),
                payload: Payload {
                    total_basis_points: Some(9_900),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Expense {
                failure: ExpenseFailure::ZeroQuantity,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Expense,
                failure: Some(FailureKind::ExpenseZeroQuantity),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::ArchivedMember {
                    member: MEMBER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupArchivedMember),
                message: DETAIL.to_owned(),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::BuiltInCategory,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupBuiltInCategory),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::Busy,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupBusy),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateCategory {
                    category: CATEGORY.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateCategory),
                message: DETAIL.to_owned(),
                payload: Payload {
                    category: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateCategoryName {
                    existing: CATEGORY.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateCategoryName),
                message: DETAIL.to_owned(),
                payload: Payload {
                    existing: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateExchangeRate {
                    currency: "RON".to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateExchangeRate),
                message: DETAIL.to_owned(),
                payload: Payload {
                    currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateExpense {
                    expense: EXPENSE.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateExpense),
                message: DETAIL.to_owned(),
                payload: Payload {
                    expense: Some(EXPENSE.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateMember {
                    member: MEMBER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateMember),
                message: DETAIL.to_owned(),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::DuplicateTransfer {
                    transfer: TRANSFER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupDuplicateTransfer),
                message: DETAIL.to_owned(),
                payload: Payload {
                    transfer: Some(TRANSFER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::EmptyName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupEmptyName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::MemberInUse {
                    member: MEMBER.to_owned(),
                    expenses: vec![EXPENSE.to_owned()],
                    transfers: vec![TRANSFER.to_owned()],
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupMemberInUse),
                message: DETAIL.to_owned(),
                payload: Payload {
                    expenses: Some(vec![EXPENSE.to_owned()]),
                    member: Some(MEMBER.to_owned()),
                    transfers: Some(vec![TRANSFER.to_owned()]),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::MissingExchangeRate {
                    currency: "RON".to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupMissingExchangeRate),
                message: DETAIL.to_owned(),
                payload: Payload {
                    currency: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::Overflow,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupOverflow),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnexpectedExchangeRate,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnexpectedExchangeRate),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnknownCategory {
                    category: CATEGORY.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnknownCategory),
                message: DETAIL.to_owned(),
                payload: Payload {
                    category: Some(CATEGORY.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnknownExpense {
                    expense: EXPENSE.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnknownExpense),
                message: DETAIL.to_owned(),
                payload: Payload {
                    expense: Some(EXPENSE.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnknownMember {
                    member: MEMBER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnknownMember),
                message: DETAIL.to_owned(),
                payload: Payload {
                    member: Some(MEMBER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnknownTransfer {
                    transfer: TRANSFER.to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnknownTransfer),
                message: DETAIL.to_owned(),
                payload: Payload {
                    transfer: Some(TRANSFER.to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::ExpenseGroup {
                failure: ExpenseGroupFailure::UnsupportedFormat { format: 3 },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::ExpenseGroup,
                failure: Some(FailureKind::ExpenseGroupUnsupportedFormat),
                message: DETAIL.to_owned(),
                payload: Payload {
                    format: Some(3),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Id {
                failure: IdFailure::InvalidId,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Id,
                failure: Some(FailureKind::IdInvalidId),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Language {
                failure: LanguageFailure::InvalidLanguage,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Language,
                failure: Some(FailureKind::LanguageInvalidLanguage),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Member {
                failure: MemberFailure::EmptyFirstName,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Member,
                failure: Some(FailureKind::MemberEmptyFirstName),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::CurrencyMismatch {
                    left: "EUR".to_owned(),
                    right: "RON".to_owned(),
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyCurrencyMismatch),
                message: DETAIL.to_owned(),
                payload: Payload {
                    left: Some("EUR".to_owned()),
                    right: Some("RON".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::InvalidAmount,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyInvalidAmount),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::InvalidExchangeRate,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyInvalidExchangeRate),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::InvalidParts,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyInvalidParts),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::InvalidPercentage,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyInvalidPercentage),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::Overflow,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyOverflow),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::TooManyDecimals {
                    currency: "EUR".to_owned(),
                    allowed: 2,
                },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyTooManyDecimals),
                message: DETAIL.to_owned(),
                payload: Payload {
                    allowed: Some(2),
                    currency: Some("EUR".to_owned()),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Money {
                failure: MoneyFailure::UnknownCurrency,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Money,
                failure: Some(FailureKind::MoneyUnknownCurrency),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Settlement {
                failure: SettlementFailure::OutOfMemory { balances: 64 },
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Settlement,
                failure: Some(FailureKind::SettlementOutOfMemory),
                message: DETAIL.to_owned(),
                payload: Payload {
                    balances: Some(64),
                    ..Payload::default()
                },
            },
        },
        Flattening {
            error: EranosError::Transfer {
                failure: TransferFailure::NonPositiveAmount,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Transfer,
                failure: Some(FailureKind::TransferNonPositiveAmount),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Transfer {
                failure: TransferFailure::SameMember,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Transfer,
                failure: Some(FailureKind::TransferSameMember),
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::Cancelled {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::Cancelled,
                failure: None,
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::IdentifierGiven {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::IdentifierGiven,
                failure: None,
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::IdentifierMissing {
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::IdentifierMissing,
                failure: None,
                message: DETAIL.to_owned(),
                payload: Payload::default(),
            },
        },
        Flattening {
            error: EranosError::InvalidJson {
                line: 2,
                column: 7,
                detail: DETAIL.to_owned(),
            },
            flat: FlatError {
                kind: ErrorKind::InvalidJson,
                failure: None,
                message: DETAIL.to_owned(),
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
                failure: None,
                message: DETAIL.to_owned(),
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
    assert_eq!(kinds.len(), 17);
    assert_eq!(failures.len(), 55);
    assert_eq!(reasons.len(), 6);
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
            failure: None,
            message: "the text is not UTF-8".to_owned(),
            payload: Payload::default(),
        }
    );
}
