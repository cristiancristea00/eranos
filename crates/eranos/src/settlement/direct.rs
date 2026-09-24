//! Pairwise debts, netted but not simplified.

use std::collections::BTreeMap;

use eranos_ledger::MemberPair;

use super::Flow;

/// One payment per pair of members with a netted debt; `pairs` maps every
/// pair, keyed with the lower member as `from`, to what `from` owes `to`.
pub(super) fn settle(pairs: &BTreeMap<MemberPair, i64>) -> Vec<Flow> {
    pairs
        .iter()
        .filter_map(|(&members, &amount)| match amount.signum() {
            1 => Some(Flow { amount, members }),
            // The ledger keeps every value above i64::MIN, so negating fits.
            -1 => Some(Flow {
                amount: -amount,
                members: members.reversed(),
            }),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
