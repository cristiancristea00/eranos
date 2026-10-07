//! The failures of building a group, of changing it, and of reading a
//! language tag.

use eranos_core::{CategoryId, ExpenseId, MemberId, TransferId};
use eranos_money::Currency;

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

/// A [`Group`](crate::Group) that could not be built, or an operation that
/// would break its consistency; the group is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GroupError {
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
        currency: Currency,
    },
    /// An entry in the group currency has a converted cost; only a local
    /// cost is in the group currency.
    #[error("an entry in the group currency cannot have an exchange rate")]
    UnexpectedExchangeRate,
    /// The same currency appears twice among the rates of a currency change.
    #[error("two exchange rates were given for {currency}")]
    DuplicateExchangeRate {
        /// The repeated currency.
        currency: Currency,
    },
    /// A member with this ID already exists.
    #[error("member {0} already exists")]
    DuplicateMember(MemberId),
    /// No member with this ID.
    #[error("unknown member {0}")]
    UnknownMember(MemberId),
    /// An archived member cannot join a new expense.
    #[error("member {0} is archived")]
    ArchivedMember(MemberId),
    /// The member is still referenced and cannot be removed; archive it
    /// instead.
    #[error("member {member} is used by {}", usage(expenses.len(), transfers.len()))]
    MemberInUse {
        /// The member that was to be removed.
        member: MemberId,
        /// The expenses that reference the member.
        expenses: Vec<ExpenseId>,
        /// The transfers that reference the member.
        transfers: Vec<TransferId>,
    },
    /// A category with this ID already exists.
    #[error("category {0} already exists")]
    DuplicateCategory(CategoryId),
    /// Another category already has this name, compared by Unicode canonical
    /// caseless matching.
    #[error("category {existing} already has this name")]
    DuplicateCategoryName {
        /// The category that has the name.
        existing: CategoryId,
    },
    /// No category with this ID.
    #[error("unknown category {0}")]
    UnknownCategory(CategoryId),
    /// The built-in Uncategorised category cannot be renamed or removed.
    #[error("the built-in category cannot be renamed or removed")]
    BuiltInCategory,
    /// An expense with this ID already exists.
    #[error("expense {0} already exists")]
    DuplicateExpense(ExpenseId),
    /// No expense with this ID.
    #[error("unknown expense {0}")]
    UnknownExpense(ExpenseId),
    /// A transfer with this ID already exists.
    #[error("transfer {0} already exists")]
    DuplicateTransfer(TransferId),
    /// No transfer with this ID.
    #[error("unknown transfer {0}")]
    UnknownTransfer(TransferId),
    /// A shared group's lock was asked for without waiting, while another
    /// call held it.
    #[error("the group is locked by another call")]
    Busy,
    /// A total would leave the range of an amount.
    #[error("the group totals would be out of range")]
    Overflow,
}

/// Builds a value from a [`GroupError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromGroupError: Sized {
    /// The value for [`GroupError::UnsupportedFormat`].
    fn unsupported_format(format: u32, message: String) -> Self;

    /// The value for [`GroupError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for [`GroupError::MissingExchangeRate`].
    fn missing_exchange_rate(currency: Currency, message: String) -> Self;

    /// The value for [`GroupError::UnexpectedExchangeRate`].
    fn unexpected_exchange_rate(message: String) -> Self;

    /// The value for [`GroupError::DuplicateExchangeRate`].
    fn duplicate_exchange_rate(currency: Currency, message: String) -> Self;

    /// The value for [`GroupError::DuplicateMember`].
    fn duplicate_member(member: MemberId, message: String) -> Self;

    /// The value for [`GroupError::UnknownMember`].
    fn unknown_member(member: MemberId, message: String) -> Self;

    /// The value for [`GroupError::ArchivedMember`].
    fn archived_member(member: MemberId, message: String) -> Self;

    /// The value for [`GroupError::MemberInUse`].
    fn member_in_use(
        member: MemberId,
        expenses: &[ExpenseId],
        transfers: &[TransferId],
        message: String,
    ) -> Self;

    /// The value for [`GroupError::DuplicateCategory`].
    fn duplicate_category(category: CategoryId, message: String) -> Self;

    /// The value for [`GroupError::DuplicateCategoryName`].
    fn duplicate_category_name(existing: CategoryId, message: String) -> Self;

    /// The value for [`GroupError::UnknownCategory`].
    fn unknown_category(category: CategoryId, message: String) -> Self;

    /// The value for [`GroupError::BuiltInCategory`].
    fn built_in_category(message: String) -> Self;

    /// The value for [`GroupError::DuplicateExpense`].
    fn duplicate_expense(expense: ExpenseId, message: String) -> Self;

    /// The value for [`GroupError::UnknownExpense`].
    fn unknown_expense(expense: ExpenseId, message: String) -> Self;

    /// The value for [`GroupError::DuplicateTransfer`].
    fn duplicate_transfer(transfer: TransferId, message: String) -> Self;

    /// The value for [`GroupError::UnknownTransfer`].
    fn unknown_transfer(transfer: TransferId, message: String) -> Self;

    /// The value for [`GroupError::Busy`].
    fn busy(message: String) -> Self;

    /// The value for [`GroupError::Overflow`].
    fn overflow(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_group_error(error: &GroupError) -> Self {
        let message = error.to_string();
        match error {
            GroupError::UnsupportedFormat { format } => Self::unsupported_format(*format, message),
            GroupError::EmptyName => Self::empty_name(message),
            GroupError::MissingExchangeRate { currency } => {
                Self::missing_exchange_rate(*currency, message)
            }
            GroupError::UnexpectedExchangeRate => Self::unexpected_exchange_rate(message),
            GroupError::DuplicateExchangeRate { currency } => {
                Self::duplicate_exchange_rate(*currency, message)
            }
            GroupError::DuplicateMember(member) => Self::duplicate_member(*member, message),
            GroupError::UnknownMember(member) => Self::unknown_member(*member, message),
            GroupError::ArchivedMember(member) => Self::archived_member(*member, message),
            GroupError::MemberInUse {
                member,
                expenses,
                transfers,
            } => Self::member_in_use(*member, expenses, transfers, message),
            GroupError::DuplicateCategory(category) => Self::duplicate_category(*category, message),
            GroupError::DuplicateCategoryName { existing } => {
                Self::duplicate_category_name(*existing, message)
            }
            GroupError::UnknownCategory(category) => Self::unknown_category(*category, message),
            GroupError::BuiltInCategory => Self::built_in_category(message),
            GroupError::DuplicateExpense(expense) => Self::duplicate_expense(*expense, message),
            GroupError::UnknownExpense(expense) => Self::unknown_expense(*expense, message),
            GroupError::DuplicateTransfer(transfer) => Self::duplicate_transfer(*transfer, message),
            GroupError::UnknownTransfer(transfer) => Self::unknown_transfer(*transfer, message),
            GroupError::Busy => Self::busy(message),
            GroupError::Overflow => Self::overflow(message),
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
