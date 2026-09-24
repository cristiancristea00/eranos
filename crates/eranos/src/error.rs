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
    /// Arithmetic between amounts of two different currencies.
    #[error("cannot combine {left} and {right} amounts")]
    CurrencyMismatch {
        /// The currency of the left operand.
        left: Currency,
        /// The currency of the right operand.
        right: Currency,
    },
    /// A required text field is empty after trimming.
    #[error("the {field} must not be empty")]
    Empty {
        /// The field that was empty.
        field: Field,
    },
    /// An amount that is not a decimal number.
    #[error("invalid amount: expected digits with an optional `.` and sign")]
    InvalidAmount,
    /// A calendar date that does not exist, is outside years 1 to 9999, or is
    /// not ISO 8601 text.
    #[error("invalid date: expected an existing ISO 8601 date between years 1 and 9999")]
    InvalidDate,
    /// An email address without exactly one `@`, with an empty side, or with
    /// whitespace.
    #[error("invalid email address")]
    InvalidEmail,
    /// An exchange rate that is not a positive decimal number with at most 18
    /// decimal places.
    #[error("invalid exchange rate: expected a positive number with at most 18 decimal places")]
    InvalidExchangeRate,
    /// An IBAN that failed validation.
    #[error("invalid IBAN: {0}")]
    InvalidIban(IbanError),
    /// An identifier that is not UUID text.
    #[error("invalid identifier: expected a UUID")]
    InvalidId,
    /// A language tag that does not have the shape of a BCP 47 tag.
    #[error("invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`")]
    InvalidLanguage,
    /// A number of parts that is not a positive decimal number with at most
    /// two decimal places.
    #[error("invalid parts: expected a positive number with at most 2 decimal places")]
    InvalidParts,
    /// A percentage that is not a decimal number with at most two decimal
    /// places.
    #[error("invalid percentage: expected a number with at most 2 decimal places")]
    InvalidPercentage,
    /// A phone number that is not `+` followed by 7 to 15 digits.
    #[error("invalid phone number: expected `+` followed by 7 to 15 digits")]
    InvalidPhoneNumber,
    /// A time of day that does not exist, has seconds, or is not ISO 8601
    /// text.
    #[error("invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds")]
    InvalidTime,
    /// A number or a result that does not fit its range.
    #[error("value out of range")]
    Overflow,
    /// An amount with more decimal places than its currency has.
    #[error("{currency} amounts have at most {allowed} decimal places")]
    TooManyDecimals {
        /// The currency's number of decimal places.
        allowed: u8,
        /// The currency of the amount.
        currency: Currency,
    },
    /// A currency code that is not an active ISO 4217 code.
    #[error("unknown currency code")]
    UnknownCurrency,
}

/// The text field a [`ValueError::Empty`] refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Field {
    /// [`Account::name`](crate::Account::name).
    AccountName,
    /// [`Category::name`](crate::Category::name).
    CategoryName,
    /// [`Expense::name`](crate::Expense::name).
    ExpenseName,
    /// [`Member::first_name`](crate::Member::first_name).
    FirstName,
    /// [`Group::name`](crate::Group::name).
    GroupName,
    /// [`Item::name`](crate::Item::name).
    ItemName,
    /// A [`Username`](crate::Username).
    Username,
}

impl fmt::Display for Field {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AccountName => "account name",
            Self::CategoryName => "category name",
            Self::ExpenseName => "expense name",
            Self::FirstName => "first name",
            Self::GroupName => "group name",
            Self::ItemName => "item name",
            Self::Username => "username",
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
    /// The check digits are 00, 01, or 99, which ISO 13616 never issues, or
    /// the ISO 7064 mod-97 check failed: the number contains a typo.
    #[error("the check digits do not match")]
    InvalidChecksum,
    /// Not two letters followed by two check digits.
    #[error("expected a two-letter country code followed by two check digits")]
    InvalidFormat,
    /// A letter where the country's account number has a digit, or a digit
    /// where it has a letter, by the BBAN structure of the SWIFT IBAN
    /// registry.
    #[error("the account number does not match this country's format")]
    InvalidStructure,
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
}

/// An [`Expense`](crate::Expense) or [`Item`](crate::Item) whose parts do not
/// fit together.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
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
    /// A field value is invalid.
    #[error(transparent)]
    Value(#[from] ValueError),
    /// An item quantity of zero.
    #[error("an item quantity must be at least 1")]
    ZeroQuantity,
}

/// A [`Transfer`](crate::Transfer) that cannot exist.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TransferError {
    /// The amount is zero or negative.
    #[error("the amount must be positive")]
    NonPositiveAmount,
    /// The sender and the recipient are the same member.
    #[error("a transfer needs two different members")]
    SameMember,
    /// A field value is invalid.
    #[error(transparent)]
    Value(#[from] ValueError),
}

/// A [`Group`](crate::Group) operation that would break the group's
/// consistency; the group is unchanged when one is returned.
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
    /// Another category already has this name, compared case-insensitively.
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
