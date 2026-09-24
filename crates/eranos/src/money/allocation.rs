//! Exact apportionment: splitting an amount by weights so the parts always
//! add up to the amount, plus the rounding helpers conversions and tips use.

use std::cmp::Reverse;

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
pub(crate) fn allocate(total: i64, weights: &[u64], rotation: u64) -> Vec<i64> {
    if weights.is_empty() {
        return Vec::new();
    }
    // All-zero weights say nothing about the split, so they count as equal.
    let equal = weights.iter().all(|&weight| weight == 0);
    let weight_of = |weight: u64| if equal { 1 } else { i128::from(weight) };
    let weight_sum: i128 = weights.iter().map(|&weight| weight_of(weight)).sum();
    let total = i128::from(total);
    // |total| ≤ 2^63 and weight < 2^64, so every product fits in i128.
    // Division rounds towards zero, so every part has the sign of `total`.
    let (mut parts, remainders): (Vec<i128>, Vec<u128>) = weights
        .iter()
        .map(|&weight| {
            let product = total * weight_of(weight);
            (product / weight_sum, (product % weight_sum).unsigned_abs())
        })
        .unzip();
    // Σ remainders = weight_sum × leftover, so leftover < weights.len().
    let leftover = (total - parts.iter().sum::<i128>()).unsigned_abs();
    if leftover > 0 {
        // A usize is at most 64 bits wide on every target Rust supports.
        let len = u64::try_from(weights.len()).unwrap_or(u64::MAX);
        let start = rotation % len;
        // (remainder, position from the rotation start): the largest
        // remainders sort first and equal ones by rotation; the positions
        // make every key distinct.
        let keys: Vec<(Reverse<u128>, u64)> = (0_u64..)
            .zip(remainders)
            .map(|(position, remainder)| (Reverse(remainder), (position + len - start) % len))
            .collect();
        let mut ranked = keys.clone();
        ranked.sort_unstable();
        // The key of the last part to receive a leftover unit. At least
        // leftover + 1 parts have a positive remainder, so a zero-weight
        // part never receives one.
        let last = (1_u128..)
            .zip(ranked)
            .find_map(|(rank, key)| (rank == leftover).then_some(key));
        for (part, key) in parts.iter_mut().zip(keys) {
            if last.is_some_and(|last_key| key <= last_key) {
                *part += total.signum();
            }
        }
    }
    // Every part lies between zero and `total`, so it fits back in i64.
    parts
        .into_iter()
        .map(|part| i64::try_from(part).unwrap_or_default())
        .collect()
}

/// The tie rotation for one allocation pass of an entry: FNV-1a over the
/// entry's ID bytes and the pass number.
///
/// FNV-1a is fixed by its specification, so every platform and every release
/// rotates the same way, and a synced entry splits identically everywhere.
pub(crate) fn rotation(entry: [u8; 16], pass: u64) -> u64 {
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
pub(crate) fn apply_basis_points(amount: i64, basis_points: u32) -> Option<i64> {
    let product = i128::from(amount) * i128::from(basis_points);
    i64::try_from(divide_rounding_half_away(product, 10_000)).ok()
}

#[cfg(test)]
mod tests;
