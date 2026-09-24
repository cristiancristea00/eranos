//! Tests for the exact minimum-payments solver. Its answer is shown to users
//! as "the fewest payments", so a plan with one payment too many, a plan
//! that does not clear every balance, or a payment the wrong way round would
//! be visible; an independent brute force checks the count on random small
//! groups.

use proptest::prelude::*;

use super::*;
use crate::settlement::greedy;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

/// A payment of `amount` from member `from` to member `to`.
fn flow(amount: i64, from: u8, to: u8) -> Flow {
    Flow {
        amount,
        members: MemberPair {
            from: member(from),
            to: member(to),
        },
    }
}

/// Balances for members 1, 2, 3, … in order.
fn balances(nets: &[i64]) -> Vec<(MemberId, i64)> {
    nets.iter()
        .zip(1_u8..)
        .map(|(&net, number)| (member(number), net))
        .collect()
}

/// Each member's balance after the flows are paid.
fn after(balances: &[(MemberId, i64)], flows: &[Flow]) -> Vec<i64> {
    balances
        .iter()
        .map(|&(member, net)| {
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
                    .filter(|&index| mask >> index & 1 == 1)
                    .map(|index| others[index])
                    .sum::<i64>();
            if sum == 0 {
                let remaining: Vec<i64> = (0..others.len())
                    .filter(|&index| mask >> index & 1 == 0)
                    .map(|index| others[index])
                    .collect();
                best = best.max(1 + most_groups(&remaining));
            }
        }
        best
    }
    let open: Vec<i64> = nets.iter().copied().filter(|&net| net != 0).collect();
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
    let flows = settle(&open).unwrap();
    assert_eq!(flows.len(), 4);
    assert_eq!(greedy::settle(&open).len(), 5);
    assert!(after(&open, &flows).iter().all(|&net| net == 0));
}

/// Exact opposites settle with one payment each, lowest IDs first, before
/// the dynamic program sees the rest.
#[test]
fn exact_opposites_pay_each_other_directly() {
    // Members 3 and 4 owe 5.00 each; 1 and 2 are owed 5.00 each.
    let open = balances(&[500, 500, -500, -500]);
    let (flows, rest) = pair_opposites(&open);
    assert_eq!(flows, vec![flow(500, 3, 1), flow(500, 4, 2)]);
    assert!(rest.is_empty());
    assert_eq!(settle(&open).unwrap().len(), 2);
}

/// Every zero-sum group settles with one payment fewer than its size, each
/// payment clearing one member.
#[test]
fn a_group_settles_with_one_payment_fewer_than_its_size() {
    let group = balances(&[700, 200, -300, -600]);
    let flows = settle_group(&group);
    assert_eq!(flows.len(), 3);
    assert!(after(&group, &flows).iter().all(|&net| net == 0));
    // Debtors and creditors are taken in member order.
    assert_eq!(flows[0], flow(300, 3, 1));
}

/// Settled or empty balances need no payment and no table.
#[test]
fn nothing_to_settle_needs_no_payment() {
    assert_eq!(settle(&[]), Ok(vec![]));
    assert_eq!(zero_sum_groups(&[]), Ok(vec![]));
}

/// `count` balances with no exact opposites, so all of them reach the
/// dynamic program: creditors owed 1 … count − 1 and one debtor owing their
/// sum.
fn unpaired(count: i64) -> Vec<(MemberId, i64)> {
    let mut nets: Vec<i64> = (1..count).collect();
    nets.push(-nets.iter().sum::<i64>());
    balances(&nets)
}

/// A table that cannot even be addressed is reported as out of memory
/// instead of overflowing the shift or aborting the process.
#[test]
fn a_table_beyond_the_address_space_is_out_of_memory() {
    assert_eq!(
        settle(&unpaired(64)),
        Err(SettlementError::OutOfMemory { balances: 64 })
    );
}

/// A table the allocator refuses is reported as out of memory instead of
/// aborting the process: 2^63 bytes can be addressed, but no allocation may
/// exceed `isize::MAX` bytes, so the refusal needs no memory to test.
#[test]
fn a_table_too_large_to_allocate_is_out_of_memory() {
    assert_eq!(
        settle(&unpaired(63)),
        Err(SettlementError::OutOfMemory { balances: 63 })
    );
}

/// The walk back from the full set yields groups that each sum to zero and
/// together contain every balance once.
#[test]
fn the_groups_partition_the_balances() {
    let open = balances(&[1, 3, 3, 4, -5, -6]);
    let groups = zero_sum_groups(&open).unwrap();
    assert_eq!(groups.len(), 2);
    assert!(
        groups
            .iter()
            .all(|group| group.iter().map(|&(_, net)| net).sum::<i64>() == 0)
    );
    let mut members: Vec<MemberId> = groups.iter().flatten().map(|&(member, _)| member).collect();
    members.sort_unstable();
    assert_eq!(
        members,
        open.iter().map(|&(member, _)| member).collect::<Vec<_>>()
    );
}

proptest! {
    /// The solver's count must equal the true minimum, found by exhaustive
    /// search, or "fewest payments" would be a lie.
    #[test]
    fn the_count_matches_an_exhaustive_search(nets in random_nets(8)) {
        let open = balances(&nets);
        let flows = settle(&open).unwrap();
        prop_assert_eq!(flows.len(), brute_force_minimum(&nets));
    }

    /// Every plan clears every balance with payments from debtors to
    /// creditors, and never uses more payments than greedy.
    #[test]
    fn every_plan_clears_the_balances_without_overpaying(nets in random_nets(12)) {
        let open = balances(&nets);
        let flows = settle(&open).unwrap();
        prop_assert!(after(&open, &flows).iter().all(|&net| net == 0));
        prop_assert!(flows.len() <= greedy::settle(&open).len());
        for &Flow { amount, members: MemberPair { from, to } } in &flows {
            let net = |member: MemberId| open.iter().find(|entry| entry.0 == member).map_or(0, |entry| entry.1);
            prop_assert!(amount > 0);
            prop_assert!(net(from) < 0 && net(to) > 0);
            prop_assert!(amount <= -net(from) && amount <= net(to));
        }
    }
}
