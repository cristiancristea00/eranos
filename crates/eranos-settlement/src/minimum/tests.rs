//! Tests for the exact minimum-payments solver. Its answer is shown to users
//! as "the fewest payments", so a plan with one payment too many, a plan
//! that does not clear every balance, or a payment the wrong way round would
//! be visible; an independent brute force checks the count on random small
//! groups.

use std::future::Future;
use std::pin::pin;
use std::task::{Context, Waker};

use eranos_core::MemberId;
use eranos_core::cooperative::Cooperative;
use proptest::prelude::*;

use super::*;
use crate::greedy;

/// The member `number`; members sort by number.
fn member(number: u8) -> MemberId {
    MemberId::new(&format!("member-{number:03}")).unwrap()
}

/// Runs one of the solver's computations to its end.
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

/// A payment of `amount` from member `from` to member `to`.
fn flow(from: u8, to: u8, amount: i64) -> Flow {
    Flow {
        members: MemberPair {
            from: member(from),
            to: member(to),
        },
        amount,
    }
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

/// Each member's balance after the flows are paid.
fn after(balances: &[OpenBalance], flows: &[Flow]) -> Vec<i64> {
    balances
        .iter()
        .map(|&OpenBalance { member, net }| {
            let received: i64 = flows
                .iter()
                .filter(|flow| flow.members.to == member)
                .map(|flow| flow.amount)
                .sum();
            let sent: i64 = flows
                .iter()
                .filter(|flow| flow.members.from == member)
                .map(|flow| flow.amount)
                .sum();
            net - received + sent
        })
        .collect()
}

/// The fewest payments by exhaustive search over partitions into zero-sum
/// groups, independent of the dynamic program.
fn brute_force_minimum(nets: &[i64]) -> usize {
    fn most_groups(rest: &[i64]) -> usize {
        let Some((&first, others)) = rest.split_first() else {
            return 0;
        };
        let mut best = 0;
        // Every group containing `first` that sums to zero.
        for mask in 0_u32..(1 << others.len()) {
            let sum: i64 = first
                + (0..others.len())
                    .filter(|&index| 1 == mask >> index & 1)
                    .map(|index| others[index])
                    .sum::<i64>();
            if 0 == sum {
                let remaining: Vec<i64> = (0..others.len())
                    .filter(|&index| 0 == mask >> index & 1)
                    .map(|index| others[index])
                    .collect();
                best = best.max(1 + most_groups(&remaining));
            }
        }
        best
    }
    let open: Vec<i64> = nets.iter().copied().filter(|&net| 0 != net).collect();
    open.len() - most_groups(&open)
}

/// Draws up to `count` random non-zero balances plus one that makes the sum
/// zero.
fn random_nets(count: usize) -> impl Strategy<Value = Vec<i64>> {
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

/// Where the greedy strategy needs five payments, the two zero-sum groups
/// {1, 4, −5} and {3, 3, −6} settle with four.
#[test]
fn it_finds_zero_sum_groups_greedy_misses() {
    let open = balances(&[1, 3, 3, 4, -5, -6]);
    let flows = completed(settle(&open)).unwrap();
    assert_eq!(flows.len(), 4);
    assert_eq!(greedy::settle(&open).len(), 5);
    assert!(after(&open, &flows).iter().all(|&net| 0 == net));
}

/// Exact opposites settle with one payment each, lowest IDs first, before
/// the dynamic program sees the rest.
#[test]
fn exact_opposites_pay_each_other_directly() {
    // Members 3 and 4 owe 5.00 each; 1 and 2 are owed 5.00 each.
    let open = balances(&[500, 500, -500, -500]);
    assert_eq!(
        completed(settle(&open)),
        Ok(vec![flow(3, 1, 500), flow(4, 2, 500)])
    );
}

/// Every zero-sum group settles with one payment fewer than its size, each
/// payment clearing one member.
#[test]
fn a_group_settles_with_one_payment_fewer_than_its_size() {
    let group = balances(&[700, 200, -300, -600]);
    let flows = settle_group(&group);
    assert_eq!(flows.len(), 3);
    assert!(after(&group, &flows).iter().all(|&net| 0 == net));
    // Debtors and creditors are taken in member order.
    assert_eq!(flows[0], flow(3, 1, 300));
}

/// Settled or empty balances need no payment and no table.
#[test]
fn nothing_to_settle_needs_no_payment() {
    assert_eq!(completed(settle(&[])), Ok(vec![]));
    assert_eq!(completed(zero_sum_groups(&[])), Ok(vec![]));
}

/// `count` balances with no exact opposites, so all of them reach the
/// dynamic program: creditors owed 1 … count − 1 and one debtor owing their
/// sum.
fn unpaired(count: i64) -> Vec<OpenBalance> {
    let mut nets: Vec<i64> = (1..count).collect();
    nets.push(-nets.iter().sum::<i64>());
    balances(&nets)
}

/// A long exact plan would hold its thread without a pause, or pause where
/// it needs none: the table of 16 balances fills in one poll, and that of
/// 17, which is twice as large, yields once.
#[test]
fn the_table_fill_yields_once_per_budget_of_work() {
    assert_eq!(pending_polls(settle(&unpaired(16))), 0);
    assert_eq!(pending_polls(settle(&unpaired(17))), 1);
    // Yielding changes no payment.
    assert_eq!(
        completed(settle(&unpaired(17))).map(|flows| flows.len()),
        Ok(16)
    );
}

/// A table that cannot even be addressed is reported as out of memory
/// instead of overflowing the shift or aborting the process.
#[test]
fn a_table_beyond_the_address_space_is_out_of_memory() {
    assert_eq!(
        completed(settle(&unpaired(64))),
        Err(SettlementError::OutOfMemory { balances: 64 })
    );
}

/// A table the allocator refuses is reported as out of memory instead of
/// aborting the process: 2^63 bytes can be addressed, but no allocation may
/// exceed `isize::MAX` bytes, so the refusal needs no memory to test.
#[test]
fn a_table_too_large_to_allocate_is_out_of_memory() {
    assert_eq!(
        completed(settle(&unpaired(63))),
        Err(SettlementError::OutOfMemory { balances: 63 })
    );
}

/// The walk back from the full set yields groups that each sum to zero and
/// together contain every balance once.
#[test]
fn the_groups_partition_the_balances() {
    let open = balances(&[1, 3, 3, 4, -5, -6]);
    let groups = completed(zero_sum_groups(&open)).unwrap();
    assert_eq!(groups.len(), 2);
    assert!(
        groups
            .iter()
            .all(|group| 0 == group.iter().map(|balance| balance.net).sum::<i64>())
    );
    let mut members: Vec<MemberId> = groups
        .iter()
        .flatten()
        .map(|balance| balance.member)
        .collect();
    members.sort_unstable();
    assert_eq!(
        members,
        open.iter()
            .map(|balance| balance.member)
            .collect::<Vec<_>>()
    );
}

proptest! {
    /// The solver's count must equal the true minimum, found by exhaustive
    /// search, or "fewest payments" would be a lie.
    #[test]
    fn the_count_matches_an_exhaustive_search(nets in random_nets(8)) {
        let open = balances(&nets);
        let flows = completed(settle(&open)).unwrap();
        prop_assert_eq!(flows.len(), brute_force_minimum(&nets));
    }

    /// Every plan clears every balance with payments from debtors to
    /// creditors, and never uses more payments than greedy.
    #[test]
    fn every_plan_clears_the_balances_without_overpaying(nets in random_nets(12)) {
        let open = balances(&nets);
        let flows = completed(settle(&open)).unwrap();
        prop_assert!(after(&open, &flows).iter().all(|&net| 0 == net));
        prop_assert!(flows.len() <= greedy::settle(&open).len());
        for &Flow { members: MemberPair { from, to }, amount } in &flows {
            let net = |member: MemberId| open.iter().find(|balance| balance.member == member).map_or(0, |balance| balance.net);
            prop_assert!(0 < amount);
            prop_assert!(0 > net(from) && 0 < net(to));
            prop_assert!(amount <= -net(from) && amount <= net(to));
        }
    }
}
