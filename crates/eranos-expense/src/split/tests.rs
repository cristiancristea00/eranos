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

// Three members, in identifier order.

/// Ana, a member.
fn ana() -> MemberId {
    MemberId::new("ana").unwrap()
}

/// Bob, a member.
fn bob() -> MemberId {
    MemberId::new("bob").unwrap()
}

/// Carol, a member.
fn carol() -> MemberId {
    MemberId::new("carol").unwrap()
}

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

/// The member at `index` of a generated split; members sort by index.
fn member_at(index: usize) -> MemberId {
    MemberId::new(&format!("member-{index:03}")).unwrap()
}

/// A share would hand its member or its value to the wrong caller, and an
/// app reading a split back would show another member's share.
#[test]
fn shares_keep_their_member_and_value() {
    // Parts.
    let share = MemberParts::new(bob(), parts("1.5"));
    assert_eq!(share.member(), bob());
    assert_eq!(share.parts(), parts("1.5"));
    // A percentage.
    let percentage = MemberPercentage::new(carol(), per_cent("70"));
    assert_eq!(percentage.member(), carol());
    assert_eq!(percentage.percentage(), per_cent("70"));
    // An exact amount.
    let amount = MemberAmount::new(ana(), euros(1_250));
    assert_eq!(amount.member(), ana());
    assert_eq!(amount.amount(), euros(1_250));
}

/// Saved groups would stop loading, or load with members and values
/// swapped: a share is the two-element array `[member, value]` that format 2
/// wrote and format 3 keeps, and loading still checks the value.
#[cfg(feature = "serde")]
#[test]
fn every_share_serialises_as_a_member_and_value_array() {
    let member = "ana";
    // Each kind of share, and back.
    let share = MemberParts::new(ana(), parts("1.5"));
    assert_eq!(
        serde_json::to_value(share).unwrap(),
        serde_json::json!([member, 150])
    );
    assert_eq!(
        serde_json::from_value::<MemberParts>(serde_json::json!([member, 150])).unwrap(),
        share
    );
    let percentage = MemberPercentage::new(ana(), per_cent("70"));
    assert_eq!(
        serde_json::to_value(percentage).unwrap(),
        serde_json::json!([member, 7000])
    );
    assert_eq!(
        serde_json::from_value::<MemberPercentage>(serde_json::json!([member, 7000])).unwrap(),
        percentage
    );
    let amount = MemberAmount::new(ana(), euros(1_250));
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
    let order = [carol(), ana(), bob()];
    // Equal splits.
    assert_eq!(Split::equal(order).members().collect::<Vec<_>>(), order);
    // Parts.
    let parts_split = Split::parts([
        MemberParts::new(carol(), Parts::ONE),
        MemberParts::new(ana(), Parts::ONE),
        MemberParts::new(bob(), Parts::ONE),
    ]);
    assert_eq!(parts_split.members().collect::<Vec<_>>(), order);
    // Percentages.
    let split = Split::percentages([
        MemberPercentage::new(carol(), per_cent("20")),
        MemberPercentage::new(ana(), per_cent("30")),
        MemberPercentage::new(bob(), per_cent("50")),
    ]);
    assert_eq!(split.members().collect::<Vec<_>>(), order);
    // Amounts.
    let amounts_split = Split::amounts([
        MemberAmount::new(carol(), euros(1)),
        MemberAmount::new(ana(), euros(2)),
        MemberAmount::new(bob(), euros(3)),
    ]);
    assert_eq!(amounts_split.members().collect::<Vec<_>>(), order);
}

/// Payers given as one member, an array, or a vector would split another
/// way than the equal split among them that each stands for.
#[test]
fn a_member_an_array_and_a_vector_convert_to_an_equal_split() {
    assert_eq!(Split::from(ana()), Split::equal([ana()]));
    assert_eq!(Split::from([ana(), bob()]), Split::equal([ana(), bob()]));
    assert_eq!(
        Split::from(vec![ana(), bob()]),
        Split::equal([ana(), bob()])
    );
}

/// Split forms would show the wrong number of participants.
#[test]
fn len_counts_the_listed_members() {
    // One of every form.
    assert_eq!(Split::equal([ana(), bob(), carol()]).len(), 3);
    assert_eq!(
        Split::parts([
            MemberParts::new(ana(), Parts::ONE),
            MemberParts::new(bob(), Parts::ONE)
        ])
        .len(),
        2
    );
    assert_eq!(
        Split::percentages([MemberPercentage::new(ana(), Percentage::HUNDRED)]).len(),
        1
    );
    assert_eq!(
        Split::amounts([MemberAmount::new(ana(), euros(1))]).len(),
        1
    );
    // Only a split without members is empty.
    assert!(!Split::equal([ana()]).is_empty());
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
        Split::equal([ana(), bob(), ana()]).validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: ana() })
    );
    assert_eq!(
        Split::equal([ana(), bob(), bob(), ana()]).validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: bob() })
    );
    // Parts, percentages, and amounts.
    let parts_twice = Split::parts([
        MemberParts::new(carol(), Parts::ONE),
        MemberParts::new(carol(), Parts::ONE),
    ]);
    let percentages = Split::percentages([
        MemberPercentage::new(bob(), per_cent("50")),
        MemberPercentage::new(bob(), per_cent("50")),
    ]);
    let amounts = Split::amounts([
        MemberAmount::new(ana(), euros(500)),
        MemberAmount::new(ana(), euros(500)),
    ]);
    assert_eq!(
        parts_twice.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: carol() })
    );
    assert_eq!(
        percentages.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: bob() })
    );
    assert_eq!(
        amounts.validate(euros(1_000)),
        Err(ExpenseError::DuplicateMember { member: ana() })
    );
}

/// Equal and parts splits of a valid amount would be refused.
#[test]
fn equal_and_parts_splits_fit_any_total() {
    // Equal splits.
    assert_eq!(Split::equal([ana(), bob()]).validate(euros(1)), Ok(()));
    // Parts.
    let split = Split::parts([
        MemberParts::new(ana(), parts("2")),
        MemberParts::new(bob(), parts("0.5")),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
}

/// A member listed at 0 % would appear to share an expense they pay nothing
/// of.
#[test]
fn a_zero_percentage_is_rejected() {
    // Zero next to 100 %.
    let split = Split::percentages([
        MemberPercentage::new(ana(), Percentage::ZERO),
        MemberPercentage::new(bob(), Percentage::HUNDRED),
    ]);
    assert_eq!(
        split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: ana() })
    );
}

/// Percentages that miss 100 % would leave part of the amount unpaid or
/// charge too much.
#[test]
fn percentages_must_add_up_to_exactly_100() {
    // Exactly 100 %.
    let split = Split::percentages([
        MemberPercentage::new(ana(), per_cent("70")),
        MemberPercentage::new(bob(), per_cent("30")),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
    let single_member_split =
        Split::percentages([MemberPercentage::new(ana(), Percentage::HUNDRED)]);
    assert_eq!(single_member_split.validate(euros(1_000)), Ok(()));
    // One basis point short or over.
    let short = Split::percentages([
        MemberPercentage::new(ana(), per_cent("50")),
        MemberPercentage::new(bob(), per_cent("49.99")),
    ]);
    let over = Split::percentages([
        MemberPercentage::new(ana(), per_cent("50")),
        MemberPercentage::new(bob(), per_cent("50.01")),
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
    let huge = Percentage::MAX;
    assert_eq!(
        Split::percentages([
            MemberPercentage::new(ana(), huge),
            MemberPercentage::new(bob(), huge),
            MemberPercentage::new(carol(), huge)
        ])
        .validate(euros(1_000)),
        Err(ExpenseError::PercentagesDoNotAddUp {
            total_basis_points: 6_442_450_941
        })
    );
}

/// Dollars would be counted as euros in a euro expense.
#[test]
fn amounts_in_another_currency_are_rejected() {
    // A dollar share of a euro total.
    let dollars = Money::from_minor_units(500, CurrencyCode::USD);
    let split = Split::amounts([
        MemberAmount::new(ana(), euros(500)),
        MemberAmount::new(bob(), dollars),
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
        MemberAmount::new(ana(), euros(0)),
        MemberAmount::new(bob(), euros(1_000)),
    ]);
    assert_eq!(
        split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: ana() })
    );
    // Negative.
    let negative_share_split = Split::amounts([
        MemberAmount::new(ana(), euros(1_500)),
        MemberAmount::new(bob(), euros(-500)),
    ]);
    assert_eq!(
        negative_share_split.validate(euros(1_000)),
        Err(ExpenseError::NonPositiveShare { member: bob() })
    );
}

/// Exact amounts that miss the total would leave part of the amount unpaid
/// or charge too much.
#[test]
fn amounts_must_add_up_to_the_total() {
    // Exactly the total.
    let split = Split::amounts([
        MemberAmount::new(ana(), euros(500)),
        MemberAmount::new(bob(), euros(500)),
    ]);
    assert_eq!(split.validate(euros(1_000)), Ok(()));
    // One cent short or over.
    let short = Split::amounts([
        MemberAmount::new(ana(), euros(500)),
        MemberAmount::new(bob(), euros(499)),
    ]);
    let over = Split::amounts([
        MemberAmount::new(ana(), euros(500)),
        MemberAmount::new(bob(), euros(501)),
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
        MemberAmount::new(ana(), euros(i64::MAX)),
        MemberAmount::new(bob(), euros(i64::MAX)),
    ]);
    assert_eq!(split.validate(euros(1)), Err(ExpenseError::Overflow));
    // One minor unit past the range.
    let barely_overflowing_split = Split::amounts([
        MemberAmount::new(ana(), euros(i64::MAX)),
        MemberAmount::new(bob(), euros(1)),
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
    for order in [
        [carol(), bob(), ana()],
        [bob(), carol(), ana()],
        [ana(), bob(), carol()],
    ] {
        assert_eq!(
            Split::equal(order).divide(300, 0),
            vec![share(ana(), 100), share(bob(), 100), share(carol(), 100)]
        );
    }
}

/// A cent would be lost or created when an amount does not divide evenly.
#[test]
fn an_equal_split_hands_out_the_leftover_cent() {
    // 10.00 among three: two shares of 3.33 and one of 3.34.
    let shares = Split::equal([ana(), bob(), carol()]).divide(1_000, 7);
    let mut amounts: Vec<i64> = shares.iter().map(|share| share.amount).collect();
    amounts.sort_unstable();
    assert_eq!(amounts, [333, 333, 334]);
}

/// The same member would absorb every leftover cent of a group's even
/// splits.
#[test]
fn the_rotation_decides_who_gets_the_leftover_cent() {
    let split = Split::equal([ana(), bob(), carol()]);
    // Each rotation starts at the next member in ID order.
    assert_eq!(
        split.divide(1_000, 0),
        vec![share(ana(), 334), share(bob(), 333), share(carol(), 333)]
    );
    assert_eq!(
        split.divide(1_000, 1),
        vec![share(ana(), 333), share(bob(), 334), share(carol(), 333)]
    );
    assert_eq!(
        split.divide(1_000, 2),
        vec![share(ana(), 333), share(bob(), 333), share(carol(), 334)]
    );
    // The rotation wraps around.
    assert_eq!(split.divide(1_000, 3), split.divide(1_000, 0));
    assert_eq!(split.divide(1_000, u64::MAX), split.divide(1_000, 0));
    // Positions count in ID order, not in listing order.
    assert_eq!(
        Split::equal([carol(), bob(), ana()]).divide(1_000, 1),
        vec![share(ana(), 333), share(bob(), 334), share(carol(), 333)]
    );
}

/// A couple counted as two parts would pay the same as a single person.
#[test]
fn parts_divide_in_proportion() {
    // Two parts to one.
    let split = Split::parts([
        MemberParts::new(ana(), parts("2")),
        MemberParts::new(bob(), Parts::ONE),
    ]);
    assert_eq!(
        split.divide(3_000, 0),
        vec![share(ana(), 2_000), share(bob(), 1_000)]
    );
    // Fractional parts.
    let fractional_parts_split = Split::parts([
        MemberParts::new(ana(), parts("0.5")),
        MemberParts::new(bob(), Parts::ONE),
        MemberParts::new(carol(), parts("1.5")),
    ]);
    assert_eq!(
        fractional_parts_split.divide(3_000, 0),
        vec![
            share(ana(), 500),
            share(bob(), 1_000),
            share(carol(), 1_500)
        ]
    );
}

/// A 70/30 split would charge other shares than the percentages say.
#[test]
fn percentages_divide_in_proportion() {
    // 70 % and 30 %.
    let split = Split::percentages([
        MemberPercentage::new(ana(), per_cent("70")),
        MemberPercentage::new(bob(), per_cent("30")),
    ]);
    assert_eq!(
        split.divide(10_000, 0),
        vec![share(ana(), 7_000), share(bob(), 3_000)]
    );
    // The leftover cent goes to the largest remainder, whatever the
    // rotation.
    let uneven_thirds_split = Split::percentages([
        MemberPercentage::new(ana(), per_cent("33.33")),
        MemberPercentage::new(bob(), per_cent("33.33")),
        MemberPercentage::new(carol(), per_cent("33.34")),
    ]);
    for rotation in 0..3 {
        assert_eq!(
            uneven_thirds_split.divide(100, rotation),
            vec![share(ana(), 33), share(bob(), 33), share(carol(), 34)]
        );
    }
}

/// Members who entered exact amounts would be charged something else.
#[test]
fn amounts_are_returned_exactly() {
    // Whatever the rotation.
    let split = Split::amounts([
        MemberAmount::new(bob(), euros(750)),
        MemberAmount::new(ana(), euros(1_250)),
    ]);
    for rotation in 0..4 {
        assert_eq!(
            split.divide(2_000, rotation),
            vec![share(ana(), 1_250), share(bob(), 750)]
        );
    }
}

/// An expense of zero would charge someone a cent.
#[test]
fn a_zero_total_gives_every_member_zero() {
    // Equal splits and parts.
    assert_eq!(
        Split::equal([ana(), bob(), carol()]).divide(0, 1),
        vec![share(ana(), 0), share(bob(), 0), share(carol(), 0)]
    );
    let split = Split::parts([
        MemberParts::new(ana(), parts("2")),
        MemberParts::new(bob(), Parts::ONE),
    ]);
    assert_eq!(split.divide(0, 0), vec![share(ana(), 0), share(bob(), 0)]);
}

/// A refund would be divided differently from the expense it reverses.
#[test]
fn negative_totals_divide_like_positive_ones() {
    // The mirror of a positive division.
    let split = Split::equal([ana(), bob(), carol()]);
    assert_eq!(
        split.divide(-1_000, 0),
        vec![share(ana(), -334), share(bob(), -333), share(carol(), -333)]
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
        hundredths in prop::collection::vec(1..=Parts::MAX.hundredths(), 1..=12),
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
        MemberParts::new(ana(), parts("2")),
        MemberParts::new(bob(), parts("0.5")),
    ];
    let percentages = vec![
        MemberPercentage::new(ana(), per_cent("60")),
        MemberPercentage::new(bob(), per_cent("40")),
    ];
    let amounts = vec![
        MemberAmount::new(ana(), euros(700)),
        MemberAmount::new(bob(), euros(300)),
    ];
    assert_eq!(
        ConvertedSplit::from_split(&Split::equal([ana(), carol()])),
        ConvertedSplit::Equal(vec![ana(), carol()])
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
