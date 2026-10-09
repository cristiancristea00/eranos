//! Tests for the adaptive strategy. Its promise is the exact plan while at
//! most twenty balances are left after pairing, greedy's beyond, the plan
//! named by the strategy that made it, and never a failure: an exact plan
//! for too large a group would freeze a phone, a greedy plan for a small
//! one would ask for payments the exact plan saves, and a misnamed plan
//! would tell the app it is the fewest when it is not.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Waker};

use eranos_core::cooperative::Cooperative;
use proptest::prelude::*;

use super::*;
use crate::Flow;

/// The member `number`, by position; members sort by number.
fn member(number: u8) -> usize {
    usize::from(number)
}

/// Balances for members 1, 2, 3, … in order.
fn balances(nets: &[i64]) -> Vec<OpenBalance> {
    nets.iter()
        .zip(1_u8..)
        .map(|(&net, number)| OpenBalance {
            member: member(number),
            net,
        })
        .collect()
}

/// `count` balances with no exact opposites, so all of them are left after
/// pairing: creditors owed 1 … count − 1 and one debtor owing their sum.
fn unpaired(count: i64) -> Vec<i64> {
    let mut nets: Vec<i64> = (1..count).collect();
    nets.push(-nets.iter().sum::<i64>());
    nets
}

/// Runs one of the strategy's computations to its end.
fn completed<F: Future>(future: F) -> F::Output {
    Cooperative::new(future).complete()
}

/// How many polls of `future` pend before it is ready.
fn pending_polls<F: Future>(future: F) -> usize {
    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let mut pending = 0;
    while future.as_mut().poll(&mut context).is_pending() {
        pending += 1;
    }
    pending
}

/// The exact solver's plan of `open`.
fn exact(open: &[OpenBalance]) -> Vec<Flow> {
    completed(minimum::settle(open)).unwrap()
}

/// Twenty balances left after pairing take the exact path, though the group
/// has twenty-two: the limit counts what the solver's table holds.
#[test]
fn twenty_open_balances_after_pairing_take_the_exact_path() {
    let mut nets = unpaired(20);
    nets.extend([300, -300]);
    let open = balances(&nets);
    let plan = completed(settle(&open));
    assert_eq!(plan.strategy, SettlementStrategy::MinimumTransfers);
    assert_eq!(plan.flows, exact(&open));
}

/// Twenty-one balances left after pairing settle greedily, with exactly
/// greedy's payments.
#[test]
fn twenty_one_open_balances_fall_back_to_greedy() {
    let open = balances(&unpaired(21));
    let plan = completed(settle(&open));
    assert_eq!(plan.strategy, SettlementStrategy::Greedy);
    assert_eq!(plan.flows, greedy::settle(&open));
}

/// A table the address space or the allocator refuses settles greedily
/// instead of failing: 2⁶⁴ bytes cannot be addressed, and 2⁶³ exceed what
/// one allocation may hold.
#[test]
fn a_table_that_cannot_be_allocated_falls_back_to_greedy() {
    for count in [63, 64] {
        let open = balances(&unpaired(count));
        let plan = completed(settle_within(&open, 64));
        assert_eq!(plan.strategy, SettlementStrategy::Greedy);
        assert_eq!(plan.flows, greedy::settle(&open));
    }
}

/// The exact path pauses where the exact solver does, its table of 16
/// balances in one poll and that of 17 after one yield, and the greedy path
/// never pauses.
#[test]
fn the_exact_path_yields_as_the_solver_does_and_the_fallback_never_yields() {
    assert_eq!(pending_polls(settle(&balances(&unpaired(16)))), 0);
    assert_eq!(pending_polls(settle(&balances(&unpaired(17)))), 1);
    assert_eq!(pending_polls(settle(&balances(&unpaired(21)))), 0);
}

/// Draws up to `count` random non-zero balances of small amounts, so that
/// exact opposites are common, plus one that makes the sum zero.
fn small_nets(count: usize) -> impl Strategy<Value = Vec<i64>> {
    proptest::collection::vec((1_i64..20, any::<bool>()), 1..count).prop_map(|draws| {
        let mut nets: Vec<i64> = draws
            .into_iter()
            .map(|(amount, positive)| if positive { amount } else { -amount })
            .collect();
        let sum: i64 = nets.iter().sum();
        nets.push(-sum);
        nets
    })
}

proptest! {
    /// Within the limit the plan is the exact solver's and says so; beyond
    /// it, greedy's, and says so, whatever the limit.
    #[test]
    fn the_plan_is_the_exact_one_within_the_limit_and_greedys_beyond(
        nets in small_nets(12),
        limit in 0_usize..=12,
    ) {
        let open = balances(&nets);
        let plan = completed(settle_within(&open, limit));
        if pair_opposites(&open).rest.len() <= limit {
            prop_assert_eq!(plan.strategy, SettlementStrategy::MinimumTransfers);
            prop_assert_eq!(plan.flows, exact(&open));
        } else {
            prop_assert_eq!(plan.strategy, SettlementStrategy::Greedy);
            prop_assert_eq!(plan.flows, greedy::settle(&open));
        }
    }
}
