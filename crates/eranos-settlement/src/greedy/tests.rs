//! Tests for the greedy strategy. Its promise is at most n − 1 payments that
//! clear every balance, with the largest amounts settled first; a payment
//! the wrong way round or a leftover balance would ask someone to pay money
//! they do not owe.

use proptest::prelude::*;

use super::*;

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
    assert!(settle(&[]).is_empty());
    assert!(settle(&balances(&[0, 0])).is_empty());
    assert_eq!(settle(&balances(&[0, 5, -5])), vec![flow(5, 3, 2)]);
}

proptest! {
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
