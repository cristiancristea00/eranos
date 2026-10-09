//! Exact apportionment: splitting an amount by weights so the parts always
//! add up to the amount, plus the rounding helpers conversions, tips, and
//! quantities use.

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
    let weight_of = |weight: u64| if equal { 1 } else { weight };
    let weight_sum: u128 = weights
        .iter()
        .map(|&weight| u128::from(weight_of(weight)))
        .sum();
    // A usize is at most 64 bits wide on every target Rust supports.
    let len = u64::try_from(weights.len()).unwrap_or(u64::MAX);
    let start = rotation % len;
    // Every part is the magnitude's exact share rounded down, with the sign
    // of `total`, as a division that rounds towards zero gives it. When the
    // largest product and the sum of the weights fit in 64 bits, as they do
    // for every everyday amount, the shares are divided in 64-bit
    // arithmetic, which needs no call into 128-bit division; both widths
    // give the same quotients and remainders.
    let magnitude = total.unsigned_abs();
    let largest = weights
        .iter()
        .map(|&weight| weight_of(weight))
        .max()
        .unwrap_or_default();
    let narrow_sum = u64::try_from(weight_sum)
        .ok()
        .filter(|_| largest.checked_mul(magnitude).is_some());
    let divide = |weight: u64| {
        narrow_sum.map_or_else(
            || {
                // |total| ≤ 2^63 and weight < 2^64, so the product fits in
                // u128, and the quotient, at most the magnitude, fits back in
                // u64.
                let product = u128::from(magnitude) * u128::from(weight);
                Division {
                    quotient: u64::try_from(product / weight_sum).unwrap_or_default(),
                    remainder: product % weight_sum,
                }
            },
            |sum| {
                // No weight exceeds the largest, so the product fits.
                let product = magnitude * weight;
                Division {
                    quotient: product / sum,
                    remainder: u128::from(product % sum),
                }
            },
        )
    };
    // A quotient is at most the magnitude, so it fits in i64 with the sign of
    // `total`, the most negative amount's included.
    let signed = |quotient: u64| {
        if 0 > total {
            0_i64.checked_sub_unsigned(quotient)
        } else {
            i64::try_from(quotient).ok()
        }
        .unwrap_or_default()
    };
    // Every part's exact share rounded towards zero, and its claim to one of
    // the units the rounding leaves over.
    let mut parts = Vec::with_capacity(weights.len());
    let mut claims = Vec::with_capacity(weights.len());
    let mut allocated = 0_u128;
    for (position, &weight) in (0_u64..).zip(weights) {
        let Division {
            quotient,
            remainder,
        } = divide(weight_of(weight));
        allocated += u128::from(quotient);
        parts.push(signed(quotient));
        claims.push(LeftoverClaim {
            position: (position + len - start) % len,
            remainder,
        });
    }
    // Σ remainders = weight_sum × leftover, so leftover < weights.len(), and
    // the rank of the last claim to receive a unit is an index of `claims`.
    let leftover = u128::from(magnitude) - allocated;
    if let Some(rank) = usize::try_from(leftover)
        .ok()
        .and_then(|count| count.checked_sub(1))
    {
        // The claim of the last part to receive a leftover unit: every claim
        // is distinct, so the claims ranked up to it are the same whatever
        // order the selection leaves them in. At least leftover + 1 parts
        // have a positive remainder, so a zero-weight part never receives
        // one.
        let (_, &mut last, _) = claims.select_nth_unstable(rank);
        for claim in &claims {
            let index = usize::try_from((claim.position + start) % len).ok();
            if *claim <= last
                && let Some(part) = index.and_then(|index| parts.get_mut(index))
            {
                *part += total.signum();
            }
        }
    }
    parts
}

/// A part's exact share of the magnitude, rounded down, and what the
/// rounding left.
struct Division {
    /// The share rounded down.
    quotient: u64,
    /// The share's remainder over the sum of the weights.
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

/// The tie rotation for one allocation pass of an entry: 64-bit FNV-1a over
/// the UTF-8 bytes of the entry's identifier text, then the pass number's
/// eight bytes, least significant first.
///
/// FNV-1a is fixed by its specification, so every platform and every release
/// rotates the same way, and a synced entry splits identically everywhere.
#[must_use]
pub fn rotation(entry: &str, pass: u64) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    entry
        .as_bytes()
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
///
/// A basis point is a ten-thousandth, so this is
/// [`apply_ten_thousandths`] of the basis points.
#[must_use]
pub fn apply_basis_points(amount: i64, basis_points: u32) -> Option<i64> {
    apply_ten_thousandths(amount, u64::from(basis_points))
}

/// `amount × ten_thousandths / 10 000`, rounded half away from zero; `None`
/// when the result does not fit in `i64`.
///
/// It multiplies an item's unit price by its quantity, which is held in
/// ten-thousandths: 42.00 EUR times 0.35 is 14.70 EUR, and 1.00 EUR times
/// 0.335 is 0.34 EUR. A whole quantity gives the exact product.
#[must_use]
pub fn apply_ten_thousandths(amount: i64, ten_thousandths: u64) -> Option<i64> {
    // |amount| ≤ 2^63 and ten_thousandths < 2^64, so the product fits in
    // i128.
    let product = i128::from(amount) * i128::from(ten_thousandths);
    i64::try_from(divide_rounding_half_away(product, 10_000)).ok()
}

#[cfg(test)]
mod tests;
