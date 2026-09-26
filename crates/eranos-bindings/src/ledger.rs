//! The group's totals and balances, as the bindings carry them.

use eranos::ledger as library;
use eranos_ledger::FromBalance;

use crate::money::Money;

/// Where a member stands with the group.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Balance {
    /// The others owe the member `amount`.
    GetsBack {
        /// What the member gets back.
        amount: Money,
    },
    /// The member owes the others `amount`.
    Owes {
        /// What the member owes.
        amount: Money,
    },
    /// The member owes nothing and is owed nothing.
    Settled,
}

impl FromBalance for Balance {
    fn gets_back(amount: eranos::Money) -> Self {
        Self::GetsBack {
            amount: Money::from_library(amount),
        }
    }

    fn owes(amount: eranos::Money) -> Self {
        Self::Owes {
            amount: Money::from_library(amount),
        }
    }

    fn settled() -> Self {
        Self::Settled
    }
}

/// A member's totals, in the group currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberSummary {
    /// Where the member stands.
    pub balance: Balance,
    /// The member, as their identifier.
    pub member: String,
    /// What the member is owed, less what they owe: positive when they get
    /// money back.
    pub net: Money,
    /// What the member paid for expenses.
    pub paid: Money,
    /// What the member received in transfers.
    pub received: Money,
    /// What the member sent in transfers.
    pub sent: Money,
    /// The member's share of the expenses: what they consumed.
    pub share: Money,
}

impl MemberSummary {
    /// The record of `summary`.
    fn from_library(summary: library::MemberSummary) -> Self {
        Self {
            balance: Balance::from_balance(summary.balance()),
            member: summary.member().to_string(),
            net: Money::from_library(summary.net()),
            paid: Money::from_library(summary.paid()),
            received: Money::from_library(summary.received()),
            sent: Money::from_library(summary.sent()),
            share: Money::from_library(summary.share()),
        }
    }
}

/// What the expenses of a category add up to, in the group currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CategoryTotal {
    /// The category, as its identifier.
    pub category: String,
    /// The total.
    pub total: Money,
}

impl CategoryTotal {
    /// The record of `total`.
    fn from_library(total: library::CategoryTotal) -> Self {
        Self {
            category: total.category().to_string(),
            total: Money::from_library(total.total()),
        }
    }
}

/// What one member's shares add up to in each category.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberCategoryTotals {
    /// The member, as their identifier.
    pub member: String,
    /// The member's total in every category they have a share in.
    pub totals: Vec<CategoryTotal>,
}

/// The group's totals: overall, per member, per category, and per member per
/// category, all in the group currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Summary {
    /// The total of every category.
    pub category_totals: Vec<CategoryTotal>,
    /// The totals per category of every member.
    pub member_category_totals: Vec<MemberCategoryTotals>,
    /// The totals of every member.
    pub members: Vec<MemberSummary>,
    /// What the group spent.
    pub total_spent: Money,
}

impl Summary {
    /// The record of `summary`.
    pub(crate) fn from_library(summary: &library::Summary) -> Self {
        Self {
            category_totals: summary
                .category_totals()
                .map(CategoryTotal::from_library)
                .collect(),
            member_category_totals: summary
                .members()
                .map(|member| MemberCategoryTotals {
                    member: member.member().to_string(),
                    totals: summary
                        .member_category_totals(member.member())
                        .map(CategoryTotal::from_library)
                        .collect(),
                })
                .collect(),
            members: summary.members().map(MemberSummary::from_library).collect(),
            total_spent: Money::from_library(summary.total_spent()),
        }
    }
}

/// What one member paid of an expense and what their share of it is.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberShare {
    /// The member, as their identifier.
    pub member: String,
    /// What the member paid, tip included.
    pub paid: Money,
    /// The member's share, tip included.
    pub share: Money,
}

/// How one expense divides, in the group currency.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpenseShares {
    /// What every member paid and consumed.
    pub members: Vec<MemberShare>,
    /// The tip.
    pub tip: Money,
    /// The total: the amount and the tip.
    pub total: Money,
}

impl ExpenseShares {
    /// The record of `shares`.
    pub(crate) fn from_library(shares: &library::ExpenseShares) -> Self {
        Self {
            members: shares
                .members()
                .iter()
                .map(|share| MemberShare {
                    member: share.member().to_string(),
                    paid: Money::from_library(share.paid()),
                    share: Money::from_library(share.share()),
                })
                .collect(),
            tip: Money::from_library(shares.tip()),
            total: Money::from_library(shares.total()),
        }
    }
}

#[cfg(test)]
mod tests;
