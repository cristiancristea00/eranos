//! Tests for resolving entries into group-currency shares. These numbers are
//! what every balance and settlement is built from: a share that does not add
//! up, a tip split the wrong way, a conversion rounded differently, or a
//! pairwise debt that disagrees with the net balances would all show up as
//! wrong amounts owed. The cases are worked out by hand; the properties
//! cover random splits, payers, tips, and conversions.

use std::collections::BTreeSet;

use proptest::prelude::*;

use super::*;
use crate::date::Date;
use crate::expense::{Item, Split, Tip, TipDistribution};
use crate::money::{Parts, Percentage};

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

fn entry(number: u8) -> ExpenseId {
    ExpenseId::from_bytes([number; 16])
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn day() -> Date {
    Date::new(2026, 9, 23).unwrap()
}

/// (member, paid, share) in minor units, ordered by member.
fn rows(shares: &ExpenseShares) -> Vec<(MemberId, i64, i64)> {
    shares
        .members
        .iter()
        .map(|share| {
            (
                share.member,
                share.paid.minor_units(),
                share.share.minor_units(),
            )
        })
        .collect()
}

/// What each member is owed minus what they owe, from the pairwise debts.
fn debt_nets(shares: &ExpenseShares) -> BTreeMap<MemberId, i64> {
    let mut nets = BTreeMap::new();
    for debt in &shares.debts {
        *nets.entry(debt.creditor).or_insert(0) += debt.amount;
        *nets.entry(debt.debtor).or_insert(0) -= debt.amount;
    }
    nets
}

/// A single payer is owed every other participant's share, and an even
/// split's leftover cent lands on exactly one member.
#[test]
fn a_single_payer_is_owed_every_other_share() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let taxi = Expense::builder("Taxi", eur(1000), day())
        .paid_by(ana)
        .split(Split::equal([ana, bob, carol]))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &taxi, Currency::EUR).unwrap();

    // 10.00 among three is 3.34 + 3.33 + 3.33.
    let consumed: Vec<i64> = rows(&shares).iter().map(|&(_, _, share)| share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 1000);
    assert_eq!(consumed.iter().filter(|&&share| share == 334).count(), 1);
    // Ana paid everything and is owed the other two shares.
    assert_eq!(rows(&shares)[0].1, 1000);
    assert_eq!(shares.debts.len(), 2);
    assert!(shares.debts.iter().all(|debt| debt.creditor == ana));
    assert_eq!(debt_nets(&shares)[&ana], 1000 - rows(&shares)[0].2);
    assert_eq!(shares.total, eur(1000));
    assert_eq!(shares.tip, eur(0));
}

/// A proportional tip follows consumption: whoever ordered twice as much
/// tips twice as much.
#[test]
fn a_proportional_tip_follows_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder("Dinner", eur(9000), day())
        .paid_by(ana)
        .split(Split::parts([
            (ana, Parts::from_whole(2).unwrap()),
            (bob, Parts::ONE),
        ]))
        .tip(Tip::percentage(Percentage::parse("10").unwrap()))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &dinner, Currency::EUR).unwrap();

    // 60.00 + 6.00 and 30.00 + 3.00; Ana paid the 99.00 total.
    assert_eq!(rows(&shares), vec![(ana, 9900, 6600), (bob, 0, 3300)]);
    assert_eq!(shares.tip, eur(900));
}

/// An equal tip ignores consumption: the 9.00 tip is 4.50 each.
#[test]
fn an_equal_tip_ignores_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder("Dinner", eur(9000), day())
        .paid_by(ana)
        .split(Split::parts([
            (ana, Parts::from_whole(2).unwrap()),
            (bob, Parts::ONE),
        ]))
        .tip(Tip::percentage(Percentage::parse("10").unwrap()).distributed(TipDistribution::Equal))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &dinner, Currency::EUR).unwrap();

    assert_eq!(rows(&shares), vec![(ana, 9900, 6450), (bob, 0, 3450)]);
}

/// Items divide by their own splits and the tip follows the item subtotals,
/// so a member pays only for what they had.
#[test]
fn items_divide_by_their_own_splits() {
    let (ana, bob) = (member(1), member(2));
    let pizza = Item::builder("Pizza", eur(1200))
        .shared_by([ana])
        .build()
        .unwrap();
    let beer = Item::builder("Beer", eur(450))
        .quantity(2)
        .shared_by([ana, bob])
        .build()
        .unwrap();
    let bill = Expense::builder("Bill", eur(2100), day())
        .paid_by(bob)
        .items([pizza, beer])
        .tip(Tip::fixed(eur(210)))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &bill, Currency::EUR).unwrap();

    // Ana: 12.00 + 4.50 = 16.50 plus 16.50/21 of the 2.10 tip (1.65).
    // Bob: 4.50 plus 0.45 of tip; Bob paid the 23.10 total.
    assert_eq!(rows(&shares), vec![(ana, 0, 1815), (bob, 2310, 495)]);
    assert_eq!(shares.debts.len(), 1);
    assert_eq!(
        (
            shares.debts[0].debtor,
            shares.debts[0].creditor,
            shares.debts[0].amount
        ),
        (ana, bob, 1815)
    );
}

/// With several payers every participant owes every payer in proportion to
/// what that payer covered, and the debts still net to paid minus share.
#[test]
fn several_payers_are_each_owed_their_proportion() {
    let (ana, bob, carol, dan) = (member(1), member(2), member(3), member(4));
    let hotel = Expense::builder("Hotel", eur(10_000), day())
        .payers(Split::amounts([(ana, eur(6000)), (bob, eur(4000))]))
        .split(Split::equal([ana, bob, carol, dan]))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &hotel, Currency::EUR).unwrap();

    // Ana's 60.00 covers 15.00 of each share, Bob's 40.00 the other 10.00.
    let debts: Vec<(MemberId, MemberId, i64)> = shares
        .debts
        .iter()
        .map(|debt| (debt.debtor, debt.creditor, debt.amount))
        .collect();
    assert_eq!(
        debts,
        vec![
            (bob, ana, 1500),
            (carol, ana, 1500),
            (dan, ana, 1500),
            (ana, bob, 1000),
            (carol, bob, 1000),
            (dan, bob, 1000),
        ]
    );
    // Every member's debts net to what they paid minus what they consumed.
    let nets = debt_nets(&shares);
    for (member, paid, share) in rows(&shares) {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), paid - share);
    }
}

/// A foreign expense converts its total once and re-divides it, so the
/// converted shares still add up to the converted total.
#[test]
fn a_foreign_expense_converts_its_total_once() {
    let (ana, bob) = (member(1), member(2));
    let museum = Expense::builder("Museum", eur(1000), day())
        .paid_by(ana)
        .split(Split::equal([ana, bob]))
        .exchange_rate(ExchangeRate::parse("4.9765").unwrap())
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &museum, Currency::RON).unwrap();

    // 10.00 EUR × 4.9765 = 49.765 RON, rounded half up to 49.77.
    assert_eq!(shares.total, Money::from_minor_units(4977, Currency::RON));
    let consumed: Vec<i64> = rows(&shares).iter().map(|&(_, _, share)| share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 4977);
    assert!(consumed.iter().all(|&share| share == 2488 || share == 2489));
    assert_eq!(rows(&shares)[0].1, 4977);
}

/// A converted tip is carved out of the converted total, so tip plus amount
/// still equals the total in the group currency.
#[test]
fn a_foreign_tip_is_part_of_the_converted_total() {
    let ana = member(1);
    let lunch = Expense::builder("Lunch", Money::from_minor_units(3000, Currency::JPY), day())
        .paid_by(ana)
        .split(Split::equal([ana]))
        .tip(Tip::fixed(Money::from_minor_units(300, Currency::JPY)))
        .exchange_rate(ExchangeRate::parse("0.0061").unwrap())
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &lunch, Currency::EUR).unwrap();

    // 3 300 JPY × 0.0061 = 20.13 EUR, of which one eleventh is tip.
    assert_eq!(shares.total, eur(2013));
    assert_eq!(shares.tip, eur(183));
}

/// Rates are checked against the currencies: a foreign entry without one
/// cannot be totalled, and a rate on a group-currency entry is a mistake.
#[test]
fn a_rate_must_match_the_currencies() {
    let ana = member(1);
    let builder = || {
        Expense::builder("Snack", eur(500), day())
            .paid_by(ana)
            .split(Split::equal([ana]))
    };

    // Foreign without a rate.
    let foreign = builder().build().unwrap();
    assert_eq!(
        resolve_expense(entry(1), &foreign, Currency::RON),
        Err(GroupError::MissingExchangeRate {
            currency: Currency::EUR
        })
    );
    // Group currency with a rate.
    let rated = builder()
        .exchange_rate(ExchangeRate::parse("1").unwrap())
        .build()
        .unwrap();
    assert_eq!(
        resolve_expense(entry(1), &rated, Currency::EUR),
        Err(GroupError::UnexpectedExchangeRate)
    );
}

/// A conversion beyond the range of an amount is refused instead of
/// wrapped.
#[test]
fn a_conversion_out_of_range_is_an_overflow() {
    let ana = member(1);
    let huge = Expense::builder(
        "Huge",
        Money::from_minor_units(i64::MAX / 2, Currency::JPY),
        day(),
    )
    .paid_by(ana)
    .split(Split::equal([ana]))
    .exchange_rate(ExchangeRate::parse("1000").unwrap())
    .build()
    .unwrap();
    assert_eq!(
        resolve_expense(entry(1), &huge, Currency::EUR),
        Err(GroupError::Overflow)
    );
}

/// A payer whose converted part rounds to nothing is owed nothing, instead
/// of being handed debts that do not add up.
#[test]
fn a_payer_converted_to_nothing_is_owed_nothing() {
    let (ana, bob) = (member(1), member(2));
    // 2 JPY paid 1 + 1 at 0.0061 EUR per JPY is 1 cent in total.
    let sweets = Expense::builder("Sweets", Money::from_minor_units(2, Currency::JPY), day())
        .payers(Split::equal([ana, bob]))
        .split(Split::equal([ana, bob]))
        .exchange_rate(ExchangeRate::parse("0.0061").unwrap())
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &sweets, Currency::EUR).unwrap();
    assert_eq!(shares.total, eur(1));
    let paid: Vec<i64> = rows(&shares).iter().map(|&(_, paid, _)| paid).collect();
    assert_eq!(paid.iter().sum::<i64>(), 1);
    assert!(paid.contains(&0));
    let nets = debt_nets(&shares);
    for (member, payment, share) in rows(&shares) {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), payment - share);
    }
}

/// The leftover cent of an even split moves with the expense ID, so over
/// many expenses nobody always pays it; the same ID always resolves the
/// same way.
#[test]
fn the_leftover_cent_rotates_with_the_expense_id() {
    let members = [member(1), member(2), member(3)];
    let expense = Expense::builder("Coffee", eur(100), day())
        .paid_by(members[0])
        .split(Split::equal(members))
        .build()
        .unwrap();
    let carrier = |id: ExpenseId| {
        let shares = resolve_expense(id, &expense, Currency::EUR).unwrap();
        rows(&shares)
            .iter()
            .find(|&&(_, _, share)| share == 34)
            .map(|&(member, _, _)| member)
            .unwrap()
    };

    // Deterministic for one ID.
    assert_eq!(carrier(entry(7)), carrier(entry(7)));
    // Different IDs hand the cent to different members.
    let carriers: BTreeSet<MemberId> = (0..32).map(|number| carrier(entry(number))).collect();
    assert_eq!(carriers.len(), 3);
}

/// A transfer converts like an expense and obeys the same rate rules.
#[test]
fn a_transfer_converts_like_an_expense() {
    let (ana, bob) = (member(1), member(2));
    // In the group currency, the amount is taken as is.
    let local = Transfer::builder(ana, bob, eur(2500), day())
        .build()
        .unwrap();
    assert_eq!(resolve_transfer(&local, Currency::EUR), Ok(2500));
    // In a foreign currency, it needs and uses the rate.
    let cash = Transfer::builder(
        ana,
        bob,
        Money::from_minor_units(10_000, Currency::RON),
        day(),
    )
    .exchange_rate(ExchangeRate::parse("0.2011").unwrap())
    .build()
    .unwrap();
    assert_eq!(resolve_transfer(&cash, Currency::EUR), Ok(2011));
    let unrated = Transfer::builder(
        ana,
        bob,
        Money::from_minor_units(10_000, Currency::RON),
        day(),
    )
    .build()
    .unwrap();
    assert_eq!(
        resolve_transfer(&unrated, Currency::EUR),
        Err(GroupError::MissingExchangeRate {
            currency: Currency::RON
        })
    );
    let rated = local
        .to_builder()
        .exchange_rate(ExchangeRate::parse("2").unwrap())
        .build()
        .unwrap();
    assert_eq!(
        resolve_transfer(&rated, Currency::EUR),
        Err(GroupError::UnexpectedExchangeRate)
    );
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
            (member(number), Parts::from_hundredths(hundredths).unwrap())
        }))
    };
    let currency = if foreign.is_some() {
        Currency::JPY
    } else {
        Currency::EUR
    };
    let mut builder = Expense::builder("Random", Money::from_minor_units(amount, currency), day())
        .payers(parts(payer_parts))
        .split(parts(participant_parts));
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
    if let Some((mantissa, scale)) = foreign {
        builder = builder.exchange_rate(ExchangeRate::from_decimal(mantissa, scale).unwrap());
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
        let shares = resolve_expense(ExpenseId::from_bytes(id), &expense, Currency::EUR).unwrap();
        let total = shares.total.minor_units();
        prop_assert_eq!(rows(&shares).iter().map(|&(_, paid, _)| paid).sum::<i64>(), total);
        prop_assert_eq!(rows(&shares).iter().map(|&(_, _, share)| share).sum::<i64>(), total);
        prop_assert!(shares.tip.minor_units() <= total);
        if foreign.is_none() {
            prop_assert_eq!(shares.total, expense.total());
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
        let shares = resolve_expense(ExpenseId::from_bytes(id), &expense, Currency::EUR).unwrap();
        let nets = debt_nets(&shares);
        for (member, paid, share) in rows(&shares) {
            prop_assert_eq!(nets.get(&member).copied().unwrap_or(0), paid - share);
        }
        prop_assert!(shares.debts.iter().all(|debt| debt.amount > 0 && debt.debtor != debt.creditor));
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
        Item::builder("Item", eur(price))
            .shared_by([ana, bob, carol])
            .build()
            .unwrap()
    });
    let itemised = Expense::builder("Pinned", eur(301), day())
        .payers(Split::equal([ana, bob]))
        .items(items)
        .tip(Tip::fixed(eur(2)).distributed(TipDistribution::Equal))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &itemised, Currency::EUR).unwrap();
    assert_eq!(
        rows(&shares),
        vec![(ana, 151, 101), (bob, 152, 100), (carol, 0, 102)]
    );
    let debts: Vec<(MemberId, MemberId, i64)> = shares
        .debts
        .iter()
        .map(|debt| (debt.debtor, debt.creditor, debt.amount))
        .collect();
    assert_eq!(
        debts,
        vec![
            (bob, ana, 50),
            (carol, ana, 51),
            (ana, bob, 51),
            (carol, bob, 51)
        ]
    );

    // A conversion's re-division: 300 JPY at 0.00673 is 2.02 EUR among three.
    let foreign = Expense::builder("Pinned", Money::from_minor_units(300, Currency::JPY), day())
        .paid_by(ana)
        .split(Split::equal([ana, bob, carol]))
        .exchange_rate(ExchangeRate::parse("0.00673").unwrap())
        .build()
        .unwrap();
    let converted = resolve_expense(entry(2), &foreign, Currency::EUR).unwrap();
    assert_eq!(
        rows(&converted),
        vec![(ana, 202, 67), (bob, 0, 68), (carol, 0, 67)]
    );

    // Pair debts: three payers over four equal shares, so both the first and
    // the second payer's division have ties.
    let dan = member(4);
    let hotel = Expense::builder("Pinned", eur(400), day())
        .payers(Split::amounts([
            (ana, eur(133)),
            (bob, eur(133)),
            (carol, eur(134)),
        ]))
        .split(Split::equal([ana, bob, carol, dan]))
        .build()
        .unwrap();
    let hotel_debts: Vec<(MemberId, MemberId, i64)> =
        resolve_expense(entry(3), &hotel, Currency::EUR)
            .unwrap()
            .debts
            .iter()
            .map(|debt| (debt.debtor, debt.creditor, debt.amount))
            .collect();
    assert_eq!(
        hotel_debts,
        vec![
            (bob, ana, 33),
            (carol, ana, 33),
            (dan, ana, 34),
            (ana, bob, 33),
            (carol, bob, 34),
            (dan, bob, 33),
            (ana, carol, 34),
            (bob, carol, 34),
            (dan, carol, 33),
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
    let snack = Expense::builder("Snack", eur(100), day())
        .payers(Split::amounts([(ana, eur(99)), (bob, eur(1))]))
        .split(Split::amounts([(carol, eur(99)), (dan, eur(1))]))
        .build()
        .unwrap();
    let shares = resolve_expense(entry(1), &snack, Currency::EUR).unwrap();
    let debts: Vec<(MemberId, MemberId, i64)> = shares
        .debts
        .iter()
        .map(|debt| (debt.debtor, debt.creditor, debt.amount))
        .collect();
    assert_eq!(
        debts,
        vec![(carol, ana, 98), (dan, ana, 1), (carol, bob, 1)]
    );
}
