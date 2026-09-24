//! The failure of building an expense or one of its items.

use eranos_core::MemberId;
use eranos_money::{Currency, Money};

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
