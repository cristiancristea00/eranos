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
pub use eranos_contact::ContactError;
#[doc(inline)]
pub use eranos_core::IdError;
#[doc(inline)]
pub use eranos_money::MoneyError;

use crate::group::FORMAT;
use crate::id::{CategoryId, ExpenseId, MemberId, TransferId};
use crate::money::{Currency, Money};

/// A [`Member`](crate::Member) that could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum MemberError {
    /// A first name that is blank.
    #[error("the first name must not be empty")]
    EmptyFirstName,
}

/// A [`Category`](crate::Category) that could not be built or renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum CategoryError {
    /// A name that is blank.
    #[error("the category name must not be empty")]
    EmptyName,
}

/// A language tag that could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageError {
    /// A language tag that is not a Unicode BCP 47 locale identifier (UTS #35).
    #[error("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")]
    InvalidLanguage,
}

/// An [`Expense`](crate::Expense) or [`Item`](crate::Item) whose parts do not
/// fit together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ExpenseError {
    /// The amounts of a split do not add up to the amount being split.
    #[error("the amounts add up to {actual} instead of {expected}")]
    AmountsDoNotAddUp {
        /// The sum of the listed amounts.
        actual: Money,
        /// The amount being split.
        expected: Money,
    },
    /// An amount inside the expense is in a different currency.
    #[error("expected an amount in {expected}, found one in {found}")]
    CurrencyMismatch {
        /// The expense currency.
        expected: Currency,
        /// The currency of the offending amount.
        found: Currency,
    },
    /// A member appears more than once in one split.
    #[error("member {member} appears more than once in a split")]
    DuplicateMember {
        /// The repeated member.
        member: MemberId,
    },
    /// An item whose name is blank.
    #[error("the item name must not be empty")]
    EmptyItemName,
    /// An expense whose name is blank.
    #[error("the expense name must not be empty")]
    EmptyName,
    /// A split lists no member.
    #[error("a split needs at least one member")]
    EmptySplit,
    /// The item line totals do not add up to the expense amount.
    #[error("the items add up to {actual} instead of {expected}")]
    ItemsDoNotAddUp {
        /// The sum of the item line totals.
        actual: Money,
        /// The expense amount.
        expected: Money,
    },
    /// A fixed tip below zero.
    #[error("a tip cannot be negative")]
    NegativeTip,
    /// An itemised expense without items.
    #[error("an itemised expense needs at least one item")]
    NoItems,
    /// The amount or unit price is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// A member's percentage or amount is zero or negative.
    #[error("member {member} has a share that is not positive")]
    NonPositiveShare {
        /// The member whose share is not positive.
        member: MemberId,
    },
    /// A total that does not fit the range of an amount.
    #[error("the amounts are out of range")]
    Overflow,
    /// The percentages of a split do not add up to exactly 100 %.
    #[error("the percentages add up to {total_basis_points} basis points instead of 10000 (100%)")]
    PercentagesDoNotAddUp {
        /// The sum of the percentages, in basis points (100 % = 10 000).
        total_basis_points: u64,
    },
    /// An item quantity of zero.
    #[error("an item quantity must be at least 1")]
    ZeroQuantity,
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

/// A settlement that cannot be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum SettlementError {
    /// The exact solver needs 2ⁿ bytes for `n` open balances, and that table
    /// does not fit in the address space or the allocator refused it.
    #[error("not enough memory to find the minimum transfers between {balances} members")]
    OutOfMemory {
        /// The number of members with an open balance after pairing exact
        /// opposites.
        balances: usize,
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
