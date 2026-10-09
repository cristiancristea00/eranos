//! The error in the shape a binding without enums that hold data reads:
//! the kind of error, the failure within its module, the library's message,
//! and what the failure holds, each in a field of its own.

use crate::{
    CalendarFailure, CategoryFailure, ContactFailure, CurrencyDetails, EranosError, ExpenseFailure,
    ExpenseGroupFailure, IbanReason, IdFailure, LanguageFailure, MemberFailure, MismatchedField,
    Money, MoneyFailure, PaymentAccountFailure, SettlementFailure, TransferFailure,
};

/// An [`EranosError`] as fields: its kind, the library's message, the
/// failure within its module, and what the failure holds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatError {
    /// The module that refused the call, or the binding's own case.
    pub kind: ErrorKind,
    /// The failure within its module, or the field a mismatched record gave
    /// another way; absent for the binding's other own cases.
    pub failure: Option<FailureKind>,
    /// The library's message: lowercase, without a full stop, and never the
    /// text it refused.
    pub message: String,
    /// What the failure holds, each field present for the failures that
    /// hold it.
    pub payload: Payload,
}

impl FlatError {
    /// The error of text that is not UTF-8, which a binding that takes text
    /// as bytes refuses before the library sees it.
    #[must_use]
    pub fn invalid_text() -> Self {
        Self {
            kind: ErrorKind::InvalidText,
            failure: None,
            message: "the text is not UTF-8".to_owned(),
            payload: Payload::default(),
        }
    }
}

impl From<&EranosError> for FlatError {
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per case of the error, of which there are 16"
    )]
    fn from(error: &EranosError) -> Self {
        let message = error.to_string();
        match error {
            EranosError::Id { failure, .. } => Self {
                kind: ErrorKind::Id,
                failure: Some(id(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::PaymentAccount { failure, .. } => {
                let FlatFailure { failure, payload } = payment_account(*failure);
                Self {
                    kind: ErrorKind::PaymentAccount,
                    failure: Some(failure),
                    message,
                    payload,
                }
            }
            EranosError::Calendar { failure, .. } => Self {
                kind: ErrorKind::Calendar,
                failure: Some(calendar(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::Category { failure, .. } => Self {
                kind: ErrorKind::Category,
                failure: Some(category(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::Contact { failure, .. } => Self {
                kind: ErrorKind::Contact,
                failure: Some(contact(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::Money { failure, .. } => {
                let FlatFailure { failure, payload } = money(failure);
                Self {
                    kind: ErrorKind::Money,
                    failure: Some(failure),
                    message,
                    payload,
                }
            }
            EranosError::Member { failure, .. } => Self {
                kind: ErrorKind::Member,
                failure: Some(member(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::Expense { failure, .. } => {
                let FlatFailure { failure, payload } = expense(failure);
                Self {
                    kind: ErrorKind::Expense,
                    failure: Some(failure),
                    message,
                    payload,
                }
            }
            EranosError::Settlement { failure, .. } => {
                let FlatFailure { failure, payload } = settlement(*failure);
                Self {
                    kind: ErrorKind::Settlement,
                    failure: Some(failure),
                    message,
                    payload,
                }
            }
            EranosError::Transfer { failure, .. } => Self {
                kind: ErrorKind::Transfer,
                failure: Some(transfer(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::ExpenseGroup { failure, .. } => {
                let FlatFailure { failure, payload } = expense_group(failure);
                Self {
                    kind: ErrorKind::ExpenseGroup,
                    failure: Some(failure),
                    message,
                    payload,
                }
            }
            EranosError::Language { failure, .. } => Self {
                kind: ErrorKind::Language,
                failure: Some(language(*failure)),
                message,
                payload: Payload::default(),
            },
            EranosError::MismatchedRecord { field, .. } => Self {
                kind: ErrorKind::MismatchedRecord,
                failure: Some(mismatched_record(*field)),
                message,
                payload: Payload::default(),
            },
            EranosError::InvalidJson { line, column, .. } => Self {
                kind: ErrorKind::InvalidJson,
                failure: None,
                message,
                payload: Payload {
                    column: Some(*column),
                    line: Some(*line),
                    ..Payload::default()
                },
            },
            EranosError::Cancelled { .. } => Self {
                kind: ErrorKind::Cancelled,
                failure: None,
                message,
                payload: Payload::default(),
            },
            EranosError::Released { .. } => Self {
                kind: ErrorKind::Released,
                failure: None,
                message,
                payload: Payload::default(),
            },
        }
    }
}

/// The module of the library that refused a call, or the binding's own
/// case: a case per case of [`EranosError`], and [`ErrorKind::InvalidText`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// [`EranosError::Id`].
    Id,
    /// [`EranosError::PaymentAccount`].
    PaymentAccount,
    /// [`EranosError::Calendar`].
    Calendar,
    /// [`EranosError::Category`].
    Category,
    /// [`EranosError::Contact`].
    Contact,
    /// [`EranosError::Money`].
    Money,
    /// [`EranosError::Member`].
    Member,
    /// [`EranosError::Expense`].
    Expense,
    /// [`EranosError::Settlement`].
    Settlement,
    /// [`EranosError::Transfer`].
    Transfer,
    /// [`EranosError::ExpenseGroup`].
    ExpenseGroup,
    /// [`EranosError::Language`].
    Language,
    /// [`EranosError::MismatchedRecord`].
    MismatchedRecord,
    /// [`EranosError::InvalidJson`].
    InvalidJson,
    /// Text that is not UTF-8, which a binding that takes text as bytes
    /// refuses before the library sees it.
    InvalidText,
    /// [`EranosError::Cancelled`].
    Cancelled,
    /// [`EranosError::Released`].
    Released,
}

/// The failure within its module, named after the module and the failure:
/// a case per case of every module's failure, and one per field of
/// [`MismatchedField`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureKind {
    /// [`IdFailure::Empty`].
    IdEmpty,
    /// [`IdFailure::TooLong`].
    IdTooLong,
    /// [`IdFailure::ControlCharacter`].
    IdControlCharacter,
    /// [`IdFailure::SurroundingWhitespace`].
    IdSurroundingWhitespace,
    /// [`PaymentAccountFailure::EmptyName`].
    PaymentAccountEmptyName,
    /// [`PaymentAccountFailure::InvalidIban`].
    PaymentAccountInvalidIban,
    /// [`PaymentAccountFailure::EmptyUsername`].
    PaymentAccountEmptyUsername,
    /// [`CalendarFailure::InvalidDate`].
    CalendarInvalidDate,
    /// [`CalendarFailure::InvalidTime`].
    CalendarInvalidTime,
    /// [`CategoryFailure::EmptyName`].
    CategoryEmptyName,
    /// [`ContactFailure::InvalidEmail`].
    ContactInvalidEmail,
    /// [`ContactFailure::InvalidPhoneNumber`].
    ContactInvalidPhoneNumber,
    /// [`MoneyFailure::InvalidAmount`].
    MoneyInvalidAmount,
    /// [`MoneyFailure::TooManyDecimals`].
    MoneyTooManyDecimals,
    /// [`MoneyFailure::CurrencyMismatch`].
    MoneyCurrencyMismatch,
    /// [`MoneyFailure::UnknownCurrency`].
    MoneyUnknownCurrency,
    /// [`MoneyFailure::InvalidExchangeRate`].
    MoneyInvalidExchangeRate,
    /// [`MoneyFailure::InvalidParts`].
    MoneyInvalidParts,
    /// [`MoneyFailure::InvalidPercentage`].
    MoneyInvalidPercentage,
    /// [`MoneyFailure::InvalidQuantity`].
    MoneyInvalidQuantity,
    /// [`MoneyFailure::NotFinite`].
    MoneyNotFinite,
    /// [`MoneyFailure::Overflow`].
    MoneyOverflow,
    /// [`MemberFailure::EmptyFirstName`].
    MemberEmptyFirstName,
    /// [`ExpenseFailure::EmptyName`].
    ExpenseEmptyName,
    /// [`ExpenseFailure::NonPositiveAmount`].
    ExpenseNonPositiveAmount,
    /// [`ExpenseFailure::CurrencyMismatch`].
    ExpenseCurrencyMismatch,
    /// [`ExpenseFailure::EmptySplit`].
    ExpenseEmptySplit,
    /// [`ExpenseFailure::DuplicateMember`].
    ExpenseDuplicateMember,
    /// [`ExpenseFailure::NonPositiveShare`].
    ExpenseNonPositiveShare,
    /// [`ExpenseFailure::AmountsDoNotAddUp`].
    ExpenseAmountsDoNotAddUp,
    /// [`ExpenseFailure::PercentagesDoNotAddUp`].
    ExpensePercentagesDoNotAddUp,
    /// [`ExpenseFailure::NoItems`].
    ExpenseNoItems,
    /// [`ExpenseFailure::EmptyItemName`].
    ExpenseEmptyItemName,
    /// [`ExpenseFailure::ItemsDoNotAddUp`].
    ExpenseItemsDoNotAddUp,
    /// [`ExpenseFailure::NegativeDiscount`].
    ExpenseNegativeDiscount,
    /// [`ExpenseFailure::DiscountAboveHundredPercent`].
    ExpenseDiscountAboveHundredPercent,
    /// [`ExpenseFailure::DiscountAbovePrice`].
    ExpenseDiscountAbovePrice,
    /// [`ExpenseFailure::NegativeTip`].
    ExpenseNegativeTip,
    /// [`ExpenseFailure::Overflow`].
    ExpenseOverflow,
    /// [`SettlementFailure::OutOfMemory`].
    SettlementOutOfMemory,
    /// [`TransferFailure::SameMember`].
    TransferSameMember,
    /// [`TransferFailure::NonPositiveAmount`].
    TransferNonPositiveAmount,
    /// [`ExpenseGroupFailure::UnsupportedFormat`].
    ExpenseGroupUnsupportedFormat,
    /// [`ExpenseGroupFailure::EmptyName`].
    ExpenseGroupEmptyName,
    /// [`ExpenseGroupFailure::MissingExchangeRate`].
    ExpenseGroupMissingExchangeRate,
    /// [`ExpenseGroupFailure::UnexpectedExchangeRate`].
    ExpenseGroupUnexpectedExchangeRate,
    /// [`ExpenseGroupFailure::DuplicateExchangeRate`].
    ExpenseGroupDuplicateExchangeRate,
    /// [`ExpenseGroupFailure::DuplicateMember`].
    ExpenseGroupDuplicateMember,
    /// [`ExpenseGroupFailure::UnknownMember`].
    ExpenseGroupUnknownMember,
    /// [`ExpenseGroupFailure::ArchivedMember`].
    ExpenseGroupArchivedMember,
    /// [`ExpenseGroupFailure::MemberInUse`].
    ExpenseGroupMemberInUse,
    /// [`ExpenseGroupFailure::DuplicateCategory`].
    ExpenseGroupDuplicateCategory,
    /// [`ExpenseGroupFailure::DuplicateCategoryName`].
    ExpenseGroupDuplicateCategoryName,
    /// [`ExpenseGroupFailure::UnknownCategory`].
    ExpenseGroupUnknownCategory,
    /// [`ExpenseGroupFailure::BuiltInCategory`].
    ExpenseGroupBuiltInCategory,
    /// [`ExpenseGroupFailure::DuplicateExpense`].
    ExpenseGroupDuplicateExpense,
    /// [`ExpenseGroupFailure::UnknownExpense`].
    ExpenseGroupUnknownExpense,
    /// [`ExpenseGroupFailure::DuplicateTransfer`].
    ExpenseGroupDuplicateTransfer,
    /// [`ExpenseGroupFailure::UnknownTransfer`].
    ExpenseGroupUnknownTransfer,
    /// [`ExpenseGroupFailure::Busy`].
    ExpenseGroupBusy,
    /// [`ExpenseGroupFailure::Overflow`].
    ExpenseGroupOverflow,
    /// [`LanguageFailure::InvalidLanguage`].
    LanguageInvalidLanguage,
    /// [`MismatchedField::IbanCountryCode`].
    MismatchedRecordIbanCountryCode,
    /// [`MismatchedField::PhoneInternational`].
    MismatchedRecordPhoneInternational,
    /// [`MismatchedField::CurrencyDetails`].
    MismatchedRecordCurrencyDetails,
}

/// Why an IBAN was refused, without what [`IbanReason::WrongLength`] holds,
/// which [`Payload`] carries: a case per case of [`IbanReason`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IbanReasonKind {
    /// [`IbanReason::InvalidCharacter`].
    InvalidCharacter,
    /// [`IbanReason::InvalidFormat`].
    InvalidFormat,
    /// [`IbanReason::UnknownCountry`].
    UnknownCountry,
    /// [`IbanReason::WrongLength`].
    WrongLength,
    /// [`IbanReason::InvalidStructure`].
    InvalidStructure,
    /// [`IbanReason::InvalidChecksum`].
    InvalidChecksum,
}

/// What a failure holds, each field present for the failures that hold it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Payload {
    /// What the amounts add up to: [`ExpenseFailure::AmountsDoNotAddUp`],
    /// [`ExpenseFailure::ItemsDoNotAddUp`].
    pub actual: Option<Money>,
    /// How many decimal places the currency has:
    /// [`MoneyFailure::TooManyDecimals`].
    pub allowed: Option<i64>,
    /// How many open balances the plan has:
    /// [`SettlementFailure::OutOfMemory`].
    pub balances: Option<i64>,
    /// The category, as its identifier: [`ExpenseGroupFailure::DuplicateCategory`],
    /// [`ExpenseGroupFailure::UnknownCategory`].
    pub category: Option<String>,
    /// The column where the saved group stops being one, from 1:
    /// [`EranosError::InvalidJson`].
    pub column: Option<i64>,
    /// The currency: [`ExpenseGroupFailure::DuplicateExchangeRate`],
    /// [`ExpenseGroupFailure::MissingExchangeRate`],
    /// [`MoneyFailure::TooManyDecimals`].
    pub currency: Option<CurrencyDetails>,
    /// The category that has the name, as its identifier:
    /// [`ExpenseGroupFailure::DuplicateCategoryName`].
    pub existing: Option<String>,
    /// What the amounts must add up to:
    /// [`ExpenseFailure::AmountsDoNotAddUp`],
    /// [`ExpenseFailure::ItemsDoNotAddUp`].
    pub expected_amount: Option<Money>,
    /// The currency the entry must be in:
    /// [`ExpenseFailure::CurrencyMismatch`].
    pub expected_currency: Option<CurrencyDetails>,
    /// How many characters the country's IBANs have:
    /// [`IbanReason::WrongLength`].
    pub expected_length: Option<i64>,
    /// The expense, as its identifier: [`ExpenseGroupFailure::DuplicateExpense`],
    /// [`ExpenseGroupFailure::UnknownExpense`].
    pub expense: Option<String>,
    /// The expenses that name the member: [`ExpenseGroupFailure::MemberInUse`].
    pub expenses: Option<Vec<String>>,
    /// The format the saved group names:
    /// [`ExpenseGroupFailure::UnsupportedFormat`].
    pub format: Option<i64>,
    /// The currency the part is in: [`ExpenseFailure::CurrencyMismatch`].
    pub found_currency: Option<CurrencyDetails>,
    /// How many characters the IBAN has: [`IbanReason::WrongLength`].
    pub found_length: Option<i64>,
    /// The currency of the first amount: [`MoneyFailure::CurrencyMismatch`].
    pub left: Option<CurrencyDetails>,
    /// The line where the saved group stops being one, from 1:
    /// [`EranosError::InvalidJson`].
    pub line: Option<i64>,
    /// The member, as their identifier: [`ExpenseFailure::DuplicateMember`],
    /// [`ExpenseFailure::NonPositiveShare`],
    /// [`ExpenseGroupFailure::ArchivedMember`], [`ExpenseGroupFailure::DuplicateMember`],
    /// [`ExpenseGroupFailure::MemberInUse`], [`ExpenseGroupFailure::UnknownMember`].
    pub member: Option<String>,
    /// The check the IBAN failed: [`PaymentAccountFailure::InvalidIban`].
    pub reason: Option<IbanReasonKind>,
    /// The currency of the second amount:
    /// [`MoneyFailure::CurrencyMismatch`].
    pub right: Option<CurrencyDetails>,
    /// What the percentages add up to, in basis points:
    /// [`ExpenseFailure::PercentagesDoNotAddUp`].
    pub total_basis_points: Option<i64>,
    /// The transfer, as its identifier: [`ExpenseGroupFailure::DuplicateTransfer`],
    /// [`ExpenseGroupFailure::UnknownTransfer`].
    pub transfer: Option<String>,
    /// The transfers that name the member: [`ExpenseGroupFailure::MemberInUse`].
    pub transfers: Option<Vec<String>>,
}

/// A module's failure as a [`FlatError`] holds it: the failure, and what it
/// holds.
struct FlatFailure {
    failure: FailureKind,
    payload: Payload,
}

impl FlatFailure {
    /// Returns `failure`, which holds nothing.
    fn new(failure: FailureKind) -> Self {
        Self {
            failure,
            payload: Payload::default(),
        }
    }
}

/// The account module's `failure`, with what it holds.
fn payment_account(failure: PaymentAccountFailure) -> FlatFailure {
    match failure {
        PaymentAccountFailure::EmptyName => FlatFailure::new(FailureKind::PaymentAccountEmptyName),
        PaymentAccountFailure::InvalidIban { reason } => FlatFailure {
            failure: FailureKind::PaymentAccountInvalidIban,
            payload: iban(reason),
        },
        PaymentAccountFailure::EmptyUsername => {
            FlatFailure::new(FailureKind::PaymentAccountEmptyUsername)
        }
    }
}

/// The calendar module's `failure`.
const fn calendar(failure: CalendarFailure) -> FailureKind {
    match failure {
        CalendarFailure::InvalidDate => FailureKind::CalendarInvalidDate,
        CalendarFailure::InvalidTime => FailureKind::CalendarInvalidTime,
    }
}

/// The category module's `failure`.
const fn category(failure: CategoryFailure) -> FailureKind {
    match failure {
        CategoryFailure::EmptyName => FailureKind::CategoryEmptyName,
    }
}

/// The contact module's `failure`.
const fn contact(failure: ContactFailure) -> FailureKind {
    match failure {
        ContactFailure::InvalidEmail => FailureKind::ContactInvalidEmail,
        ContactFailure::InvalidPhoneNumber => FailureKind::ContactInvalidPhoneNumber,
    }
}

/// The expense module's `failure`, with what it holds.
fn expense(failure: &ExpenseFailure) -> FlatFailure {
    match failure {
        ExpenseFailure::EmptyName => FlatFailure::new(FailureKind::ExpenseEmptyName),
        ExpenseFailure::NonPositiveAmount => {
            FlatFailure::new(FailureKind::ExpenseNonPositiveAmount)
        }
        ExpenseFailure::CurrencyMismatch { expected, found } => FlatFailure {
            failure: FailureKind::ExpenseCurrencyMismatch,
            payload: Payload {
                expected_currency: Some(expected.clone()),
                found_currency: Some(found.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::EmptySplit => FlatFailure::new(FailureKind::ExpenseEmptySplit),
        ExpenseFailure::DuplicateMember { member } => FlatFailure {
            failure: FailureKind::ExpenseDuplicateMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::NonPositiveShare { member } => FlatFailure {
            failure: FailureKind::ExpenseNonPositiveShare,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::AmountsDoNotAddUp { expected, actual } => FlatFailure {
            failure: FailureKind::ExpenseAmountsDoNotAddUp,
            payload: Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::PercentagesDoNotAddUp { total_basis_points } => FlatFailure {
            failure: FailureKind::ExpensePercentagesDoNotAddUp,
            payload: Payload {
                total_basis_points: Some(*total_basis_points),
                ..Payload::default()
            },
        },
        ExpenseFailure::NoItems => FlatFailure::new(FailureKind::ExpenseNoItems),
        ExpenseFailure::EmptyItemName => FlatFailure::new(FailureKind::ExpenseEmptyItemName),
        ExpenseFailure::ItemsDoNotAddUp { expected, actual } => FlatFailure {
            failure: FailureKind::ExpenseItemsDoNotAddUp,
            payload: Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::NegativeDiscount => FlatFailure::new(FailureKind::ExpenseNegativeDiscount),
        ExpenseFailure::DiscountAboveHundredPercent => {
            FlatFailure::new(FailureKind::ExpenseDiscountAboveHundredPercent)
        }
        ExpenseFailure::DiscountAbovePrice => {
            FlatFailure::new(FailureKind::ExpenseDiscountAbovePrice)
        }
        ExpenseFailure::NegativeTip => FlatFailure::new(FailureKind::ExpenseNegativeTip),
        ExpenseFailure::Overflow => FlatFailure::new(FailureKind::ExpenseOverflow),
    }
}

/// The group module's `failure`, with what it holds.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per failure of the group module, of which there are 19"
)]
fn expense_group(failure: &ExpenseGroupFailure) -> FlatFailure {
    match failure {
        ExpenseGroupFailure::UnsupportedFormat { format } => FlatFailure {
            failure: FailureKind::ExpenseGroupUnsupportedFormat,
            payload: Payload {
                format: Some(*format),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::EmptyName => FlatFailure::new(FailureKind::ExpenseGroupEmptyName),
        ExpenseGroupFailure::MissingExchangeRate { currency } => FlatFailure {
            failure: FailureKind::ExpenseGroupMissingExchangeRate,
            payload: Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::UnexpectedExchangeRate => {
            FlatFailure::new(FailureKind::ExpenseGroupUnexpectedExchangeRate)
        }
        ExpenseGroupFailure::DuplicateExchangeRate { currency } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateExchangeRate,
            payload: Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::DuplicateMember { member } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::UnknownMember { member } => FlatFailure {
            failure: FailureKind::ExpenseGroupUnknownMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::ArchivedMember { member } => FlatFailure {
            failure: FailureKind::ExpenseGroupArchivedMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::MemberInUse {
            member,
            expenses,
            transfers,
        } => FlatFailure {
            failure: FailureKind::ExpenseGroupMemberInUse,
            payload: Payload {
                expenses: Some(expenses.clone()),
                member: Some(member.clone()),
                transfers: Some(transfers.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::DuplicateCategory { category } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateCategory,
            payload: Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::DuplicateCategoryName { existing } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateCategoryName,
            payload: Payload {
                existing: Some(existing.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::UnknownCategory { category } => FlatFailure {
            failure: FailureKind::ExpenseGroupUnknownCategory,
            payload: Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::BuiltInCategory => {
            FlatFailure::new(FailureKind::ExpenseGroupBuiltInCategory)
        }
        ExpenseGroupFailure::DuplicateExpense { expense } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateExpense,
            payload: Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::UnknownExpense { expense } => FlatFailure {
            failure: FailureKind::ExpenseGroupUnknownExpense,
            payload: Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::DuplicateTransfer { transfer } => FlatFailure {
            failure: FailureKind::ExpenseGroupDuplicateTransfer,
            payload: Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::UnknownTransfer { transfer } => FlatFailure {
            failure: FailureKind::ExpenseGroupUnknownTransfer,
            payload: Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        },
        ExpenseGroupFailure::Busy => FlatFailure::new(FailureKind::ExpenseGroupBusy),
        ExpenseGroupFailure::Overflow => FlatFailure::new(FailureKind::ExpenseGroupOverflow),
    }
}

/// The id module's `failure`.
const fn id(failure: IdFailure) -> FailureKind {
    match failure {
        IdFailure::Empty => FailureKind::IdEmpty,
        IdFailure::TooLong => FailureKind::IdTooLong,
        IdFailure::ControlCharacter => FailureKind::IdControlCharacter,
        IdFailure::SurroundingWhitespace => FailureKind::IdSurroundingWhitespace,
    }
}

/// The language module's `failure`.
const fn language(failure: LanguageFailure) -> FailureKind {
    match failure {
        LanguageFailure::InvalidLanguage => FailureKind::LanguageInvalidLanguage,
    }
}

/// The `field` of a mismatched record, as its failure.
const fn mismatched_record(field: MismatchedField) -> FailureKind {
    match field {
        MismatchedField::IbanCountryCode => FailureKind::MismatchedRecordIbanCountryCode,
        MismatchedField::PhoneInternational => FailureKind::MismatchedRecordPhoneInternational,
        MismatchedField::CurrencyDetails => FailureKind::MismatchedRecordCurrencyDetails,
    }
}

/// The member module's `failure`.
const fn member(failure: MemberFailure) -> FailureKind {
    match failure {
        MemberFailure::EmptyFirstName => FailureKind::MemberEmptyFirstName,
    }
}

/// The money module's `failure`, with what it holds.
fn money(failure: &MoneyFailure) -> FlatFailure {
    match failure {
        MoneyFailure::InvalidAmount => FlatFailure::new(FailureKind::MoneyInvalidAmount),
        MoneyFailure::TooManyDecimals { currency, allowed } => FlatFailure {
            failure: FailureKind::MoneyTooManyDecimals,
            payload: Payload {
                allowed: Some(*allowed),
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        MoneyFailure::CurrencyMismatch { left, right } => FlatFailure {
            failure: FailureKind::MoneyCurrencyMismatch,
            payload: Payload {
                left: Some(left.clone()),
                right: Some(right.clone()),
                ..Payload::default()
            },
        },
        MoneyFailure::UnknownCurrency => FlatFailure::new(FailureKind::MoneyUnknownCurrency),
        MoneyFailure::InvalidExchangeRate => {
            FlatFailure::new(FailureKind::MoneyInvalidExchangeRate)
        }
        MoneyFailure::InvalidParts => FlatFailure::new(FailureKind::MoneyInvalidParts),
        MoneyFailure::InvalidPercentage => FlatFailure::new(FailureKind::MoneyInvalidPercentage),
        MoneyFailure::InvalidQuantity => FlatFailure::new(FailureKind::MoneyInvalidQuantity),
        MoneyFailure::NotFinite => FlatFailure::new(FailureKind::MoneyNotFinite),
        MoneyFailure::Overflow => FlatFailure::new(FailureKind::MoneyOverflow),
    }
}

/// The settlement module's `failure`, with what it holds.
fn settlement(failure: SettlementFailure) -> FlatFailure {
    match failure {
        SettlementFailure::OutOfMemory { balances } => FlatFailure {
            failure: FailureKind::SettlementOutOfMemory,
            payload: Payload {
                balances: Some(balances),
                ..Payload::default()
            },
        },
    }
}

/// The transfer module's `failure`.
const fn transfer(failure: TransferFailure) -> FailureKind {
    match failure {
        TransferFailure::SameMember => FailureKind::TransferSameMember,
        TransferFailure::NonPositiveAmount => FailureKind::TransferNonPositiveAmount,
    }
}

/// What a refused IBAN's `reason` holds: the check it failed, and for a
/// wrong length both lengths.
fn iban(reason: IbanReason) -> Payload {
    match reason {
        IbanReason::InvalidCharacter => Payload {
            reason: Some(IbanReasonKind::InvalidCharacter),
            ..Payload::default()
        },
        IbanReason::InvalidFormat => Payload {
            reason: Some(IbanReasonKind::InvalidFormat),
            ..Payload::default()
        },
        IbanReason::UnknownCountry => Payload {
            reason: Some(IbanReasonKind::UnknownCountry),
            ..Payload::default()
        },
        IbanReason::WrongLength { expected, found } => Payload {
            expected_length: Some(expected),
            found_length: Some(found),
            reason: Some(IbanReasonKind::WrongLength),
            ..Payload::default()
        },
        IbanReason::InvalidStructure => Payload {
            reason: Some(IbanReasonKind::InvalidStructure),
            ..Payload::default()
        },
        IbanReason::InvalidChecksum => Payload {
            reason: Some(IbanReasonKind::InvalidChecksum),
            ..Payload::default()
        },
    }
}

#[cfg(test)]
mod tests;
