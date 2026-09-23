//! Settling up: turning balances into payments between members.
//!
//! Every strategy pays out exactly the members' net balances: each debtor
//! pays what they owe and each creditor receives what they are owed, so
//! nobody pays more than their debt. The strategies differ in how many
//! payments they need and how much work that takes.

mod direct;
mod greedy;
mod minimum;

use crate::date::Date;
use crate::error::SettlementError;
use crate::id::MemberId;
use crate::ledger::Ledger;
use crate::money::Money;
use crate::transfer::Transfer;

/// How [`Group::settlement`](crate::Group::settlement) turns balances into
/// payments.
///
/// There is no default: the strategies trade the number of payments against
/// computation time, and the caller chooses. `n` below is the number of
/// members with a non-zero balance.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Date, Group, Member, Money, SettlementStrategy, Transfer};
///
/// let mut group = Group::builder("Flat", Currency::EUR).build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?);
/// let bob = group.add_member(Member::builder("Bob").build()?);
/// let day = Date::new(2026, 9, 23)?;
/// // Bob paid Ana 20.00 too much.
/// group.add_transfer(Transfer::builder(bob, ana, Money::parse("20.00", Currency::EUR)?, day).build()?)?;
///
/// for strategy in SettlementStrategy::all() {
///     let settlement = group.settlement(strategy)?;
///     assert_eq!(settlement.payments().len(), 1);
///     assert_eq!(settlement.payments()[0].from(), ana);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SettlementStrategy {
    /// The fewest payments possible.
    ///
    /// Finding them is NP-hard, and this solves it exactly: exact opposite
    /// balances are paired first (always optimal), then a dynamic program
    /// over all subsets of the remaining `n` balances finds the largest
    /// number of groups that each sum to zero; every such group settles with
    /// one payment fewer than its size. It needs O(2ⁿ·n) time and 2ⁿ bytes
    /// of memory, with no size limit, so every extra open balance doubles
    /// both: on an Apple M4 Pro, 12 balances take 18 µs, 16 take 0.36 ms,
    /// and 20 take 7 ms and 1 MiB; 30 would take seconds and 1 GiB. The
    /// operating system may end the process before an allocation fails, so
    /// prefer [`Greedy`](Self::Greedy) for large groups.
    MinimumTransfers,
    /// The largest debtor pays the largest creditor, repeatedly.
    ///
    /// O(n log n) time and at most n − 1 payments (85 µs for 1 000 members
    /// on an Apple M4 Pro); usually as few as
    /// [`MinimumTransfers`](Self::MinimumTransfers), but not always.
    Greedy,
    /// No simplification: everyone pays back the people who paid for them.
    ///
    /// Every participant owes every payer of each expense their share of what
    /// that payer paid; the debts between each pair of members are netted
    /// across all entries. Each payment traces back to shared expenses, at
    /// the cost of the most payments. O(p) time for p pairs with a debt,
    /// which the group keeps current as entries change (110 µs for 100
    /// members who all share every expense, 4 950 pairs, on an Apple M4
    /// Pro).
    Direct,
}

impl SettlementStrategy {
    /// Every strategy.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::MinimumTransfers, Self::Greedy, Self::Direct]
    }
}

/// The payments that settle a group, all in the group currency.
///
/// Recording every payment as a [`Transfer`] (see
/// [`Payment::to_transfer`]) brings every balance to zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    strategy: SettlementStrategy,
    payments: Vec<Payment>,
}

impl Settlement {
    /// The strategy that produced the payments.
    #[must_use]
    pub const fn strategy(&self) -> SettlementStrategy {
        self.strategy
    }

    /// The payments, ordered by payer and then by recipient.
    #[must_use]
    pub fn payments(&self) -> &[Payment] {
        &self.payments
    }

    /// The number of payments.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.payments.len()
    }

    /// Whether everyone is already settled.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.payments.is_empty()
    }
}

/// One payment of a [`Settlement`]: `from` pays `amount` to `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Payment {
    from: MemberId,
    to: MemberId,
    amount: Money,
}

impl Payment {
    /// The member who pays.
    #[must_use]
    pub const fn from(&self) -> MemberId {
        self.from
    }

    /// The member who is paid.
    #[must_use]
    pub const fn to(&self) -> MemberId {
        self.to
    }

    /// The amount, in the group currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The transfer recording this payment, made on `date`.
    #[must_use]
    pub fn to_transfer(&self, date: Date) -> Transfer {
        Transfer::new_unchecked(self.from, self.to, self.amount, date)
    }
}

/// A payment in minor units: (from, to, amount).
type Flow = (MemberId, MemberId, i64);

/// Settles the balances of `ledger` with `strategy`.
pub(crate) fn settle(
    strategy: SettlementStrategy,
    ledger: &Ledger,
) -> Result<Settlement, SettlementError> {
    let mut flows = match strategy {
        SettlementStrategy::MinimumTransfers => minimum::settle(&ledger.balances())?,
        SettlementStrategy::Greedy => greedy::settle(&ledger.balances()),
        SettlementStrategy::Direct => direct::settle(ledger.pairs()),
    };
    flows.sort_unstable();
    let currency = ledger.currency();
    let payments = flows
        .into_iter()
        .map(|(from, to, amount)| Payment {
            from,
            to,
            amount: Money::from_minor_units(amount, currency),
        })
        .collect();
    Ok(Settlement { strategy, payments })
}

#[cfg(test)]
mod tests;
