//! Tests for resolving entries into group-currency shares. These numbers are
//! what every balance and settlement is built from: a share that does not add
//! up, a tip split the wrong way, a conversion rounded differently, or a
//! pairwise debt that disagrees with the net balances would all show up as
//! wrong amounts owed. The cases are worked out by hand; the properties
//! cover random splits, payers, tips, and conversions.

use std::collections::BTreeSet;

use eranos_calendar::CalendarDate;
use eranos_core::ExpenseId;
use eranos_money::{Cost, Parts, Percentage, Quantity};
use proptest::prelude::*;

use super::*;
use crate::{ExpenseItem, MemberAmount, MemberParts, Split, TipDistribution, TipRule};

/// Identifier text of one to 64 letters, digits, and hyphens.
const IDENTIFIER: &str = "[A-Za-z0-9-]{1,64}";

/// `expense` under the identifier `id`, which seeds its tie rotation.
fn identified(expense: &Expense, id: ExpenseId) -> Expense {
    expense.to_builder().id(id).build().unwrap()
}

/// The member `number`; members sort by number.
fn member(number: u8) -> MemberId {
    MemberId::new(&format!("member-{number:03}")).unwrap()
}

/// The expense `number`, whose identifier seeds its tie rotation.
fn entry(number: u8) -> ExpenseId {
    ExpenseId::new(&format!("expense-{number:03}")).unwrap()
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn day() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

/// What one member paid and consumed of a resolved expense, in minor
/// units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row {
    member: MemberId,
    paid: i64,
    share: i64,
}

/// The row of `member`, who paid `paid` and consumed `share`.
const fn row(member: MemberId, paid: i64, share: i64) -> Row {
    Row {
        member,
        paid,
        share,
    }
}

/// Every member's row, ordered by member.
fn rows(resolved: &ResolvedExpense) -> Vec<Row> {
    resolved
        .shares
        .members()
        .iter()
        .map(|share| {
            row(
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
const fn debt(debtor: MemberId, creditor: MemberId, amount: i64) -> Debt {
    Debt {
        members: MemberPair {
            from: debtor,
            to: creditor,
        },
        amount,
    }
}

/// A single payer is owed every other participant's share, and an even
/// split's leftover cent lands on exactly one member.
#[test]
fn a_single_payer_is_owed_every_other_share() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let taxi = Expense::builder(
        "Taxi",
        eur(1000),
        ana,
        Split::equal([ana, bob, carol]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&taxi, entry(1)), CurrencyCode::EUR).unwrap();

    // 10.00 among three is 3.34 + 3.33 + 3.33.
    let consumed: Vec<i64> = rows(&resolved).iter().map(|row| row.share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 1000);
    assert_eq!(consumed.iter().filter(|&&share| 334 == share).count(), 1);
    // Ana paid everything and is owed the other two shares.
    assert_eq!(rows(&resolved)[0].paid, 1000);
    assert_eq!(resolved.debts.len(), 2);
    assert!(resolved.debts.iter().all(|debt| debt.members.to == ana));
    assert_eq!(debt_nets(&resolved)[&ana], 1000 - rows(&resolved)[0].share);
    assert_eq!(resolved.shares.total(), eur(1000));
    assert_eq!(resolved.shares.tip(), eur(0));
}

/// A proportional tip follows consumption: whoever ordered twice as much
/// tips twice as much.
#[test]
fn a_proportional_tip_follows_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder(
        "Dinner",
        eur(9000),
        ana,
        Split::parts([
            MemberParts::new(ana, Parts::from_whole(2).unwrap()),
            MemberParts::new(bob, Parts::ONE),
        ]),
        day(),
    )
    .tip(TipRule::percentage(Percentage::parse("10").unwrap()))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&dinner, entry(1)), CurrencyCode::EUR).unwrap();

    // 60.00 + 6.00 and 30.00 + 3.00; Ana paid the 99.00 total.
    assert_eq!(
        rows(&resolved),
        vec![row(ana, 9900, 6600), row(bob, 0, 3300)]
    );
    assert_eq!(resolved.shares.tip(), eur(900));
}

/// An equal tip ignores consumption: the 9.00 tip is 4.50 each.
#[test]
fn an_equal_tip_ignores_the_shares() {
    let (ana, bob) = (member(1), member(2));
    let dinner = Expense::builder(
        "Dinner",
        eur(9000),
        ana,
        Split::parts([
            MemberParts::new(ana, Parts::from_whole(2).unwrap()),
            MemberParts::new(bob, Parts::ONE),
        ]),
        day(),
    )
    .tip(TipRule::percentage(Percentage::parse("10").unwrap()).distributed(TipDistribution::Equal))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&dinner, entry(1)), CurrencyCode::EUR).unwrap();

    assert_eq!(
        rows(&resolved),
        vec![row(ana, 9900, 6450), row(bob, 0, 3450)]
    );
}

/// Items divide by their own splits and the tip follows the item subtotals,
/// so a member pays only for what they had.
#[test]
fn items_divide_by_their_own_splits() {
    let (ana, bob) = (member(1), member(2));
    let pizza = ExpenseItem::builder("Pizza", eur(1200), [ana])
        .build()
        .unwrap();
    let beer = ExpenseItem::builder("Beer", eur(450), [ana, bob])
        .quantity(Quantity::from_whole(2).unwrap())
        .build()
        .unwrap();
    let bill = Expense::itemised_builder("Bill", eur(2100), bob, [pizza, beer], day())
        .tip(TipRule::fixed(eur(210)))
        .build()
        .unwrap();
    let resolved = resolve_expense(&identified(&bill, entry(1)), CurrencyCode::EUR).unwrap();

    // Ana: 12.00 + 4.50 = 16.50 plus 16.50/21 of the 2.10 tip (1.65).
    // Bob: 4.50 plus 0.45 of tip; Bob paid the 23.10 total.
    assert_eq!(
        rows(&resolved),
        vec![row(ana, 0, 1815), row(bob, 2310, 495)]
    );
    assert_eq!(resolved.debts, vec![debt(ana, bob, 1815)]);
}

/// An item of a fractional quantity divides its rounded line total, the
/// amount the expense's items add up to; dividing the unrounded product
/// would hand out a share of a cent that no line holds.
#[test]
fn a_fractional_item_divides_its_rounded_line_total() {
    let (ana, bob) = (member(1), member(2));
    // 0.335 kg at 1.00 a kilogram is 33.5 cents, a line of 0.34.
    let olives = ExpenseItem::builder("Olives", eur(100), [ana, bob])
        .quantity(Quantity::parse("0.335").unwrap())
        .build()
        .unwrap();
    let bread = ExpenseItem::builder("Bread", eur(166), [ana])
        .build()
        .unwrap();
    let bill = Expense::itemised_builder("Market", eur(200), bob, [olives, bread], day())
        .build()
        .unwrap();
    let resolved = resolve_expense(&identified(&bill, entry(1)), CurrencyCode::EUR).unwrap();

    // Ana: 1.66 + 0.17; Bob: 0.17, and he paid the 2.00.
    assert_eq!(rows(&resolved), vec![row(ana, 0, 183), row(bob, 200, 17)]);
}

/// A free line, whose total rounds to zero, divides to zeros: a member who
/// shares only it would be charged a cent the line does not cost, or be
/// dropped from the expense they took part in.
#[test]
fn a_free_line_divides_to_zeros() {
    let (ana, bob) = (member(1), member(2));
    // 0.4 of a cent rounds to zero.
    let mint = ExpenseItem::builder("Mint", eur(1), [ana, bob])
        .quantity(Quantity::parse("0.4").unwrap())
        .build()
        .unwrap();
    let bread = ExpenseItem::builder("Bread", eur(200), [ana])
        .build()
        .unwrap();
    let bill = Expense::itemised_builder("Market", eur(200), bob, [mint, bread], day())
        .tip(TipRule::fixed(eur(10)))
        .build()
        .unwrap();
    let resolved = resolve_expense(&identified(&bill, entry(1)), CurrencyCode::EUR).unwrap();

    // Ana consumed the bread and its whole proportional tip; Bob, who paid
    // the 2.10, consumed nothing.
    assert_eq!(rows(&resolved), vec![row(ana, 0, 210), row(bob, 210, 0)]);
    assert_eq!(resolved.debts, vec![debt(ana, bob, 210)]);
}

/// With several payers every participant owes every payer in proportion to
/// what that payer covered, and the debts still net to paid minus share.
#[test]
fn several_payers_are_each_owed_their_proportion() {
    let (ana, bob, carol, dan) = (member(1), member(2), member(3), member(4));
    let hotel = Expense::builder_with_payers(
        "Hotel",
        eur(10_000),
        Split::amounts([
            MemberAmount::new(ana, eur(6000)),
            MemberAmount::new(bob, eur(4000)),
        ]),
        Split::equal([ana, bob, carol, dan]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&hotel, entry(1)), CurrencyCode::EUR).unwrap();

    // Ana's 60.00 covers 15.00 of each share, Bob's 40.00 the other 10.00.
    assert_eq!(
        resolved.debts,
        vec![
            debt(bob, ana, 1500),
            debt(carol, ana, 1500),
            debt(dan, ana, 1500),
            debt(ana, bob, 1000),
            debt(carol, bob, 1000),
            debt(dan, bob, 1000),
        ]
    );
    // Every member's debts net to what they paid minus what they consumed.
    let nets = debt_nets(&resolved);
    for Row {
        member,
        paid,
        share,
    } in rows(&resolved)
    {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), paid - share);
    }
}

/// A foreign expense converts its total once and re-divides it, so the
/// converted shares still add up to the converted total.
#[test]
fn a_foreign_expense_converts_its_total_once() {
    let (ana, bob) = (member(1), member(2));
    let museum = Expense::builder(
        "Museum",
        eur(1000).at(ExchangeRate::parse("4.9765").unwrap()),
        ana,
        Split::equal([ana, bob]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&museum, entry(1)), CurrencyCode::RON).unwrap();

    // 10.00 EUR × 4.9765 = 49.765 RON, rounded half up to 49.77.
    assert_eq!(
        resolved.shares.total(),
        Money::from_minor_units(4977, CurrencyCode::RON)
    );
    let consumed: Vec<i64> = rows(&resolved).iter().map(|row| row.share).collect();
    assert_eq!(consumed.iter().sum::<i64>(), 4977);
    assert!(consumed.iter().all(|&share| 2488 == share || 2489 == share));
    assert_eq!(rows(&resolved)[0].paid, 4977);
}

/// A currency change resolves each expense at its new rate before the
/// expense takes it: the shares at a rate would otherwise differ from the
/// shares of the expense at that rate, and a group's totals from a rebuild.
#[test]
fn resolving_at_a_rate_resolves_the_expense_at_that_rate() {
    let (ana, bob) = (member(1), member(2));
    let rate = ExchangeRate::parse("4.9765").unwrap();
    let local = identified(
        &Expense::builder("Museum", eur(1000), ana, Split::equal([ana, bob]), day())
            .build()
            .unwrap(),
        entry(1),
    );
    let converted = local.clone().with_exchange_rate(Some(rate));
    assert_eq!(
        resolve_expense_at(&local, Some(rate), CurrencyCode::RON),
        resolve_expense(&converted, CurrencyCode::RON)
    );
    assert_eq!(
        resolve_expense_at(&converted, None, CurrencyCode::EUR),
        resolve_expense(&local, CurrencyCode::EUR)
    );
}

/// A converted tip is carved out of the converted total, so tip plus amount
/// still equals the total in the group currency.
#[test]
fn a_foreign_tip_is_part_of_the_converted_total() {
    let ana = member(1);
    let lunch = Expense::builder(
        "Lunch",
        Money::from_minor_units(3000, CurrencyCode::JPY).at(ExchangeRate::parse("0.0061").unwrap()),
        ana,
        Split::equal([ana]),
        day(),
    )
    .tip(TipRule::fixed(Money::from_minor_units(
        300,
        CurrencyCode::JPY,
    )))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&lunch, entry(1)), CurrencyCode::EUR).unwrap();

    // 3 300 JPY × 0.0061 = 20.13 EUR, of which one eleventh is tip.
    assert_eq!(resolved.shares.total(), eur(2013));
    assert_eq!(resolved.shares.tip(), eur(183));
}

/// A currency pegged at par, such as the Bermudian dollar to the US dollar,
/// would be converted at another rate.
#[test]
fn a_conversion_at_one_converts_a_pegged_currency() {
    let ana = member(1);
    let bermudian_dollars = Money::from_minor_units(5_000, CurrencyCode::BMD);
    // At 1, the amount is taken as is.
    let pegged = Expense::builder(
        "Ferry",
        bermudian_dollars.at(ExchangeRate::ONE),
        ana,
        Split::equal([ana]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&pegged, entry(1)), CurrencyCode::USD).unwrap();
    assert_eq!(
        resolved.shares.total(),
        Money::from_minor_units(5_000, CurrencyCode::USD)
    );
}

/// A conversion beyond the range of an amount is refused instead of
/// wrapped.
#[test]
fn a_conversion_out_of_range_is_an_overflow() {
    let ana = member(1);
    let huge = Expense::builder(
        "Huge",
        Money::from_minor_units(i64::MAX / 2, CurrencyCode::JPY)
            .at(ExchangeRate::parse("1000").unwrap()),
        ana,
        Split::equal([ana]),
        day(),
    )
    .build()
    .unwrap();
    assert_eq!(
        resolve_expense(&identified(&huge, entry(1)), CurrencyCode::EUR),
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
        "Sweets",
        Money::from_minor_units(2, CurrencyCode::JPY).at(ExchangeRate::parse("0.0061").unwrap()),
        Split::equal([ana, bob]),
        Split::equal([ana, bob]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&sweets, entry(1)), CurrencyCode::EUR).unwrap();
    assert_eq!(resolved.shares.total(), eur(1));
    let paid: Vec<i64> = rows(&resolved).iter().map(|row| row.paid).collect();
    assert_eq!(paid.iter().sum::<i64>(), 1);
    assert!(paid.contains(&0));
    let nets = debt_nets(&resolved);
    for Row {
        member,
        paid: payment,
        share,
    } in rows(&resolved)
    {
        assert_eq!(nets.get(&member).copied().unwrap_or(0), payment - share);
    }
}

/// The leftover cent of an even split moves with the expense ID, so over
/// many expenses nobody always pays it; the same ID always resolves the
/// same way.
#[test]
fn the_leftover_cent_rotates_with_the_expense_id() {
    let members = [member(1), member(2), member(3)];
    let expense = Expense::builder("Coffee", eur(100), members[0], Split::equal(members), day())
        .build()
        .unwrap();
    let carrier = |id: ExpenseId| {
        let resolved = resolve_expense(&identified(&expense, id), CurrencyCode::EUR).unwrap();
        rows(&resolved)
            .iter()
            .find(|row| 34 == row.share)
            .map(|row| row.member)
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
        Some((mantissa, scale)) => Money::from_minor_units(amount, CurrencyCode::JPY)
            .at(ExchangeRate::from_decimal(mantissa, scale).unwrap()),
    };
    let mut builder = Expense::builder_with_payers(
        "Random",
        cost,
        parts(payer_parts),
        parts(participant_parts),
        day(),
    );
    if let Some(basis_points) = tip_basis_points {
        let distribution = if equal_tip {
            TipDistribution::Equal
        } else {
            TipDistribution::Proportional
        };
        builder = builder.tip(
            TipRule::percentage(Percentage::from_basis_points(basis_points))
                .distributed(distribution),
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
        id in IDENTIFIER,
    ) {
        let expense = random_expense(amount, &participant_parts, &payer_parts, tip_basis_points, equal_tip, foreign);
        let resolved = resolve_expense(&identified(&expense, ExpenseId::new(&id).unwrap()), CurrencyCode::EUR).unwrap();
        let total = resolved.shares.total().minor_units();
        prop_assert_eq!(rows(&resolved).iter().map(|row| row.paid).sum::<i64>(), total);
        prop_assert_eq!(rows(&resolved).iter().map(|row| row.share).sum::<i64>(), total);
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
        id in IDENTIFIER,
    ) {
        let expense = random_expense(amount, &participant_parts, &payer_parts, tip_basis_points, false, foreign);
        let resolved = resolve_expense(&identified(&expense, ExpenseId::new(&id).unwrap()), CurrencyCode::EUR).unwrap();
        let nets = debt_nets(&resolved);
        for Row { member, paid, share } in rows(&resolved) {
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
        ExpenseItem::builder("Item", eur(price), [ana, bob, carol])
            .build()
            .unwrap()
    });
    let itemised = Expense::itemised_builder_with_payers(
        "Pinned",
        eur(301),
        Split::equal([ana, bob]),
        items,
        day(),
    )
    .tip(TipRule::fixed(eur(2)).distributed(TipDistribution::Equal))
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&itemised, entry(1)), CurrencyCode::EUR).unwrap();
    // The items' leftover cents go to Ana, to Carol, and to Bob and Carol,
    // and the tip's to Ana and Bob, so each consumes 1.01; Bob pays the odd
    // cent of the 3.03.
    assert_eq!(
        rows(&resolved),
        vec![row(ana, 151, 101), row(bob, 152, 101), row(carol, 0, 101)]
    );
    // Ana's 1.51 over three shares of 1.01 gives Bob the leftover cent.
    assert_eq!(
        resolved.debts,
        vec![
            debt(bob, ana, 51),
            debt(carol, ana, 50),
            debt(ana, bob, 51),
            debt(carol, bob, 51)
        ]
    );

    // A conversion's re-division: 300 JPY at 0.00673 is 2.02 EUR among three.
    let foreign = Expense::builder(
        "Pinned",
        Money::from_minor_units(300, CurrencyCode::JPY).at(ExchangeRate::parse("0.00673").unwrap()),
        ana,
        Split::equal([ana, bob, carol]),
        day(),
    )
    .build()
    .unwrap();
    let converted = resolve_expense(&identified(&foreign, entry(2)), CurrencyCode::EUR).unwrap();
    // The leftover cent of the re-division goes to Carol.
    assert_eq!(
        rows(&converted),
        vec![row(ana, 202, 67), row(bob, 0, 67), row(carol, 0, 68)]
    );

    // Pair debts: three payers over four equal shares, so both the first and
    // the second payer's division have ties.
    let dan = member(4);
    let hotel = Expense::builder_with_payers(
        "Pinned",
        eur(400),
        Split::amounts([
            MemberAmount::new(ana, eur(133)),
            MemberAmount::new(bob, eur(133)),
            MemberAmount::new(carol, eur(134)),
        ]),
        Split::equal([ana, bob, carol, dan]),
        day(),
    )
    .build()
    .unwrap();
    assert_eq!(
        resolve_expense(&identified(&hotel, entry(3)), CurrencyCode::EUR)
            .unwrap()
            .debts,
        vec![
            debt(bob, ana, 34),
            debt(carol, ana, 33),
            debt(dan, ana, 33),
            debt(ana, bob, 34),
            debt(carol, bob, 33),
            debt(dan, bob, 33),
            debt(ana, carol, 33),
            debt(bob, carol, 33),
            debt(dan, carol, 34),
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
        "Snack",
        eur(100),
        Split::amounts([
            MemberAmount::new(ana, eur(99)),
            MemberAmount::new(bob, eur(1)),
        ]),
        Split::amounts([
            MemberAmount::new(carol, eur(99)),
            MemberAmount::new(dan, eur(1)),
        ]),
        day(),
    )
    .build()
    .unwrap();
    let resolved = resolve_expense(&identified(&snack, entry(1)), CurrencyCode::EUR).unwrap();
    assert_eq!(
        resolved.debts,
        vec![debt(carol, ana, 98), debt(dan, ana, 1), debt(carol, bob, 1)]
    );
}
