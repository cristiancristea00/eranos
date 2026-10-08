//! The error of the C and C++ APIs: the module that refused a call, the failure within it, the library's
//! message, and what the failure holds.

/// The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it in the
/// namespace `eranos::detail`.
#[diplomat::bridge]
#[diplomat::attr(cpp, namespace = "eranos::detail")]
pub mod ffi {
    use crate::generated::records::ffi::EranosCurrencyDetails;
    use crate::values::ffi::{EranosMoney, EranosTextList};
    use diplomat_runtime::DiplomatWrite;

    /// The module of the library that refused a call, or the core's own case.
    #[derive(Debug)]
    #[diplomat::enum_convert(eranos_bindings::ErrorKind)]
    pub enum EranosErrorKind {
        /// An `IdError`, about an identifier.
        Id,
        /// A `PaymentAccountError`, about a payment account: its IBAN, username, or label.
        PaymentAccount,
        /// A `CalendarError`, about a date or a time of day.
        Calendar,
        /// A `CategoryError`, about a category.
        Category,
        /// A `ContactError`, about an email address or a phone number.
        Contact,
        /// A `MoneyError`, about an amount, a currency, a rate, or a ratio.
        Money,
        /// A `MemberError`, about a member.
        Member,
        /// An `ExpenseError`, about an expense: its amount, split, items, or tip.
        Expense,
        /// A `SettlementError`, about a settlement.
        Settlement,
        /// A `TransferError`, about a transfer.
        Transfer,
        /// An `ExpenseGroupError`, about how an entry fits the group, or the group's lock.
        ExpenseGroup,
        /// A `LanguageError`, about a language tag.
        Language,
        /// A saved group that is not JSON of the saved form, at `line` and `column`.
        InvalidJson,
        /// Text that is not UTF-8, which the C and C++ APIs refuse before the library sees it.
        InvalidText,
        /// The call was cancelled through its `Cancellation`, before it changed anything.
        Cancelled,
        /// The read handle was released, so it reads the group no more.
        Released,
    }

    /// The failure within its module, named after the module and the failure.
    #[derive(Debug)]
    #[diplomat::enum_convert(eranos_bindings::FailureKind)]
    pub enum EranosFailure {
        /// An identifier that is empty.
        IdEmpty,
        /// An identifier longer than 64 bytes of UTF-8.
        IdTooLong,
        /// An identifier that holds a control character, such as a tab or a line break.
        IdControlCharacter,
        /// An identifier that starts or ends with whitespace.
        IdSurroundingWhitespace,
        /// An account whose name is blank.
        PaymentAccountEmptyName,
        /// An IBAN that failed validation, and why.
        PaymentAccountInvalidIban,
        /// A username that is blank.
        PaymentAccountEmptyUsername,
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
        /// An amount that is not a decimal number.
        MoneyInvalidAmount,
        /// An amount with more decimal places than its currency has.
        MoneyTooManyDecimals,
        /// Arithmetic between amounts of two different currencies.
        MoneyCurrencyMismatch,
        /// A currency code that is not an active ISO 4217 code.
        MoneyUnknownCurrency,
        /// An exchange rate that is not a positive decimal number with at most 18 decimal places.
        MoneyInvalidExchangeRate,
        /// A number of parts that is not a positive decimal number with at most two decimal places.
        MoneyInvalidParts,
        /// A percentage that is not a decimal number with at most two decimal places.
        MoneyInvalidPercentage,
        /// A double that is NaN or infinite, which no amount, percentage, number of parts, or
        /// exchange rate is.
        MoneyNotFinite,
        /// A number or a result that does not fit its range.
        MoneyOverflow,
        /// A first name that is blank.
        MemberEmptyFirstName,
        /// An expense whose name is blank.
        ExpenseEmptyName,
        /// The amount or unit price is zero or negative.
        ExpenseNonPositiveAmount,
        /// An amount inside the expense is in a different currency.
        ExpenseCurrencyMismatch,
        /// A split lists no member.
        ExpenseEmptySplit,
        /// A member appears more than once in one split.
        ExpenseDuplicateMember,
        /// A member's percentage or amount is zero or negative.
        ExpenseNonPositiveShare,
        /// The amounts of a split do not add up to the amount being split.
        ExpenseAmountsDoNotAddUp,
        /// The percentages of a split do not add up to exactly 100 %.
        ExpensePercentagesDoNotAddUp,
        /// An itemised expense without items.
        ExpenseNoItems,
        /// An item whose name is blank.
        ExpenseEmptyItemName,
        /// An item quantity of zero.
        ExpenseZeroQuantity,
        /// The item line totals do not add up to the expense amount.
        ExpenseItemsDoNotAddUp,
        /// A fixed tip below zero.
        ExpenseNegativeTip,
        /// A total that does not fit the range of an amount.
        ExpenseOverflow,
        /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in
        /// the address space or the allocator refused it.
        SettlementOutOfMemory,
        /// The sender and the recipient are the same member.
        TransferSameMember,
        /// The amount is zero or negative.
        TransferNonPositiveAmount,
        /// A saved group in a format this version does not read: a newer one, whose fields it would
        /// drop, or an earlier one.
        ExpenseGroupUnsupportedFormat,
        /// A group name that is blank.
        ExpenseGroupEmptyName,
        /// An entry in another currency than the group's has no exchange rate into the group
        /// currency: its cost is local, or a currency change gave no rate for its currency.
        ExpenseGroupMissingExchangeRate,
        /// An entry in the group currency has a converted cost; only a local cost is in the group
        /// currency.
        ExpenseGroupUnexpectedExchangeRate,
        /// The same currency appears twice among the rates of a currency change.
        ExpenseGroupDuplicateExchangeRate,
        /// A member with this ID already exists.
        ExpenseGroupDuplicateMember,
        /// No member with this ID.
        ExpenseGroupUnknownMember,
        /// An archived member cannot join a new expense.
        ExpenseGroupArchivedMember,
        /// The member is still referenced and cannot be removed; archive it instead.
        ExpenseGroupMemberInUse,
        /// A category with this ID already exists.
        ExpenseGroupDuplicateCategory,
        /// Another category already has this name, compared by Unicode canonical caseless matching.
        ExpenseGroupDuplicateCategoryName,
        /// No category with this ID.
        ExpenseGroupUnknownCategory,
        /// The built-in Uncategorised category cannot be renamed or removed.
        ExpenseGroupBuiltInCategory,
        /// An expense with this ID already exists.
        ExpenseGroupDuplicateExpense,
        /// No expense with this ID.
        ExpenseGroupUnknownExpense,
        /// A transfer with this ID already exists.
        ExpenseGroupDuplicateTransfer,
        /// No transfer with this ID.
        ExpenseGroupUnknownTransfer,
        /// A shared group's lock was asked for without waiting, while another call held it.
        ExpenseGroupBusy,
        /// A total would leave the range of an amount.
        ExpenseGroupOverflow,
        /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
        LanguageInvalidLanguage,
    }

    /// Why an IBAN was refused.
    #[derive(Debug)]
    #[diplomat::enum_convert(eranos_bindings::IbanReasonKind)]
    pub enum EranosIbanReason {
        /// A character other than an ASCII letter, a digit, or whitespace (which is dropped).
        InvalidCharacter,
        /// Not two letters followed by two check digits.
        InvalidFormat,
        /// A country code without an entry in the SWIFT IBAN registry.
        UnknownCountry,
        /// The wrong number of characters for the country.
        WrongLength,
        /// A letter where the country's account number has a digit, or a digit where it has a
        /// letter, by the BBAN structure of the SWIFT IBAN registry.
        InvalidStructure,
        /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97
        /// check failed: the number contains a typo.
        InvalidChecksum,
    }

    /// A failure a call reports: its kind, the failure within its module, the library's message,
    /// and what the failure holds, each getter present for the failures that hold its field.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosError(pub(crate) eranos_bindings::FlatError);

    impl EranosError {
        /// The module that refused the call, or the core's own case.
        #[must_use]
        pub fn kind(&self) -> EranosErrorKind {
            self.0.kind.into()
        }

        /// The failure within its module, absent for the core's own cases.
        #[must_use]
        pub fn failure(&self) -> Option<EranosFailure> {
            self.0.failure.map(Into::into)
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
        /// `MissingExchangeRate`, `TooManyDecimals`.
        #[must_use]
        pub fn currency(&self) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .payload
                .currency
                .clone()
                .map(|currency| Box::new(EranosCurrencyDetails(currency)))
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

        /// The expense's currency. Present for the failure `CurrencyMismatch`.
        #[must_use]
        pub fn expected_currency(&self) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .payload
                .expected_currency
                .clone()
                .map(|currency| Box::new(EranosCurrencyDetails(currency)))
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

        /// The currency of the amount inside it. Present for the failure `CurrencyMismatch`.
        #[must_use]
        pub fn found_currency(&self) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .payload
                .found_currency
                .clone()
                .map(|currency| Box::new(EranosCurrencyDetails(currency)))
        }

        /// How many characters the IBAN has. Present for the failure `WrongLength`.
        #[must_use]
        pub const fn found_length(&self) -> Option<i64> {
            self.0.payload.found_length
        }

        /// The currency of the first amount. Present for the failure `CurrencyMismatch`.
        #[must_use]
        pub fn left(&self) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .payload
                .left
                .clone()
                .map(|currency| Box::new(EranosCurrencyDetails(currency)))
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
        pub fn reason(&self) -> Option<EranosIbanReason> {
            self.0.payload.reason.map(Into::into)
        }

        /// The currency of the second amount. Present for the failure `CurrencyMismatch`.
        #[must_use]
        pub fn right(&self) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .payload
                .right
                .clone()
                .map(|currency| Box::new(EranosCurrencyDetails(currency)))
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

impl From<crate::Failure> for ffi::EranosError {
    fn from(failure: crate::Failure) -> Self {
        Self(match failure {
            crate::Failure::InvalidText => eranos_bindings::FlatError::invalid_text(),
            crate::Failure::Library(error) => eranos_bindings::FlatError::from(&error),
        })
    }
}
