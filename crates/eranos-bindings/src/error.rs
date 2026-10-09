//! The one error every binding reports: a case per module of the library,
//! holding that module's failure and the library's message, and the
//! binding's own cases.

use std::fmt;

use eranos::error::{
    CalendarError, CategoryError, ContactError, ExpenseError, ExpenseGroupError, IbanError,
    IdError, LanguageError, MemberError, MoneyError, PaymentAccountError, SettlementError,
    TransferError,
};
use eranos::{CategoryId, CurrencyCode, ExpenseId, MemberId, TransferId};
use eranos_calendar::FromCalendarError;
use eranos_category::FromCategoryError;
use eranos_contact::FromContactError;
use eranos_core::FromIdError;
use eranos_expense::FromExpenseError;
use eranos_expense_group::{FromExpenseGroupError, FromLanguageError};
use eranos_member::FromMemberError;
use eranos_money::FromMoneyError;
use eranos_payment_account::{FromIbanError, FromPaymentAccountError};
use eranos_settlement::FromSettlementError;
use eranos_transfer::FromTransferError;

use crate::{CurrencyDetails, Money};

/// Why a call failed.
///
/// A case per module of the library whose rules refuse a call, holding the
/// module's failure, with a case per variant of the module's error and what
/// it holds, and the library's message; and the binding's own cases. A
/// synchronous call on wasm that finds the group locked by another call
/// fails with [`ExpenseGroupFailure::Busy`].
#[eranos_glue_macros::mirrored(error)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EranosError {
    /// An [`IdError`], about an identifier.
    Id {
        /// Which failure, with what it holds.
        failure: IdFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`PaymentAccountError`], about a payment account: its IBAN, username, or
    /// label.
    PaymentAccount {
        /// Which failure, with what it holds.
        failure: PaymentAccountFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`CalendarError`], about a date or a time of day.
    Calendar {
        /// Which failure, with what it holds.
        failure: CalendarFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`CategoryError`], about a category.
    Category {
        /// Which failure, with what it holds.
        failure: CategoryFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`ContactError`], about an email address or a phone number.
    Contact {
        /// Which failure, with what it holds.
        failure: ContactFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`MoneyError`], about an amount, a currency, a rate, or a ratio.
    Money {
        /// Which failure, with what it holds.
        failure: MoneyFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`MemberError`], about a member.
    Member {
        /// Which failure, with what it holds.
        failure: MemberFailure,
        /// The library's message.
        detail: String,
    },
    /// An [`ExpenseError`], about an expense: its amount, split, items, or tip.
    Expense {
        /// Which failure, with what it holds.
        failure: ExpenseFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`SettlementError`], about a settlement.
    Settlement {
        /// Which failure, with what it holds.
        failure: SettlementFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`TransferError`], about a transfer.
    Transfer {
        /// Which failure, with what it holds.
        failure: TransferFailure,
        /// The library's message.
        detail: String,
    },
    /// An [`ExpenseGroupError`], about how an entry fits the group, or the group's
    /// lock.
    ExpenseGroup {
        /// Which failure, with what it holds.
        failure: ExpenseGroupFailure,
        /// The library's message.
        detail: String,
    },
    /// A [`LanguageError`], about a language tag.
    Language {
        /// Which failure, with what it holds.
        failure: LanguageFailure,
        /// The library's message.
        detail: String,
    },
    /// A record whose derived text is not what the library derives from
    /// its other fields, so it names no value the library writes.
    MismatchedRecord {
        /// The field whose text is not the library's.
        field: MismatchedField,
        /// What went wrong.
        detail: String,
    },
    /// A saved group that is not JSON of the saved form, at `line` and
    /// `column`.
    InvalidJson {
        /// The line of the first character the reader refused, from 1.
        line: i64,
        /// The column of the first character the reader refused, from 1.
        column: i64,
        /// What went wrong, and where.
        detail: String,
    },
    /// The call was cancelled through its
    /// [`Cancellation`](crate::Cancellation), before it changed anything.
    Cancelled {
        /// What went wrong.
        detail: String,
    },
    /// A read or write handle was released, so it reaches the group no
    /// more.
    Released {
        /// What went wrong.
        detail: String,
    },
}

impl EranosError {
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
            line,
            column,
            detail: format!("the saved group {fault}, at line {line}, column {column}"),
        }
    }

    /// The error of the case `Cancelled`.
    pub(crate) fn cancelled() -> Self {
        Self::Cancelled {
            detail: "the call was cancelled".to_owned(),
        }
    }

    /// The error of the case `Released`.
    pub(crate) fn released() -> Self {
        Self::Released {
            detail: "the handle was released".to_owned(),
        }
    }

    /// The error of the case `MismatchedRecord`, for `field`.
    pub(crate) fn mismatched_record(field: MismatchedField) -> Self {
        let detail = match field {
            MismatchedField::IbanCountryCode => {
                "the IBAN's country code is not the one the library derives from it"
            }
            MismatchedField::PhoneInternational => {
                "the phone number's international form is not the one the library writes for it"
            }
            MismatchedField::CurrencyDetails => {
                "the currency's details are not the library's for its code"
            }
        };
        Self::MismatchedRecord {
            field,
            detail: detail.to_owned(),
        }
    }

    /// What went wrong, as the library words it.
    #[must_use]
    pub fn detail(&self) -> &str {
        match self {
            Self::Id { detail, .. }
            | Self::PaymentAccount { detail, .. }
            | Self::Calendar { detail, .. }
            | Self::Category { detail, .. }
            | Self::Contact { detail, .. }
            | Self::Money { detail, .. }
            | Self::Member { detail, .. }
            | Self::Expense { detail, .. }
            | Self::Settlement { detail, .. }
            | Self::Transfer { detail, .. }
            | Self::ExpenseGroup { detail, .. }
            | Self::Language { detail, .. }
            | Self::MismatchedRecord { detail, .. }
            | Self::InvalidJson { detail, .. }
            | Self::Cancelled { detail, .. }
            | Self::Released { detail, .. } => detail,
        }
    }
}

impl fmt::Display for EranosError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.detail())
    }
}

impl std::error::Error for EranosError {}

/// What an [`PaymentAccountError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentAccountFailure {
    /// [`PaymentAccountError::EmptyName`].
    EmptyName,
    /// [`PaymentAccountError::InvalidIban`].
    InvalidIban {
        /// Why the IBAN was refused.
        reason: IbanReason,
    },
    /// [`PaymentAccountError::EmptyUsername`].
    EmptyUsername,
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
    /// [`ExpenseError::EmptyName`].
    EmptyName,
    /// [`ExpenseError::NonPositiveAmount`].
    NonPositiveAmount,
    /// [`ExpenseError::CurrencyMismatch`].
    CurrencyMismatch {
        /// The currency the entry must be in.
        expected: CurrencyDetails,
        /// The currency the part is in.
        found: CurrencyDetails,
    },
    /// [`ExpenseError::EmptySplit`].
    EmptySplit,
    /// [`ExpenseError::DuplicateMember`].
    DuplicateMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseError::NonPositiveShare`].
    NonPositiveShare {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseError::AmountsDoNotAddUp`].
    AmountsDoNotAddUp {
        /// What the parts must add up to.
        expected: Money,
        /// What the parts add up to.
        actual: Money,
    },
    /// [`ExpenseError::PercentagesDoNotAddUp`].
    PercentagesDoNotAddUp {
        /// What the percentages add up to, in basis points.
        total_basis_points: i64,
    },
    /// [`ExpenseError::NoItems`].
    NoItems,
    /// [`ExpenseError::EmptyItemName`].
    EmptyItemName,
    /// [`ExpenseError::ItemsDoNotAddUp`].
    ItemsDoNotAddUp {
        /// What the parts must add up to.
        expected: Money,
        /// What the parts add up to.
        actual: Money,
    },
    /// [`ExpenseError::NegativeDiscount`].
    NegativeDiscount,
    /// [`ExpenseError::DiscountAboveHundredPercent`].
    DiscountAboveHundredPercent,
    /// [`ExpenseError::DiscountAbovePrice`].
    DiscountAbovePrice,
    /// [`ExpenseError::NegativeTip`].
    NegativeTip,
    /// [`ExpenseError::Overflow`].
    Overflow,
}

/// What a [`ExpenseGroupError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpenseGroupFailure {
    /// [`ExpenseGroupError::UnsupportedFormat`].
    UnsupportedFormat {
        /// The format the saved group names.
        format: i64,
    },
    /// [`ExpenseGroupError::EmptyName`].
    EmptyName,
    /// [`ExpenseGroupError::MissingExchangeRate`].
    MissingExchangeRate {
        /// The currency.
        currency: CurrencyDetails,
    },
    /// [`ExpenseGroupError::UnexpectedExchangeRate`].
    UnexpectedExchangeRate,
    /// [`ExpenseGroupError::DuplicateExchangeRate`].
    DuplicateExchangeRate {
        /// The currency.
        currency: CurrencyDetails,
    },
    /// [`ExpenseGroupError::DuplicateMember`].
    DuplicateMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseGroupError::UnknownMember`].
    UnknownMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseGroupError::ArchivedMember`].
    ArchivedMember {
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseGroupError::MemberInUse`].
    MemberInUse {
        /// The member, as their identifier.
        member: String,
        /// The expenses that reference the member, as identifiers.
        expenses: Vec<String>,
        /// The transfers that reference the member, as identifiers.
        transfers: Vec<String>,
    },
    /// [`ExpenseGroupError::DuplicateCategory`].
    DuplicateCategory {
        /// The category, as its identifier.
        category: String,
    },
    /// [`ExpenseGroupError::DuplicateCategoryName`].
    DuplicateCategoryName {
        /// The category that has the name, as its identifier.
        existing: String,
    },
    /// [`ExpenseGroupError::UnknownCategory`].
    UnknownCategory {
        /// The category, as its identifier.
        category: String,
    },
    /// [`ExpenseGroupError::BuiltInCategory`].
    BuiltInCategory,
    /// [`ExpenseGroupError::DuplicateExpense`].
    DuplicateExpense {
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`ExpenseGroupError::UnknownExpense`].
    UnknownExpense {
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`ExpenseGroupError::DuplicateTransfer`].
    DuplicateTransfer {
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`ExpenseGroupError::UnknownTransfer`].
    UnknownTransfer {
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`ExpenseGroupError::Busy`].
    Busy,
    /// [`ExpenseGroupError::Overflow`].
    Overflow,
}

/// What an [`IdError`] reports: a case per variant, with
/// what the variant holds.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdFailure {
    /// [`IdError::Empty`].
    Empty,
    /// [`IdError::TooLong`].
    TooLong,
    /// [`IdError::ControlCharacter`].
    ControlCharacter,
    /// [`IdError::SurroundingWhitespace`].
    SurroundingWhitespace,
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
    /// [`MoneyError::InvalidAmount`].
    InvalidAmount,
    /// [`MoneyError::TooManyDecimals`].
    TooManyDecimals {
        /// The currency.
        currency: CurrencyDetails,
        /// How many decimal places the currency allows.
        allowed: i64,
    },
    /// [`MoneyError::CurrencyMismatch`].
    CurrencyMismatch {
        /// The currency of the first amount.
        left: CurrencyDetails,
        /// The currency of the second amount.
        right: CurrencyDetails,
    },
    /// [`MoneyError::UnknownCurrency`].
    UnknownCurrency,
    /// [`MoneyError::InvalidExchangeRate`].
    InvalidExchangeRate,
    /// [`MoneyError::InvalidParts`].
    InvalidParts,
    /// [`MoneyError::InvalidPercentage`].
    InvalidPercentage,
    /// [`MoneyError::InvalidQuantity`].
    InvalidQuantity,
    /// [`MoneyError::NotFinite`].
    NotFinite,
    /// [`MoneyError::Overflow`].
    Overflow,
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
    /// [`TransferError::SameMember`].
    SameMember,
    /// [`TransferError::NonPositiveAmount`].
    NonPositiveAmount,
}

/// The field of a record that the library derives from the record's other
/// fields, and which a record gave another way: what
/// [`EranosError::MismatchedRecord`] names.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MismatchedField {
    /// The country code of an IBAN, which the library derives from the
    /// IBAN.
    IbanCountryCode,
    /// The international form of a phone number, which the library writes
    /// from its E.164 form.
    PhoneInternational,
    /// The name, numeric code, or decimal places of a currency, which the
    /// library holds for its code; a code in lower case names no currency
    /// the library lists either.
    CurrencyDetails,
}

/// Why an IBAN was refused: the [`IbanError`] inside
/// [`PaymentAccountFailure::InvalidIban`].
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IbanReason {
    /// [`IbanError::InvalidCharacter`].
    InvalidCharacter,
    /// [`IbanError::InvalidFormat`].
    InvalidFormat,
    /// [`IbanError::UnknownCountry`].
    UnknownCountry,
    /// [`IbanError::WrongLength`].
    WrongLength {
        /// The length the country's IBANs have.
        expected: i64,
        /// The length of the IBAN given.
        found: i64,
    },
    /// [`IbanError::InvalidStructure`].
    InvalidStructure,
    /// [`IbanError::InvalidChecksum`].
    InvalidChecksum,
}

impl FromIbanError for IbanReason {
    fn invalid_character() -> Self {
        Self::InvalidCharacter
    }

    fn invalid_format() -> Self {
        Self::InvalidFormat
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

    fn invalid_structure() -> Self {
        Self::InvalidStructure
    }

    fn invalid_checksum() -> Self {
        Self::InvalidChecksum
    }
}

impl FromIdError for EranosError {
    fn empty(message: String) -> Self {
        Self::Id {
            failure: IdFailure::Empty,
            detail: message,
        }
    }

    fn too_long(message: String) -> Self {
        Self::Id {
            failure: IdFailure::TooLong,
            detail: message,
        }
    }

    fn control_character(message: String) -> Self {
        Self::Id {
            failure: IdFailure::ControlCharacter,
            detail: message,
        }
    }

    fn surrounding_whitespace(message: String) -> Self {
        Self::Id {
            failure: IdFailure::SurroundingWhitespace,
            detail: message,
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
            failure: CalendarFailure::InvalidDate,
            detail: message,
        }
    }

    fn invalid_time(message: String) -> Self {
        Self::Calendar {
            failure: CalendarFailure::InvalidTime,
            detail: message,
        }
    }
}

impl From<CalendarError> for EranosError {
    fn from(error: CalendarError) -> Self {
        Self::from_calendar_error(error)
    }
}

impl FromMoneyError for EranosError {
    fn invalid_amount(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::InvalidAmount,
            detail: message,
        }
    }

    fn too_many_decimals(currency: CurrencyCode, allowed: u8, message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::TooManyDecimals {
                currency: CurrencyDetails::from_library(currency),
                allowed: i64::from(allowed),
            },
            detail: message,
        }
    }

    fn currency_mismatch(left: CurrencyCode, right: CurrencyCode, message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::CurrencyMismatch {
                left: CurrencyDetails::from_library(left),
                right: CurrencyDetails::from_library(right),
            },
            detail: message,
        }
    }

    fn unknown_currency(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::UnknownCurrency,
            detail: message,
        }
    }

    fn invalid_exchange_rate(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::InvalidExchangeRate,
            detail: message,
        }
    }

    fn invalid_parts(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::InvalidParts,
            detail: message,
        }
    }

    fn invalid_percentage(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::InvalidPercentage,
            detail: message,
        }
    }

    fn invalid_quantity(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::InvalidQuantity,
            detail: message,
        }
    }

    fn not_finite(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::NotFinite,
            detail: message,
        }
    }

    fn overflow(message: String) -> Self {
        Self::Money {
            failure: MoneyFailure::Overflow,
            detail: message,
        }
    }
}

impl From<MoneyError> for EranosError {
    fn from(error: MoneyError) -> Self {
        Self::from_money_error(&error)
    }
}

impl FromPaymentAccountError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::PaymentAccount {
            failure: PaymentAccountFailure::EmptyName,
            detail: message,
        }
    }

    fn invalid_iban(reason: IbanError, message: String) -> Self {
        Self::PaymentAccount {
            failure: PaymentAccountFailure::InvalidIban {
                reason: IbanReason::from_iban_error(&reason),
            },
            detail: message,
        }
    }

    fn empty_username(message: String) -> Self {
        Self::PaymentAccount {
            failure: PaymentAccountFailure::EmptyUsername,
            detail: message,
        }
    }
}

impl From<PaymentAccountError> for EranosError {
    fn from(error: PaymentAccountError) -> Self {
        Self::from_payment_account_error(&error)
    }
}

impl FromContactError for EranosError {
    fn invalid_email(message: String) -> Self {
        Self::Contact {
            failure: ContactFailure::InvalidEmail,
            detail: message,
        }
    }

    fn invalid_phone_number(message: String) -> Self {
        Self::Contact {
            failure: ContactFailure::InvalidPhoneNumber,
            detail: message,
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
            failure: MemberFailure::EmptyFirstName,
            detail: message,
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
            failure: CategoryFailure::EmptyName,
            detail: message,
        }
    }
}

impl From<CategoryError> for EranosError {
    fn from(error: CategoryError) -> Self {
        Self::from_category_error(error)
    }
}

impl FromExpenseError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::EmptyName,
            detail: message,
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::NonPositiveAmount,
            detail: message,
        }
    }

    fn currency_mismatch(expected: CurrencyCode, found: CurrencyCode, message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::CurrencyMismatch {
                expected: CurrencyDetails::from_library(expected),
                found: CurrencyDetails::from_library(found),
            },
            detail: message,
        }
    }

    fn empty_split(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::EmptySplit,
            detail: message,
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::DuplicateMember {
                member: member.to_string(),
            },
            detail: message,
        }
    }

    fn non_positive_share(member: MemberId, message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::NonPositiveShare {
                member: member.to_string(),
            },
            detail: message,
        }
    }

    fn amounts_do_not_add_up(
        expected: eranos::Money,
        actual: eranos::Money,
        message: String,
    ) -> Self {
        Self::Expense {
            failure: ExpenseFailure::AmountsDoNotAddUp {
                expected: Money::from_library(expected),
                actual: Money::from_library(actual),
            },
            detail: message,
        }
    }

    fn percentages_do_not_add_up(total_basis_points: u64, message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::PercentagesDoNotAddUp {
                total_basis_points: count(total_basis_points),
            },
            detail: message,
        }
    }

    fn no_items(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::NoItems,
            detail: message,
        }
    }

    fn empty_item_name(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::EmptyItemName,
            detail: message,
        }
    }

    fn items_do_not_add_up(
        expected: eranos::Money,
        actual: eranos::Money,
        message: String,
    ) -> Self {
        Self::Expense {
            failure: ExpenseFailure::ItemsDoNotAddUp {
                expected: Money::from_library(expected),
                actual: Money::from_library(actual),
            },
            detail: message,
        }
    }

    fn negative_discount(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::NegativeDiscount,
            detail: message,
        }
    }

    fn discount_above_hundred_percent(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::DiscountAboveHundredPercent,
            detail: message,
        }
    }

    fn discount_above_price(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::DiscountAbovePrice,
            detail: message,
        }
    }

    fn negative_tip(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::NegativeTip,
            detail: message,
        }
    }

    fn overflow(message: String) -> Self {
        Self::Expense {
            failure: ExpenseFailure::Overflow,
            detail: message,
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
            failure: SettlementFailure::OutOfMemory {
                balances: count(balances),
            },
            detail: message,
        }
    }
}

impl From<SettlementError> for EranosError {
    fn from(error: SettlementError) -> Self {
        Self::from_settlement_error(&error)
    }
}

impl FromTransferError for EranosError {
    fn same_member(message: String) -> Self {
        Self::Transfer {
            failure: TransferFailure::SameMember,
            detail: message,
        }
    }

    fn non_positive_amount(message: String) -> Self {
        Self::Transfer {
            failure: TransferFailure::NonPositiveAmount,
            detail: message,
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
            failure: LanguageFailure::InvalidLanguage,
            detail: message,
        }
    }
}

impl From<LanguageError> for EranosError {
    fn from(error: LanguageError) -> Self {
        Self::from_language_error(error)
    }
}

impl FromExpenseGroupError for EranosError {
    fn unsupported_format(format: u32, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnsupportedFormat {
                format: i64::from(format),
            },
            detail: message,
        }
    }

    fn empty_name(message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::EmptyName,
            detail: message,
        }
    }

    fn missing_exchange_rate(currency: CurrencyCode, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::MissingExchangeRate {
                currency: CurrencyDetails::from_library(currency),
            },
            detail: message,
        }
    }

    fn unexpected_exchange_rate(message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnexpectedExchangeRate,
            detail: message,
        }
    }

    fn duplicate_exchange_rate(currency: CurrencyCode, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateExchangeRate {
                currency: CurrencyDetails::from_library(currency),
            },
            detail: message,
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateMember {
                member: member.to_string(),
            },
            detail: message,
        }
    }

    fn unknown_member(member: MemberId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownMember {
                member: member.to_string(),
            },
            detail: message,
        }
    }

    fn archived_member(member: MemberId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::ArchivedMember {
                member: member.to_string(),
            },
            detail: message,
        }
    }

    fn member_in_use(
        member: MemberId,
        expenses: &[ExpenseId],
        transfers: &[TransferId],
        message: String,
    ) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::MemberInUse {
                member: member.to_string(),
                expenses: identifiers(expenses),
                transfers: identifiers(transfers),
            },
            detail: message,
        }
    }

    fn duplicate_category(category: CategoryId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateCategory {
                category: category.to_string(),
            },
            detail: message,
        }
    }

    fn duplicate_category_name(existing: CategoryId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateCategoryName {
                existing: existing.to_string(),
            },
            detail: message,
        }
    }

    fn unknown_category(category: CategoryId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownCategory {
                category: category.to_string(),
            },
            detail: message,
        }
    }

    fn built_in_category(message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::BuiltInCategory,
            detail: message,
        }
    }

    fn duplicate_expense(expense: ExpenseId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateExpense {
                expense: expense.to_string(),
            },
            detail: message,
        }
    }

    fn unknown_expense(expense: ExpenseId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownExpense {
                expense: expense.to_string(),
            },
            detail: message,
        }
    }

    fn duplicate_transfer(transfer: TransferId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::DuplicateTransfer {
                transfer: transfer.to_string(),
            },
            detail: message,
        }
    }

    fn unknown_transfer(transfer: TransferId, message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownTransfer {
                transfer: transfer.to_string(),
            },
            detail: message,
        }
    }

    fn busy(message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            detail: message,
        }
    }

    fn overflow(message: String) -> Self {
        Self::ExpenseGroup {
            failure: ExpenseGroupFailure::Overflow,
            detail: message,
        }
    }
}

impl From<ExpenseGroupError> for EranosError {
    fn from(error: ExpenseGroupError) -> Self {
        Self::from_expense_group_error(&error)
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
