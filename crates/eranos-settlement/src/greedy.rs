//! Largest debtor pays largest creditor, after exact opposites where they
//! save a payment.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use eranos_core::MemberId;
use eranos_ledger::MemberPair;

use super::{Flow, OpenBalance};
use crate::opposites::{Pairing, pair_opposites};

/// What a debtor still owes or a creditor is still owed.
///
/// Ordered by amount, then by member in reverse, so that a max-heap yields
/// the largest amount first and, among equal amounts, the lower member ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Outstanding {
    amount: u64,
    member: MemberId,
}

impl Ord for Outstanding {
    fn cmp(&self, other: &Self) -> Ordering {
        self.amount
            .cmp(&other.amount)
            .then_with(|| other.member.cmp(&self.member))
    }
}

impl PartialOrd for Outstanding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Settles `balances`, ordered by member, with the fewer payments of two
/// plans: the largest debtor paying the largest creditor throughout, and the
/// same after exact opposites have paid each other; the first on a tie.
///
/// Every payment clears at least one member, so at most n − 1 payments
/// settle n balances. The plan kept is within 3/2 of the fewest payments:
/// the pairs belong to a minimal plan, and the m balances they leave split
/// into zero-sum groups of at least three members, so they need at least
/// 2m/3 payments where the second plan makes at most m − 1.
pub(super) fn settle(balances: &[OpenBalance]) -> Vec<Flow> {
    settle_paired(balances, pair_opposites(balances))
}

/// The plan of [`settle`], given the pairing of the exact opposites among
/// `balances`.
pub(super) fn settle_paired(balances: &[OpenBalance], pairing: Pairing) -> Vec<Flow> {
    let largest = largest_first(balances);
    if pairing.flows.is_empty() {
        // Nothing was paired, so the second plan would be the first.
        return largest;
    }
    let Pairing { mut flows, rest } = pairing;
    flows.extend(largest_first(&rest));
    if flows.len() < largest.len() {
        flows
    } else {
        largest
    }
}

/// Settles `balances` by repeatedly having the largest debtor pay the
/// largest creditor the smaller of the two amounts; ties go to the lower
/// member ID.
fn largest_first(balances: &[OpenBalance]) -> Vec<Flow> {
    let mut creditors: BinaryHeap<Outstanding> = BinaryHeap::new();
    let mut debtors: BinaryHeap<Outstanding> = BinaryHeap::new();
    for &OpenBalance { member, net } in balances {
        let outstanding = Outstanding {
            amount: net.unsigned_abs(),
            member,
        };
        match net.signum() {
            1 => creditors.push(outstanding),
            -1 => debtors.push(outstanding),
            _ => {}
        }
    }
    let mut flows = Vec::with_capacity(balances.len().saturating_sub(1));
    while let (Some(creditor), Some(debtor)) = (creditors.pop(), debtors.pop()) {
        let amount = creditor.amount.min(debtor.amount);
        flows.push(Flow {
            // Balances are within ±i64::MAX, so every amount fits back in i64.
            amount: i64::try_from(amount).unwrap_or(i64::MAX),
            members: MemberPair {
                from: debtor.member,
                to: creditor.member,
            },
        });
        if creditor.amount > amount {
            creditors.push(Outstanding {
                amount: creditor.amount - amount,
                member: creditor.member,
            });
        }
        if debtor.amount > amount {
            debtors.push(Outstanding {
                amount: debtor.amount - amount,
                member: debtor.member,
            });
        }
    }
    flows
}

#[cfg(test)]
mod tests;
