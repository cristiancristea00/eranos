//! Tests for exact apportionment. Every split, tip, conversion, and debt in
//! the library goes through `allocate`, so a part that does not add up here
//! is money that appears or vanishes in every group; the rotation hash is
//! pinned because synced devices must split the same entry identically.

use proptest::prelude::*;

use super::*;

/// The exact share `total × weight / Σ weights`, as a rational
/// (numerator, denominator) in 128-bit arithmetic.
fn exact_share(total: i64, weights: &[u64], index: usize) -> (i128, i128) {
    let sum: i128 = weights.iter().map(|&weight| i128::from(weight)).sum();
    (i128::from(total) * i128::from(weights[index]), sum)
}

proptest! {
    /// A split whose parts do not add up to the amount would create or lose
    /// money in the group's balances.
    #[test]
    fn parts_always_add_up_to_the_total(
        total in -1_000_000_000_000_i64..1_000_000_000_000,
        weights in proptest::collection::vec(0_u64..1_000_000, 1..12),
        rotation in any::<u64>(),
    ) {
        let parts = allocate(total, &weights, rotation);
        prop_assert_eq!(parts.len(), weights.len());
        prop_assert_eq!(parts.iter().map(|&part| i128::from(part)).sum::<i128>(), i128::from(total));
    }

    /// A part further than one minor unit from its exact share would charge
    /// someone for somebody else's consumption.
    #[test]
    fn every_part_is_its_exact_share_rounded_down_or_up(
        total in 0_i64..1_000_000_000_000,
        weights in proptest::collection::vec(0_u64..1_000_000, 1..12),
        rotation in any::<u64>(),
    ) {
        prop_assume!(weights.iter().any(|&weight| weight > 0));
        let parts = allocate(total, &weights, rotation);
        for (index, &part) in parts.iter().enumerate() {
            let (numerator, denominator) = exact_share(total, &weights, index);
            let floor = numerator / denominator;
            prop_assert!(i128::from(part) == floor || i128::from(part) == floor + 1);
        }
    }
}

/// Even splits must spread their leftover cents across entries instead of
/// always charging the first-listed member.
#[test]
fn an_even_split_gives_the_leftover_from_the_rotation_start() {
    // 10 among three: one part carries the extra unit.
    assert_eq!(allocate(10, &[1, 1, 1], 0), vec![4, 3, 3]);
    assert_eq!(allocate(10, &[1, 1, 1], 1), vec![3, 4, 3]);
    assert_eq!(allocate(10, &[1, 1, 1], 2), vec![3, 3, 4]);
    // The rotation wraps around the number of parts.
    assert_eq!(allocate(10, &[1, 1, 1], 3), vec![4, 3, 3]);
    // Two leftover units go to consecutive parts from the start.
    assert_eq!(allocate(11, &[1, 1, 1], 2), vec![4, 3, 4]);
}

/// The largest-remainder rule decides first; only equal remainders consult
/// the rotation, or proportional splits would round against the member who
/// is closest to the next unit.
#[test]
fn the_largest_remainder_wins_before_the_rotation() {
    // 10 split 1:2 is 3.33 and 6.67: the larger remainder gets the unit.
    for rotation in 0..4 {
        assert_eq!(allocate(10, &[1, 2], rotation), vec![3, 7]);
    }
}

/// The leftover unit follows the fraction, not the weight: a smaller share
/// that is closer to its next unit gets it first.
#[test]
fn the_leftover_follows_the_fraction_not_the_weight() {
    // 3 split 5:2 is 2.14 and 0.86: the second part is closer to 1.
    for rotation in 0..4 {
        assert_eq!(allocate(3, &[5, 2], rotation), vec![2, 1]);
    }
}

/// A zero-weight part must stay zero, or a member outside a split would pay
/// for it.
#[test]
fn a_zero_weight_never_receives_a_leftover_unit() {
    for rotation in 0..6 {
        let parts = allocate(5, &[0, 1, 1], rotation);
        assert_eq!(parts[0], 0);
        assert_eq!(parts.iter().sum::<i64>(), 5);
    }
}

/// Negative totals mirror positive ones, so reversing an amount reverses
/// every part exactly.
#[test]
fn negative_totals_split_symmetrically() {
    assert_eq!(allocate(-10, &[1, 1, 1], 0), vec![-4, -3, -3]);
    assert_eq!(allocate(-10, &[1, 2], 0), vec![-3, -7]);
}

/// Weights that are all zero must still carry the whole total, or money
/// would vanish; they count as equal.
#[test]
fn all_zero_weights_count_as_equal() {
    // Split evenly, leftover from the rotation start.
    assert_eq!(allocate(5, &[0, 0], 0), vec![3, 2]);
    assert_eq!(allocate(5, &[0, 0], 1), vec![2, 3]);
    assert_eq!(allocate(-5, &[0, 0], 0), vec![-3, -2]);
    // Nothing to split gives zeros, whatever the weights.
    assert_eq!(allocate(0, &[0, 0], 0), vec![0, 0]);
    assert_eq!(allocate(0, &[3, 1], 7), vec![0, 0]);
}

/// No weights means no parts: there is nobody to carry the total, and
/// every caller has at least one weight.
#[test]
fn no_weights_give_no_parts() {
    assert_eq!(allocate(5, &[], 0), Vec::<i64>::new());
    assert_eq!(allocate(0, &[], 0), Vec::<i64>::new());
}

/// The 128-bit intermediates must hold the extreme products, or large
/// amounts would wrap into garbage.
#[test]
fn extreme_totals_and_weights_do_not_overflow() {
    // The largest total with the largest weights.
    let parts = allocate(i64::MAX, &[u64::MAX, u64::MAX, 1], 1);
    assert_eq!(
        parts.iter().map(|&part| i128::from(part)).sum::<i128>(),
        i128::from(i64::MAX)
    );
    // The smallest total, whose magnitude does not fit in i64.
    assert_eq!(
        allocate(i64::MIN, &[1, 1], 0),
        vec![i64::MIN / 2, i64::MIN / 2]
    );
    // All of the smallest total to a single part.
    assert_eq!(allocate(i64::MIN, &[0, 1], 0), vec![0, i64::MIN]);
}

/// The rotation must be the same on every platform and release, or synced
/// devices would split the same entry differently.
#[test]
fn the_rotation_is_fnv1a_over_the_entry_and_the_pass() {
    // Reference values computed independently of this implementation.
    assert_eq!(rotation([0; 16], 0), 0x81d2_3fd7_003c_2305);
    assert_eq!(rotation([0; 16], 1), 0x62d7_78cd_f54c_d8e4);
    let bytes: [u8; 16] = std::array::from_fn(|index| u8::try_from(index).unwrap());
    assert_eq!(rotation(bytes, 7), 0x1a65_92dd_385c_f9b2);
}

/// Conversions and percentage tips round half away from zero, the EU
/// conversion rule; any other rounding would disagree with bank statements.
#[test]
fn division_rounds_half_away_from_zero() {
    // Exact halves go away from zero on both sides.
    assert_eq!(divide_rounding_half_away(5, 2), 3);
    assert_eq!(divide_rounding_half_away(-5, 2), -3);
    // Below half rounds towards zero, above half away from it.
    assert_eq!(divide_rounding_half_away(4, 3), 1);
    assert_eq!(divide_rounding_half_away(5, 3), 2);
    assert_eq!(divide_rounding_half_away(-4, 3), -1);
    assert_eq!(divide_rounding_half_away(-5, 3), -2);
    // Exact quotients are unchanged.
    assert_eq!(divide_rounding_half_away(0, 7), 0);
    assert_eq!(divide_rounding_half_away(21, 7), 3);
}

/// Percentage tips must round like the receipt: half a cent up.
#[test]
fn basis_points_round_half_away_from_zero() {
    // 12.5 % of 10.00 is exact.
    assert_eq!(apply_basis_points(1000, 1250), Some(125));
    // 12.5 % of 0.99 is 12.375 cents.
    assert_eq!(apply_basis_points(99, 1250), Some(12));
    // 15 % of 0.10 is 1.5 cents.
    assert_eq!(apply_basis_points(10, 1500), Some(2));
    // A result beyond i64 is refused instead of wrapped.
    assert_eq!(apply_basis_points(i64::MAX, 20_000), None);
}
