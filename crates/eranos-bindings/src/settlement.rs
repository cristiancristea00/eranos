//! Settlements and their payments, as the bindings carry them.

use eranos::settlement as library;
use eranos_settlement::FromSettlementStrategy;

use crate::money::Money;

/// How balances turn into payments.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettlementStrategy {
    /// The fewest payments while at most 20 balances are left after exact
    /// opposites pay each other, the greedy plan beyond; the settlement names
    /// the strategy whose plan it is.
    Adaptive,
    /// Everyone pays back the people who paid for them.
    Direct,
    /// The largest debtor pays the largest creditor, repeatedly, after exact
    /// opposites pay each other where that saves a payment.
    Greedy,
    /// The fewest payments possible.
    MinimumTransfers,
}

impl SettlementStrategy {
    /// The library's strategy.
    pub(crate) const fn to_library(self) -> library::SettlementStrategy {
        match self {
            Self::Adaptive => library::SettlementStrategy::Adaptive,
            Self::Direct => library::SettlementStrategy::Direct,
            Self::Greedy => library::SettlementStrategy::Greedy,
            Self::MinimumTransfers => library::SettlementStrategy::MinimumTransfers,
        }
    }
}

impl FromSettlementStrategy for SettlementStrategy {
    fn adaptive() -> Self {
        Self::Adaptive
    }

    fn direct() -> Self {
        Self::Direct
    }

    fn greedy() -> Self {
        Self::Greedy
    }

    fn minimum_transfers() -> Self {
        Self::MinimumTransfers
    }
}

/// One payment of a settlement: `from` pays `amount` to `to`.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Payment {
    /// The amount, in the group currency.
    pub amount: Money,
    /// The member who pays, as their identifier.
    pub from: String,
    /// The member who is paid, as their identifier.
    pub to: String,
}

/// The payments that settle every balance.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settlement {
    /// The payments, ordered by payer and then by recipient.
    pub payments: Vec<Payment>,
    /// The strategy that produced them: the one asked for, or, for the
    /// adaptive strategy, the one whose plan it chose.
    pub strategy: SettlementStrategy,
}

impl Settlement {
    /// The record of `settlement`.
    pub(crate) fn from_library(settlement: &library::Settlement) -> Self {
        Self {
            payments: settlement
                .payments()
                .iter()
                .map(|payment| Payment {
                    amount: Money::from_library(payment.amount()),
                    from: payment.from().to_string(),
                    to: payment.to().to_string(),
                })
                .collect(),
            strategy: SettlementStrategy::from_settlement_strategy(settlement.strategy()),
        }
    }
}

#[cfg(test)]
mod tests;
