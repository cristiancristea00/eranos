//! Tests for pairing exact opposites. Both the exact solver and greedy rely
//! on what is left: an opposite pair left over would cost the solver a
//! doubled table and break greedy's bound, and a payment between members who
//! are not exact opposites would leave one of them with a balance.

use eranos_core::MemberId;
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

/// The open balance `net` of member `number`.
fn balance(number: u8, net: i64) -> OpenBalance {
    OpenBalance {
        member: member(number),
        net,
    }
}

/// Balances for members 1, 2, 3, … in order.
fn balances(nets: &[i64]) -> Vec<OpenBalance> {
    nets.iter()
        .zip(1_u8..)
        .map(|(&net, number)| balance(number, net))
        .collect()
}

/// Exact opposites settle with one payment each, lowest IDs first, and
/// leave nothing when every balance has its opposite.
#[test]
fn exact_opposites_pay_each_other_directly() {
    // Members 3 and 4 owe 5.00 each; 1 and 2 are owed 5.00 each.
    let open = balances(&[500, 500, -500, -500]);
    let Pairing { flows, rest } = pair_opposites(&open);
    assert_eq!(flows, vec![flow(500, 3, 1), flow(500, 4, 2)]);
    assert!(rest.is_empty());
}

/// Balances with nothing to pair and nothing settled are left as they were
/// handed in, without a copy, which most groups' are.
#[test]
fn balances_without_a_pair_are_left_without_a_copy() {
    let open = balances(&[1, 2, -3]);
    let Pairing { flows, rest } = pair_opposites(&open);
    assert_eq!(flows, Vec::<Flow>::new());
    assert!(matches!(rest, Cow::Borrowed(left) if std::ptr::eq(left, open.as_slice())));
}

/// A creditor or a debtor whose amount has no opposite left stays open, a
/// debt larger than every credit finds none, and settled members are left
/// out.
#[test]
fn a_surplus_of_one_amount_stays_unpaired() {
    let open = balances(&[5, 5, 0, -5, 3, -8, -3, -3]);
    let Pairing { flows, rest } = pair_opposites(&open);
    assert_eq!(flows, vec![flow(5, 4, 1), flow(3, 7, 5)]);
    assert_eq!(rest, vec![balance(2, 5), balance(6, -8), balance(8, -3)]);
}

proptest! {
    /// Every payment joins a debtor and a creditor of the same amount, every
    /// other open balance is left, and no two balances left are opposites.
    #[test]
    fn no_exact_opposites_remain_after_pairing(
        draws in proptest::collection::vec((0_i64..6, any::<bool>()), 0..12),
    ) {
        let nets: Vec<i64> = draws
            .into_iter()
            .map(|(amount, positive)| if positive { amount } else { -amount })
            .collect();
        let open = balances(&nets);
        let net_of = |member: MemberId| open.iter().find(|balance| balance.member == member).map_or(0, |balance| balance.net);
        let Pairing { flows, rest } = pair_opposites(&open);
        for flow in &flows {
            prop_assert!(0 < flow.amount);
            prop_assert_eq!(net_of(flow.members.from), -flow.amount);
            prop_assert_eq!(net_of(flow.members.to), flow.amount);
        }
        prop_assert_eq!(
            2 * flows.len() + rest.len(),
            nets.iter().filter(|&&net| 0 != net).count()
        );
        for left in rest.iter() {
            prop_assert!(0 != left.net);
            prop_assert!(rest.iter().all(|other| -left.net != other.net));
        }
    }
}
