//! The fewest payments while they are quick to find, Greedy's beyond.

use super::{OpenBalance, Plan, SettlementStrategy};
use crate::opposites::pair_opposites;
use crate::{greedy, minimum};

/// The most balances left after pairing exact opposites that the exact
/// solver settles: their table is 1 MiB, and both its size and the time to
/// fill it double with every balance more (docs/ARCHITECTURE.md § 7).
/// Another limit would settle some groups with other payments, so it is part
/// of what the version promises (docs/VERSIONING.md).
const EXACT_LIMIT: usize = 20;

/// Settles `balances`, ordered by member, with the exact solver when at most
/// [`EXACT_LIMIT`] balances are left after pairing exact opposites, and
/// greedily otherwise.
pub(super) async fn settle(balances: &[OpenBalance]) -> Plan {
    settle_within(balances, EXACT_LIMIT).await
}

/// Settles `balances` with the exact solver when at most `limit` balances
/// are left after pairing exact opposites and its table can be allocated,
/// and greedily otherwise; the plan names the strategy whose plan it is.
///
/// The pairs are found once: the exact plan is the pairs and the exact plan
/// of the balances left, as the exact solver's own, and the greedy plan is
/// computed from the same pairing, as greedy's own.
async fn settle_within(balances: &[OpenBalance], limit: usize) -> Plan {
    let pairing = pair_opposites(balances);
    if pairing.rest.len() <= limit
        && let Ok(exact) = minimum::settle_rest(&pairing.rest).await
    {
        let mut flows = pairing.flows;
        flows.extend(exact);
        return Plan {
            flows,
            strategy: SettlementStrategy::MinimumTransfers,
        };
    }
    Plan {
        flows: greedy::settle_paired(balances, pairing),
        strategy: SettlementStrategy::Greedy,
    }
}

#[cfg(test)]
mod tests;
