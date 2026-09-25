//! The one error every binding reports: a case per failure of the library,
//! holding what the failure holds and the library's message, and the
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
/// A case per failure of the library, named after it, with its crate's name in
/// front where two crates share a name, holding what the failure holds and the
/// library's message; and the binding's own cases. `Busy` is a synchronous
/// call on wasm that found the group locked by another call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EranosError {
    /// [`AccountError::EmptyName`].
    AccountEmptyName {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::AmountsDoNotAddUp`].
    AmountsDoNotAddUp {
        /// What the parts add up to.
        actual: Money,
        /// The library's message.
        detail: String,
        /// What the parts must add up to.
        expected: Money,
    },
    /// [`GroupError::ArchivedMember`].
    ArchivedMember {
        /// The library's message.
        detail: String,
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::BuiltInCategory`].
    BuiltInCategory {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::Busy`].
    Busy {
        /// The library's message.
        detail: String,
    },
    /// The call was cancelled through its
    /// [`Cancellation`](crate::Cancellation), before it changed anything.
    Cancelled {
        /// What went wrong.
        detail: String,
    },
    /// [`CategoryError::EmptyName`].
    CategoryEmptyName {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::DuplicateCategory`].
    DuplicateCategory {
        /// The category, as its identifier.
        category: String,
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::DuplicateCategoryName`].
    DuplicateCategoryName {
        /// The library's message.
        detail: String,
        /// The category that has the name, as its identifier.
        existing: String,
    },
    /// [`GroupError::DuplicateExchangeRate`].
    DuplicateExchangeRate {
        /// The currency, as its ISO 4217 code.
        currency: String,
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::DuplicateExpense`].
    DuplicateExpense {
        /// The library's message.
        detail: String,
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`GroupError::DuplicateTransfer`].
    DuplicateTransfer {
        /// The library's message.
        detail: String,
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`MemberError::EmptyFirstName`].
    EmptyFirstName {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::EmptyItemName`].
    EmptyItemName {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::EmptySplit`].
    EmptySplit {
        /// The library's message.
        detail: String,
    },
    /// [`AccountError::EmptyUsername`].
    EmptyUsername {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::CurrencyMismatch`].
    ExpenseCurrencyMismatch {
        /// The library's message.
        detail: String,
        /// The currency the entry must be in, as its ISO 4217 code.
        expected: String,
        /// The currency the part is in, as its ISO 4217 code.
        found: String,
    },
    /// [`ExpenseError::DuplicateMember`].
    ExpenseDuplicateMember {
        /// The library's message.
        detail: String,
        /// The member, as their identifier.
        member: String,
    },
    /// [`ExpenseError::EmptyName`].
    ExpenseEmptyName {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::NonPositiveAmount`].
    ExpenseNonPositiveAmount {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::Overflow`].
    ExpenseOverflow {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::DuplicateMember`].
    GroupDuplicateMember {
        /// The library's message.
        detail: String,
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::EmptyName`].
    GroupEmptyName {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::Overflow`].
    GroupOverflow {
        /// The library's message.
        detail: String,
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
    /// [`MoneyError::InvalidAmount`].
    InvalidAmount {
        /// The library's message.
        detail: String,
    },
    /// [`CalendarError::InvalidDate`].
    InvalidDate {
        /// The library's message.
        detail: String,
    },
    /// [`ContactError::InvalidEmail`].
    InvalidEmail {
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::InvalidExchangeRate`].
    InvalidExchangeRate {
        /// The library's message.
        detail: String,
    },
    /// [`AccountError::InvalidIban`].
    InvalidIban {
        /// The library's message.
        detail: String,
        /// Why the IBAN was refused.
        reason: IbanReason,
    },
    /// [`IdError::InvalidId`].
    InvalidId {
        /// The library's message.
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
    /// [`LanguageError::InvalidLanguage`].
    InvalidLanguage {
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::InvalidParts`].
    InvalidParts {
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::InvalidPercentage`].
    InvalidPercentage {
        /// The library's message.
        detail: String,
    },
    /// [`ContactError::InvalidPhoneNumber`].
    InvalidPhoneNumber {
        /// The library's message.
        detail: String,
    },
    /// [`CalendarError::InvalidTime`].
    InvalidTime {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::ItemsDoNotAddUp`].
    ItemsDoNotAddUp {
        /// What the parts add up to.
        actual: Money,
        /// The library's message.
        detail: String,
        /// What the parts must add up to.
        expected: Money,
    },
    /// [`GroupError::MemberInUse`].
    MemberInUse {
        /// The library's message.
        detail: String,
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
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::CurrencyMismatch`].
    MoneyCurrencyMismatch {
        /// The library's message.
        detail: String,
        /// The currency of the first amount, as its ISO 4217 code.
        left: String,
        /// The currency of the second amount, as its ISO 4217 code.
        right: String,
    },
    /// [`MoneyError::Overflow`].
    MoneyOverflow {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::NegativeTip`].
    NegativeTip {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::NoItems`].
    NoItems {
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::NonPositiveShare`].
    NonPositiveShare {
        /// The library's message.
        detail: String,
        /// The member, as their identifier.
        member: String,
    },
    /// [`SettlementError::OutOfMemory`].
    OutOfMemory {
        /// How many balances the exact solver was asked to settle.
        balances: i64,
        /// The library's message.
        detail: String,
    },
    /// [`ExpenseError::PercentagesDoNotAddUp`].
    PercentagesDoNotAddUp {
        /// The library's message.
        detail: String,
        /// What the percentages add up to, in basis points.
        total_basis_points: i64,
    },
    /// The read handle was released, so it reads the group no more.
    Released {
        /// What went wrong.
        detail: String,
    },
    /// [`TransferError::SameMember`].
    SameMember {
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::TooManyDecimals`].
    TooManyDecimals {
        /// How many decimal places the currency allows.
        allowed: i64,
        /// The currency, as its ISO 4217 code.
        currency: String,
        /// The library's message.
        detail: String,
    },
    /// [`TransferError::NonPositiveAmount`].
    TransferNonPositiveAmount {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::UnexpectedExchangeRate`].
    UnexpectedExchangeRate {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::UnknownCategory`].
    UnknownCategory {
        /// The category, as its identifier.
        category: String,
        /// The library's message.
        detail: String,
    },
    /// [`MoneyError::UnknownCurrency`].
    UnknownCurrency {
        /// The library's message.
        detail: String,
    },
    /// [`GroupError::UnknownExpense`].
    UnknownExpense {
        /// The library's message.
        detail: String,
        /// The expense, as its identifier.
        expense: String,
    },
    /// [`GroupError::UnknownMember`].
    UnknownMember {
        /// The library's message.
        detail: String,
        /// The member, as their identifier.
        member: String,
    },
    /// [`GroupError::UnknownTransfer`].
    UnknownTransfer {
        /// The library's message.
        detail: String,
        /// The transfer, as its identifier.
        transfer: String,
    },
    /// [`GroupError::UnsupportedFormat`].
    UnsupportedFormat {
        /// The library's message.
        detail: String,
        /// The format the saved group names.
        format: i64,
    },
    /// [`ExpenseError::ZeroQuantity`].
    ZeroQuantity {
        /// The library's message.
        detail: String,
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
            Self::AccountEmptyName { detail, .. }
            | Self::AmountsDoNotAddUp { detail, .. }
            | Self::ArchivedMember { detail, .. }
            | Self::BuiltInCategory { detail, .. }
            | Self::Busy { detail, .. }
            | Self::Cancelled { detail, .. }
            | Self::CategoryEmptyName { detail, .. }
            | Self::DuplicateCategory { detail, .. }
            | Self::DuplicateCategoryName { detail, .. }
            | Self::DuplicateExchangeRate { detail, .. }
            | Self::DuplicateExpense { detail, .. }
            | Self::DuplicateTransfer { detail, .. }
            | Self::EmptyFirstName { detail, .. }
            | Self::EmptyItemName { detail, .. }
            | Self::EmptySplit { detail, .. }
            | Self::EmptyUsername { detail, .. }
            | Self::ExpenseCurrencyMismatch { detail, .. }
            | Self::ExpenseDuplicateMember { detail, .. }
            | Self::ExpenseEmptyName { detail, .. }
            | Self::ExpenseNonPositiveAmount { detail, .. }
            | Self::ExpenseOverflow { detail, .. }
            | Self::GroupDuplicateMember { detail, .. }
            | Self::GroupEmptyName { detail, .. }
            | Self::GroupOverflow { detail, .. }
            | Self::IdentifierGiven { detail, .. }
            | Self::IdentifierMissing { detail, .. }
            | Self::InvalidAmount { detail, .. }
            | Self::InvalidDate { detail, .. }
            | Self::InvalidEmail { detail, .. }
            | Self::InvalidExchangeRate { detail, .. }
            | Self::InvalidIban { detail, .. }
            | Self::InvalidId { detail, .. }
            | Self::InvalidJson { detail, .. }
            | Self::InvalidLanguage { detail, .. }
            | Self::InvalidParts { detail, .. }
            | Self::InvalidPercentage { detail, .. }
            | Self::InvalidPhoneNumber { detail, .. }
            | Self::InvalidTime { detail, .. }
            | Self::ItemsDoNotAddUp { detail, .. }
            | Self::MemberInUse { detail, .. }
            | Self::MissingExchangeRate { detail, .. }
            | Self::MoneyCurrencyMismatch { detail, .. }
            | Self::MoneyOverflow { detail, .. }
            | Self::NegativeTip { detail, .. }
            | Self::NoItems { detail, .. }
            | Self::NonPositiveShare { detail, .. }
            | Self::OutOfMemory { detail, .. }
            | Self::PercentagesDoNotAddUp { detail, .. }
            | Self::Released { detail, .. }
            | Self::SameMember { detail, .. }
            | Self::TooManyDecimals { detail, .. }
            | Self::TransferNonPositiveAmount { detail, .. }
            | Self::UnexpectedExchangeRate { detail, .. }
            | Self::UnknownCategory { detail, .. }
            | Self::UnknownCurrency { detail, .. }
            | Self::UnknownExpense { detail, .. }
            | Self::UnknownMember { detail, .. }
            | Self::UnknownTransfer { detail, .. }
            | Self::UnsupportedFormat { detail, .. }
            | Self::ZeroQuantity { detail, .. } => detail,
        }
    }
}

impl fmt::Display for EranosError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.detail())
    }
}

impl std::error::Error for EranosError {}

/// Why an IBAN was refused: the [`IbanError`] inside
/// [`EranosError::InvalidIban`].
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
        Self::InvalidId { detail: message }
    }
}

impl From<IdError> for EranosError {
    fn from(error: IdError) -> Self {
        Self::from_id_error(error)
    }
}

impl FromCalendarError for EranosError {
    fn invalid_date(message: String) -> Self {
        Self::InvalidDate { detail: message }
    }

    fn invalid_time(message: String) -> Self {
        Self::InvalidTime { detail: message }
    }
}

impl From<CalendarError> for EranosError {
    fn from(error: CalendarError) -> Self {
        Self::from_calendar_error(error)
    }
}

impl FromMoneyError for EranosError {
    fn currency_mismatch(left: Currency, message: String, right: Currency) -> Self {
        Self::MoneyCurrencyMismatch {
            detail: message,
            left: left.code().to_owned(),
            right: right.code().to_owned(),
        }
    }

    fn invalid_amount(message: String) -> Self {
        Self::InvalidAmount { detail: message }
    }

    fn invalid_exchange_rate(message: String) -> Self {
        Self::InvalidExchangeRate { detail: message }
    }

    fn invalid_parts(message: String) -> Self {
        Self::InvalidParts { detail: message }
    }

    fn invalid_percentage(message: String) -> Self {
        Self::InvalidPercentage { detail: message }
    }

    fn overflow(message: String) -> Self {
        Self::MoneyOverflow { detail: message }
    }

    fn too_many_decimals(allowed: u8, currency: Currency, message: String) -> Self {
        Self::TooManyDecimals {
            allowed: i64::from(allowed),
            currency: currency.code().to_owned(),
            detail: message,
        }
    }

    fn unknown_currency(message: String) -> Self {
        Self::UnknownCurrency { detail: message }
    }
}

impl From<MoneyError> for EranosError {
    fn from(error: MoneyError) -> Self {
        Self::from_money_error(&error)
    }
}

impl FromAccountError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::AccountEmptyName { detail: message }
    }

    fn empty_username(message: String) -> Self {
        Self::EmptyUsername { detail: message }
    }

    fn invalid_iban(message: String, reason: IbanError) -> Self {
        Self::InvalidIban {
            detail: message,
            reason: IbanReason::from_iban_error(&reason),
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
        Self::InvalidEmail { detail: message }
    }

    fn invalid_phone_number(message: String) -> Self {
        Self::InvalidPhoneNumber { detail: message }
    }
}

impl From<ContactError> for EranosError {
    fn from(error: ContactError) -> Self {
        Self::from_contact_error(error)
    }
}

impl FromMemberError for EranosError {
    fn empty_first_name(message: String) -> Self {
        Self::EmptyFirstName { detail: message }
    }
}

impl From<MemberError> for EranosError {
    fn from(error: MemberError) -> Self {
        Self::from_member_error(error)
    }
}

impl FromCategoryError for EranosError {
    fn empty_name(message: String) -> Self {
        Self::CategoryEmptyName { detail: message }
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
        Self::AmountsDoNotAddUp {
            actual: Money::from_library(actual),
            detail: message,
            expected: Money::from_library(expected),
        }
    }

    fn currency_mismatch(expected: Currency, found: Currency, message: String) -> Self {
        Self::ExpenseCurrencyMismatch {
            detail: message,
            expected: expected.code().to_owned(),
            found: found.code().to_owned(),
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::ExpenseDuplicateMember {
            detail: message,
            member: member.to_string(),
        }
    }

    fn empty_item_name(message: String) -> Self {
        Self::EmptyItemName { detail: message }
    }

    fn empty_name(message: String) -> Self {
        Self::ExpenseEmptyName { detail: message }
    }

    fn empty_split(message: String) -> Self {
        Self::EmptySplit { detail: message }
    }

    fn items_do_not_add_up(
        actual: eranos::Money,
        expected: eranos::Money,
        message: String,
    ) -> Self {
        Self::ItemsDoNotAddUp {
            actual: Money::from_library(actual),
            detail: message,
            expected: Money::from_library(expected),
        }
    }

    fn negative_tip(message: String) -> Self {
        Self::NegativeTip { detail: message }
    }

    fn no_items(message: String) -> Self {
        Self::NoItems { detail: message }
    }

    fn non_positive_amount(message: String) -> Self {
        Self::ExpenseNonPositiveAmount { detail: message }
    }

    fn non_positive_share(member: MemberId, message: String) -> Self {
        Self::NonPositiveShare {
            detail: message,
            member: member.to_string(),
        }
    }

    fn overflow(message: String) -> Self {
        Self::ExpenseOverflow { detail: message }
    }

    fn percentages_do_not_add_up(message: String, total_basis_points: u64) -> Self {
        Self::PercentagesDoNotAddUp {
            detail: message,
            total_basis_points: count(total_basis_points),
        }
    }

    fn zero_quantity(message: String) -> Self {
        Self::ZeroQuantity { detail: message }
    }
}

impl From<ExpenseError> for EranosError {
    fn from(error: ExpenseError) -> Self {
        Self::from_expense_error(&error)
    }
}

impl FromSettlementError for EranosError {
    fn out_of_memory(balances: usize, message: String) -> Self {
        Self::OutOfMemory {
            balances: count(balances),
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
    fn non_positive_amount(message: String) -> Self {
        Self::TransferNonPositiveAmount { detail: message }
    }

    fn same_member(message: String) -> Self {
        Self::SameMember { detail: message }
    }
}

impl From<TransferError> for EranosError {
    fn from(error: TransferError) -> Self {
        Self::from_transfer_error(error)
    }
}

impl FromLanguageError for EranosError {
    fn invalid_language(message: String) -> Self {
        Self::InvalidLanguage { detail: message }
    }
}

impl From<LanguageError> for EranosError {
    fn from(error: LanguageError) -> Self {
        Self::from_language_error(error)
    }
}

impl FromGroupError for EranosError {
    fn archived_member(member: MemberId, message: String) -> Self {
        Self::ArchivedMember {
            detail: message,
            member: member.to_string(),
        }
    }

    fn built_in_category(message: String) -> Self {
        Self::BuiltInCategory { detail: message }
    }

    fn busy(message: String) -> Self {
        Self::Busy { detail: message }
    }

    fn duplicate_category(category: CategoryId, message: String) -> Self {
        Self::DuplicateCategory {
            category: category.to_string(),
            detail: message,
        }
    }

    fn duplicate_category_name(existing: CategoryId, message: String) -> Self {
        Self::DuplicateCategoryName {
            detail: message,
            existing: existing.to_string(),
        }
    }

    fn duplicate_exchange_rate(currency: Currency, message: String) -> Self {
        Self::DuplicateExchangeRate {
            currency: currency.code().to_owned(),
            detail: message,
        }
    }

    fn duplicate_expense(expense: ExpenseId, message: String) -> Self {
        Self::DuplicateExpense {
            detail: message,
            expense: expense.to_string(),
        }
    }

    fn duplicate_member(member: MemberId, message: String) -> Self {
        Self::GroupDuplicateMember {
            detail: message,
            member: member.to_string(),
        }
    }

    fn duplicate_transfer(message: String, transfer: TransferId) -> Self {
        Self::DuplicateTransfer {
            detail: message,
            transfer: transfer.to_string(),
        }
    }

    fn empty_name(message: String) -> Self {
        Self::GroupEmptyName { detail: message }
    }

    fn member_in_use(
        expenses: &[ExpenseId],
        member: MemberId,
        message: String,
        transfers: &[TransferId],
    ) -> Self {
        Self::MemberInUse {
            detail: message,
            expenses: identifiers(expenses),
            member: member.to_string(),
            transfers: identifiers(transfers),
        }
    }

    fn missing_exchange_rate(currency: Currency, message: String) -> Self {
        Self::MissingExchangeRate {
            currency: currency.code().to_owned(),
            detail: message,
        }
    }

    fn overflow(message: String) -> Self {
        Self::GroupOverflow { detail: message }
    }

    fn unexpected_exchange_rate(message: String) -> Self {
        Self::UnexpectedExchangeRate { detail: message }
    }

    fn unknown_category(category: CategoryId, message: String) -> Self {
        Self::UnknownCategory {
            category: category.to_string(),
            detail: message,
        }
    }

    fn unknown_expense(expense: ExpenseId, message: String) -> Self {
        Self::UnknownExpense {
            detail: message,
            expense: expense.to_string(),
        }
    }

    fn unknown_member(member: MemberId, message: String) -> Self {
        Self::UnknownMember {
            detail: message,
            member: member.to_string(),
        }
    }

    fn unknown_transfer(message: String, transfer: TransferId) -> Self {
        Self::UnknownTransfer {
            detail: message,
            transfer: transfer.to_string(),
        }
    }

    fn unsupported_format(format: u32, message: String) -> Self {
        Self::UnsupportedFormat {
            detail: message,
            format: i64::from(format),
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
