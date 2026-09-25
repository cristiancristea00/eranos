//! The error of the C and C++ APIs: the module that refused a call, the failure within it, the library's
//! message, and what the failure holds.

/// The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it in the
/// namespace `eranos::detail`.
#[diplomat::bridge]
#[diplomat::attr(cpp, namespace = "eranos::detail")]
pub mod ffi {
    use crate::values::ffi::{EranosMoney, EranosTextList};
    use diplomat_runtime::DiplomatWrite;

    /// The module of the library that refused a call, or the core's own case.
    #[derive(Debug)]
    pub enum EranosErrorKind {
        /// An `AccountError`, about a payment account: its IBAN, username, or label.
        Account,
        /// A `CalendarError`, about a date or a time of day.
        Calendar,
        /// The call was cancelled through its `Cancellation`, before it changed anything.
        Cancelled,
        /// A `CategoryError`, about a category.
        Category,
        /// A `ContactError`, about an email address or a phone number.
        Contact,
        /// An `ExpenseError`, about an expense: its amount, split, items, or tip.
        Expense,
        /// A `GroupError`, about how an entry fits the group, or the group's lock.
        Group,
        /// An `IdError`, about an identifier.
        Id,
        /// An entry to add carried an identifier; the group gives a new entry its own.
        IdentifierGiven,
        /// An entry to replace carried no identifier to find it by.
        IdentifierMissing,
        /// A saved group that is not JSON of the saved form, at `line` and `column`.
        InvalidJson,
        /// Text that is not UTF-8, which the C and C++ APIs refuse before the library sees it.
        InvalidText,
        /// A `LanguageError`, about a language tag.
        Language,
        /// A `MemberError`, about a member.
        Member,
        /// A `MoneyError`, about an amount, a currency, a rate, or a ratio.
        Money,
        /// The read handle was released, so it reads the group no more.
        Released,
        /// A `SettlementError`, about a settlement.
        Settlement,
        /// A `TransferError`, about a transfer.
        Transfer,
    }

    /// The failure within its module, named after the module and the failure.
    #[derive(Debug)]
    pub enum EranosFailure {
        /// An account whose name is blank.
        AccountEmptyName,
        /// A username that is blank.
        AccountEmptyUsername,
        /// An IBAN that failed validation, and why.
        AccountInvalidIban,
        /// A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601
        /// text.
        CalendarInvalidDate,
        /// A time of day that does not exist, has seconds, or is not ISO 8601 text.
        CalendarInvalidTime,
        /// A name that is blank.
        CategoryEmptyName,
        /// An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around
        /// an `@`, a malformed one, one too long, or whitespace or an invisible character.
        ContactInvalidEmail,
        /// A phone number that does not start with `+`, holds a letter or an extension, or is not
        /// valid in its country's numbering plan.
        ContactInvalidPhoneNumber,
        /// The amounts of a split do not add up to the amount being split.
        ExpenseAmountsDoNotAddUp,
        /// An amount inside the expense is in a different currency.
        ExpenseCurrencyMismatch,
        /// A member appears more than once in one split.
        ExpenseDuplicateMember,
        /// An item whose name is blank.
        ExpenseEmptyItemName,
        /// An expense whose name is blank.
        ExpenseEmptyName,
        /// A split lists no member.
        ExpenseEmptySplit,
        /// The item line totals do not add up to the expense amount.
        ExpenseItemsDoNotAddUp,
        /// A fixed tip below zero.
        ExpenseNegativeTip,
        /// An itemised expense without items.
        ExpenseNoItems,
        /// The amount or unit price is zero or negative.
        ExpenseNonPositiveAmount,
        /// A member's percentage or amount is zero or negative.
        ExpenseNonPositiveShare,
        /// A total that does not fit the range of an amount.
        ExpenseOverflow,
        /// The percentages of a split do not add up to exactly 100 %.
        ExpensePercentagesDoNotAddUp,
        /// An item quantity of zero.
        ExpenseZeroQuantity,
        /// An archived member cannot join a new expense.
        GroupArchivedMember,
        /// The built-in Uncategorised category cannot be renamed or removed.
        GroupBuiltInCategory,
        /// A shared group's lock was asked for without waiting, while another call held it.
        GroupBusy,
        /// A category with this ID already exists.
        GroupDuplicateCategory,
        /// Another category already has this name, compared by Unicode canonical caseless matching.
        GroupDuplicateCategoryName,
        /// The same currency appears twice among the rates of a currency change.
        GroupDuplicateExchangeRate,
        /// An expense with this ID already exists.
        GroupDuplicateExpense,
        /// A member with this ID already exists.
        GroupDuplicateMember,
        /// A transfer with this ID already exists.
        GroupDuplicateTransfer,
        /// A group name that is blank.
        GroupEmptyName,
        /// The member is still referenced and cannot be removed; archive it instead.
        GroupMemberInUse,
        /// An entry in another currency than the group's has no exchange rate into the group
        /// currency: its cost is local, or a currency change gave no rate for its currency.
        GroupMissingExchangeRate,
        /// A total would leave the range of an amount.
        GroupOverflow,
        /// An entry in the group currency has a converted cost; only a local cost is in the group
        /// currency.
        GroupUnexpectedExchangeRate,
        /// No category with this ID.
        GroupUnknownCategory,
        /// No expense with this ID.
        GroupUnknownExpense,
        /// No member with this ID.
        GroupUnknownMember,
        /// No transfer with this ID.
        GroupUnknownTransfer,
        /// A saved group in a format this version does not read: a newer one, whose fields it would
        /// drop, or an earlier one.
        GroupUnsupportedFormat,
        /// Text that is not a UUID.
        IdInvalidId,
        /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
        LanguageInvalidLanguage,
        /// A first name that is blank.
        MemberEmptyFirstName,
        /// Arithmetic between amounts of two different currencies.
        MoneyCurrencyMismatch,
        /// An amount that is not a decimal number.
        MoneyInvalidAmount,
        /// An exchange rate that is not a positive decimal number with at most 18 decimal places.
        MoneyInvalidExchangeRate,
        /// A number of parts that is not a positive decimal number with at most two decimal places.
        MoneyInvalidParts,
        /// A percentage that is not a decimal number with at most two decimal places.
        MoneyInvalidPercentage,
        /// A number or a result that does not fit its range.
        MoneyOverflow,
        /// An amount with more decimal places than its currency has.
        MoneyTooManyDecimals,
        /// A currency code that is not an active ISO 4217 code.
        MoneyUnknownCurrency,
        /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in
        /// the address space or the allocator refused it.
        SettlementOutOfMemory,
        /// The amount is zero or negative.
        TransferNonPositiveAmount,
        /// The sender and the recipient are the same member.
        TransferSameMember,
    }

    /// Why an IBAN was refused.
    #[derive(Debug)]
    pub enum EranosIbanReason {
        /// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
        InvalidCharacter,
        /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97
        /// check failed: the number contains a typo.
        InvalidChecksum,
        /// Not two letters followed by two check digits.
        InvalidFormat,
        /// A letter where the country's account number has a digit, or a digit where it has a
        /// letter, by the BBAN structure of the SWIFT IBAN registry.
        InvalidStructure,
        /// A country code without an entry in the SWIFT IBAN registry.
        UnknownCountry,
        /// The wrong number of characters for the country.
        WrongLength,
    }

    /// A failure a call reports: its kind, the failure within its module, the library's message,
    /// and what the failure holds, each getter present for the failures that hold its field.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosError(pub(crate) super::Flat);

    impl EranosError {
        /// The module that refused the call, or the core's own case.
        #[must_use]
        pub const fn kind(&self) -> EranosErrorKind {
            self.0.kind
        }

        /// The failure within its module, absent for the core's own cases.
        #[must_use]
        pub const fn failure(&self) -> Option<EranosFailure> {
            self.0.failure
        }

        /// Writes the library's message: lowercase, without a full stop, and never the text it
        /// refused.
        pub fn message(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.message);
        }

        /// What the amounts add up to. Present for the failures `AmountsDoNotAddUp`,
        /// `ItemsDoNotAddUp`.
        #[must_use]
        pub fn actual(&self) -> Option<Box<EranosMoney>> {
            self.0
                .payload
                .actual
                .clone()
                .map(|money| Box::new(EranosMoney(money)))
        }

        /// How many decimal places the currency has. Present for the failure `TooManyDecimals`.
        #[must_use]
        pub const fn allowed(&self) -> Option<i64> {
            self.0.payload.allowed
        }

        /// How many open balances the plan has. Present for the failure `OutOfMemory`.
        #[must_use]
        pub const fn balances(&self) -> Option<i64> {
            self.0.payload.balances
        }

        /// The identifier another category has. Present for the failures `DuplicateCategory`,
        /// `UnknownCategory`. Writes it, if present; absent otherwise.
        pub fn category(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.category.as_deref())
        }

        /// The column of the saved group where the text stops being one, from 1. Present for the
        /// failure `InvalidJson`.
        #[must_use]
        pub const fn column(&self) -> Option<i64> {
            self.0.payload.column
        }

        /// The currency that appears twice. Present for the failures `DuplicateExchangeRate`,
        /// `MissingExchangeRate`, `TooManyDecimals`. Writes it, if present; absent otherwise.
        pub fn currency(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.currency.as_deref())
        }

        /// The category that has the name. Present for the failure `DuplicateCategoryName`. Writes
        /// it, if present; absent otherwise.
        pub fn existing(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.existing.as_deref())
        }

        /// The amount being split. Present for the failures `AmountsDoNotAddUp`, `ItemsDoNotAddUp`.
        #[must_use]
        pub fn expected_amount(&self) -> Option<Box<EranosMoney>> {
            self.0
                .payload
                .expected_amount
                .clone()
                .map(|money| Box::new(EranosMoney(money)))
        }

        /// The expense's currency. Present for the failure `CurrencyMismatch`. Writes it, if
        /// present; absent otherwise.
        pub fn expected_currency(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.expected_currency.as_deref())
        }

        /// How many characters the country's IBANs have. Present for the failure `WrongLength`.
        #[must_use]
        pub const fn expected_length(&self) -> Option<i64> {
            self.0.payload.expected_length
        }

        /// The identifier another expense has. Present for the failures `DuplicateExpense`,
        /// `UnknownExpense`. Writes it, if present; absent otherwise.
        pub fn expense(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.expense.as_deref())
        }

        /// The expenses that name the member. Present for the failure `MemberInUse`.
        #[must_use]
        pub fn expenses(&self) -> Option<Box<EranosTextList>> {
            self.0
                .payload
                .expenses
                .clone()
                .map(|list| Box::new(EranosTextList(list)))
        }

        /// The format the saved group names. Present for the failure `UnsupportedFormat`.
        #[must_use]
        pub const fn format(&self) -> Option<i64> {
            self.0.payload.format
        }

        /// The currency of the amount inside it. Present for the failure `CurrencyMismatch`. Writes
        /// it, if present; absent otherwise.
        pub fn found_currency(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.found_currency.as_deref())
        }

        /// How many characters the IBAN has. Present for the failure `WrongLength`.
        #[must_use]
        pub const fn found_length(&self) -> Option<i64> {
            self.0.payload.found_length
        }

        /// The currency of the first amount. Present for the failure `CurrencyMismatch`. Writes it,
        /// if present; absent otherwise.
        pub fn left(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.left.as_deref())
        }

        /// The line of the saved group where the text stops being one, from 1. Present for the
        /// failure `InvalidJson`.
        #[must_use]
        pub const fn line(&self) -> Option<i64> {
            self.0.payload.line
        }

        /// The member who appears more than once. Present for the failures `DuplicateMember`,
        /// `NonPositiveShare`, `ArchivedMember`, `DuplicateMember`, `MemberInUse`, `UnknownMember`.
        /// Writes it, if present; absent otherwise.
        pub fn member(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.member.as_deref())
        }

        /// The check the IBAN failed. Present for the failure `InvalidIban`.
        #[must_use]
        pub const fn reason(&self) -> Option<EranosIbanReason> {
            self.0.payload.reason
        }

        /// The currency of the second amount. Present for the failure `CurrencyMismatch`. Writes
        /// it, if present; absent otherwise.
        pub fn right(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.right.as_deref())
        }

        /// What the percentages add up to, in basis points. Present for the failure
        /// `PercentagesDoNotAddUp`.
        #[must_use]
        pub const fn total_basis_points(&self) -> Option<i64> {
            self.0.payload.total_basis_points
        }

        /// The identifier another transfer has. Present for the failures `DuplicateTransfer`,
        /// `UnknownTransfer`. Writes it, if present; absent otherwise.
        pub fn transfer(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.payload.transfer.as_deref())
        }

        /// The transfers that name the member. Present for the failure `MemberInUse`.
        #[must_use]
        pub fn transfers(&self) -> Option<Box<EranosTextList>> {
            self.0
                .payload
                .transfers
                .clone()
                .map(|list| Box::new(EranosTextList(list)))
        }
    }
}

/// What an error holds, read once when it is made: every getter reads a field.
#[derive(Debug, Clone, Default)]
pub(crate) struct Payload {
    actual: Option<eranos_bindings::Money>,
    allowed: Option<i64>,
    balances: Option<i64>,
    category: Option<String>,
    column: Option<i64>,
    currency: Option<String>,
    existing: Option<String>,
    expected_amount: Option<eranos_bindings::Money>,
    expected_currency: Option<String>,
    expected_length: Option<i64>,
    expense: Option<String>,
    expenses: Option<Vec<String>>,
    format: Option<i64>,
    found_currency: Option<String>,
    found_length: Option<i64>,
    left: Option<String>,
    line: Option<i64>,
    member: Option<String>,
    reason: Option<ffi::EranosIbanReason>,
    right: Option<String>,
    total_basis_points: Option<i64>,
    transfer: Option<String>,
    transfers: Option<Vec<String>>,
}

/// An error's kind, failure, message, and payload.
#[derive(Debug, Clone)]
pub(crate) struct Flat {
    kind: ffi::EranosErrorKind,
    failure: Option<ffi::EranosFailure>,
    message: String,
    payload: Payload,
}

impl From<crate::Failure> for ffi::EranosError {
    fn from(failure: crate::Failure) -> Self {
        Self(flatten(&failure))
    }
}

/// Reads `failure` into what the getters read.
fn flatten(failure: &crate::Failure) -> Flat {
    match failure {
        crate::Failure::InvalidText => Flat {
            kind: ffi::EranosErrorKind::InvalidText,
            failure: None,
            message: "the text is not UTF-8".to_owned(),
            payload: Payload::default(),
        },
        crate::Failure::Library(error) => library(error),
    }
}

/// Reads the library's `error`.
fn library(error: &eranos_bindings::EranosError) -> Flat {
    let message = error.to_string();
    let (kind, failure, payload) = match error {
        eranos_bindings::EranosError::Account { failure, .. } => {
            let (failure, payload) = account(*failure);
            (ffi::EranosErrorKind::Account, Some(failure), payload)
        }
        eranos_bindings::EranosError::Calendar { failure, .. } => {
            let (failure, payload) = calendar(*failure);
            (ffi::EranosErrorKind::Calendar, Some(failure), payload)
        }
        eranos_bindings::EranosError::Cancelled { .. } => {
            (ffi::EranosErrorKind::Cancelled, None, Payload::default())
        }
        eranos_bindings::EranosError::Category { failure, .. } => {
            let (failure, payload) = category(*failure);
            (ffi::EranosErrorKind::Category, Some(failure), payload)
        }
        eranos_bindings::EranosError::Contact { failure, .. } => {
            let (failure, payload) = contact(*failure);
            (ffi::EranosErrorKind::Contact, Some(failure), payload)
        }
        eranos_bindings::EranosError::Expense { failure, .. } => {
            let (failure, payload) = expense(failure);
            (ffi::EranosErrorKind::Expense, Some(failure), payload)
        }
        eranos_bindings::EranosError::Group { failure, .. } => {
            let (failure, payload) = group(failure);
            (ffi::EranosErrorKind::Group, Some(failure), payload)
        }
        eranos_bindings::EranosError::Id { failure, .. } => {
            let (failure, payload) = id(*failure);
            (ffi::EranosErrorKind::Id, Some(failure), payload)
        }
        eranos_bindings::EranosError::IdentifierGiven { .. } => (
            ffi::EranosErrorKind::IdentifierGiven,
            None,
            Payload::default(),
        ),
        eranos_bindings::EranosError::IdentifierMissing { .. } => (
            ffi::EranosErrorKind::IdentifierMissing,
            None,
            Payload::default(),
        ),
        eranos_bindings::EranosError::InvalidJson { column, line, .. } => (
            ffi::EranosErrorKind::InvalidJson,
            None,
            Payload {
                column: Some(*column),
                line: Some(*line),
                ..Payload::default()
            },
        ),
        eranos_bindings::EranosError::Language { failure, .. } => {
            let (failure, payload) = language(*failure);
            (ffi::EranosErrorKind::Language, Some(failure), payload)
        }
        eranos_bindings::EranosError::Member { failure, .. } => {
            let (failure, payload) = member(*failure);
            (ffi::EranosErrorKind::Member, Some(failure), payload)
        }
        eranos_bindings::EranosError::Money { failure, .. } => {
            let (failure, payload) = money(failure);
            (ffi::EranosErrorKind::Money, Some(failure), payload)
        }
        eranos_bindings::EranosError::Released { .. } => {
            (ffi::EranosErrorKind::Released, None, Payload::default())
        }
        eranos_bindings::EranosError::Settlement { failure, .. } => {
            let (failure, payload) = settlement(*failure);
            (ffi::EranosErrorKind::Settlement, Some(failure), payload)
        }
        eranos_bindings::EranosError::Transfer { failure, .. } => {
            let (failure, payload) = transfer(*failure);
            (ffi::EranosErrorKind::Transfer, Some(failure), payload)
        }
    };
    Flat {
        kind,
        failure,
        message,
        payload,
    }
}

/// Reads the account module's `failure`.
fn account(failure: eranos_bindings::AccountFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::AccountFailure::EmptyName => {
            (ffi::EranosFailure::AccountEmptyName, Payload::default())
        }
        eranos_bindings::AccountFailure::EmptyUsername => {
            (ffi::EranosFailure::AccountEmptyUsername, Payload::default())
        }
        eranos_bindings::AccountFailure::InvalidIban { reason } => {
            (ffi::EranosFailure::AccountInvalidIban, iban(&reason))
        }
    }
}

/// Reads the calendar module's `failure`.
fn calendar(failure: eranos_bindings::CalendarFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::CalendarFailure::InvalidDate => {
            (ffi::EranosFailure::CalendarInvalidDate, Payload::default())
        }
        eranos_bindings::CalendarFailure::InvalidTime => {
            (ffi::EranosFailure::CalendarInvalidTime, Payload::default())
        }
    }
}

/// Reads the category module's `failure`.
fn category(failure: eranos_bindings::CategoryFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::CategoryFailure::EmptyName => {
            (ffi::EranosFailure::CategoryEmptyName, Payload::default())
        }
    }
}

/// Reads the contact module's `failure`.
fn contact(failure: eranos_bindings::ContactFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::ContactFailure::InvalidEmail => {
            (ffi::EranosFailure::ContactInvalidEmail, Payload::default())
        }
        eranos_bindings::ContactFailure::InvalidPhoneNumber => (
            ffi::EranosFailure::ContactInvalidPhoneNumber,
            Payload::default(),
        ),
    }
}

/// Reads the expense module's `failure`.
fn expense(failure: &eranos_bindings::ExpenseFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::ExpenseFailure::AmountsDoNotAddUp { actual, expected } => (
            ffi::EranosFailure::ExpenseAmountsDoNotAddUp,
            Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::CurrencyMismatch { expected, found } => (
            ffi::EranosFailure::ExpenseCurrencyMismatch,
            Payload {
                expected_currency: Some(expected.clone()),
                found_currency: Some(found.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::DuplicateMember { member } => (
            ffi::EranosFailure::ExpenseDuplicateMember,
            Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::EmptyItemName => {
            (ffi::EranosFailure::ExpenseEmptyItemName, Payload::default())
        }
        eranos_bindings::ExpenseFailure::EmptyName => {
            (ffi::EranosFailure::ExpenseEmptyName, Payload::default())
        }
        eranos_bindings::ExpenseFailure::EmptySplit => {
            (ffi::EranosFailure::ExpenseEmptySplit, Payload::default())
        }
        eranos_bindings::ExpenseFailure::ItemsDoNotAddUp { actual, expected } => (
            ffi::EranosFailure::ExpenseItemsDoNotAddUp,
            Payload {
                actual: Some(actual.clone()),
                expected_amount: Some(expected.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::NegativeTip => {
            (ffi::EranosFailure::ExpenseNegativeTip, Payload::default())
        }
        eranos_bindings::ExpenseFailure::NoItems => {
            (ffi::EranosFailure::ExpenseNoItems, Payload::default())
        }
        eranos_bindings::ExpenseFailure::NonPositiveAmount => (
            ffi::EranosFailure::ExpenseNonPositiveAmount,
            Payload::default(),
        ),
        eranos_bindings::ExpenseFailure::NonPositiveShare { member } => (
            ffi::EranosFailure::ExpenseNonPositiveShare,
            Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::Overflow => {
            (ffi::EranosFailure::ExpenseOverflow, Payload::default())
        }
        eranos_bindings::ExpenseFailure::PercentagesDoNotAddUp { total_basis_points } => (
            ffi::EranosFailure::ExpensePercentagesDoNotAddUp,
            Payload {
                total_basis_points: Some(*total_basis_points),
                ..Payload::default()
            },
        ),
        eranos_bindings::ExpenseFailure::ZeroQuantity => {
            (ffi::EranosFailure::ExpenseZeroQuantity, Payload::default())
        }
    }
}

/// Reads the group module's `failure`.
#[expect(
    clippy::too_many_lines,
    reason = "one arm per failure of the group module, of which there are 19"
)]
fn group(failure: &eranos_bindings::GroupFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::GroupFailure::ArchivedMember { member } => (
            ffi::EranosFailure::GroupArchivedMember,
            Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::BuiltInCategory => {
            (ffi::EranosFailure::GroupBuiltInCategory, Payload::default())
        }
        eranos_bindings::GroupFailure::Busy => (ffi::EranosFailure::GroupBusy, Payload::default()),
        eranos_bindings::GroupFailure::DuplicateCategory { category } => (
            ffi::EranosFailure::GroupDuplicateCategory,
            Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::DuplicateCategoryName { existing } => (
            ffi::EranosFailure::GroupDuplicateCategoryName,
            Payload {
                existing: Some(existing.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::DuplicateExchangeRate { currency } => (
            ffi::EranosFailure::GroupDuplicateExchangeRate,
            Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::DuplicateExpense { expense } => (
            ffi::EranosFailure::GroupDuplicateExpense,
            Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::DuplicateMember { member } => (
            ffi::EranosFailure::GroupDuplicateMember,
            Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::DuplicateTransfer { transfer } => (
            ffi::EranosFailure::GroupDuplicateTransfer,
            Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::EmptyName => {
            (ffi::EranosFailure::GroupEmptyName, Payload::default())
        }
        eranos_bindings::GroupFailure::MemberInUse {
            expenses,
            member,
            transfers,
        } => (
            ffi::EranosFailure::GroupMemberInUse,
            Payload {
                expenses: Some(expenses.clone()),
                member: Some(member.clone()),
                transfers: Some(transfers.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::MissingExchangeRate { currency } => (
            ffi::EranosFailure::GroupMissingExchangeRate,
            Payload {
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::Overflow => {
            (ffi::EranosFailure::GroupOverflow, Payload::default())
        }
        eranos_bindings::GroupFailure::UnexpectedExchangeRate => (
            ffi::EranosFailure::GroupUnexpectedExchangeRate,
            Payload::default(),
        ),
        eranos_bindings::GroupFailure::UnknownCategory { category } => (
            ffi::EranosFailure::GroupUnknownCategory,
            Payload {
                category: Some(category.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::UnknownExpense { expense } => (
            ffi::EranosFailure::GroupUnknownExpense,
            Payload {
                expense: Some(expense.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::UnknownMember { member } => (
            ffi::EranosFailure::GroupUnknownMember,
            Payload {
                member: Some(member.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::UnknownTransfer { transfer } => (
            ffi::EranosFailure::GroupUnknownTransfer,
            Payload {
                transfer: Some(transfer.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::GroupFailure::UnsupportedFormat { format } => (
            ffi::EranosFailure::GroupUnsupportedFormat,
            Payload {
                format: Some(*format),
                ..Payload::default()
            },
        ),
    }
}

/// Reads the id module's `failure`.
fn id(failure: eranos_bindings::IdFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::IdFailure::InvalidId => {
            (ffi::EranosFailure::IdInvalidId, Payload::default())
        }
    }
}

/// Reads the language module's `failure`.
fn language(failure: eranos_bindings::LanguageFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::LanguageFailure::InvalidLanguage => (
            ffi::EranosFailure::LanguageInvalidLanguage,
            Payload::default(),
        ),
    }
}

/// Reads the member module's `failure`.
fn member(failure: eranos_bindings::MemberFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::MemberFailure::EmptyFirstName => {
            (ffi::EranosFailure::MemberEmptyFirstName, Payload::default())
        }
    }
}

/// Reads the money module's `failure`.
fn money(failure: &eranos_bindings::MoneyFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::MoneyFailure::CurrencyMismatch { left, right } => (
            ffi::EranosFailure::MoneyCurrencyMismatch,
            Payload {
                left: Some(left.clone()),
                right: Some(right.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::MoneyFailure::InvalidAmount => {
            (ffi::EranosFailure::MoneyInvalidAmount, Payload::default())
        }
        eranos_bindings::MoneyFailure::InvalidExchangeRate => (
            ffi::EranosFailure::MoneyInvalidExchangeRate,
            Payload::default(),
        ),
        eranos_bindings::MoneyFailure::InvalidParts => {
            (ffi::EranosFailure::MoneyInvalidParts, Payload::default())
        }
        eranos_bindings::MoneyFailure::InvalidPercentage => (
            ffi::EranosFailure::MoneyInvalidPercentage,
            Payload::default(),
        ),
        eranos_bindings::MoneyFailure::Overflow => {
            (ffi::EranosFailure::MoneyOverflow, Payload::default())
        }
        eranos_bindings::MoneyFailure::TooManyDecimals { allowed, currency } => (
            ffi::EranosFailure::MoneyTooManyDecimals,
            Payload {
                allowed: Some(*allowed),
                currency: Some(currency.clone()),
                ..Payload::default()
            },
        ),
        eranos_bindings::MoneyFailure::UnknownCurrency => {
            (ffi::EranosFailure::MoneyUnknownCurrency, Payload::default())
        }
    }
}

/// Reads the settlement module's `failure`.
fn settlement(failure: eranos_bindings::SettlementFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::SettlementFailure::OutOfMemory { balances } => (
            ffi::EranosFailure::SettlementOutOfMemory,
            Payload {
                balances: Some(balances),
                ..Payload::default()
            },
        ),
    }
}

/// Reads the transfer module's `failure`.
fn transfer(failure: eranos_bindings::TransferFailure) -> (ffi::EranosFailure, Payload) {
    match failure {
        eranos_bindings::TransferFailure::NonPositiveAmount => (
            ffi::EranosFailure::TransferNonPositiveAmount,
            Payload::default(),
        ),
        eranos_bindings::TransferFailure::SameMember => {
            (ffi::EranosFailure::TransferSameMember, Payload::default())
        }
    }
}

/// Reads the reason of a refused IBAN.
fn iban(reason: &eranos_bindings::IbanReason) -> Payload {
    match reason {
        eranos_bindings::IbanReason::InvalidCharacter => Payload {
            reason: Some(ffi::EranosIbanReason::InvalidCharacter),
            ..Payload::default()
        },
        eranos_bindings::IbanReason::InvalidChecksum => Payload {
            reason: Some(ffi::EranosIbanReason::InvalidChecksum),
            ..Payload::default()
        },
        eranos_bindings::IbanReason::InvalidFormat => Payload {
            reason: Some(ffi::EranosIbanReason::InvalidFormat),
            ..Payload::default()
        },
        eranos_bindings::IbanReason::InvalidStructure => Payload {
            reason: Some(ffi::EranosIbanReason::InvalidStructure),
            ..Payload::default()
        },
        eranos_bindings::IbanReason::UnknownCountry => Payload {
            reason: Some(ffi::EranosIbanReason::UnknownCountry),
            ..Payload::default()
        },
        eranos_bindings::IbanReason::WrongLength { expected, found } => Payload {
            reason: Some(ffi::EranosIbanReason::WrongLength),
            expected_length: Some(*expected),
            found_length: Some(*found),
            ..Payload::default()
        },
    }
}
