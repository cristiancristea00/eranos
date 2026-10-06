//! Exact opposites: a debtor who owes exactly what a creditor is owed.

use eranos_ledger::MemberPair;

use super::{Flow, OpenBalance};

/// The payments between exact opposites and the balances they leave.
#[derive(Debug)]
pub(super) struct Pairing {
    /// One payment for each pair of exact opposites.
    pub(super) flows: Vec<Flow>,
    /// The non-zero balances left, in their order among those paired.
    pub(super) rest: Vec<OpenBalance>,
}

/// Settles every debtor whose debt exactly matches a creditor's credit with
/// one payment, lowest member IDs first.
///
/// Taking such a pair as its own group is always optimal: if an optimal
/// partition put the two members in groups G and H, then {x, −x} and
/// (G ∪ H) ∖ {x, −x} is a partition with as many zero-sum groups. No two
/// balances left are exact opposites, so every zero-sum group among them has
/// at least three members.
pub(super) fn pair_opposites(balances: &[OpenBalance]) -> Pairing {
    // Creditors by amount, then by member, so the first of each amount has
    // the lowest ID.
    let mut creditors: Vec<OpenBalance> = balances
        .iter()
        .copied()
        .filter(|balance| 0 < balance.net)
        .collect();
    creditors.sort_unstable_by(|left, right| {
        left.net
            .cmp(&right.net)
            .then_with(|| left.member.cmp(&right.member))
    });
    // At the first creditor of each amount, the next one of that amount
    // still unpaid.
    let mut next: Vec<usize> = (0..creditors.len()).collect();
    let mut paired = Vec::new();
    let mut flows = Vec::new();
    for debtor in balances.iter().filter(|balance| 0 > balance.net) {
        // The ledger keeps every value above i64::MIN, so negating fits.
        let debt = -debtor.net;
        let first = creditors.partition_point(|creditor| creditor.net < debt);
        let Some(candidate) = next.get_mut(first) else {
            continue;
        };
        if let Some(creditor) = creditors.get(*candidate)
            && debt == creditor.net
        {
            *candidate += 1;
            flows.push(Flow {
                amount: debt,
                members: MemberPair {
                    from: debtor.member,
                    to: creditor.member,
                },
            });
            paired.push(debtor.member);
            paired.push(creditor.member);
        }
    }
    paired.sort_unstable();
    let rest = balances
        .iter()
        .copied()
        .filter(|balance| 0 != balance.net && paired.binary_search(&balance.member).is_err())
        .collect();
    Pairing { flows, rest }
}

#[cfg(test)]
mod tests;
