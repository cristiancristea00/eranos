//! Pairwise debts, netted but not simplified.

use std::collections::BTreeMap;

use super::Flow;
use crate::id::MemberId;

/// One payment per pair of members with a netted debt; `pairs` maps (lower,
/// higher) to what lower owes higher.
pub(super) fn settle(pairs: &BTreeMap<(MemberId, MemberId), i64>) -> Vec<Flow> {
    pairs
        .iter()
        .filter_map(|(&(lower, higher), &amount)| match amount.signum() {
            1 => Some((lower, higher, amount)),
            // The ledger keeps every value above i64::MIN, so negating fits.
            -1 => Some((higher, lower, -amount)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
