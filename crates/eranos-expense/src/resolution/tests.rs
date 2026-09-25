//! Tests for resolving entries into group-currency shares. These numbers are
//! what every balance and settlement is built from: a share that does not add
//! up, a tip split the wrong way, a conversion rounded differently, or a
//! pairwise debt that disagrees with the net balances would all show up as
//! wrong amounts owed. The cases are worked out by hand; the properties
//! cover random splits, payers, tips, and conversions.

use std::collections::BTreeSet;

use eranos_calendar::CalendarDate;
use eranos_core::ExpenseId;
use eranos_money::{Cost, Parts, Percentage};
use proptest::prelude::*;

use super::*;
use crate::{Item, MemberAmount, MemberParts, Split, Tip, TipDistribution};

/// `expense` under the identifier `id`, which seeds its tie rotation.
fn identified(expense: &Expense, id: ExpenseId) -> Expense {
    expense.to_builder().with_id(id).build().unwrap()
}

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

fn entry(number: u8) -> ExpenseId {
    ExpenseId::from_bytes([number; 16])
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

fn day() -> CalendarDate {
    CalendarDate::new(23, 9, 2026).unwrap()
}

/// (member, paid, share) in minor units, ordered by member.
fn rows(resolved: &ResolvedExpense) -> Vec<(MemberId, i64, i64)> {
    resolved
        .shares
        .members()
        .iter()
        .map(|share| {
            (
                share.member(),
                share.paid().minor_units(),
                share.share().minor_units(),
            )
        })
        .collect()
}

/// What each member is owed minus what they owe, from the pairwise debts.
fn debt_nets(resolved: &ResolvedExpense) -> BTreeMap<MemberId, i64> {
    let mut nets = BTreeMap::new();
    for debt in &resolved.debts {
        *nets.entry(debt.members.to).or_insert(0) += debt.amount;
        *nets.entry(debt.members.from).or_insert(0) -= debt.amount;
    }
    nets
}

/// A debt of `amount` from `debtor` to `creditor`.
const fn debt(amount: i64, debtor: MemberId, creditor: MemberId) -> Debt {
    Debt {
        amount,
        members: MemberPair {
            from: debtor,
            to: creditor,
        },
    }
}

/// A single payer is owed every other participant's share, and an even
/// split's leftover cent lands on exactly one member.
#[test]
fn a_single_payer_is_owed_every_other_share() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let taxi = Expense::builder(
        eur(1000),
        day(),
        "Taxi",
        ana,
        Split::equal([ana, bob, carol]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&taxi, entry(1)), Currency::EUR).unwrap();

    // 10.00 among three is 3.34 + 3.33 + 3.33.
    let consumed: Vec<i64> = rows(&resolved).iter().map(|&(_, _, share)| share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 1000);
    assert_eq!(consumed.iter().filter(|&&share| 334 == share).count(), 1);
    // Ana paid everything and is owed the other two shares.
    assert_eq!(rows(&resolved)[0].1, 1000);
    assert_eq!(resolved.debts.len(), 2);
    assert!(resolved.debts.iter().all(|debt| debt.members.to == ana));
    assert_eq!(debt_nets(&resolved)[&ana], 1000 - rows(&resolved)[0].2);
    assert_eq!(resolved.shares.total(), eur(1000));
    assert_eq!(resolved.shares.tip(), eur(0));
}

/// A proportional tip follows consumption: whoever ordered twice as much
/// tips twice as much.
#[test]
fn a_proportional_tip_follows_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder(
        eur(9000),
        day(),
        "Dinner",
        ana,
        Split::parts([
            MemberParts::new(ana, Parts::from_whole(2).unwrap()),
            MemberParts::new(bob, Parts::ONE),
        ]),
    )
    .tip(Tip::percentage(Percentage::parse("10").unwrap()))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&dinner, entry(1)), Currency::EUR).unwrap();

    // 60.00 + 6.00 and 30.00 + 3.00; Ana paid the 99.00 total.
    assert_eq!(rows(&resolved), vec![(ana, 9900, 6600), (bob, 0, 3300)]);
    assert_eq!(resolved.shares.tip(), eur(900));
}

/// An equal tip ignores consumption: the 9.00 tip is 4.50 each.
#[test]
fn an_equal_tip_ignores_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder(
        eur(9000),
        day(),
        "Dinner",
        ana,
        Split::parts([
            MemberParts::new(ana, Parts::from_whole(2).unwrap()),
            MemberParts::new(bob, Parts::ONE),
        ]),
    )
    .tip(Tip::percentage(Percentage::parse("10").unwrap()).distributed(TipDistribution::Equal))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&dinner, entry(1)), Currency::EUR).unwrap();

    assert_eq!(rows(&resolved), vec![(ana, 9900, 6450), (bob, 0, 3450)]);
}

/// Items divide by their own splits and the tip follows the item subtotals,
/// so a member pays only for what they had.
#[test]
fn items_divide_by_their_own_splits() {
    let (ana, bob) = (member(1), member(2));
    let pizza = Item::builder([ana], "Pizza", eur(1200)).build().unwrap();
    let beer = Item::builder([ana, bob], "Beer", eur(450))
        .quantity(2)
        .build()
        .unwrap();
    let bill = Expense::itemised_builder(eur(2100), day(), [pizza, beer], "Bill", bob)
        .tip(Tip::fixed(eur(210)))
        .build()
        .unwrap();
    let resolved = resolve_expense(&identified(&bill, entry(1)), Currency::EUR).unwrap();

    // Ana: 12.00 + 4.50 = 16.50 plus 16.50/21 of the 2.10 tip (1.65).
    // Bob: 4.50 plus 0.45 of tip; Bob paid the 23.10 total.
    assert_eq!(rows(&resolved), vec![(ana, 0, 1815), (bob, 2310, 495)]);
    assert_eq!(resolved.debts, vec![debt(1815, ana, bob)]);
}

/// With several payers every participant owes every payer in proportion to
/// what that payer covered, and the debts still net to paid minus share.
#[test]
fn several_payers_are_each_owed_their_proportion() {
    let (ana, bob, carol, dan) = (member(1), member(2), member(3), member(4));
    let hotel = Expense::builder_with_payers(
        eur(10_000),
        day(),
        "Hotel",
        Split::amounts([
            MemberAmount::new(eur(6000), ana),
            MemberAmount::new(eur(4000), bob),
        ]),
        Split::equal([ana, bob, carol, dan]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&hotel, entry(1)), Currency::EUR).unwrap();

    // Ana's 60.00 covers 15.00 of each share, Bob's 40.00 the other 10.00.
    assert_eq!(
        resolved.debts,
        vec![
            debt(1500, bob, ana),
            debt(1500, carol, ana),
            debt(1500, dan, ana),
            debt(1000, ana, bob),
            debt(1000, carol, bob),
            debt(1000, dan, bob),
        ]
    );
    // Every member's debts net to what they paid minus what they consumed.
    let nets = debt_nets(&resolved);
    for (member, paid, share) in rows(&resolved) {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), paid - share);
    }
}

/// A foreign expense converts its total once and re-divides it, so the
/// converted shares still add up to the converted total.
#[test]
fn a_foreign_expense_converts_its_total_once() {
    let (ana, bob) = (member(1), member(2));
    let museum = Expense::builder(
        eur(1000).at(ExchangeRate::parse("4.9765").unwrap()),
        day(),
        "Museum",
        ana,
        Split::equal([ana, bob]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&museum, entry(1)), Currency::RON).unwrap();

    // 10.00 EUR × 4.9765 = 49.765 RON, rounded half up to 49.77.
    assert_eq!(
        resolved.shares.total(),
        Money::from_minor_units(Currency::RON, 4977)
    );
    let consumed: Vec<i64> = rows(&resolved).iter().map(|&(_, _, share)| share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 4977);
    assert!(consumed.iter().all(|&share| 2488 == share || 2489 == share));
    assert_eq!(rows(&resolved)[0].1, 4977);
}

/// A converted tip is carved out of the converted total, so tip plus amount
/// still equals the total in the group currency.
#[test]
fn a_foreign_tip_is_part_of_the_converted_total() {
    let ana = member(1);
    let lunch = Expense::builder(
        Money::from_minor_units(Currency::JPY, 3000).at(ExchangeRate::parse("0.0061").unwrap()),
        day(),
        "Lunch",
        ana,
        Split::equal([ana]),
    )
    .tip(Tip::fixed(Money::from_minor_units(Currency::JPY, 300)))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&lunch, entry(1)), Currency::EUR).unwrap();

    // 3 300 JPY × 0.0061 = 20.13 EUR, of which one eleventh is tip.
    assert_eq!(resolved.shares.total(), eur(2013));
    assert_eq!(resolved.shares.tip(), eur(183));
}

/// A currency pegged at par, such as the Bermudian dollar to the US dollar,
/// would be converted at another rate.
#[test]
fn a_conversion_at_one_converts_a_pegged_currency() {
    let ana = member(1);
    let bermudian_dollars = Money::from_minor_units(Currency::BMD, 5_000);
    // At 1, the amount is taken as is.
    let pegged = Expense::builder(
        bermudian_dollars.at(ExchangeRate::ONE),
        day(),
        "Ferry",
        ana,
        Split::equal([ana]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&pegged, entry(1)), Currency::USD).unwrap();
    assert_eq!(
        resolved.shares.total(),
        Money::from_minor_units(Currency::USD, 5_000)
    );
}

/// A conversion beyond the range of an amount is refused instead of
/// wrapped.
#[test]
fn a_conversion_out_of_range_is_an_overflow() {
    let ana = member(1);
    let huge = Expense::builder(
        Money::from_minor_units(Currency::JPY, i64::MAX / 2)
            .at(ExchangeRate::parse("1000").unwrap()),
        day(),
        "Huge",
        ana,
        Split::equal([ana]),
    )
    .build()
    .unwrap();
    assert_eq!(
        resolve_expense(&identified(&huge, entry(1)), Currency::EUR),
        Err(MoneyError::Overflow)
    );
}

/// A payer whose converted part rounds to nothing is owed nothing, instead
/// of being handed debts that do not add up.
#[test]
fn a_payer_converted_to_nothing_is_owed_nothing() {
    let (ana, bob) = (member(1), member(2));
    // 2 JPY paid 1 + 1 at 0.0061 EUR per JPY is 1 cent in total.
    let sweets = Expense::builder_with_payers(
        Money::from_minor_units(Currency::JPY, 2).at(ExchangeRate::parse("0.0061").unwrap()),
        day(),
        "Sweets",
        Split::equal([ana, bob]),
        Split::equal([ana, bob]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&sweets, entry(1)), Currency::EUR).unwrap();
    assert_eq!(resolved.shares.total(), eur(1));
    let paid: Vec<i64> = rows(&resolved).iter().map(|&(_, paid, _)| paid).collect();
    assert_eq!(paid.iter().sum::<i64>(), 1);
    assert!(paid.contains(&0));
    let nets = debt_nets(&resolved);
    for (member, payment, share) in rows(&resolved) {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), payment - share);
    }
}

/// The leftover cent of an even split moves with the expense ID, so over
/// many expenses nobody always pays it; the same ID always resolves the
/// same way.
#[test]
fn the_leftover_cent_rotates_with_the_expense_id() {
    let members = [member(1), member(2), member(3)];
    let expense = Expense::builder(eur(100), day(), "Coffee", members[0], Split::equal(members))
        .build()
        .unwrap();
    let carrier = |id: ExpenseId| {
        let resolved = resolve_expense(&identified(&expense, id), Currency::EUR).unwrap();
        rows(&resolved)
            .iter()
            .find(|&&(_, _, share)| 34 == share)
            .map(|&(member, _, _)| member)
            .unwrap()
    };

    // Deterministic for one ID.
    assert_eq!(carrier(entry(7)), carrier(entry(7)));
    // Different IDs hand the cent to different members.
    let carriers: BTreeSet<MemberId> = (0..32).map(|number| carrier(entry(number))).collect();
    assert_eq!(carriers.len(), 3);
}

/// Builds a random valid expense from primitive draws: participants and
/// payers by parts, an optional tip, and an optional conversion from JPY.
fn random_expense(
    amount: i64,
    participant_parts: &[u32],
    payer_parts: &[u32],
    tip_basis_points: Option<u32>,
    equal_tip: bool,
    foreign: Option<(u64, u8)>,
) -> Expense {
    let parts = |weights: &[u32]| {
        Split::parts(weights.iter().zip(1_u8..).map(|(&hundredths, number)| {
            MemberParts::new(member(number), Parts::from_hundredths(hundredths).unwrap())
        }))
    };
    let cost = match foreign {
        None => Cost::from(eur(amount)),
        Some((mantissa, scale)) => Money::from_minor_units(Currency::JPY, amount)
            .at(ExchangeRate::from_decimal(mantissa, scale).unwrap()),
    };
    let mut builder = Expense::builder_with_payers(
        cost,
        day(),
        "Random",
        parts(payer_parts),
        parts(participant_parts),
    );
    if let Some(basis_points) = tip_basis_points {
        let distribution = if equal_tip {
            TipDistribution::Equal
        } else {
            TipDistribution::Proportional
        };
        builder = builder.tip(
            Tip::percentage(Percentage::from_basis_points(basis_points)).distributed(distribution),
        );
    }
    builder.build().unwrap()
}

proptest! {
    /// Paid and consumed amounts must both add up to the expense total, or
    /// the group's balances would no longer sum to zero.
    #[test]
    fn paid_and_shared_both_add_up_to_the_total(
        amount in 1_i64..10_000_000,
        participant_parts in proptest::collection::vec(1_u32..1_000, 1..6),
        payer_parts in proptest::collection::vec(1_u32..1_000, 1..4),
        tip_basis_points in proptest::option::of(0_u32..3_000),
        equal_tip in any::<bool>(),
        foreign in proptest::option::of((1_u64..100_000, 0_u8..6)),
        id in any::<[u8; 16]>(),
    ) {
        let expense = random_expense(amount, &participant_parts, &payer_parts, tip_basis_points, equal_tip, foreign);
        let resolved = resolve_expense(&identified(&expense, ExpenseId::from_bytes(id)), Currency::EUR).unwrap();
        let total = resolved.shares.total().minor_units();
        prop_assert_eq!(rows(&resolved).iter().map(|&(_, paid, _)| paid).sum::<i64>(), total);
        prop_assert_eq!(rows(&resolved).iter().map(|&(_, _, share)| share).sum::<i64>(), total);
        prop_assert!(resolved.shares.tip().minor_units() <= total);
        if foreign.is_none() {
            prop_assert_eq!(resolved.shares.total(), expense.total());
        }
    }

    /// The pairwise debts behind the direct settlement must net, for every
    /// member, to what they paid minus what they consumed; otherwise the
    /// direct settlement would not clear the balances.
    #[test]
    fn pairwise_debts_net_to_paid_minus_share(
        amount in 1_i64..10_000_000,
        participant_parts in proptest::collection::vec(1_u32..1_000, 1..6),
        payer_parts in proptest::collection::vec(1_u32..1_000, 1..4),
        tip_basis_points in proptest::option::of(0_u32..3_000),
        foreign in proptest::option::of((1_u64..100_000, 0_u8..6)),
        id in any::<[u8; 16]>(),
    ) {
        let expense = random_expense(amount, &participant_parts, &payer_parts, tip_basis_points, false, foreign);
        let resolved = resolve_expense(&identified(&expense, ExpenseId::from_bytes(id)), Currency::EUR).unwrap();
        let nets = debt_nets(&resolved);
        for (member, paid, share) in rows(&resolved) {
            prop_assert_eq!(nets.get(&member).copied().unwrap_or(0), paid - share);
        }
        prop_assert!(resolved.debts.iter().all(|debt| 0 < debt.amount && debt.members.from != debt.members.to));
    }
}

/// The tie rotation of every pass is part of the stored numbers: two
/// releases, or two devices, must hand the same leftover cents to the same
/// members for the same expense, so these outcomes are pinned.
#[test]
fn the_rotation_of_every_pass_is_pinned() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    // Three items with a leftover cent or two each, an equal tip of 2 cents
    // among three, and 3.03 paid by two: every pass has ties to break.
    let items = [100, 100, 101].map(|price| {
        Item::builder([ana, bob, carol], "Item", eur(price))
            .build()
            .unwrap()
    });
    let itemised = Expense::itemised_builder_with_payers(
        eur(301),
        day(),
        items,
        "Pinned",
        Split::equal([ana, bob]),
    )
    .tip(Tip::fixed(eur(2)).distributed(TipDistribution::Equal))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&itemised, entry(1)), Currency::EUR).unwrap();
    assert_eq!(
        rows(&resolved),
        vec![(ana, 151, 101), (bob, 152, 100), (carol, 0, 102)]
    );
    assert_eq!(
        resolved.debts,
        vec![
            debt(50, bob, ana),
            debt(51, carol, ana),
            debt(51, ana, bob),
            debt(51, carol, bob)
        ]
    );

    // A conversion's re-division: 300 JPY at 0.00673 is 2.02 EUR among three.
    let foreign = Expense::builder(
        Money::from_minor_units(Currency::JPY, 300).at(ExchangeRate::parse("0.00673").unwrap()),
        day(),
        "Pinned",
        ana,
        Split::equal([ana, bob, carol]),
    )
    .build()
    .unwrap();
    let converted = resolve_expense(&identified(&foreign, entry(2)), Currency::EUR).unwrap();
    assert_eq!(
        rows(&converted),
        vec![(ana, 202, 67), (bob, 0, 68), (carol, 0, 67)]
    );

    // Pair debts: three payers over four equal shares, so both the first and
    // the second payer's division have ties.
    let dan = member(4);
    let hotel = Expense::builder_with_payers(
        eur(400),
        day(),
        "Pinned",
        Split::amounts([
            MemberAmount::new(eur(133), ana),
            MemberAmount::new(eur(133), bob),
            MemberAmount::new(eur(134), carol),
        ]),
        Split::equal([ana, bob, carol, dan]),
    )
    .build()
    .unwrap();
    assert_eq!(
        resolve_expense(&identified(&hotel, entry(3)), Currency::EUR)
            .unwrap()
            .debts,
        vec![
            debt(33, bob, ana),
            debt(33, carol, ana),
            debt(34, dan, ana),
            debt(33, ana, bob),
            debt(34, carol, bob),
            debt(33, dan, bob),
            debt(34, ana, carol),
            debt(34, bob, carol),
            debt(33, dan, carol),
        ]
    );
}

/// A portion that rounds to nothing is no debt: a zero debt would add a
/// zero payment to the direct settlement.
#[test]
fn a_zero_portion_is_no_debt() {
    let (ana, bob, carol, dan) = (member(1), member(2), member(3), member(4));
    // Ana's 99 cents cover all of Dan's single cent, so Bob's cent is
    // Carol's alone.
    let snack = Expense::builder_with_payers(
        eur(100),
        day(),
        "Snack",
        Split::amounts([
            MemberAmount::new(eur(99), ana),
            MemberAmount::new(eur(1), bob),
        ]),
        Split::amounts([
            MemberAmount::new(eur(99), carol),
            MemberAmount::new(eur(1), dan),
        ]),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&snack, entry(1)), Currency::EUR).unwrap();
    assert_eq!(
        resolved.debts,
        vec![debt(98, carol, ana), debt(1, dan, ana), debt(1, carol, bob)]
    );
}
