//! The error in the shape a binding without enums that hold data reads:
//! the kind of error, the failure within its module, the library's message,
//! and what the failure holds, each in a field of its own.

use crate::{
    AccountFailure, CalendarFailure, CategoryFailure, ContactFailure, EranosError, ExpenseFailure,
    GroupFailure, IbanReason, IdFailure, LanguageFailure, MemberFailure, Money, MoneyFailure,
    SettlementFailure, TransferFailure,
};

/// An [`EranosError`] as fields: its kind, the library's message, the
/// failure within its module, and what the failure holds.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatError {
    /// The module that refused the call, or the binding's own case.
    pub kind: ErrorKind,
    /// The library's message: lowercase, without a full stop, and never the
    /// text it refused.
    pub message: String,
    /// The failure within its module, absent for the binding's own cases.
    pub failure: Option<FailureKind>,
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
            message: "the text is not UTF-8".to_owned(),
            failure: None,
            payload: Payload::default(),
        }
    }
}

impl From<&EranosError> for FlatError {
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per case of the error, of which there are 17"
    )]
    fn from(error: &EranosError) -> Self {
        let message = error.to_string();
        match error {
            EranosError::Account { failure, .. } => {
                let FlatFailure { failure, payload } = account(*failure);
                Self {
                    kind: ErrorKind::Account,
                    message,
                    failure: Some(failure),
                    payload,
                }
            }
            EranosError::Calendar { failure, .. } => Self {
                kind: ErrorKind::Calendar,
                message,
                failure: Some(calendar(*failure)),
                payload: Payload::default(),
            },
            EranosError::Cancelled { .. } => Self {
                kind: ErrorKind::Cancelled,
                message,
                failure: None,
                payload: Payload::default(),
            },
            EranosError::Category { failure, .. } => Self {
                kind: ErrorKind::Category,
                message,
                failure: Some(category(*failure)),
                payload: Payload::default(),
            },
            EranosError::Contact { failure, .. } => Self {
                kind: ErrorKind::Contact,
                message,
                failure: Some(contact(*failure)),
                payload: Payload::default(),
            },
            EranosError::Expense { failure, .. } => {
                let FlatFailure { failure, payload } = expense(failure);
                Self {
                    kind: ErrorKind::Expense,
                    message,
                    failure: Some(failure),
                    payload,
                }
            }
            EranosError::Group { failure, .. } => {
                let FlatFailure { failure, payload } = group(failure);
                Self {
                    kind: ErrorKind::Group,
                    message,
                    failure: Some(failure),
                    payload,
                }
            }
            EranosError::Id { failure, .. } => Self {
                kind: ErrorKind::Id,
                message,
                failure: Some(id(*failure)),
                payload: Payload::default(),
            },
            EranosError::IdentifierGiven { .. } => Self {
                kind: ErrorKind::IdentifierGiven,
                message,
                failure: None,
                payload: Payload::default(),
            },
            EranosError::IdentifierMissing { .. } => Self {
                kind: ErrorKind::IdentifierMissing,
                message,
                failure: None,
                payload: Payload::default(),
            },
            EranosError::InvalidJson { column, line, .. } => Self {
                kind: ErrorKind::InvalidJson,
                message,
                failure: None,
                payload: Payload {
                    column: Some(*column),
                    line: Some(*line),
                    ..Payload::default()
                },
            },
            EranosError::Language { failure, .. } => Self {
                kind: ErrorKind::Language,
                message,
                failure: Some(language(*failure)),
                payload: Payload::default(),
            },
            EranosError::Member { failure, .. } => Self {
                kind: ErrorKind::Member,
                message,
                failure: Some(member(*failure)),
                payload: Payload::default(),
            },
            EranosError::Money { failure, .. } => {
                let FlatFailure { failure, payload } = money(failure);
                Self {
                    kind: ErrorKind::Money,
                    message,
                    failure: Some(failure),
                    payload,
                }
            }
            EranosError::Released { .. } => Self {
                kind: ErrorKind::Released,
                message,
                failure: None,
                payload: Payload::default(),
            },
            EranosError::Settlement { failure, .. } => {
                let FlatFailure { failure, payload } = settlement(*failure);
                Self {
                    kind: ErrorKind::Settlement,
                    message,
                    failure: Some(failure),
                    payload,
                }
            }
            EranosError::Transfer { failure, .. } => Self {
                kind: ErrorKind::Transfer,
                message,
                failure: Some(transfer(*failure)),
                payload: Payload::default(),
            },
        }
    }
}

/// The module of the library that refused a call, or the binding's own
/// case: a case per case of [`EranosError`], and [`ErrorKind::InvalidText`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// [`EranosError::Account`].
    Account,
    /// [`EranosError::Calendar`].
    Calendar,
    /// [`EranosError::Cancelled`].
    Cancelled,
    /// [`EranosError::Category`].
    Category,
    /// [`EranosError::Contact`].
    Contact,
    /// [`EranosError::Expense`].
    Expense,
    /// [`EranosError::Group`].
    Group,
    /// [`EranosError::Id`].
    Id,
    /// [`EranosError::IdentifierGiven`].
    IdentifierGiven,
    /// [`EranosError::IdentifierMissing`].
    IdentifierMissing,
    /// [`EranosError::InvalidJson`].
    InvalidJson,
    /// Text that is not UTF-8, which a binding that takes text as bytes
    /// refuses before the library sees it.
    InvalidText,
    /// [`EranosError::Language`].
    Language,
    /// [`EranosError::Member`].
    Member,
    /// [`EranosError::Money`].
    Money,
    /// [`EranosError::Released`].
    Released,
    /// [`EranosError::Settlement`].
    Settlement,
    /// [`EranosError::Transfer`].
    Transfer,
}

/// The failure within its module, named after the module and the failure:
/// a case per case of every module's failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureKind {
    /// [`AccountFailure::EmptyName`].
    AccountEmptyName,
    /// [`AccountFailure::EmptyUsername`].
    AccountEmptyUsername,
    /// [`AccountFailure::InvalidIban`].
    AccountInvalidIban,
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
    /// [`ExpenseFailure::AmountsDoNotAddUp`].
    ExpenseAmountsDoNotAddUp,
    /// [`ExpenseFailure::CurrencyMismatch`].
    ExpenseCurrencyMismatch,
    /// [`ExpenseFailure::DuplicateMember`].
    ExpenseDuplicateMember,
    /// [`ExpenseFailure::EmptyItemName`].
    ExpenseEmptyItemName,
    /// [`ExpenseFailure::EmptyName`].
    ExpenseEmptyName,
    /// [`ExpenseFailure::EmptySplit`].
    ExpenseEmptySplit,
    /// [`ExpenseFailure::ItemsDoNotAddUp`].
    ExpenseItemsDoNotAddUp,
    /// [`ExpenseFailure::NegativeTip`].
    ExpenseNegativeTip,
    /// [`ExpenseFailure::NoItems`].
    ExpenseNoItems,
    /// [`ExpenseFailure::NonPositiveAmount`].
    ExpenseNonPositiveAmount,
    /// [`ExpenseFailure::NonPositiveShare`].
    ExpenseNonPositiveShare,
    /// [`ExpenseFailure::Overflow`].
    ExpenseOverflow,
    /// [`ExpenseFailure::PercentagesDoNotAddUp`].
    ExpensePercentagesDoNotAddUp,
    /// [`ExpenseFailure::ZeroQuantity`].
    ExpenseZeroQuantity,
    /// [`GroupFailure::ArchivedMember`].
    GroupArchivedMember,
    /// [`GroupFailure::BuiltInCategory`].
    GroupBuiltInCategory,
    /// [`GroupFailure::Busy`].
    GroupBusy,
    /// [`GroupFailure::DuplicateCategory`].
    GroupDuplicateCategory,
    /// [`GroupFailure::DuplicateCategoryName`].
    GroupDuplicateCategoryName,
    /// [`GroupFailure::DuplicateExchangeRate`].
    GroupDuplicateExchangeRate,
    /// [`GroupFailure::DuplicateExpense`].
    GroupDuplicateExpense,
    /// [`GroupFailure::DuplicateMember`].
    GroupDuplicateMember,
    /// [`GroupFailure::DuplicateTransfer`].
    GroupDuplicateTransfer,
    /// [`GroupFailure::EmptyName`].
    GroupEmptyName,
    /// [`GroupFailure::MemberInUse`].
    GroupMemberInUse,
    /// [`GroupFailure::MissingExchangeRate`].
    GroupMissingExchangeRate,
    /// [`GroupFailure::Overflow`].
    GroupOverflow,
    /// [`GroupFailure::UnexpectedExchangeRate`].
    GroupUnexpectedExchangeRate,
    /// [`GroupFailure::UnknownCategory`].
    GroupUnknownCategory,
    /// [`GroupFailure::UnknownExpense`].
    GroupUnknownExpense,
    /// [`GroupFailure::UnknownMember`].
    GroupUnknownMember,
    /// [`GroupFailure::UnknownTransfer`].
    GroupUnknownTransfer,
    /// [`GroupFailure::UnsupportedFormat`].
    GroupUnsupportedFormat,
    /// [`IdFailure::InvalidId`].
    IdInvalidId,
    /// [`LanguageFailure::InvalidLanguage`].
    LanguageInvalidLanguage,
    /// [`MemberFailure::EmptyFirstName`].
    MemberEmptyFirstName,
    /// [`MoneyFailure::CurrencyMismatch`].
    MoneyCurrencyMismatch,
    /// [`MoneyFailure::InvalidAmount`].
    MoneyInvalidAmount,
    /// [`MoneyFailure::InvalidExchangeRate`].
    MoneyInvalidExchangeRate,
    /// [`MoneyFailure::InvalidParts`].
    MoneyInvalidParts,
    /// [`MoneyFailure::InvalidPercentage`].
    MoneyInvalidPercentage,
    /// [`MoneyFailure::Overflow`].
    MoneyOverflow,
    /// [`MoneyFailure::TooManyDecimals`].
    MoneyTooManyDecimals,
    /// [`MoneyFailure::UnknownCurrency`].
    MoneyUnknownCurrency,
    /// [`SettlementFailure::OutOfMemory`].
    SettlementOutOfMemory,
    /// [`TransferFailure::NonPositiveAmount`].
    TransferNonPositiveAmount,
    /// [`TransferFailure::SameMember`].
    TransferSameMember,
}

/// Why an IBAN was refused, without what [`IbanReason::WrongLength`] holds,
/// which [`Payload`] carries: a case per case of [`IbanReason`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IbanReasonKind {
    /// [`IbanReason::InvalidCharacter`].
    InvalidCharacter,
    /// [`IbanReason::InvalidChecksum`].
    InvalidChecksum,
    /// [`IbanReason::InvalidFormat`].
    InvalidFormat,
    /// [`IbanReason::InvalidStructure`].
    InvalidStructure,
    /// [`IbanReason::UnknownCountry`].
    UnknownCountry,
    /// [`IbanReason::WrongLength`].
    WrongLength,
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
    /// The category, as its identifier: [`GroupFailure::DuplicateCategory`],
    /// [`GroupFailure::UnknownCategory`].
    pub category: Option<String>,
    /// The column where the saved group stops being one, from 1:
    /// [`EranosError::InvalidJson`].
    pub column: Option<i64>,
    /// The currency, as its ISO 4217 code:
    /// [`GroupFailure::DuplicateExchangeRate`],
    /// [`GroupFailure::MissingExchangeRate`],
    /// [`MoneyFailure::TooManyDecimals`].
    pub currency: Option<String>,
    /// The category that has the name, as its identifier:
    /// [`GroupFailure::DuplicateCategoryName`].
    pub existing: Option<String>,
    /// What the amounts must add up to:
    /// [`ExpenseFailure::AmountsDoNotAddUp`],
    /// [`ExpenseFailure::ItemsDoNotAddUp`].
    pub expected_amount: Option<Money>,
    /// The currency the entry must be in:
    /// [`ExpenseFailure::CurrencyMismatch`].
    pub expected_currency: Option<String>,
    /// How many characters the country's IBANs have:
    /// [`IbanReason::WrongLength`].
    pub expected_length: Option<i64>,
    /// The expense, as its identifier: [`GroupFailure::DuplicateExpense`],
    /// [`GroupFailure::UnknownExpense`].
    pub expense: Option<String>,
    /// The expenses that name the member: [`GroupFailure::MemberInUse`].
    pub expenses: Option<Vec<String>>,
    /// The format the saved group names:
    /// [`GroupFailure::UnsupportedFormat`].
    pub format: Option<i64>,
    /// The currency the part is in: [`ExpenseFailure::CurrencyMismatch`].
    pub found_currency: Option<String>,
    /// How many characters the IBAN has: [`IbanReason::WrongLength`].
    pub found_length: Option<i64>,
    /// The currency of the first amount: [`MoneyFailure::CurrencyMismatch`].
    pub left: Option<String>,
    /// The line where the saved group stops being one, from 1:
    /// [`EranosError::InvalidJson`].
    pub line: Option<i64>,
    /// The member, as their identifier: [`ExpenseFailure::DuplicateMember`],
    /// [`ExpenseFailure::NonPositiveShare`],
    /// [`GroupFailure::ArchivedMember`], [`GroupFailure::DuplicateMember`],
    /// [`GroupFailure::MemberInUse`], [`GroupFailure::UnknownMember`].
    pub member: Option<String>,
    /// The check the IBAN failed: [`AccountFailure::InvalidIban`].
    pub reason: Option<IbanReasonKind>,
    /// The currency of the second amount:
    /// [`MoneyFailure::CurrencyMismatch`].
    pub right: Option<String>,
    /// What the percentages add up to, in basis points:
    /// [`ExpenseFailure::PercentagesDoNotAddUp`].
    pub total_basis_points: Option<i64>,
    /// The transfer, as its identifier: [`GroupFailure::DuplicateTransfer`],
    /// [`GroupFailure::UnknownTransfer`].
    pub transfer: Option<String>,
    /// The transfers that name the member: [`GroupFailure::MemberInUse`].
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
fn account(failure: AccountFailure) -> FlatFailure {
    match failure {
        AccountFailure::EmptyName => FlatFailure::new(FailureKind::AccountEmptyName),
        AccountFailure::EmptyUsername => FlatFailure::new(FailureKind::AccountEmptyUsername),
        AccountFailure::InvalidIban { reason } => FlatFailure {
            failure: FailureKind::AccountInvalidIban,
            payload: iban(reason),
        },
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
        ExpenseFailure::AmountsDoNotAddUp { actual, expected } => FlatFailure {
            failure: FailureKind::ExpenseAmountsDoNotAddUp,
            payload: Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::CurrencyMismatch { expected, found } => FlatFailure {
            failure: FailureKind::ExpenseCurrencyMismatch,
            payload: Payload {
                expected_currency: Some(expected.clone()),
                found_currency: Some(found.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::DuplicateMember { member } => FlatFailure {
            failure: FailureKind::ExpenseDuplicateMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::EmptyItemName => FlatFailure::new(FailureKind::ExpenseEmptyItemName),
        ExpenseFailure::EmptyName => FlatFailure::new(FailureKind::ExpenseEmptyName),
        ExpenseFailure::EmptySplit => FlatFailure::new(FailureKind::ExpenseEmptySplit),
        ExpenseFailure::ItemsDoNotAddUp { actual, expected } => FlatFailure {
            failure: FailureKind::ExpenseItemsDoNotAddUp,
            payload: Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::NegativeTip => FlatFailure::new(FailureKind::ExpenseNegativeTip),
        ExpenseFailure::NoItems => FlatFailure::new(FailureKind::ExpenseNoItems),
        ExpenseFailure::NonPositiveAmount => {
            FlatFailure::new(FailureKind::ExpenseNonPositiveAmount)
        }
        ExpenseFailure::NonPositiveShare { member } => FlatFailure {
            failure: FailureKind::ExpenseNonPositiveShare,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        ExpenseFailure::Overflow => FlatFailure::new(FailureKind::ExpenseOverflow),
        ExpenseFailure::PercentagesDoNotAddUp { total_basis_points } => FlatFailure {
            failure: FailureKind::ExpensePercentagesDoNotAddUp,
            payload: Payload {
                total_basis_points: Some(*total_basis_points),
                ..Payload::default()
            },
        },
        ExpenseFailure::ZeroQuantity => FlatFailure::new(FailureKind::ExpenseZeroQuantity),
    }
}

/// The group module's `failure`, with what it holds.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per failure of the group module, of which there are 19"
)]
fn group(failure: &GroupFailure) -> FlatFailure {
    match failure {
        GroupFailure::ArchivedMember { member } => FlatFailure {
            failure: FailureKind::GroupArchivedMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::BuiltInCategory => FlatFailure::new(FailureKind::GroupBuiltInCategory),
        GroupFailure::Busy => FlatFailure::new(FailureKind::GroupBusy),
        GroupFailure::DuplicateCategory { category } => FlatFailure {
            failure: FailureKind::GroupDuplicateCategory,
            payload: Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::DuplicateCategoryName { existing } => FlatFailure {
            failure: FailureKind::GroupDuplicateCategoryName,
            payload: Payload {
                existing: Some(existing.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::DuplicateExchangeRate { currency } => FlatFailure {
            failure: FailureKind::GroupDuplicateExchangeRate,
            payload: Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::DuplicateExpense { expense } => FlatFailure {
            failure: FailureKind::GroupDuplicateExpense,
            payload: Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::DuplicateMember { member } => FlatFailure {
            failure: FailureKind::GroupDuplicateMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::DuplicateTransfer { transfer } => FlatFailure {
            failure: FailureKind::GroupDuplicateTransfer,
            payload: Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::EmptyName => FlatFailure::new(FailureKind::GroupEmptyName),
        GroupFailure::MemberInUse {
            expenses,
            member,
            transfers,
        } => FlatFailure {
            failure: FailureKind::GroupMemberInUse,
            payload: Payload {
                expenses: Some(expenses.clone()),
                member: Some(member.clone()),
                transfers: Some(transfers.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::MissingExchangeRate { currency } => FlatFailure {
            failure: FailureKind::GroupMissingExchangeRate,
            payload: Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::Overflow => FlatFailure::new(FailureKind::GroupOverflow),
        GroupFailure::UnexpectedExchangeRate => {
            FlatFailure::new(FailureKind::GroupUnexpectedExchangeRate)
        }
        GroupFailure::UnknownCategory { category } => FlatFailure {
            failure: FailureKind::GroupUnknownCategory,
            payload: Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::UnknownExpense { expense } => FlatFailure {
            failure: FailureKind::GroupUnknownExpense,
            payload: Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::UnknownMember { member } => FlatFailure {
            failure: FailureKind::GroupUnknownMember,
            payload: Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::UnknownTransfer { transfer } => FlatFailure {
            failure: FailureKind::GroupUnknownTransfer,
            payload: Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        },
        GroupFailure::UnsupportedFormat { format } => FlatFailure {
            failure: FailureKind::GroupUnsupportedFormat,
            payload: Payload {
                format: Some(*format),
                ..Payload::default()
            },
        },
    }
}

/// The id module's `failure`.
const fn id(failure: IdFailure) -> FailureKind {
    match failure {
        IdFailure::InvalidId => FailureKind::IdInvalidId,
    }
}

/// The language module's `failure`.
const fn language(failure: LanguageFailure) -> FailureKind {
    match failure {
        LanguageFailure::InvalidLanguage => FailureKind::LanguageInvalidLanguage,
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
        MoneyFailure::CurrencyMismatch { left, right } => FlatFailure {
            failure: FailureKind::MoneyCurrencyMismatch,
            payload: Payload {
                left: Some(left.clone()),
                right: Some(right.clone()),
                ..Payload::default()
            },
        },
        MoneyFailure::InvalidAmount => FlatFailure::new(FailureKind::MoneyInvalidAmount),
        MoneyFailure::InvalidExchangeRate => {
            FlatFailure::new(FailureKind::MoneyInvalidExchangeRate)
        }
        MoneyFailure::InvalidParts => FlatFailure::new(FailureKind::MoneyInvalidParts),
        MoneyFailure::InvalidPercentage => FlatFailure::new(FailureKind::MoneyInvalidPercentage),
        MoneyFailure::Overflow => FlatFailure::new(FailureKind::MoneyOverflow),
        MoneyFailure::TooManyDecimals { allowed, currency } => FlatFailure {
            failure: FailureKind::MoneyTooManyDecimals,
            payload: Payload {
                allowed: Some(*allowed),
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        },
        MoneyFailure::UnknownCurrency => FlatFailure::new(FailureKind::MoneyUnknownCurrency),
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
        TransferFailure::NonPositiveAmount => FailureKind::TransferNonPositiveAmount,
        TransferFailure::SameMember => FailureKind::TransferSameMember,
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
        IbanReason::InvalidChecksum => Payload {
            reason: Some(IbanReasonKind::InvalidChecksum),
            ..Payload::default()
        },
        IbanReason::InvalidFormat => Payload {
            reason: Some(IbanReasonKind::InvalidFormat),
            ..Payload::default()
        },
        IbanReason::InvalidStructure => Payload {
            reason: Some(IbanReasonKind::InvalidStructure),
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
    }
}

#[cfg(test)]
mod tests;
