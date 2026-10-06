//! Exact apportionment: splitting an amount by weights so the parts always
//! add up to the amount, plus the rounding helpers conversions and tips use.

use std::cmp::Ordering;

/// Splits `total` into parts proportional to `weights` that add up to exactly
/// `total`.
///
/// Largest-remainder (Hamilton) method: every part is `total × weight / Σ
/// weights` rounded towards zero, and the leftover units go one each to the
/// parts with the largest remainders, so every part is its exact share
/// rounded down or up. Equal remainders are ordered from position `rotation %
/// len` onwards, which spreads the leftover of even splits across callers
/// that vary the rotation. Weights that are all zero count as equal, so any
/// non-empty weights carry the whole total; empty weights give no parts.
#[must_use]
pub fn allocate(total: i64, weights: &[u64], rotation: u64) -> Vec<i64> {
    if weights.is_empty() {
        return Vec::new();
    }
    // All-zero weights say nothing about the split, so they count as equal.
    let equal = weights.iter().all(|&weight| 0 == weight);
    let weight_of = |weight: u64| if equal { 1 } else { i128::from(weight) };
    let weight_sum: i128 = weights.iter().map(|&weight| weight_of(weight)).sum();
    let total = i128::from(total);
    // |total| ≤ 2^63 and weight < 2^64, so every product fits in i128.
    // Division rounds towards zero, so every part has the sign of `total`.
    let mut parts: Vec<Part> = weights
        .iter()
        .map(|&weight| {
            let product = total * weight_of(weight);
            Part {
                amount: product / weight_sum,
                remainder: (product % weight_sum).unsigned_abs(),
            }
        })
        .collect();
    // Σ remainders = weight_sum × leftover, so leftover < weights.len().
    let leftover = (total - parts.iter().map(|part| part.amount).sum::<i128>()).unsigned_abs();
    if 0 < leftover {
        // A usize is at most 64 bits wide on every target Rust supports.
        let len = u64::try_from(weights.len()).unwrap_or(u64::MAX);
        let start = rotation % len;
        let claims: Vec<LeftoverClaim> = (0_u64..)
            .zip(&parts)
            .map(|(position, part)| LeftoverClaim {
                position: (position + len - start) % len,
                remainder: part.remainder,
            })
            .collect();
        let mut ranked = claims.clone();
        ranked.sort_unstable();
        // The claim of the last part to receive a leftover unit. At least
        // leftover + 1 parts have a positive remainder, so a zero-weight
        // part never receives one.
        let last = (1_u128..)
            .zip(ranked)
            .find_map(|(rank, claim)| (rank == leftover).then_some(claim));
        for (part, claim) in parts.iter_mut().zip(claims) {
            if last.is_some_and(|last_claim| claim <= last_claim) {
                part.amount += total.signum();
            }
        }
    }
    // Every part lies between zero and `total`, so it fits back in i64.
    parts
        .into_iter()
        .map(|part| i64::try_from(part.amount).unwrap_or_default())
        .collect()
}

/// One part of an allocation: its exact share rounded towards zero, until it
/// receives a leftover unit, and the remainder of that rounding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Part {
    amount: i128,
    remainder: u128,
}

/// A part's claim to a leftover unit: its remainder, and its position counted
/// from the rotation start, which makes every claim of an allocation
/// distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LeftoverClaim {
    position: u64,
    remainder: u128,
}

// Written out, so that it does not depend on the order of the fields: the
// largest remainders first, and equal ones by position, so that ties go
// round from the rotation start.
impl Ord for LeftoverClaim {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .remainder
            .cmp(&self.remainder)
            .then_with(|| self.position.cmp(&other.position))
    }
}

impl PartialOrd for LeftoverClaim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The tie rotation for one allocation pass of an entry: FNV-1a over the
/// entry's ID bytes and the pass number.
///
/// FNV-1a is fixed by its specification, so every platform and every release
/// rotates the same way, and a synced entry splits identically everywhere.
#[must_use]
pub fn rotation(entry: [u8; 16], pass: u64) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    entry
        .iter()
        .chain(&pass.to_le_bytes())
        .fold(OFFSET_BASIS, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
}

/// `numerator / denominator` rounded half away from zero; `denominator` must
/// be positive.
pub(crate) const fn divide_rounding_half_away(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    let remainder = (numerator % denominator).unsigned_abs();
    // remainder < denominator ≤ i128::MAX, so doubling fits in u128.
    if remainder * 2 >= denominator.unsigned_abs() {
        quotient + numerator.signum()
    } else {
        quotient
    }
}

/// `amount × basis_points / 10 000`, rounded half away from zero; `None` when
/// the result does not fit in `i64`.
#[must_use]
pub fn apply_basis_points(amount: i64, basis_points: u32) -> Option<i64> {
    let product = i128::from(amount) * i128::from(basis_points);
    i64::try_from(divide_rounding_half_away(product, 10_000)).ok()
}

#[cfg(test)]
mod tests;
