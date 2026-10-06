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
//! re-export them; nor does it re-export [`FromSettlementError`] and
//! [`FromSettlementStrategy`], through which a crate that tells every variant
//! apart converts a failure or a strategy without the wildcard arm a `match`
//! outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod adaptive;
mod direct;
mod error;
mod greedy;
mod minimum;
mod opposites;

use std::future::Future;
use std::sync::OnceLock;

use eranos_core::MemberId;
use eranos_core::cooperative::Cooperative;
use eranos_ledger::{Ledger, MemberPair};
use eranos_money::{Currency, Money};

pub use error::{FromSettlementError, SettlementError};

/// How `Group::settlement` turns balances into payments.
///
/// There is no default: the strategies trade the number of payments against
/// computation time, and the caller chooses; an app that wants the fewest
/// payments without risking a long computation chooses
/// [`Adaptive`](Self::Adaptive). `n` below is the number of members with a
/// non-zero balance.
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
    /// The fewest payments while they are quick to find, Greedy's beyond.
    ///
    /// Exact opposite balances are paired first. When at most 20 balances are
    /// left, the exact solver settles them and the plan is the one
    /// [`MinimumTransfers`](Self::MinimumTransfers) gives; when more are left,
    /// or when the solver's table of 2ⁿ bytes cannot be allocated, the plan is
    /// the one [`Greedy`](Self::Greedy) gives. [`Settlement::strategy`] names
    /// the strategy whose plan it is. It never fails, and it yields only while
    /// the exact solver fills its table.
    ///
    /// Within the limit it takes the time of
    /// [`MinimumTransfers`](Self::MinimumTransfers) for at most 20 balances
    /// and a table of at most 1 MiB, and beyond it the time of
    /// [`Greedy`](Self::Greedy).
    ///
    /// The limit is part of the numbers the library promises: another limit
    /// would give some groups other payments, so it changes only in a
    /// breaking release.
    Adaptive,
    /// No simplification: everyone pays back the people who paid for them.
    ///
    /// Every participant owes every payer of each expense their share of what
    /// that payer paid; the debts between each pair of members are netted
    /// across all entries. Each payment traces back to shared expenses, at
    /// the cost of the most payments. O(p) time for p pairs with a debt,
    /// which the group keeps current as entries change.
    Direct,
    /// The largest debtor pays the largest creditor, repeatedly, after exact
    /// opposites pay each other where that saves a payment.
    ///
    /// Two plans are computed: the largest debtor paying the largest creditor
    /// throughout, and the same after every debtor who owes exactly what a
    /// creditor is owed has paid them; the one with fewer payments is kept,
    /// the first on a tie. O(n log n) time and at most n − 1 payments,
    /// within 3/2 of [`MinimumTransfers`](Self::MinimumTransfers) and
    /// usually as few, but not always.
    Greedy,
    /// The fewest payments possible.
    ///
    /// Finding them is NP-hard, and this solves it exactly: exact opposite
    /// balances are paired first (always optimal), then a dynamic program
    /// over all subsets of the remaining `n` balances finds the largest
    /// number of groups that each sum to zero; every such group settles with
    /// one payment fewer than its size. It needs O(2ⁿ·n) time and 2ⁿ bytes
    /// of memory, with no size limit, so every extra open balance doubles
    /// both: 20 balances take 1 MiB and 30 take 1 GiB. The operating system
    /// may end the process before an allocation fails, so
    /// prefer [`Adaptive`](Self::Adaptive), which settles large groups
    /// greedily.
    MinimumTransfers,
}

impl SettlementStrategy {
    /// Every strategy, in the order they are declared.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::Adaptive,
            Self::Direct,
            Self::Greedy,
            Self::MinimumTransfers,
        ]
    }
}

/// Builds a value from a [`SettlementStrategy`], one method per strategy.
pub trait FromSettlementStrategy: Sized {
    /// The value for the adaptive strategy.
    fn adaptive() -> Self;

    /// The value for the direct strategy.
    fn direct() -> Self;

    /// The value for the greedy strategy.
    fn greedy() -> Self;

    /// The value for the strategy of the fewest payments.
    fn minimum_transfers() -> Self;

    /// The value for `strategy`, from the method of its variant.
    #[must_use]
    fn from_settlement_strategy(strategy: SettlementStrategy) -> Self {
        match strategy {
            SettlementStrategy::Adaptive => Self::adaptive(),
            SettlementStrategy::Direct => Self::direct(),
            SettlementStrategy::Greedy => Self::greedy(),
            SettlementStrategy::MinimumTransfers => Self::minimum_transfers(),
        }
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

    /// The strategy that produced the payments: the one asked for, or, when
    /// [`SettlementStrategy::Adaptive`] was, the one whose plan it chose,
    /// [`SettlementStrategy::MinimumTransfers`] or
    /// [`SettlementStrategy::Greedy`].
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

/// The payments of one strategy, in minor units, and the strategy whose
/// plan they are.
#[derive(Debug)]
struct Plan {
    flows: Vec<Flow>,
    strategy: SettlementStrategy,
}

/// A member's net balance in minor units while it is open: positive when the
/// member is owed, negative when they owe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenBalance {
    member: MemberId,
    net: i64,
}

/// One settlement per strategy, computed on first request and kept until
/// the balances change.
///
/// The cache holds no ledger: its owner asks with the same ledger every
/// time and clears it whenever a balance changes, as the group does. A plan
/// is kept once it is computed, never while it is: no request waits for
/// another's computation, and one dropped before it completes keeps
/// nothing.
#[derive(Debug, Clone, Default)]
pub struct Settlements {
    adaptive: OnceLock<Result<Settlement, SettlementError>>,
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
        self.get_async(strategy, ledger).complete()
    }

    /// The settlement of `ledger` by `strategy`, computed with
    /// [`settle_async`] the first time it is asked for.
    ///
    /// # Errors
    ///
    /// As for [`settle`], computed once and kept like a plan.
    pub fn get_async(
        &self,
        strategy: SettlementStrategy,
        ledger: &Ledger,
    ) -> Cooperative<impl Future<Output = Result<&Settlement, SettlementError>> + Send> {
        let slot = self.slot(strategy);
        Cooperative::new(async move {
            let kept = if let Some(kept) = slot.get() {
                kept
            } else {
                let computed = settle_async(strategy, ledger).await;
                // Of two requests computing at once, the first to complete
                // keeps its plan, which equals the other's.
                slot.get_or_init(|| computed)
            };
            kept.as_ref().map_err(|&error| error)
        })
    }

    /// Drops every kept settlement.
    pub fn clear(&mut self) {
        self.adaptive.take();
        self.direct.take();
        self.greedy.take();
        self.minimum_transfers.take();
    }

    const fn slot(
        &self,
        strategy: SettlementStrategy,
    ) -> &OnceLock<Result<Settlement, SettlementError>> {
        match strategy {
            SettlementStrategy::Adaptive => &self.adaptive,
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
/// fit in memory, for [`SettlementStrategy::MinimumTransfers`] only:
/// [`SettlementStrategy::Adaptive`] settles greedily instead.
pub fn settle(
    strategy: SettlementStrategy,
    ledger: &Ledger,
) -> Result<Settlement, SettlementError> {
    settle_async(strategy, ledger).complete()
}

/// Settles the balances of `ledger` with `strategy`, yielding as it works.
///
/// The exact solver yields while it fills its table, for the fewest payments
/// and for the adaptive strategy's exact plans; the direct and greedy
/// strategies never yield.
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
    let Plan {
        mut flows,
        strategy: computed_by,
    } = match strategy {
        SettlementStrategy::Adaptive => adaptive::settle(&balances(ledger)).await,
        SettlementStrategy::Direct => Plan {
            flows: direct::settle(ledger.pairs()),
            strategy,
        },
        SettlementStrategy::Greedy => Plan {
            flows: greedy::settle(&balances(ledger)),
            strategy,
        },
        SettlementStrategy::MinimumTransfers => Plan {
            flows: minimum::settle(&balances(ledger)).await?,
            strategy,
        },
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
    Ok(Settlement {
        payments,
        strategy: computed_by,
    })
}

/// Every non-zero net balance of `ledger`, ordered by member.
fn balances(ledger: &Ledger) -> Vec<OpenBalance> {
    ledger
        .summary()
        .members()
        .map(|member| OpenBalance {
            member: member.member(),
            net: member.net().minor_units(),
        })
        .filter(|balance| 0 != balance.net)
        .collect()
}

#[cfg(test)]
mod tests;
