//! The fewest payments: an exact solver for an NP-hard problem.
//!
//! Settling n balances takes n − g payments, where g is the largest number of
//! groups the balances can be split into so that each group sums to zero
//! (Verhoeff, "Settling multiple debts efficiently", 2004). The solver pairs
//! exact opposites, then finds g for the rest with a dynamic program over
//! subsets, and settles each group with one payment fewer than its size.

use std::ops::Range;

use eranos_core::cooperative::Pacer;

use super::{Flow, OpenBalance, Positions};
use crate::error::SettlementError;
use crate::opposites::{Pairing, pair_opposites};

/// How many subsets the table fill computes between two checks of the
/// pacer.
const SUBSETS_PER_CHECK: usize = 1 << 12;

/// The work between two yields of the table fill, counted as subsets times
/// balances, so a table of 16 balances or fewer fills without a yield.
const UNITS_PER_YIELD: u64 = 1 << 21;

/// Settles `balances`, ordered by member, with the fewest payments, yielding
/// while it fills its table.
///
/// # Errors
///
/// [`SettlementError::OutOfMemory`] when the 2ⁿ-byte table for the `n`
/// unpaired balances cannot be allocated.
pub(super) async fn settle(balances: &[OpenBalance]) -> Result<Vec<Flow>, SettlementError> {
    let Pairing { mut flows, rest } = pair_opposites(balances);
    flows.extend(settle_rest(&rest).await?);
    Ok(flows)
}

/// Settles `rest`, balances among which no two are exact opposites, with the
/// fewest payments, yielding while it fills its table.
///
/// # Errors
///
/// [`SettlementError::OutOfMemory`] when the 2ⁿ-byte table for the `n`
/// balances cannot be allocated.
pub(super) async fn settle_rest(rest: &[OpenBalance]) -> Result<Vec<Flow>, SettlementError> {
    let mut flows = Vec::new();
    for group in zero_sum_groups(rest).await? {
        flows.extend(settle_group(&group));
    }
    Ok(flows)
}

/// Partitions `balances` into the largest number of groups that each sum to
/// zero.
///
/// `table[S]` is the largest number of zero-sum groups the subset S (a
/// bitmask) splits into, counting S itself only when its sum is zero:
/// `table[S] = max over i in S of table[S ∖ {i}] + [ΣS = 0]`. Walking back
/// from the full set along that maximum orders the members so that the
/// prefix sums hit zero exactly at the group boundaries.
async fn zero_sum_groups(
    balances: &[OpenBalance],
) -> Result<Vec<Vec<OpenBalance>>, SettlementError> {
    let count = balances.len();
    if 0 == count {
        return Ok(Vec::new());
    }
    let out_of_memory = SettlementError::OutOfMemory { balances: count };
    let size = u32::try_from(count)
        .ok()
        .and_then(|bits| 1_usize.checked_shl(bits))
        .ok_or(out_of_memory)?;
    let mut table: Vec<u8> = Vec::new();
    table.try_reserve_exact(size).map_err(|_| out_of_memory)?;

    let nets: Vec<i128> = balances
        .iter()
        .map(|balance| i128::from(balance.net))
        .collect();
    // Subsets in increasing order, one chunk at a time between the pacer's
    // checks.
    table.push(0);
    let mut pacer = Pacer::new(UNITS_PER_YIELD);
    let mut start = 1;
    while start < size {
        let end = start.saturating_add(SUBSETS_PER_CHECK).min(size);
        fill_subsets(&mut table, start..end, &nets);
        // Counted per chunk, not per member of each subset: the aarch64
        // baseline has no scalar popcount, so `count_ones` in the inner loop
        // would cost a vector sequence for every subset. A chunk has at most
        // 2¹² subsets of fewer than 64 balances, so its work fits in a u64.
        let units = (end - start) * count;
        pacer
            .advance(u64::try_from(units).unwrap_or_default())
            .await;
        start = end;
    }

    // Walk back from the full set, removing the lowest member whose removal
    // keeps the maximum: the reversed removal order has zero prefix sums
    // exactly at the group boundaries. The recurrence guarantees such a
    // member exists in every non-empty subset, and the empty one has none.
    let removable = |subset: usize| {
        let closes_group = u8::from(
            0 == members_of(subset)
                .map(|position| net_of(&nets, position))
                .sum::<i128>(),
        );
        let target = groups_in(&table, subset);
        members_of(subset)
            .find(|&position| groups_in(&table, subset & !(1 << position)) + closes_group == target)
    };
    let mut order = Vec::with_capacity(count);
    let mut subset = size - 1;
    while let Some(position) = removable(subset) {
        order.push(position);
        subset &= !(1 << position);
    }

    let mut groups = Vec::new();
    let mut group = Vec::new();
    let mut prefix = 0_i128;
    for &balance in order
        .iter()
        .rev()
        .filter_map(|&position| balances.get(position))
    {
        group.push(balance);
        prefix += i128::from(balance.net);
        if 0 == prefix {
            groups.push(std::mem::take(&mut group));
        }
    }
    Ok(groups)
}

/// Pushes the table's entries for `subsets`, the next ones in increasing
/// order: every S ∖ {i} is smaller than S, so its entry is already in the
/// table when S is pushed.
// Not inlined: inlined into the future of `zero_sum_groups`, the loop
// compiles to slower code, as the exact solver's benchmarks show.
#[inline(never)]
fn fill_subsets(table: &mut Vec<u8>, subsets: Range<usize>, nets: &[i128]) {
    for subset in subsets {
        let mut best = 0;
        let mut total = 0_i128;
        for position in members_of(subset) {
            total += net_of(nets, position);
            best = best.max(groups_in(table, subset & !(1 << position)));
        }
        // At most count / 2 ≤ 32 groups on 64-bit targets, so no u8
        // overflows.
        table.push(best + u8::from(0 == total));
    }
}

/// The net balance at `position` of `nets`, or zero past their end, which
/// no member of a subset of the balances reaches.
fn net_of(nets: &[i128], position: usize) -> i128 {
    nets.get(position).copied().unwrap_or(0)
}

/// The most zero-sum groups `subset` splits into, from `table`, or zero past
/// its end, which no subset it is read for reaches: an entry is read only
/// after it is pushed.
fn groups_in(table: &[u8], subset: usize) -> u8 {
    table.get(subset).copied().unwrap_or(0)
}

/// The positions of the set bits of `subset`, lowest first.
fn members_of(mut subset: usize) -> impl Iterator<Item = usize> {
    std::iter::from_fn(move || {
        if 0 == subset {
            return None;
        }
        // A bit position is below usize::BITS, so it always fits in usize.
        let position = usize::try_from(subset.trailing_zeros()).unwrap_or_default();
        subset &= subset - 1;
        Some(position)
    })
}

/// Settles one zero-sum group with one payment fewer than its size: debtors
/// and creditors in member order, each payment the overlap of the current
/// debt and credit (the northwest-corner rule).
///
/// A group from a maximum partition has no zero-sum subgroup, so no debt and
/// credit ever clear together before the last payment, and every payment but
/// the last clears exactly one member.
fn settle_group(group: &[OpenBalance]) -> Vec<Flow> {
    let mut debtors: Vec<OpenBalance> = group
        .iter()
        .copied()
        .filter(|balance| 0 > balance.net)
        .collect();
    let mut creditors: Vec<OpenBalance> = group
        .iter()
        .copied()
        .filter(|balance| 0 < balance.net)
        .collect();
    debtors.sort_unstable_by_key(|balance| balance.member);
    creditors.sort_unstable_by_key(|balance| balance.member);
    let mut flows = Vec::with_capacity(group.len().saturating_sub(1));
    let mut debtors = debtors.into_iter();
    let mut creditors = creditors.into_iter();
    let (mut debtor, mut creditor) = (debtors.next(), creditors.next());
    while let (Some(owing), Some(owed)) = (&mut debtor, &mut creditor) {
        // A debtor's net is negative and above i64::MIN, so negating fits.
        let amount = (-owing.net).min(owed.net);
        flows.push(Flow {
            members: Positions {
                from: owing.member,
                to: owed.member,
            },
            amount,
        });
        owing.net += amount;
        owed.net -= amount;
        if 0 == owing.net {
            debtor = debtors.next();
        }
        if 0 == owed.net {
            creditor = creditors.next();
        }
    }
    flows
}

#[cfg(test)]
mod tests;
