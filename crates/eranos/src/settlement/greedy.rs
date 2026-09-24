//! Largest debtor pays largest creditor.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::Flow;
use crate::id::MemberId;
use crate::ledger::MemberPair;

/// Settles `balances` (member, net; positive = is owed) by repeatedly having
/// the largest debtor pay the largest creditor the smaller of the two
/// amounts; ties go to the lower member ID.
///
/// Every payment clears at least one of the two, so at most n − 1 payments
/// settle n balances.
pub(super) fn settle(balances: &[(MemberId, i64)]) -> Vec<Flow> {
    // Max-heaps on (amount, lower ID first).
    let mut creditors: BinaryHeap<(u64, Reverse<MemberId>)> = BinaryHeap::new();
    let mut debtors: BinaryHeap<(u64, Reverse<MemberId>)> = BinaryHeap::new();
    for &(member, net) in balances {
        match net.signum() {
            1 => creditors.push((net.unsigned_abs(), Reverse(member))),
            -1 => debtors.push((net.unsigned_abs(), Reverse(member))),
            _ => {}
        }
    }
    let mut flows = Vec::with_capacity(balances.len().saturating_sub(1));
    while let (Some((credit, Reverse(creditor))), Some((debt, Reverse(debtor)))) =
        (creditors.pop(), debtors.pop())
    {
        let amount = credit.min(debt);
        flows.push(Flow {
            // Balances are within ±i64::MAX, so every amount fits back in i64.
            amount: i64::try_from(amount).unwrap_or(i64::MAX),
            members: MemberPair {
                from: debtor,
                to: creditor,
            },
        });
        if credit > amount {
            creditors.push((credit - amount, Reverse(creditor)));
        }
        if debt > amount {
            debtors.push((debt - amount, Reverse(debtor)));
        }
    }
    flows
}

#[cfg(test)]
mod tests;
