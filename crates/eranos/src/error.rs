//! Error types, one per kind of operation.
//!
//! No error echoes the text it rejected: inputs include account numbers and
//! contact details, and error messages end up in logs.

use std::fmt;

use crate::id::{CategoryId, ExpenseId, MemberId, TransferId};
use crate::money::{Currency, Money};

/// A single input value that failed validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ValueError {
    /// A required text field is empty after trimming.
    #[error("the {field} must not be empty")]
    Empty {
        /// The field that was empty.
        field: Field,
    },
    /// An email address without exactly one `@`, with an empty side, or with
    /// whitespace.
    #[error("invalid email address")]
    InvalidEmail,
    /// A phone number that is not `+` followed by 7 to 15 digits.
    #[error("invalid phone number: expected `+` followed by 7 to 15 digits")]
    InvalidPhoneNumber,
    /// An IBAN that failed validation.
    #[error("invalid IBAN: {0}")]
    InvalidIban(IbanError),
    /// An identifier that is not UUID text.
    #[error("invalid identifier: expected a UUID")]
    InvalidId,
    /// A currency code that is not an active ISO 4217 code.
    #[error("unknown currency code")]
    UnknownCurrency,
    /// An amount that is not a decimal number.
    #[error("invalid amount: expected digits with an optional `.` and sign")]
    InvalidAmount,
    /// An amount with more decimal places than its currency has.
    #[error("{currency} amounts have at most {allowed} decimal places")]
    TooManyDecimals {
        /// The currency's number of decimal places.
        allowed: u8,
        /// The currency of the amount.
        currency: Currency,
    },
    /// A percentage that is not a decimal number with at most two decimal
    /// places.
    #[error("invalid percentage: expected a number with at most 2 decimal places")]
    InvalidPercentage,
    /// A number of parts that is not a positive decimal number with at most
    /// two decimal places.
    #[error("invalid parts: expected a positive number with at most 2 decimal places")]
    InvalidParts,
    /// An exchange rate that is not a positive decimal number with at most 18
    /// decimal places.
    #[error("invalid exchange rate: expected a positive number with at most 18 decimal places")]
    InvalidExchangeRate,
    /// A calendar date that does not exist or is not `YYYY-MM-DD`.
    #[error("invalid date: expected an existing `YYYY-MM-DD` date between years 1 and 9999")]
    InvalidDate,
    /// A time of day that does not exist or is not `hh:mm`.
    #[error("invalid time: expected `hh:mm` between 00:00 and 23:59")]
    InvalidTime,
    /// A language tag that does not have the shape of a BCP 47 tag.
    #[error("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")]
    InvalidLanguage,
    /// Arithmetic between amounts of two different currencies.
    #[error("cannot combine {left} and {right} amounts")]
    CurrencyMismatch {
        /// The currency of the left operand.
        left: Currency,
        /// The currency of the right operand.
        right: Currency,
    },
    /// A number or a result that does not fit its range.
    #[error("value out of range")]
    Overflow,
}

/// The text field a [`ValueError::Empty`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Field {
    /// [`Group::name`](crate::Group::name).
    GroupName,
    /// [`Member::first_name`](crate::Member::first_name).
    FirstName,
    /// [`Account::name`](crate::Account::name).
    AccountName,
    /// A [`Username`](crate::Username).
    Username,
    /// [`Category::name`](crate::Category::name).
    CategoryName,
    /// [`Expense::name`](crate::Expense::name).
    ExpenseName,
    /// [`Item::name`](crate::Item::name).
    ItemName,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::GroupName => "group name",
            Self::FirstName => "first name",
            Self::AccountName => "account name",
            Self::Username => "username",
            Self::CategoryName => "category name",
            Self::ExpenseName => "expense name",
            Self::ItemName => "item name",
        })
    }
}

/// Why an IBAN was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum IbanError {
    /// A character other than an ASCII letter, a digit, or whitespace (which
    /// is dropped).
    #[error("only letters, digits, and spaces are allowed")]
    InvalidCharacter,
    /// Not two letters followed by two check digits.
    #[error("expected a two-letter country code followed by two check digits")]
    InvalidFormat,
    /// A country code without an entry in the SWIFT IBAN registry.
    #[error("the country code is not in the IBAN registry")]
    UnknownCountry,
    /// The wrong number of characters for the country.
    #[error("expected {expected} characters for this country, found {found}")]
    WrongLength {
        /// The registry length for the country.
        expected: usize,
        /// The number of characters after removing spaces.
        found: usize,
    },
    /// The ISO 7064 mod-97 check failed: the number contains a typo.
    #[error("the check digits do not match")]
    InvalidChecksum,
}

/// An [`Expense`](crate::Expense) or [`Item`](crate::Item) whose parts do not
/// fit together.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum ExpenseError {
    /// A field value is invalid.
    #[error(transparent)]
    Value(#[from] ValueError),
    /// The amount or unit price is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// Nobody paid: neither `paid_by` nor `payers` was set.
    #[error("the expense has no payers")]
    MissingPayers,
    /// Both `paid_by` and `payers` were set.
    #[error("the payers were set twice, through `paid_by` and `payers`")]
    ConflictingPayers,
    /// Nobody shares the cost: neither `split` nor `items` was set.
    #[error("the expense has no split")]
    MissingSplit,
    /// Both `split` and `items` were set, or both `split` and `shared_by`.
    #[error("the split was set twice, as a whole and item by item")]
    ConflictingSplit,
    /// A split lists no member.
    #[error("a split needs at least one member")]
    EmptySplit,
    /// An itemised expense without items.
    #[error("an itemised expense needs at least one item")]
    NoItems,
    /// A member appears more than once in one split.
    #[error("member {member} appears more than once in a split")]
    DuplicateMember {
        /// The repeated member.
        member: MemberId,
    },
    /// A member's percentage or amount is zero or negative.
    #[error("member {member} has a share that is not positive")]
    NonPositiveShare {
        /// The member whose share is not positive.
        member: MemberId,
    },
    /// The percentages of a split do not add up to exactly 100 %.
    #[error("the percentages add up to {total_basis_points} basis points instead of 10000 (100%)")]
    PercentagesDoNotAddUp {
        /// The sum of the percentages, in basis points (100 % = 10 000).
        total_basis_points: u64,
    },
    /// The amounts of a split do not add up to the amount being split.
    #[error("the amounts add up to {actual} instead of {expected}")]
    AmountsDoNotAddUp {
        /// The sum of the listed amounts.
        actual: Money,
        /// The amount being split.
        expected: Money,
    },
    /// The item line totals do not add up to the expense amount.
    #[error("the items add up to {actual} instead of {expected}")]
    ItemsDoNotAddUp {
        /// The sum of the item line totals.
        actual: Money,
        /// The expense amount.
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
    /// An item quantity of zero.
    #[error("an item quantity must be at least 1")]
    ZeroQuantity,
    /// A fixed tip below zero.
    #[error("a tip cannot be negative")]
    NegativeTip,
    /// A total that does not fit the range of an amount.
    #[error("the amounts are out of range")]
    Overflow,
}

/// A [`Transfer`](crate::Transfer) that cannot exist.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TransferError {
    /// A field value is invalid.
    #[error(transparent)]
    Value(#[from] ValueError),
    /// The sender and the recipient are the same member.
    #[error("a transfer needs two different members")]
    SameMember,
    /// The amount is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
}

/// A [`Group`](crate::Group) operation that would break the group's
/// consistency; the group is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum GroupError {
    /// No member with this ID.
    #[error("unknown member {0}")]
    UnknownMember(MemberId),
    /// No category with this ID.
    #[error("unknown category {0}")]
    UnknownCategory(CategoryId),
    /// No expense with this ID.
    #[error("unknown expense {0}")]
    UnknownExpense(ExpenseId),
    /// No transfer with this ID.
    #[error("unknown transfer {0}")]
    UnknownTransfer(TransferId),
    /// A member with this ID already exists.
    #[error("member {0} already exists")]
    DuplicateMember(MemberId),
    /// A category with this ID already exists.
    #[error("category {0} already exists")]
    DuplicateCategory(CategoryId),
    /// An expense with this ID already exists.
    #[error("expense {0} already exists")]
    DuplicateExpense(ExpenseId),
    /// A transfer with this ID already exists.
    #[error("transfer {0} already exists")]
    DuplicateTransfer(TransferId),
    /// Another category already has this name, compared case-insensitively.
    #[error("category {existing} already has this name")]
    DuplicateCategoryName {
        /// The category that has the name.
        existing: CategoryId,
    },
    /// An archived member cannot join a new expense.
    #[error("member {0} is archived")]
    ArchivedMember(MemberId),
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
    /// The category is still referenced and cannot be removed.
    #[error("category {category} is used by {}", count(expenses.len(), "expense", "expenses"))]
    CategoryInUse {
        /// The category that was to be removed.
        category: CategoryId,
        /// The expenses that reference the category.
        expenses: Vec<ExpenseId>,
    },
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
    /// A total would leave the range of an amount.
    #[error("the group totals would be out of range")]
    Overflow,
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
