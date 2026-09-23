//! Tests for the greedy strategy. Its promise is at most n − 1 payments that
//! clear every balance, with the largest amounts settled first; a payment
//! the wrong way round or a leftover balance would ask someone to pay money
//! they do not owe.

use proptest::prelude::*;

use super::*;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

fn balances(nets: &[i64]) -> Vec<(MemberId, i64)> {
    nets.iter()
        .zip(1_u8..)
        .map(|(&net, number)| (member(number), net))
        .collect()
}

/// The largest debtor pays the largest creditor first, and every payment
/// clears one of the two.
#[test]
fn the_largest_debtor_pays_the_largest_creditor_first() {
    // Owed 50 and 30; owing 60 and 20.
    let flows = settle(&balances(&[50, 30, -60, -20]));
    assert_eq!(
        flows,
        vec![
            (member(3), member(1), 50),
            (member(4), member(2), 20),
            (member(3), member(2), 10)
        ]
    );
}

/// Equal amounts are taken in member order, so the plan is the same on
/// every device.
#[test]
fn ties_go_to_the_lower_member() {
    let flows = settle(&balances(&[10, 10, -10, -10]));
    assert_eq!(
        flows,
        vec![(member(3), member(1), 10), (member(4), member(2), 10)]
    );
}

/// Settled members are ignored and an all-settled group needs nothing.
#[test]
fn settled_balances_need_no_payment() {
    assert!(settle(&[]).is_empty());
    assert!(settle(&balances(&[0, 0])).is_empty());
    assert_eq!(
        settle(&balances(&[0, 5, -5])),
        vec![(member(3), member(2), 5)]
    );
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
        let count = nets.iter().filter(|&&net| net != 0).count();
        prop_assert!(flows.len() <= count.saturating_sub(1));
        for &(member, net) in &open {
            let received: i64 = flows.iter().filter(|flow| flow.1 == member).map(|flow| flow.2).sum();
            let sent: i64 = flows.iter().filter(|flow| flow.0 == member).map(|flow| flow.2).sum();
            prop_assert_eq!(net - received + sent, 0);
            prop_assert!(net >= 0 || received == 0);
            prop_assert!(net <= 0 || sent == 0);
        }
    }
}
