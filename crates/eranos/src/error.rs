//! Error types, one per module: every function reports the failure of its
//! own module, and no error wraps another.
//!
//! No error echoes the text it rejected: inputs include account numbers and
//! contact details, and error messages end up in logs.

#[doc(inline)]
pub use eranos_account::{AccountError, IbanError};
#[doc(inline)]
pub use eranos_calendar::CalendarError;
#[doc(inline)]
pub use eranos_category::CategoryError;
#[doc(inline)]
pub use eranos_contact::ContactError;
#[doc(inline)]
pub use eranos_core::IdError;
#[doc(inline)]
pub use eranos_expense::ExpenseError;
#[doc(inline)]
pub use eranos_member::MemberError;
#[doc(inline)]
pub use eranos_money::MoneyError;
#[doc(inline)]
pub use eranos_settlement::SettlementError;

use crate::group::FORMAT;
use crate::id::{CategoryId, ExpenseId, MemberId, TransferId};
use crate::money::Currency;

/// A language tag that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageError {
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    #[error("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")]
    InvalidLanguage,
}

/// A [`Transfer`](crate::Transfer) that cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum TransferError {
    /// The amount is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// The sender and the recipient are the same member.
    #[error("a transfer needs two different members")]
    SameMember,
}

/// A [`Group`](crate::Group) that could not be built, or an operation that
/// would break its consistency; the group is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GroupError {
    /// An archived member cannot join a new expense.
    #[error("member {0} is archived")]
    ArchivedMember(MemberId),
    /// The built-in Uncategorised category cannot be renamed or removed.
    #[error("the built-in category cannot be renamed or removed")]
    BuiltInCategory,
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
    /// The same currency appears twice among the rates of a currency change.
    #[error("two exchange rates were given for {currency}")]
    DuplicateExchangeRate {
        /// The repeated currency.
        currency: Currency,
    },
    /// An expense with this ID already exists.
    #[error("expense {0} already exists")]
    DuplicateExpense(ExpenseId),
    /// A member with this ID already exists.
    #[error("member {0} already exists")]
    DuplicateMember(MemberId),
    /// A transfer with this ID already exists.
    #[error("transfer {0} already exists")]
    DuplicateTransfer(TransferId),
    /// A group name that is blank.
    #[error("the group name must not be empty")]
    EmptyName,
    /// The member is still referenced and cannot be removed; archive it
    /// instead.
    #[error("member {member} is used by {}", usage(expenses.len(), transfers.len()))]
    MemberInUse {
        /// The expenses that reference the member.
        expenses: Vec<ExpenseId>,
        /// The member that was to be removed.
        member: MemberId,
        /// The transfers that reference the member.
        transfers: Vec<TransferId>,
    },
    /// An entry in another currency than the group's has no exchange rate
    /// into the group currency: its cost is local, or a currency change
    /// gave no rate for its currency.
    #[error("an exchange rate from {currency} into the group currency is required")]
    MissingExchangeRate {
        /// The currency without a rate.
        currency: Currency,
    },
    /// A total would leave the range of an amount.
    #[error("the group totals would be out of range")]
    Overflow,
    /// An entry in the group currency has a converted cost; only a local
    /// cost is in the group currency.
    #[error("an entry in the group currency cannot have an exchange rate")]
    UnexpectedExchangeRate,
    /// No category with this ID.
    #[error("unknown category {0}")]
    UnknownCategory(CategoryId),
    /// No expense with this ID.
    #[error("unknown expense {0}")]
    UnknownExpense(ExpenseId),
    /// No member with this ID.
    #[error("unknown member {0}")]
    UnknownMember(MemberId),
    /// No transfer with this ID.
    #[error("unknown transfer {0}")]
    UnknownTransfer(TransferId),
    /// A saved group in a format this version does not read: a newer one,
    /// whose fields it would drop, or an earlier one.
    #[error("unsupported group format {format}: this version reads format {FORMAT}")]
    UnsupportedFormat {
        /// The format the saved group names.
        format: u32,
    },
}

/// "1 expense", "2 expenses": a count with its noun in the right number.
fn count(number: usize, singular: &str, plural: &str) -> String {
    format!("{number} {}", if number == 1 { singular } else { plural })
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
