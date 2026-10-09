//! The failure of building an expense or one of its items.

use eranos_core::MemberId;
use eranos_money::{CurrencyCode, Money};

/// An [`Expense`](crate::Expense) or [`ExpenseItem`](crate::ExpenseItem) whose parts do not
/// fit together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum ExpenseError {
    /// An expense whose name is blank.
    #[error("the expense name must not be empty")]
    EmptyName,
    /// The amount or unit price is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// An amount inside the expense is in a different currency.
    #[error("expected an amount in {expected}, found one in {found}")]
    CurrencyMismatch {
        /// The expense currency.
        expected: CurrencyCode,
        /// The currency of the offending amount.
        found: CurrencyCode,
    },
    /// A split lists no member.
    #[error("a split needs at least one member")]
    EmptySplit,
    /// A member appears more than once in one split.
    #[error("a member appears more than once in the split")]
    DuplicateMember {
        /// The repeated member.
        member: MemberId,
    },
    /// A member's percentage or amount is zero or negative.
    #[error("a member's share is not positive")]
    NonPositiveShare {
        /// The member whose share is not positive.
        member: MemberId,
    },
    /// The amounts of a split do not add up to the amount being split.
    #[error("the amounts add up to {actual} instead of {expected}")]
    AmountsDoNotAddUp {
        /// The amount being split.
        expected: Money,
        /// The sum of the listed amounts.
        actual: Money,
    },
    /// The percentages of a split do not add up to exactly 100 %.
    #[error("the percentages add up to {total_basis_points} basis points instead of 10000 (100%)")]
    PercentagesDoNotAddUp {
        /// The sum of the percentages, in basis points (100 % = 10 000).
        total_basis_points: u64,
    },
    /// An itemised expense without items.
    #[error("an itemised expense needs at least one item")]
    NoItems,
    /// An item whose name is blank.
    #[error("the item name must not be empty")]
    EmptyItemName,
    /// The item line totals do not add up to the expense amount.
    #[error("the items add up to {actual} instead of {expected}")]
    ItemsDoNotAddUp {
        /// The expense amount.
        expected: Money,
        /// The sum of the item line totals.
        actual: Money,
    },
    /// A fixed discount below zero.
    #[error("a discount cannot be negative")]
    NegativeDiscount,
    /// A percentage discount above 100 %.
    #[error("a discount cannot be more than 100%")]
    DiscountAboveHundredPercent,
    /// A fixed discount larger than what it reduces: the line total before
    /// the discount, or the unit price for a discount off each unit.
    #[error("the discount is larger than the price it reduces")]
    DiscountAbovePrice,
    /// A fixed tip below zero.
    #[error("a tip cannot be negative")]
    NegativeTip,
    /// A total that does not fit the range of an amount.
    #[error("the amounts are out of range")]
    Overflow,
}

/// Builds a value from an [`ExpenseError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromExpenseError: Sized {
    /// The value for [`ExpenseError::EmptyName`].
    fn empty_name(message: String) -> Self;

    /// The value for [`ExpenseError::NonPositiveAmount`].
    fn non_positive_amount(message: String) -> Self;

    /// The value for [`ExpenseError::CurrencyMismatch`].
    fn currency_mismatch(expected: CurrencyCode, found: CurrencyCode, message: String) -> Self;

    /// The value for [`ExpenseError::EmptySplit`].
    fn empty_split(message: String) -> Self;

    /// The value for [`ExpenseError::DuplicateMember`].
    fn duplicate_member(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseError::NonPositiveShare`].
    fn non_positive_share(member: MemberId, message: String) -> Self;

    /// The value for [`ExpenseError::AmountsDoNotAddUp`].
    fn amounts_do_not_add_up(expected: Money, actual: Money, message: String) -> Self;

    /// The value for [`ExpenseError::PercentagesDoNotAddUp`].
    fn percentages_do_not_add_up(total_basis_points: u64, message: String) -> Self;

    /// The value for [`ExpenseError::NoItems`].
    fn no_items(message: String) -> Self;

    /// The value for [`ExpenseError::EmptyItemName`].
    fn empty_item_name(message: String) -> Self;

    /// The value for [`ExpenseError::ItemsDoNotAddUp`].
    fn items_do_not_add_up(expected: Money, actual: Money, message: String) -> Self;

    /// The value for [`ExpenseError::NegativeDiscount`].
    fn negative_discount(message: String) -> Self;

    /// The value for [`ExpenseError::DiscountAboveHundredPercent`].
    fn discount_above_hundred_percent(message: String) -> Self;

    /// The value for [`ExpenseError::DiscountAbovePrice`].
    fn discount_above_price(message: String) -> Self;

    /// The value for [`ExpenseError::NegativeTip`].
    fn negative_tip(message: String) -> Self;

    /// The value for [`ExpenseError::Overflow`].
    fn overflow(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_expense_error(error: &ExpenseError) -> Self {
        let message = error.to_string();
        match error {
            ExpenseError::EmptyName => Self::empty_name(message),
            ExpenseError::NonPositiveAmount => Self::non_positive_amount(message),
            ExpenseError::CurrencyMismatch { expected, found } => {
                Self::currency_mismatch(*expected, *found, message)
            }
            ExpenseError::EmptySplit => Self::empty_split(message),
            ExpenseError::DuplicateMember { member } => Self::duplicate_member(*member, message),
            ExpenseError::NonPositiveShare { member } => Self::non_positive_share(*member, message),
            ExpenseError::AmountsDoNotAddUp { expected, actual } => {
                Self::amounts_do_not_add_up(*expected, *actual, message)
            }
            ExpenseError::PercentagesDoNotAddUp { total_basis_points } => {
                Self::percentages_do_not_add_up(*total_basis_points, message)
            }
            ExpenseError::NoItems => Self::no_items(message),
            ExpenseError::EmptyItemName => Self::empty_item_name(message),
            ExpenseError::ItemsDoNotAddUp { expected, actual } => {
                Self::items_do_not_add_up(*expected, *actual, message)
            }
            ExpenseError::NegativeDiscount => Self::negative_discount(message),
            ExpenseError::DiscountAboveHundredPercent => {
                Self::discount_above_hundred_percent(message)
            }
            ExpenseError::DiscountAbovePrice => Self::discount_above_price(message),
            ExpenseError::NegativeTip => Self::negative_tip(message),
            ExpenseError::Overflow => Self::overflow(message),
        }
    }
}

#[cfg(test)]
mod tests;
