//! Every expense divides its amount through a `Split`, once for the payers
//! and once for the participants. These tests keep invalid splits (empty,
//! with a member twice, with a share that is not positive, or with
//! percentages or amounts that do not add up) out, and keep every division
//! exact: the shares add up to the amount to the minor unit, each is within
//! one minor unit of its exact value, and none depends on the order the
//! members were listed in; a conversion through `FromSplit` keeps the way of
//! dividing and its shares.

use eranos_money::CurrencyCode;
use proptest::prelude::*;

use super::*;

// Three members, in ID order.
const ANA: MemberId = MemberId::from_bytes([1; 16]);
const BOB: MemberId = MemberId::from_bytes([2; 16]);
const CAROL: MemberId = MemberId::from_bytes([3; 16]);

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn per_cent(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

fn parts(text: &str) -> Parts {
    Parts::parse(text).unwrap()
}

/// The share of `member`: `amount` minor units.
const fn share(member: MemberId, amount: i64) -> Share {
    Share { member, amount }
}

/// The member at `index` of a generated split.
fn member_at(index: usize) -> MemberId {
    MemberId::from_bytes([u8::try_from(index).unwrap(); 16])
}

/// A share would hand its member or its value to the wrong caller, and an
/// app reading a split back would show another member's share.
#[test]
fn shares_keep_their_member_and_value() {
    // Parts.
    let share = MemberParts::new(BOB, parts("1.5"));
    assert_eq!(share.member(), BOB);
    assert_eq!(share.parts(), parts("1.5"));
    // A percentage.
    let percentage = MemberPercentage::new(CAROL, per_cent("70"));
    assert_eq!(percentage.member(), CAROL);
    assert_eq!(percentage.percentage(), per_cent("70"));
    // An exact amount.
    let amount = MemberAmount::new(ANA, euros(1_250));
    assert_eq!(amount.member(), ANA);
    assert_eq!(amount.amount(), euros(1_250));
}

/// Groups saved before shares had names would stop loading, or load with
/// members and values swapped: a share is still the two-element array
/// `[member, value]` of format 2, and loading still checks the value.
#[cfg(feature = "serde")]
#[test]
fn every_share_serialises_as_a_member_and_value_array() {
    let member = "01010101-0101-0101-0101-010101010101";
    // Each kind of share, and back.
    let share = MemberParts::new(ANA, parts("1.5"));
    assert_eq!(
        serde_json::to_value(share).unwrap(),
        serde_json::json!([member, 150])
    );
    assert_eq!(
        serde_json::from_value::<MemberParts>(serde_json::json!([member, 150])).unwrap(),
        share
    );
    let percentage = MemberPercentage::new(ANA, per_cent("70"));
    assert_eq!(
        serde_json::to_value(percentage).unwrap(),
        serde_json::json!([member, 7000])
    );
    assert_eq!(
        serde_json::from_value::<MemberPercentage>(serde_json::json!([member, 7000])).unwrap(),
        percentage
    );
    let amount = MemberAmount::new(ANA, euros(1_250));
    let amount_json = serde_json::json!([member, {"currency": "EUR", "minor_units": 1250}]);
    assert_eq!(serde_json::to_value(amount).unwrap(), amount_json);
    assert_eq!(
        serde_json::from_value::<MemberAmount>(amount_json).unwrap(),
        amount
    );
    // Inside a split.
    assert_eq!(
        serde_json::to_value(Split::parts([share])).unwrap(),
        serde_json::json!({"parts": [[member, 150]]})
    );
    // Zero parts and a malformed pair are refused.
    assert!(serde_json::from_value::<MemberParts>(serde_json::json!([member, 0])).is_err());
    assert!(
        serde_json::from_value::<MemberParts>(serde_json::json!({"member": member, "parts": 150}))
            .is_err()
    );
    assert!(serde_json::from_value::<MemberPercentage>(serde_json::json!([member])).is_err());
    assert!(serde_json::from_value::<MemberAmount>(serde_json::json!([150, member])).is_err());
}

/// Split forms would list members in another order than the user entered
/// them.
#[test]
fn members_are_listed_in_the_order_given() {
    let order = [CAROL, ANA, BOB];
    // Equal splits.
    assert_eq!(Split::equal(order).members().collect::<Vec<_>>(), order);
    // Parts.
    let parts_split = Split::parts([
        MemberParts::new(CAROL, Parts::ONE),
        MemberParts::new(ANA, Parts::ONE),
        MemberParts::new(BOB, Parts::ONE),
    ]);
    assert_eq!(parts_split.members().collect::<Vec<_>>(), order);
    // Percentages.
    let split = Split::percentages([
        MemberPercentage::new(CAROL, per_cent("20")),
        MemberPercentage::new(ANA, per_cent("30")),
        MemberPercentage::new(BOB, per_cent("50")),
    ]);
    assert_eq!(split.members().collect::<Vec<_>>(), order);
    // Amounts.
    let amounts_split = Split::amounts([
        MemberAmount::new(CAROL, euros(1)),
        MemberAmount::new(ANA, euros(2)),
        MemberAmount::new(BOB, euros(3)),
    ]);
    assert_eq!(amounts_split.members().collect::<Vec<_>>(), order);
}

/// Split forms would show the wrong number of participants.
#[test]
fn len_counts_the_listed_members() {
    // One of every form.
    assert_eq!(Split::equal([ANA, BOB, CAROL]).len(), 3);
    assert_eq!(
        Split::parts([
            MemberParts::new(ANA, Parts::ONE),
            MemberParts::new(BOB, Parts::ONE)
        ])
        .len(),
        2
    );
    assert_eq!(
        Split::percentages([MemberPercentage::new(ANA, Percentage::HUNDRED)]).len(),
        1
    );
    assert_eq!(Split::amounts([MemberAmount::new(ANA, euros(1))]).len(), 1);
    // Only a split without members is empty.
    assert!(!Split::equal([ANA]).is_empty());
    for split in [
        Split::equal([]),
        Split::parts([]),
        Split::percentages([]),
        Split::amounts([]),
    ] {
        assert_eq!(split.len(), 0);
        assert!(split.is_empty());
    }
}

/// An expense nobody shares would be accepted and its amount would vanish
/// from every balance.
#[test]
fn an_empty_split_is_rejected() {
    // Every form.
    for split in [
        Split::equal([]),
        Split::parts([]),
        Split::percentages([]),
        Split::amounts([]),
    ] {
        assert_eq!(split.validate(euros(1_000)), Err(ExpenseError::EmptySplit));
    }
}

/// A member listed twice would pay twice their share.
#[test]
fn a_member_listed_twice_is_rejected() {
    // Equal splits name the first repeated member.
    assert_eq!(
        Split::equal([ANA, BOB, ANA]).validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: ANA })
    );
    assert_eq!(
        Split::equal([ANA, BOB, BOB, ANA]).validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: BOB })
    );
    // Parts, percentages, and amounts.
    let parts_twice = Split::parts([
        MemberParts::new(CAROL, Parts::ONE),
        MemberParts::new(CAROL, Parts::ONE),
    ]);
    let percentages = Split::percentages([
        MemberPercentage::new(BOB, per_cent("50")),
        MemberPercentage::new(BOB, per_cent("50")),
    ]);
    let amounts = Split::amounts([
        MemberAmount::new(ANA, euros(500)),
        MemberAmount::new(ANA, euros(500)),
    ]);
    assert_eq!(
        parts_twice.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: CAROL })
    );
    assert_eq!(
        percentages.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: BOB })
    );
    assert_eq!(
        amounts.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: ANA })
    );
}

/// Equal and parts splits of a valid amount would be refused.
#[test]
fn equal_and_parts_splits_fit_any_total() {
    // Equal splits.
    assert_eq!(Split::equal([ANA, BOB]).validate(euros(1)), Ok(()));
    // Parts.
    let split = Split::parts([
        MemberParts::new(ANA, parts("2")),
        MemberParts::new(BOB, parts("0.5")),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
}

/// A member listed at 0 % would appear to share an expense they pay nothing
/// of.
#[test]
fn a_zero_percentage_is_rejected() {
    // Zero next to 100 %.
    let split = Split::percentages([
        MemberPercentage::new(ANA, Percentage::ZERO),
        MemberPercentage::new(BOB, Percentage::HUNDRED),
    ]);
    assert_eq!(
        split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: ANA })
    );
}

/// Percentages that miss 100 % would leave part of the amount unpaid or
/// charge too much.
#[test]
fn percentages_must_add_up_to_exactly_100() {
    // Exactly 100 %.
    let split = Split::percentages([
        MemberPercentage::new(ANA, per_cent("70")),
        MemberPercentage::new(BOB, per_cent("30")),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
    let single_member_split = Split::percentages([MemberPercentage::new(ANA, Percentage::HUNDRED)]);
    assert_eq!(single_member_split.validate(euros(1_000)), Ok(()));
    // One basis point short or over.
    let short = Split::percentages([
        MemberPercentage::new(ANA, per_cent("50")),
        MemberPercentage::new(BOB, per_cent("49.99")),
    ]);
    let over = Split::percentages([
        MemberPercentage::new(ANA, per_cent("50")),
        MemberPercentage::new(BOB, per_cent("50.01")),
    ]);
    assert_eq!(
        short.validate(euros(1_000)),
        Err(ExpenseError::PercentagesDoNotAddUp {
            total_basis_points: 9_999
        })
    );
    assert_eq!(
        over.validate(euros(1_000)),
        Err(ExpenseError::PercentagesDoNotAddUp {
            total_basis_points: 10_001
        })
    );
    // Totals beyond 32 bits are reported exactly.
    let huge = Percentage::from_basis_points(u32::MAX);
    assert_eq!(
        Split::percentages([
            MemberPercentage::new(ANA, huge),
            MemberPercentage::new(BOB, huge)
        ])
        .validate(euros(1_000)),
        Err(ExpenseError::PercentagesDoNotAddUp {
            total_basis_points: 8_589_934_590
        })
    );
}

/// Dollars would be counted as euros in a euro expense.
#[test]
fn amounts_in_another_currency_are_rejected() {
    // A dollar share of a euro total.
    let dollars = Money::from_minor_units(500, CurrencyCode::USD);
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(500)),
        MemberAmount::new(BOB, dollars),
    ]);
    assert_eq!(
        split.validate(euros(1_000)),
        Err(ExpenseError::CurrencyMismatch {
            expected: CurrencyCode::EUR,
            found: CurrencyCode::USD
        })
    );
}

/// A member listed with nothing to pay, or with a negative share that
/// others pay for, would be accepted.
#[test]
fn zero_or_negative_amounts_are_rejected() {
    // Zero.
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(0)),
        MemberAmount::new(BOB, euros(1_000)),
    ]);
    assert_eq!(
        split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: ANA })
    );
    // Negative.
    let negative_share_split = Split::amounts([
        MemberAmount::new(ANA, euros(1_500)),
        MemberAmount::new(BOB, euros(-500)),
    ]);
    assert_eq!(
        negative_share_split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: BOB })
    );
}

/// Exact amounts that miss the total would leave part of the amount unpaid
/// or charge too much.
#[test]
fn amounts_must_add_up_to_the_total() {
    // Exactly the total.
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(500)),
        MemberAmount::new(BOB, euros(500)),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
    // One cent short or over.
    let short = Split::amounts([
        MemberAmount::new(ANA, euros(500)),
        MemberAmount::new(BOB, euros(499)),
    ]);
    let over = Split::amounts([
        MemberAmount::new(ANA, euros(500)),
        MemberAmount::new(BOB, euros(501)),
    ]);
    assert_eq!(
        short.validate(euros(1_000)),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(1_000),
            actual: euros(999)
        })
    );
    assert_eq!(
        over.validate(euros(1_000)),
        Err(ExpenseError::AmountsDoNotAddUp {
            expected: euros(1_000),
            actual: euros(1_001)
        })
    );
}

/// Amounts whose sum leaves the 64-bit range would wrap around and could
/// appear to add up.
#[test]
fn amounts_summing_past_the_range_report_overflow() {
    // Far past the range.
    let split = Split::amounts([
        MemberAmount::new(ANA, euros(i64::MAX)),
        MemberAmount::new(BOB, euros(i64::MAX)),
    ]);
    assert_eq!(split.validate(euros(1)), Err(ExpenseError::Overflow));
    // One minor unit past the range.
    let barely_overflowing_split = Split::amounts([
        MemberAmount::new(ANA, euros(i64::MAX)),
        MemberAmount::new(BOB, euros(1)),
    ]);
    assert_eq!(
        barely_overflowing_split.validate(euros(i64::MAX)),
        Err(ExpenseError::Overflow)
    );
}

/// The same expense would split differently on two devices that list its
/// members in different orders.
#[test]
fn shares_are_sorted_by_member_whatever_the_listing_order() {
    // Listed backwards and shuffled.
    for order in [[CAROL, BOB, ANA], [BOB, CAROL, ANA], [ANA, BOB, CAROL]] {
        assert_eq!(
            Split::equal(order).divide(300, 0),
            vec![share(ANA, 100), share(BOB, 100), share(CAROL, 100)]
        );
    }
}

/// A cent would be lost or created when an amount does not divide evenly.
#[test]
fn an_equal_split_hands_out_the_leftover_cent() {
    // 10.00 among three: two shares of 3.33 and one of 3.34.
    let shares = Split::equal([ANA, BOB, CAROL]).divide(1_000, 7);
    let mut amounts: Vec<i64> = shares.iter().map(|share| share.amount).collect();
    amounts.sort_unstable();
    assert_eq!(amounts, [333, 333, 334]);
}

/// The same member would absorb every leftover cent of a group's even
/// splits.
#[test]
fn the_rotation_decides_who_gets_the_leftover_cent() {
    let split = Split::equal([ANA, BOB, CAROL]);
    // Each rotation starts at the next member in ID order.
    assert_eq!(
        split.divide(1_000, 0),
        vec![share(ANA, 334), share(BOB, 333), share(CAROL, 333)]
    );
    assert_eq!(
        split.divide(1_000, 1),
        vec![share(ANA, 333), share(BOB, 334), share(CAROL, 333)]
    );
    assert_eq!(
        split.divide(1_000, 2),
        vec![share(ANA, 333), share(BOB, 333), share(CAROL, 334)]
    );
    // The rotation wraps around.
    assert_eq!(split.divide(1_000, 3), split.divide(1_000, 0));
    assert_eq!(split.divide(1_000, u64::MAX), split.divide(1_000, 0));
    // Positions count in ID order, not in listing order.
    assert_eq!(
        Split::equal([CAROL, BOB, ANA]).divide(1_000, 1),
        vec![share(ANA, 333), share(BOB, 334), share(CAROL, 333)]
    );
}

/// A couple counted as two parts would pay the same as a single person.
#[test]
fn parts_divide_in_proportion() {
    // Two parts to one.
    let split = Split::parts([
        MemberParts::new(ANA, parts("2")),
        MemberParts::new(BOB, Parts::ONE),
    ]);
    assert_eq!(
        split.divide(3_000, 0),
        vec![share(ANA, 2_000), share(BOB, 1_000)]
    );
    // Fractional parts.
    let fractional_parts_split = Split::parts([
        MemberParts::new(ANA, parts("0.5")),
        MemberParts::new(BOB, Parts::ONE),
        MemberParts::new(CAROL, parts("1.5")),
    ]);
    assert_eq!(
        fractional_parts_split.divide(3_000, 0),
        vec![share(ANA, 500), share(BOB, 1_000), share(CAROL, 1_500)]
    );
}

/// A 70/30 split would charge other shares than the percentages say.
#[test]
fn percentages_divide_in_proportion() {
    // 70 % and 30 %.
    let split = Split::percentages([
        MemberPercentage::new(ANA, per_cent("70")),
        MemberPercentage::new(BOB, per_cent("30")),
    ]);
    assert_eq!(
        split.divide(10_000, 0),
        vec![share(ANA, 7_000), share(BOB, 3_000)]
    );
    // The leftover cent goes to the largest remainder, whatever the
    // rotation.
    let uneven_thirds_split = Split::percentages([
        MemberPercentage::new(ANA, per_cent("33.33")),
        MemberPercentage::new(BOB, per_cent("33.33")),
        MemberPercentage::new(CAROL, per_cent("33.34")),
    ]);
    for rotation in 0..3 {
        assert_eq!(
            uneven_thirds_split.divide(100, rotation),
            vec![share(ANA, 33), share(BOB, 33), share(CAROL, 34)]
        );
    }
}

/// Members who entered exact amounts would be charged something else.
#[test]
fn amounts_are_returned_exactly() {
    // Whatever the rotation.
    let split = Split::amounts([
        MemberAmount::new(BOB, euros(750)),
        MemberAmount::new(ANA, euros(1_250)),
    ]);
    for rotation in 0..4 {
        assert_eq!(
            split.divide(2_000, rotation),
            vec![share(ANA, 1_250), share(BOB, 750)]
        );
    }
}

/// An expense of zero would charge someone a cent.
#[test]
fn a_zero_total_gives_every_member_zero() {
    // Equal splits and parts.
    assert_eq!(
        Split::equal([ANA, BOB, CAROL]).divide(0, 1),
        vec![share(ANA, 0), share(BOB, 0), share(CAROL, 0)]
    );
    let split = Split::parts([
        MemberParts::new(ANA, parts("2")),
        MemberParts::new(BOB, Parts::ONE),
    ]);
    assert_eq!(split.divide(0, 0), vec![share(ANA, 0), share(BOB, 0)]);
}

/// A refund would be divided differently from the expense it reverses.
#[test]
fn negative_totals_divide_like_positive_ones() {
    // The mirror of a positive division.
    let split = Split::equal([ANA, BOB, CAROL]);
    assert_eq!(
        split.divide(-1_000, 0),
        vec![share(ANA, -334), share(BOB, -333), share(CAROL, -333)]
    );
}

/// An empty split has no members to give shares to; validation refuses it
/// before any expense divides by it.
#[test]
fn an_empty_split_divides_into_no_shares() {
    assert_eq!(Split::equal([]).divide(100, 0), Vec::new());
    assert_eq!(
        Split::equal([]).validate(euros(100)),
        Err(ExpenseError::EmptySplit)
    );
}

proptest! {
    /// A division would lose or create minor units, or hand a member more
    /// than one unit away from their exact share.
    #[test]
    fn divisions_are_exact_and_within_one_unit(
        hundredths in prop::collection::vec(1..=u32::MAX, 1..=12),
        total in any::<i64>(),
        rotation in any::<u64>(),
    ) {
        let split = Split::parts(hundredths.iter().enumerate().map(|(index, &weight)| {
            MemberParts::new(member_at(index), Parts::from_hundredths(weight).unwrap())
        }));
        let shares = split.divide(total, rotation);
        prop_assert_eq!(shares.len(), hundredths.len());
        // Exact: the shares add up to the total.
        let sum: i128 = shares.iter().map(|share| i128::from(share.amount)).sum();
        prop_assert_eq!(sum, i128::from(total));
        // Fair: |share − total × weight / Σ weights| < 1.
        let weight_sum: i128 = hundredths.iter().map(|&weight| i128::from(weight)).sum();
        for (index, &weight) in hundredths.iter().enumerate() {
            let Share { member, amount } = shares[index];
            prop_assert_eq!(member, member_at(index));
            let deviation =
                i128::from(amount) * weight_sum - i128::from(total) * i128::from(weight);
            prop_assert!(deviation.abs() < weight_sum);
        }
    }

    /// Two devices listing the same members in different orders would
    /// split the same expense differently.
    #[test]
    fn listing_order_never_changes_the_shares(
        hundredths in prop::collection::vec(1..=10_000_u32, 1..=12),
        total in any::<i64>(),
        rotation in any::<u64>(),
    ) {
        let listed: Vec<MemberParts> = hundredths
            .iter()
            .enumerate()
            .map(|(index, &weight)| {
                MemberParts::new(member_at(index), Parts::from_hundredths(weight).unwrap())
            })
            .collect();
        let reversed: Vec<MemberParts> = listed.iter().rev().copied().collect();
        prop_assert_eq!(
            Split::parts(listed).divide(total, rotation),
            Split::parts(reversed).divide(total, rotation)
        );
    }
}

/// A split as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedSplit {
    Equal(Vec<MemberId>),
    Parts(Vec<MemberParts>),
    Percentages(Vec<MemberPercentage>),
    Amounts(Vec<MemberAmount>),
}

impl FromSplit for ConvertedSplit {
    fn equal(members: &[MemberId]) -> Self {
        Self::Equal(members.to_vec())
    }

    fn parts(shares: &[MemberParts]) -> Self {
        Self::Parts(shares.to_vec())
    }

    fn percentages(shares: &[MemberPercentage]) -> Self {
        Self::Percentages(shares.to_vec())
    }

    fn amounts(shares: &[MemberAmount]) -> Self {
        Self::Amounts(shares.to_vec())
    }
}

/// A conversion that called another variant's method, or lost a share,
/// would hand the bindings another division than the library's.
#[test]
fn a_split_converts_through_the_method_of_its_variant() {
    let shares = vec![
        MemberParts::new(ANA, parts("2")),
        MemberParts::new(BOB, parts("0.5")),
    ];
    let percentages = vec![
        MemberPercentage::new(ANA, per_cent("60")),
        MemberPercentage::new(BOB, per_cent("40")),
    ];
    let amounts = vec![
        MemberAmount::new(ANA, euros(700)),
        MemberAmount::new(BOB, euros(300)),
    ];
    assert_eq!(
        ConvertedSplit::from_split(&Split::equal([ANA, CAROL])),
        ConvertedSplit::Equal(vec![ANA, CAROL])
    );
    assert_eq!(
        ConvertedSplit::from_split(&Split::parts(shares.clone())),
        ConvertedSplit::Parts(shares)
    );
    assert_eq!(
        ConvertedSplit::from_split(&Split::percentages(percentages.clone())),
        ConvertedSplit::Percentages(percentages)
    );
    assert_eq!(
        ConvertedSplit::from_split(&Split::amounts(amounts.clone())),
        ConvertedSplit::Amounts(amounts)
    );
}
