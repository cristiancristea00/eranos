//! Settling up: turning balances into payments between members.
//!
//! Every strategy pays out exactly the members' net balances: each debtor
//! pays what they owe and each creditor receives what they are owed, so
//! nobody pays more than their debt. The strategies differ in how many
//! payments they need and how much work that takes.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`Settlements`], the group's cache of plans,
//! [`settle`], and [`settle_async`] are the group's, and the facade does not
//! re-export them.
#![doc(test(attr(deny(warnings))))]

mod direct;
mod error;
mod greedy;
mod minimum;

use std::future::Future;
use std::sync::OnceLock;

use eranos_core::MemberId;
use eranos_core::cooperative::Cooperative;
use eranos_ledger::{Ledger, MemberPair};
use eranos_money::{Currency, Money};

pub use error::SettlementError;

/// How `Group::settlement` turns balances into payments.
///
/// There is no default: the strategies trade the number of payments against
/// computation time, and the caller chooses. `n` below is the number of
/// members with a non-zero balance.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, CalendarDate, Group, Member, Money, SettlementStrategy, Transfer};
///
/// let mut group = Group::builder(Currency::EUR, "Flat").build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?)?;
/// let bob = group.add_member(Member::builder("Bob").build()?)?;
/// let day = CalendarDate::new(23, 9, 2026)?;
/// // Bob paid Ana 20.00 too much.
/// group.add_transfer(Transfer::builder(Money::parse("20.00", Currency::EUR)?, day, bob, ana).build()?)?;
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
    /// The largest debtor pays the largest creditor, repeatedly.
    ///
    /// O(n log n) time and at most n − 1 payments (55 µs for 1 000 members
    /// on an Apple M4 Pro); usually as few as
    /// [`MinimumTransfers`](Self::MinimumTransfers), but not always.
    Greedy,
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
}

impl SettlementStrategy {
    /// Every strategy, in the order they are declared.
    #[must_use]
    pub const fn all() -> [Self; 3] {
        [Self::Direct, Self::Greedy, Self::MinimumTransfers]
    }
}

/// The payments that settle a group, all in the group currency.
///
/// Recording every payment as a transfer, with `Transfer::from_payment`,
/// brings every balance to zero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    payments: Vec<Payment>,
    strategy: SettlementStrategy,
}

impl Settlement {
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

    /// The strategy that produced the payments.
    #[must_use]
    pub const fn strategy(&self) -> SettlementStrategy {
        self.strategy
    }
}

/// One payment of a [`Settlement`]: `from` pays `amount` to `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Payment {
    amount: Money,
    members: MemberPair,
}

impl Payment {
    /// The amount, in the group currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The member who pays.
    #[must_use]
    pub const fn from(&self) -> MemberId {
        self.members.from
    }

    /// The member who is paid.
    #[must_use]
    pub const fn to(&self) -> MemberId {
        self.members.to
    }
}

/// A payment in minor units, before it becomes a [`Payment`] in the group
/// currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Flow {
    amount: i64,
    members: MemberPair,
}

impl Flow {
    /// The payment this flow makes, in `currency`.
    const fn into_payment(self, currency: Currency) -> Payment {
        Payment {
            amount: Money::from_minor_units(currency, self.amount),
            members: self.members,
        }
    }
}

/// One settlement per strategy, computed on first request and kept until
/// the balances change.
///
/// The cache holds no ledger: its owner asks with the same ledger every
/// time and clears it whenever a balance changes, as the group does.
#[derive(Debug, Clone, Default)]
pub struct Settlements {
    direct: OnceLock<Result<Settlement, SettlementError>>,
    greedy: OnceLock<Result<Settlement, SettlementError>>,
    minimum_transfers: OnceLock<Result<Settlement, SettlementError>>,
}

impl Settlements {
    /// The settlement of `ledger` by `strategy`, computed the first time it
    /// is asked for.
    ///
    /// # Errors
    ///
    /// As for [`settle`], computed once and kept like a plan.
    pub fn get(
        &self,
        strategy: SettlementStrategy,
        ledger: &Ledger,
    ) -> Result<&Settlement, SettlementError> {
        self.slot(strategy)
            .get_or_init(|| settle(strategy, ledger))
            .as_ref()
            .map_err(|&error| error)
    }

    /// Drops every kept settlement.
    pub fn clear(&mut self) {
        self.direct.take();
        self.greedy.take();
        self.minimum_transfers.take();
    }

    const fn slot(
        &self,
        strategy: SettlementStrategy,
    ) -> &OnceLock<Result<Settlement, SettlementError>> {
        match strategy {
            SettlementStrategy::Direct => &self.direct,
            SettlementStrategy::Greedy => &self.greedy,
            SettlementStrategy::MinimumTransfers => &self.minimum_transfers,
        }
    }
}

/// Settles the balances of `ledger` with `strategy`.
///
/// # Errors
///
/// [`SettlementError::OutOfMemory`] when the exact solver's table does not
/// fit in memory, for [`SettlementStrategy::MinimumTransfers`] only.
pub fn settle(
    strategy: SettlementStrategy,
    ledger: &Ledger,
) -> Result<Settlement, SettlementError> {
    settle_async(strategy, ledger).complete()
}

/// Settles the balances of `ledger` with `strategy`, yielding while the
/// exact solver fills its table; the direct and greedy strategies never
/// yield.
///
/// # Errors
///
/// As for [`settle`].
pub fn settle_async(
    strategy: SettlementStrategy,
    ledger: &Ledger,
) -> Cooperative<impl Future<Output = Result<Settlement, SettlementError>> + Send + '_> {
    Cooperative::new(settle_paced(strategy, ledger))
}

/// The computation of [`settle_async`].
async fn settle_paced(
    strategy: SettlementStrategy,
    ledger: &Ledger,
) -> Result<Settlement, SettlementError> {
    let mut flows = match strategy {
        SettlementStrategy::Direct => direct::settle(ledger.pairs()),
        SettlementStrategy::Greedy => greedy::settle(&balances(ledger)),
        SettlementStrategy::MinimumTransfers => minimum::settle(&balances(ledger)).await?,
    };
    // By payer, then recipient, as `Settlement::payments` promises. Every
    // strategy pays each pair of members at most once (the direct one pays
    // each ledger pair once, and the others clear a debtor or a creditor
    // with every payment), so the pair alone orders the flows.
    flows.sort_unstable_by_key(|flow| flow.members);
    let currency = ledger.currency();
    let payments = flows
        .into_iter()
        .map(|flow| flow.into_payment(currency))
        .collect();
    Ok(Settlement { payments, strategy })
}

/// Every non-zero net balance of `ledger`, ordered by member.
fn balances(ledger: &Ledger) -> Vec<(MemberId, i64)> {
    ledger
        .summary()
        .members()
        .map(|member| (member.member(), member.net().minor_units()))
        .filter(|&(_, net)| net != 0)
        .collect()
}

#[cfg(test)]
mod tests;
