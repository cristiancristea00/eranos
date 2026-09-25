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

/// Builds a value from an [`ExpenseError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromExpenseError: Sized {
    /// The value for [`ExpenseError::AmountsDoNotAddUp`].
    fn amounts_do_not_add_up(actual: Money, expected: Money, message: String) -> Self;

    /// The value for [`ExpenseError::CurrencyMismatch`].
    fn currency_mismatch(expected: Currency, found: Currency, message: String) -> Self;

    /// The value for [`ExpenseError::DuplicateMember`].
    fn duplicate_member(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseError::EmptyItemName`].
    fn empty_item_name(message: String) -> Self;

    /// The value for [`ExpenseError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for [`ExpenseError::EmptySplit`].
    fn empty_split(message: String) -> Self;

    /// The value for [`ExpenseError::ItemsDoNotAddUp`].
    fn items_do_not_add_up(actual: Money, expected: Money, message: String) -> Self;

    /// The value for [`ExpenseError::NegativeTip`].
    fn negative_tip(message: String) -> Self;

    /// The value for [`ExpenseError::NoItems`].
    fn no_items(message: String) -> Self;

    /// The value for [`ExpenseError::NonPositiveAmount`].
    fn non_positive_amount(message: String) -> Self;

    /// The value for [`ExpenseError::NonPositiveShare`].
    fn non_positive_share(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseError::Overflow`].
    fn overflow(message: String) -> Self;

    /// The value for [`ExpenseError::PercentagesDoNotAddUp`].
    fn percentages_do_not_add_up(message: String, total_basis_points: u64) -> Self;

    /// The value for [`ExpenseError::ZeroQuantity`].
    fn zero_quantity(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_expense_error(error: &ExpenseError) -> Self {
        let message = error.to_string();
        match error {
            ExpenseError::AmountsDoNotAddUp { actual, expected } => {
                Self::amounts_do_not_add_up(*actual, *expected, message)
            }
            ExpenseError::CurrencyMismatch { expected, found } => {
                Self::currency_mismatch(*expected, *found, message)
            }
            ExpenseError::DuplicateMember { member } => Self::duplicate_member(*member, message),
            ExpenseError::EmptyItemName => Self::empty_item_name(message),
            ExpenseError::EmptyName => Self::empty_name(message),
            ExpenseError::EmptySplit => Self::empty_split(message),
            ExpenseError::ItemsDoNotAddUp { actual, expected } => {
                Self::items_do_not_add_up(*actual, *expected, message)
            }
            ExpenseError::NegativeTip => Self::negative_tip(message),
            ExpenseError::NoItems => Self::no_items(message),
            ExpenseError::NonPositiveAmount => Self::non_positive_amount(message),
            ExpenseError::NonPositiveShare { member } => Self::non_positive_share(*member, message),
            ExpenseError::Overflow => Self::overflow(message),
            ExpenseError::PercentagesDoNotAddUp { total_basis_points } => {
                Self::percentages_do_not_add_up(message, *total_basis_points)
            }
            ExpenseError::ZeroQuantity => Self::zero_quantity(message),
        }
    }
}

#[cfg(test)]
mod tests;
