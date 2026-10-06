//! Tests for the greedy strategy. Its promise is at most n − 1 payments that
//! clear every balance, with the largest amounts settled first, never more
//! payments than the largest-first plan alone, and within 3/2 of the fewest;
//! a payment the wrong way round or a leftover balance would ask someone to
//! pay money they do not owe.

use eranos_core::cooperative::Cooperative;
use proptest::prelude::*;

use super::*;
use crate::minimum;

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

fn balances(nets: &[i64]) -> Vec<OpenBalance> {
    nets.iter()
        .zip(1_u8..)
        .map(|(&net, number)| OpenBalance {
            member: member(number),
            net,
        })
        .collect()
}

/// The largest debtor pays the largest creditor first, and every payment
/// clears one of the two.
#[test]
fn the_largest_debtor_pays_the_largest_creditor_first() {
    // Owed 50 and 30; owing 60 and 20.
    let flows = settle(&balances(&[50, 30, -60, -20]));
    assert_eq!(flows, vec![flow(50, 3, 1), flow(20, 4, 2), flow(10, 3, 2)]);
}

/// Equal amounts are taken in member order, so the plan is the same on
/// every device.
#[test]
fn ties_go_to_the_lower_member() {
    let flows = settle(&balances(&[10, 10, -10, -10]));
    assert_eq!(flows, vec![flow(10, 3, 1), flow(10, 4, 2)]);
}

/// Settled members are ignored and an all-settled group needs nothing.
#[test]
fn settled_balances_need_no_payment() {
    assert_eq!(settle(&[]), Vec::<Flow>::new());
    assert_eq!(settle(&balances(&[0, 0])), Vec::<Flow>::new());
    assert_eq!(settle(&balances(&[0, 5, -5])), vec![flow(5, 3, 2)]);
}

/// Where the largest amounts meet first, the 5.00 owed and the 5.00 owing
/// end up in separate payments; paired first, they need one payment between
/// them, and the plan one payment fewer.
#[test]
fn settling_exact_opposites_first_saves_a_payment() {
    let open = balances(&[-3, -3, -5, 6, 5]);
    assert_eq!(largest_first(&open).len(), 4);
    assert_eq!(
        settle(&open),
        vec![flow(5, 3, 5), flow(3, 1, 4), flow(3, 2, 4)]
    );
}

/// Pairing first is not always better: here it leaves balances that take
/// one payment more, so the largest-first plan, already the fewest, stays.
#[test]
fn the_largest_first_plan_stays_when_pairing_first_costs_a_payment() {
    let open = balances(&[-14, -4, -16, -9, 20, 16, 18, -7, -4]);
    let pairing = pair_opposites(&open);
    assert_eq!(pairing.flows.len() + largest_first(&pairing.rest).len(), 7);
    assert_eq!(largest_first(&open).len(), 6);
    assert_eq!(settle(&open), largest_first(&open));
}

/// Draws up to `count` random non-zero balances of small amounts, so that
/// exact opposites and zero-sum groups are common, plus one that makes the
/// sum zero.
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
    /// A group would get more payments than the largest-first plan alone
    /// makes, or a plan that changed without saving a payment.
    #[test]
    fn pairing_first_wins_only_by_a_payment_fewer(nets in small_nets(12)) {
        let open = balances(&nets);
        let plan = settle(&open);
        let largest = largest_first(&open);
        prop_assert!(plan.len() <= largest.len());
        prop_assert!(plan.len() < largest.len() || plan == largest);
    }

    /// The documented bound: never more than half again the fewest payments.
    #[test]
    fn greedy_stays_within_three_halves_of_the_minimum(nets in small_nets(10)) {
        let open = balances(&nets);
        let fewest = Cooperative::new(minimum::settle(&open)).complete().unwrap();
        prop_assert!(2 * settle(&open).len() <= 3 * fewest.len());
    }

    /// Greedy must clear every balance with at most n − 1 payments, each
    /// from a debtor to a creditor.
    #[test]
    fn at_most_n_minus_one_payments_clear_every_balance(
        draws in proptest::collection::vec((1_i64..1_000_000, any::<bool>()), 1..30),
    ) {
        let mut nets: Vec<i64> = draws.into_iter().map(|(amount, positive)| if positive { amount } else { -amount }).collect();
        let sum: i64 = nets.iter().sum();
        nets.push(-sum);
        let open = balances(&nets);
        let flows = settle(&open);
        let count = nets.iter().filter(|&&net| 0 != net).count();
        prop_assert!(flows.len() <= count.saturating_sub(1));
        for &OpenBalance { member, net } in &open {
            let received: i64 = flows.iter().filter(|flow| flow.members.to == member).map(|flow| flow.amount).sum();
            let sent: i64 = flows.iter().filter(|flow| flow.members.from == member).map(|flow| flow.amount).sum();
            prop_assert_eq!(net - received + sent, 0);
            prop_assert!(0 <= net || 0 == received);
            prop_assert!(0 >= net || 0 == sent);
        }
    }
}
