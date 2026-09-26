//! The one error every binding reports: a case per module of the library,
//! holding that module's failure and the library's message, and the
//! binding's own cases.

use std::fmt;

use eranos::error::{
    AccountError, CalendarError, CategoryError, ContactError, ExpenseError, GroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, SettlementError, TransferError,
};
use eranos::{CategoryId, Currency, ExpenseId, MemberId, TransferId};
use eranos_account::{FromAccountError, FromIbanError};
use eranos_calendar::FromCalendarError;
use eranos_category::FromCategoryError;
use eranos_contact::FromContactError;
use eranos_core::FromIdError;
use eranos_expense::FromExpenseError;
use eranos_group::{FromGroupError, FromLanguageError};
use eranos_member::FromMemberError;
use eranos_money::FromMoneyError;
use eranos_settlement::FromSettlementError;
use eranos_transfer::FromTransferError;

use crate::Money;

/// Why a call failed.
///
/// A case per module of the library whose rules refuse a call, holding the
/// module's failure, with a case per variant of the module's error and what
/// it holds, and the library's message; and the binding's own cases. A
/// synchronous call on wasm that finds the group locked by another call
/// fails with [`GroupFailure::Busy`].
#[eranos_glue_macros::mirrored(error)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EranosError {
    /// An [`AccountError`], about a payment account: its IBAN, username, or
    /// label.
    Account {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: AccountFailure,
    },
    /// A [`CalendarError`], about a date or a time of day.
    Calendar {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: CalendarFailure,
    },
    /// The call was cancelled through its
    /// [`Cancellation`](crate::Cancellation), before it changed anything.
    Cancelled {
        /// What went wrong.
        detail: String,
    },
    /// A [`CategoryError`], about a category.
    Category {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: CategoryFailure,
    },
    /// A [`ContactError`], about an email address or a phone number.
    Contact {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: ContactFailure,
    },
    /// An [`ExpenseError`], about an expense: its amount, split, items, or tip.
    Expense {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: ExpenseFailure,
    },
    /// A [`GroupError`], about how an entry fits the group, or the group's
    /// lock.
    Group {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: GroupFailure,
    },
    /// An [`IdError`], about an identifier.
    Id {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: IdFailure,
    },
    /// An entry to add carried an identifier; the group gives a new entry its
    /// own.
    IdentifierGiven {
        /// What went wrong.
        detail: String,
    },
    /// An entry to replace carried no identifier to find it by.
    IdentifierMissing {
        /// What went wrong.
        detail: String,
    },
    /// A saved group that is not JSON of the saved form, at `line` and
    /// `column`.
    InvalidJson {
        /// The column of the first character the reader refused, from 1.
        column: i64,
        /// What went wrong, and where.
        detail: String,
        /// The line of the first character the reader refused, from 1.
        line: i64,
    },
    /// A [`LanguageError`], about a language tag.
    Language {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: LanguageFailure,
    },
    /// A [`MemberError`], about a member.
    Member {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: MemberFailure,
    },
    /// A [`MoneyError`], about an amount, a currency, a rate, or a ratio.
    Money {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: MoneyFailure,
    },
    /// The read handle was released, so it reads the group no more.
    Released {
        /// What went wrong.
        detail: String,
    },
    /// A [`SettlementError`], about a settlement.
    Settlement {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: SettlementFailure,
    },
    /// A [`TransferError`], about a transfer.
    Transfer {
        /// The library's message.
        detail: String,
        /// Which failure, with what it holds.
        failure: TransferFailure,
    },
}

impl EranosError {
    /// The error of the case `Cancelled`.
    pub(crate) fn cancelled() -> Self {
        Self::Cancelled {
            detail: "the call was cancelled".to_owned(),
        }
    }

    /// The error of the case `IdentifierGiven`.
    pub(crate) fn identifier_given() -> Self {
        Self::IdentifierGiven {
            detail: "an entry to add must not carry an identifier".to_owned(),
        }
    }

    /// The error of the case `IdentifierMissing`.
    pub(crate) fn identifier_missing() -> Self {
        Self::IdentifierMissing {
            detail: "an entry to replace must carry its identifier".to_owned(),
        }
    }

    /// The error of the case `Released`.
    pub(crate) fn released() -> Self {
        Self::Released {
            detail: "the read handle was released".to_owned(),
        }
    }

    /// The error of a saved group that is not JSON of the saved form. The
    /// detail names the fault and where it is, never the text there,
    /// which can hold account numbers.
    pub(crate) fn invalid_json(error: &serde_json::Error) -> Self {
        let (line, column) = (count(error.line()), count(error.column()));
        let fault = if error.is_syntax() {
            "is not valid JSON"
        } else if error.is_eof() {
            "ends early"
        } else {
            "has a value its format does not allow"
        };
        Self::InvalidJson {
            column,
            detail: format!("the saved group {fault}, at line {line}, column {column}"),
            line,
        }
    }

    /// What went wrong, as the library words it.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Account { detail, .. }
            | Self::Calendar { detail, .. }
            | Self::Cancelled { detail, .. }
            | Self::Category { detail, .. }
            | Self::Contact { detail, .. }
            | Self::Expense { detail, .. }
            | Self::Group { detail, .. }
            | Self::Id { detail, .. }
            | Self::IdentifierGiven { detail, .. }
            | Self::IdentifierMissing { detail, .. }
            | Self::InvalidJson { detail, .. }
            | Self::Language { detail, .. }
            | Self::Member { detail, .. }
            | Self::Money { detail, .. }
            | Self::Released { detail, .. }
            | Self::Settlement { detail, .. }
            | Self::Transfer { detail, .. } => detail,
        }
    }
}

impl fmt::Display for EranosError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.detail())
    }
}

impl std::error::Error for EranosError {}

/// What an [`AccountError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountFailure {
    /// [`AccountError::EmptyName`].
    EmptyName,
    /// [`AccountError::EmptyUsername`].
    EmptyUsername,
    /// [`AccountError::InvalidIban`].
    InvalidIban {
        /// Why the IBAN was refused.
        reason: IbanReason,
    },
}

/// What a [`CalendarError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalendarFailure {
    /// [`CalendarError::InvalidDate`].
    InvalidDate,
    /// [`CalendarError::InvalidTime`].
    InvalidTime,
}

/// What a [`CategoryError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CategoryFailure {
    /// [`CategoryError::EmptyName`].
    EmptyName,
}

/// What a [`ContactError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContactFailure {
    /// [`ContactError::InvalidEmail`].
    InvalidEmail,
    /// [`ContactError::InvalidPhoneNumber`].
    InvalidPhoneNumber,
}

/// What an [`ExpenseError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpenseFailure {
    /// [`ExpenseError::AmountsDoNotAddUp`].
    AmountsDoNotAddUp {
        /// What the parts add up to.
        actual: Money,
        /// What the parts must add up to.
        expected: Money,
    },
    /// [`ExpenseError::CurrencyMismatch`].
    CurrencyMismatch {
        /// The currency the entry must be in, as its ISO 4217 code.
        expected: String,
        /// The currency the part is in, as its ISO 4217 code.
        found: String,
    },
    /// [`ExpenseError::DuplicateMember`].
    DuplicateMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseError::EmptyItemName`].
    EmptyItemName,
    /// [`ExpenseError::EmptyName`].
    EmptyName,
    /// [`ExpenseError::EmptySplit`].
    EmptySplit,
    /// [`ExpenseError::ItemsDoNotAddUp`].
    ItemsDoNotAddUp {
        /// What the parts add up to.
        actual: Money,
        /// What the parts must add up to.
        expected: Money,
    },
    /// [`ExpenseError::NegativeTip`].
    NegativeTip,
    /// [`ExpenseError::NoItems`].
    NoItems,
    /// [`ExpenseError::NonPositiveAmount`].
    NonPositiveAmount,
    /// [`ExpenseError::NonPositiveShare`].
    NonPositiveShare {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseError::Overflow`].
    Overflow,
    /// [`ExpenseError::PercentagesDoNotAddUp`].
    PercentagesDoNotAddUp {
        /// What the percentages add up to, in basis points.
        total_basis_points: i64,
    },
    /// [`ExpenseError::ZeroQuantity`].
    ZeroQuantity,
}

/// What a [`GroupError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GroupFailure {
    /// [`GroupError::ArchivedMember`].
    ArchivedMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::BuiltInCategory`].
    BuiltInCategory,
    /// [`GroupError::Busy`].
    Busy,
    /// [`GroupError::DuplicateCategory`].
    DuplicateCategory {
        /// The category, as its identifier.
        category: String,
    },
    /// [`GroupError::DuplicateCategoryName`].
    DuplicateCategoryName {
        /// The category that has the name, as its identifier.
        existing: String,
    },
    /// [`GroupError::DuplicateExchangeRate`].
    DuplicateExchangeRate {
        /// The currency, as its ISO 4217 code.
        currency: String,
    },
    /// [`GroupError::DuplicateExpense`].
    DuplicateExpense {
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`GroupError::DuplicateMember`].
    DuplicateMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::DuplicateTransfer`].
    DuplicateTransfer {
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`GroupError::EmptyName`].
    EmptyName,
    /// [`GroupError::MemberInUse`].
    MemberInUse {
        /// The expenses that reference the member, as identifiers.
        expenses: Vec<String>,
        /// The member, as their identifier.
        member: String,
        /// The transfers that reference the member, as identifiers.
        transfers: Vec<String>,
    },
    /// [`GroupError::MissingExchangeRate`].
    MissingExchangeRate {
        /// The currency, as its ISO 4217 code.
        currency: String,
    },
    /// [`GroupError::Overflow`].
    Overflow,
    /// [`GroupError::UnexpectedExchangeRate`].
    UnexpectedExchangeRate,
    /// [`GroupError::UnknownCategory`].
    UnknownCategory {
        /// The category, as its identifier.
        category: String,
    },
    /// [`GroupError::UnknownExpense`].
    UnknownExpense {
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`GroupError::UnknownMember`].
    UnknownMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::UnknownTransfer`].
    UnknownTransfer {
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`GroupError::UnsupportedFormat`].
    UnsupportedFormat {
        /// The format the saved group names.
        format: i64,
    },
}

/// What an [`IdError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdFailure {
    /// [`IdError::InvalidId`].
    InvalidId,
}

/// What a [`LanguageError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageFailure {
    /// [`LanguageError::InvalidLanguage`].
    InvalidLanguage,
}

/// What a [`MemberError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberFailure {
    /// [`MemberError::EmptyFirstName`].
    EmptyFirstName,
}

/// What a [`MoneyError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoneyFailure {
    /// [`MoneyError::CurrencyMismatch`].
    CurrencyMismatch {
        /// The currency of the first amount, as its ISO 4217 code.
        left: String,
        /// The currency of the second amount, as its ISO 4217 code.
        right: String,
    },
    /// [`MoneyError::InvalidAmount`].
    InvalidAmount,
    /// [`MoneyError::InvalidExchangeRate`].
    InvalidExchangeRate,
    /// [`MoneyError::InvalidParts`].
    InvalidParts,
    /// [`MoneyError::InvalidPercentage`].
    InvalidPercentage,
    /// [`MoneyError::Overflow`].
    Overflow,
    /// [`MoneyError::TooManyDecimals`].
    TooManyDecimals {
        /// How many decimal places the currency allows.
        allowed: i64,
        /// The currency, as its ISO 4217 code.
        currency: String,
    },
    /// [`MoneyError::UnknownCurrency`].
    UnknownCurrency,
}

/// What a [`SettlementError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettlementFailure {
    /// [`SettlementError::OutOfMemory`].
    OutOfMemory {
        /// How many balances the exact solver was asked to settle.
        balances: i64,
    },
}

/// What a [`TransferError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferFailure {
    /// [`TransferError::NonPositiveAmount`].
    NonPositiveAmount,
    /// [`TransferError::SameMember`].
    SameMember,
}

/// Why an IBAN was refused: the [`IbanError`] inside
/// [`AccountFailure::InvalidIban`].
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IbanReason {
    /// [`IbanError::InvalidCharacter`].
    InvalidCharacter,
    /// [`IbanError::InvalidChecksum`].
    InvalidChecksum,
    /// [`IbanError::InvalidFormat`].
    InvalidFormat,
    /// [`IbanError::InvalidStructure`].
    InvalidStructure,
    /// [`IbanError::UnknownCountry`].
    UnknownCountry,
    /// [`IbanError::WrongLength`].
    WrongLength {
        /// The length the country's IBANs have.
        expected: i64,
        /// The length of the IBAN given.
        found: i64,
    },
}

impl FromIbanError for IbanReason {
    fn invalid_character() -> Self {
        Self::InvalidCharacter
    }

    fn invalid_checksum() -> Self {
        Self::InvalidChecksum
    }

    fn invalid_format() -> Self {
        Self::InvalidFormat
    }

    fn invalid_structure() -> Self {
        Self::InvalidStructure
    }

    fn unknown_country() -> Self {
        Self::UnknownCountry
    }

    fn wrong_length(expected: usize, found: usize) -> Self {
        Self::WrongLength {
            expected: count(expected),
            found: count(found),
        }
    }
}

impl FromIdError for EranosError {
    fn invalid_id(message: String) -> Self {
        Self::Id {
            detail: message,
            failure: IdFailure::InvalidId,
        }
    }
}

impl From<IdError> for EranosError {
    fn from(error: IdError) -> Self {
        Self::from_id_error(error)
    }
}

impl FromCalendarError for EranosError {
    fn invalid_date(message: String) -> Self {
        Self::Calendar {
            detail: message,
            failure: CalendarFailure::InvalidDate,
        }
    }

    fn invalid_time(message: String) -> Self {
        Self::Calendar {
            detail: message,
            failure: CalendarFailure::InvalidTime,
        }
    }
}

impl From<CalendarError> for EranosError {
    fn from(error: CalendarError) -> Self {
        Self::from_calendar_error(error)
    }
}

impl FromMoneyError for EranosError {
    fn currency_mismatch(left: Currency, message: String, right: Currency) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::CurrencyMismatch {
                left: left.code().to_owned(),
                right: right.code().to_owned(),
            },
        }
    }

    fn invalid_amount(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::InvalidAmount,
        }
    }

    fn invalid_exchange_rate(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::InvalidExchangeRate,
        }
    }

    fn invalid_parts(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::InvalidParts,
        }
    }

    fn invalid_percentage(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::InvalidPercentage,
        }
    }

    fn overflow(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::Overflow,
        }
    }

    fn too_many_decimals(allowed: u8, currency: Currency, message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::TooManyDecimals {
                allowed: i64::from(allowed),
                currency: currency.code().to_owned(),
            },
        }
    }

    fn unknown_currency(message: String) -> Self {
        Self::Money {
            detail: message,
            failure: MoneyFailure::UnknownCurrency,
        }
    }
}

impl From<MoneyError> for EranosError {
    fn from(error: MoneyError) -> Self {
        Self::from_money_error(&error)
    }
}

impl FromAccountError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::Account {
            detail: message,
            failure: AccountFailure::EmptyName,
        }
    }

    fn empty_username(message: String) -> Self {
        Self::Account {
            detail: message,
            failure: AccountFailure::EmptyUsername,
        }
    }

    fn invalid_iban(message: String, reason: IbanError) -> Self {
        Self::Account {
            detail: message,
            failure: AccountFailure::InvalidIban {
                reason: IbanReason::from_iban_error(&reason),
            },
        }
    }
}

impl From<AccountError> for EranosError {
    fn from(error: AccountError) -> Self {
        Self::from_account_error(&error)
    }
}

impl FromContactError for EranosError {
    fn invalid_email(message: String) -> Self {
        Self::Contact {
            detail: message,
            failure: ContactFailure::InvalidEmail,
        }
    }

    fn invalid_phone_number(message: String) -> Self {
        Self::Contact {
            detail: message,
            failure: ContactFailure::InvalidPhoneNumber,
        }
    }
}

impl From<ContactError> for EranosError {
    fn from(error: ContactError) -> Self {
        Self::from_contact_error(error)
    }
}

impl FromMemberError for EranosError {
    fn empty_first_name(message: String) -> Self {
        Self::Member {
            detail: message,
            failure: MemberFailure::EmptyFirstName,
        }
    }
}

impl From<MemberError> for EranosError {
    fn from(error: MemberError) -> Self {
        Self::from_member_error(error)
    }
}

impl FromCategoryError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::Category {
            detail: message,
            failure: CategoryFailure::EmptyName,
        }
    }
}

impl From<CategoryError> for EranosError {
    fn from(error: CategoryError) -> Self {
        Self::from_category_error(error)
    }
}

impl FromExpenseError for EranosError {
    fn amounts_do_not_add_up(
        actual: eranos::Money,
        expected: eranos::Money,
        message: String,
    ) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::AmountsDoNotAddUp {
                actual: Money::from_library(actual),
                expected: Money::from_library(expected),
            },
        }
    }

    fn currency_mismatch(expected: Currency, found: Currency, message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::CurrencyMismatch {
                expected: expected.code().to_owned(),
                found: found.code().to_owned(),
            },
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::DuplicateMember {
                member: member.to_string(),
            },
        }
    }

    fn empty_item_name(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::EmptyItemName,
        }
    }

    fn empty_name(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::EmptyName,
        }
    }

    fn empty_split(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::EmptySplit,
        }
    }

    fn items_do_not_add_up(
        actual: eranos::Money,
        expected: eranos::Money,
        message: String,
    ) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::ItemsDoNotAddUp {
                actual: Money::from_library(actual),
                expected: Money::from_library(expected),
            },
        }
    }

    fn negative_tip(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::NegativeTip,
        }
    }

    fn no_items(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::NoItems,
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::NonPositiveAmount,
        }
    }

    fn non_positive_share(member: MemberId, message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::NonPositiveShare {
                member: member.to_string(),
            },
        }
    }

    fn overflow(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::Overflow,
        }
    }

    fn percentages_do_not_add_up(message: String, total_basis_points: u64) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::PercentagesDoNotAddUp {
                total_basis_points: count(total_basis_points),
            },
        }
    }

    fn zero_quantity(message: String) -> Self {
        Self::Expense {
            detail: message,
            failure: ExpenseFailure::ZeroQuantity,
        }
    }
}

impl From<ExpenseError> for EranosError {
    fn from(error: ExpenseError) -> Self {
        Self::from_expense_error(&error)
    }
}

impl FromSettlementError for EranosError {
    fn out_of_memory(balances: usize, message: String) -> Self {
        Self::Settlement {
            detail: message,
            failure: SettlementFailure::OutOfMemory {
                balances: count(balances),
            },
        }
    }
}

impl From<SettlementError> for EranosError {
    fn from(error: SettlementError) -> Self {
        Self::from_settlement_error(&error)
    }
}

impl FromTransferError for EranosError {
    fn non_positive_amount(message: String) -> Self {
        Self::Transfer {
            detail: message,
            failure: TransferFailure::NonPositiveAmount,
        }
    }

    fn same_member(message: String) -> Self {
        Self::Transfer {
            detail: message,
            failure: TransferFailure::SameMember,
        }
    }
}

impl From<TransferError> for EranosError {
    fn from(error: TransferError) -> Self {
        Self::from_transfer_error(error)
    }
}

impl FromLanguageError for EranosError {
    fn invalid_language(message: String) -> Self {
        Self::Language {
            detail: message,
            failure: LanguageFailure::InvalidLanguage,
        }
    }
}

impl From<LanguageError> for EranosError {
    fn from(error: LanguageError) -> Self {
        Self::from_language_error(error)
    }
}

impl FromGroupError for EranosError {
    fn archived_member(member: MemberId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::ArchivedMember {
                member: member.to_string(),
            },
        }
    }

    fn built_in_category(message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::BuiltInCategory,
        }
    }

    fn busy(message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::Busy,
        }
    }

    fn duplicate_category(category: CategoryId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateCategory {
                category: category.to_string(),
            },
        }
    }

    fn duplicate_category_name(existing: CategoryId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateCategoryName {
                existing: existing.to_string(),
            },
        }
    }

    fn duplicate_exchange_rate(currency: Currency, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateExchangeRate {
                currency: currency.code().to_owned(),
            },
        }
    }

    fn duplicate_expense(expense: ExpenseId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateExpense {
                expense: expense.to_string(),
            },
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateMember {
                member: member.to_string(),
            },
        }
    }

    fn duplicate_transfer(message: String, transfer: TransferId) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::DuplicateTransfer {
                transfer: transfer.to_string(),
            },
        }
    }

    fn empty_name(message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::EmptyName,
        }
    }

    fn member_in_use(
        expenses: &[ExpenseId],
        member: MemberId,
        message: String,
        transfers: &[TransferId],
    ) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::MemberInUse {
                expenses: identifiers(expenses),
                member: member.to_string(),
                transfers: identifiers(transfers),
            },
        }
    }

    fn missing_exchange_rate(currency: Currency, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::MissingExchangeRate {
                currency: currency.code().to_owned(),
            },
        }
    }

    fn overflow(message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::Overflow,
        }
    }

    fn unexpected_exchange_rate(message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnexpectedExchangeRate,
        }
    }

    fn unknown_category(category: CategoryId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnknownCategory {
                category: category.to_string(),
            },
        }
    }

    fn unknown_expense(expense: ExpenseId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnknownExpense {
                expense: expense.to_string(),
            },
        }
    }

    fn unknown_member(member: MemberId, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnknownMember {
                member: member.to_string(),
            },
        }
    }

    fn unknown_transfer(message: String, transfer: TransferId) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnknownTransfer {
                transfer: transfer.to_string(),
            },
        }
    }

    fn unsupported_format(format: u32, message: String) -> Self {
        Self::Group {
            detail: message,
            failure: GroupFailure::UnsupportedFormat {
                format: i64::from(format),
            },
        }
    }
}

impl From<GroupError> for EranosError {
    fn from(error: GroupError) -> Self {
        Self::from_group_error(&error)
    }
}

/// A count the library holds as an unsigned integer, as the bindings carry it;
/// every count here is far below `i64::MAX`, so the fallback never applies.
fn count(number: impl TryInto<i64>) -> i64 {
    number.try_into().unwrap_or(i64::MAX)
}

/// Identifiers, as the bindings carry them.
fn identifiers(identifiers: &[impl ToString]) -> Vec<String> {
    identifiers.iter().map(ToString::to_string).collect()
}

#[cfg(test)]
mod tests;
