//! The failures of building a group, of changing it, and of reading a
//! language tag.

use eranos_core::{CategoryId, ExpenseId, MemberId, TransferId};
use eranos_money::CurrencyCode;

use crate::FORMAT;

/// A language tag that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageError {
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    #[error("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")]
    InvalidLanguage,
}

/// Builds a value from a [`LanguageError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromLanguageError: Sized {
    /// The value for [`LanguageError::InvalidLanguage`].
    fn invalid_language(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_language_error(error: LanguageError) -> Self {
        let message = error.to_string();
        match error {
            LanguageError::InvalidLanguage => Self::invalid_language(message),
        }
    }
}

/// A [`ExpenseGroup`](crate::ExpenseGroup) that could not be built, or an operation that
/// would break its consistency; the group is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ExpenseGroupError {
    /// A saved group in a format this version does not read: a newer one,
    /// whose fields it would drop, or an earlier one.
    #[error("unsupported group format {format}: this version reads format {FORMAT}")]
    UnsupportedFormat {
        /// The format the saved group names.
        format: u32,
    },
    /// A group name that is blank.
    #[error("the group name must not be empty")]
    EmptyName,
    /// An entry in another currency than the group's has no exchange rate
    /// into the group currency: its cost is local, or a currency change
    /// gave no rate for its currency.
    #[error("an exchange rate from {currency} into the group currency is required")]
    MissingExchangeRate {
        /// The currency without a rate.
        currency: CurrencyCode,
    },
    /// An entry in the group currency has a converted cost; only a local
    /// cost is in the group currency.
    #[error("an entry in the group currency cannot have an exchange rate")]
    UnexpectedExchangeRate,
    /// The same currency appears twice among the rates of a currency change.
    #[error("two exchange rates were given for {currency}")]
    DuplicateExchangeRate {
        /// The repeated currency.
        currency: CurrencyCode,
    },
    /// A member with this ID already exists.
    #[error("a member with this identifier already exists")]
    DuplicateMember(MemberId),
    /// No member with this ID.
    #[error("no member has this identifier")]
    UnknownMember(MemberId),
    /// An archived member cannot join a new expense.
    #[error("the member is archived")]
    ArchivedMember(MemberId),
    /// The member is still referenced and cannot be removed; archive it
    /// instead.
    #[error("the member is used by {}", usage(expenses.len(), transfers.len()))]
    MemberInUse {
        /// The member that was to be removed.
        member: MemberId,
        /// The expenses that reference the member.
        expenses: Vec<ExpenseId>,
        /// The transfers that reference the member.
        transfers: Vec<TransferId>,
    },
    /// A category with this ID already exists.
    #[error("a category with this identifier already exists")]
    DuplicateCategory(CategoryId),
    /// Another category already has this name, compared by Unicode canonical
    /// caseless matching.
    #[error("another category already has this name")]
    DuplicateCategoryName {
        /// The category that has the name.
        existing: CategoryId,
    },
    /// No category with this ID.
    #[error("no category has this identifier")]
    UnknownCategory(CategoryId),
    /// The built-in Uncategorised category cannot be renamed or removed.
    #[error("the built-in category cannot be renamed or removed")]
    BuiltInCategory,
    /// An expense with this ID already exists.
    #[error("an expense with this identifier already exists")]
    DuplicateExpense(ExpenseId),
    /// No expense with this ID.
    #[error("no expense has this identifier")]
    UnknownExpense(ExpenseId),
    /// A transfer with this ID already exists.
    #[error("a transfer with this identifier already exists")]
    DuplicateTransfer(TransferId),
    /// No transfer with this ID.
    #[error("no transfer has this identifier")]
    UnknownTransfer(TransferId),
    /// A shared group's lock was asked for without waiting, while another
    /// call held it.
    #[error("the group is locked by another call")]
    Busy,
    /// A total would leave the range of an amount.
    #[error("the group totals would be out of range")]
    Overflow,
}

/// Builds a value from a [`ExpenseGroupError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromExpenseGroupError: Sized {
    /// The value for [`ExpenseGroupError::UnsupportedFormat`].
    fn unsupported_format(format: u32, message: String) -> Self;

    /// The value for [`ExpenseGroupError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for [`ExpenseGroupError::MissingExchangeRate`].
    fn missing_exchange_rate(currency: CurrencyCode, message: String) -> Self;

    /// The value for [`ExpenseGroupError::UnexpectedExchangeRate`].
    fn unexpected_exchange_rate(message: String) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateExchangeRate`].
    fn duplicate_exchange_rate(currency: CurrencyCode, message: String) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateMember`].
    fn duplicate_member(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::UnknownMember`].
    fn unknown_member(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::ArchivedMember`].
    fn archived_member(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::MemberInUse`].
    fn member_in_use(
        member: MemberId,
        expenses: &[ExpenseId],
        transfers: &[TransferId],
        message: String,
    ) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateCategory`].
    fn duplicate_category(category: CategoryId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateCategoryName`].
    fn duplicate_category_name(existing: CategoryId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::UnknownCategory`].
    fn unknown_category(category: CategoryId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::BuiltInCategory`].
    fn built_in_category(message: String) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateExpense`].
    fn duplicate_expense(expense: ExpenseId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::UnknownExpense`].
    fn unknown_expense(expense: ExpenseId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::DuplicateTransfer`].
    fn duplicate_transfer(transfer: TransferId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::UnknownTransfer`].
    fn unknown_transfer(transfer: TransferId, message: String) -> Self;

    /// The value for [`ExpenseGroupError::Busy`].
    fn busy(message: String) -> Self;

    /// The value for [`ExpenseGroupError::Overflow`].
    fn overflow(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_expense_group_error(error: &ExpenseGroupError) -> Self {
        let message = error.to_string();
        match error {
            ExpenseGroupError::UnsupportedFormat { format } => {
                Self::unsupported_format(*format, message)
            }
            ExpenseGroupError::EmptyName => Self::empty_name(message),
            ExpenseGroupError::MissingExchangeRate { currency } => {
                Self::missing_exchange_rate(*currency, message)
            }
            ExpenseGroupError::UnexpectedExchangeRate => Self::unexpected_exchange_rate(message),
            ExpenseGroupError::DuplicateExchangeRate { currency } => {
                Self::duplicate_exchange_rate(*currency, message)
            }
            ExpenseGroupError::DuplicateMember(member) => Self::duplicate_member(*member, message),
            ExpenseGroupError::UnknownMember(member) => Self::unknown_member(*member, message),
            ExpenseGroupError::ArchivedMember(member) => Self::archived_member(*member, message),
            ExpenseGroupError::MemberInUse {
                member,
                expenses,
                transfers,
            } => Self::member_in_use(*member, expenses, transfers, message),
            ExpenseGroupError::DuplicateCategory(category) => {
                Self::duplicate_category(*category, message)
            }
            ExpenseGroupError::DuplicateCategoryName { existing } => {
                Self::duplicate_category_name(*existing, message)
            }
            ExpenseGroupError::UnknownCategory(category) => {
                Self::unknown_category(*category, message)
            }
            ExpenseGroupError::BuiltInCategory => Self::built_in_category(message),
            ExpenseGroupError::DuplicateExpense(expense) => {
                Self::duplicate_expense(*expense, message)
            }
            ExpenseGroupError::UnknownExpense(expense) => Self::unknown_expense(*expense, message),
            ExpenseGroupError::DuplicateTransfer(transfer) => {
                Self::duplicate_transfer(*transfer, message)
            }
            ExpenseGroupError::UnknownTransfer(transfer) => {
                Self::unknown_transfer(*transfer, message)
            }
            ExpenseGroupError::Busy => Self::busy(message),
            ExpenseGroupError::Overflow => Self::overflow(message),
        }
    }
}

/// "1 expense", "2 expenses": a count with its noun in the right number.
fn count(number: usize, singular: &str, plural: &str) -> String {
    format!("{number} {}", if 1 == number { singular } else { plural })
}

/// What still references a member: "2 expenses and 1 transfer",
/// "1 expense", or "3 transfers".
fn usage(expenses: usize, transfers: usize) -> String {
    match (expenses, transfers) {
        (0, transfers) => count(transfers, "transfer", "transfers"),
        (expenses, 0) => count(expenses, "expense", "expenses"),
        (expenses, transfers) => format!(
            "{} and {}",
            count(expenses, "expense", "expenses"),
            count(transfers, "transfer", "transfers")
        ),
    }
}

#[cfg(test)]
mod tests;
