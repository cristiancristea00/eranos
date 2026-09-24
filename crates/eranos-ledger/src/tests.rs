//! Tests for the running totals. The summary is what the apps show as the
//! group's spending and everyone's balance, and it is never recomputed: a
//! delta applied twice, a row left behind at zero, a pair keyed the wrong
//! way round, or a half-applied change after an overflow would leave the
//! numbers permanently wrong.

use super::*;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

fn category(number: u8) -> CategoryId {
    CategoryId::from_bytes([number; 16])
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

/// The pair of `from` owing or paying `to`.
const fn pair(from: MemberId, to: MemberId) -> MemberPair {
    MemberPair { from, to }
}

/// An expense of `total` paid by `payer` and consumed by `participants`,
/// with the payer owed every other share.
fn expense(total: i64, payer: MemberId, participants: &[(MemberId, i64)]) -> ResolvedExpense {
    let mut members: BTreeMap<MemberId, (i64, i64)> = BTreeMap::new();
    members.entry(payer).or_default().0 = total;
    for &(participant, share) in participants {
        members.entry(participant).or_default().1 = share;
    }
    let debts = participants
        .iter()
        .filter(|&&(participant, _)| participant != payer)
        .map(|&(debtor, amount)| Debt {
            amount,
            members: pair(debtor, payer),
        })
        .collect();
    let members = members
        .into_iter()
        .map(|(member, (paid, share))| MemberShare::new(member, eur(paid), eur(share)))
        .collect();
    ResolvedExpense::new(debts, members, eur(0), eur(total))
}

fn applied(ledger: &mut Ledger, change: impl FnOnce(&mut Delta)) -> Result<(), MoneyError> {
    let mut delta = Delta::default();
    change(&mut delta);
    ledger.apply(&delta)
}

/// Adding an expense moves every total by exactly its shares: spending,
/// paid, consumed, net, per category, and per member per category.
#[test]
fn an_expense_moves_every_total_by_its_shares() {
    let (ana, bob) = (member(1), member(2));
    let food = category(1);
    let mut ledger = Ledger::new(Currency::EUR);
    let dinner = expense(3000, ana, &[(ana, 1000), (bob, 2000)]);
    applied(&mut ledger, |delta| delta.add_expense(&dinner, food, 1)).unwrap();

    let summary = ledger.summary();
    assert_eq!(summary.total_spent(), eur(3000));
    // Ana paid 30.00 and consumed 10.00; Bob consumed 20.00.
    assert_eq!(summary.member(ana).paid(), eur(3000));
    assert_eq!(summary.member(ana).share(), eur(1000));
    assert_eq!(summary.member(ana).net(), eur(2000));
    assert_eq!(summary.member(bob).net(), eur(-2000));
    // Categories count the whole total; per member they count the share.
    assert_eq!(summary.category_total(food), eur(3000));
    assert_eq!(summary.category_total(CategoryId::UNCATEGORISED), eur(0));
    assert_eq!(summary.member_category_total(bob, food), eur(2000));
    // Bob owes Ana his share, pairwise.
    assert_eq!(ledger.pairs().get(&pair(ana, bob)), Some(&-2000));
}

/// Removing an entry is the exact inverse of adding it, and leaves no zero
/// rows behind, so the ledger compares equal to one that never saw it.
#[test]
fn removing_an_entry_restores_the_ledger_exactly() {
    let (ana, bob) = (member(1), member(2));
    let mut ledger = Ledger::new(Currency::EUR);
    let first = expense(3000, ana, &[(ana, 1500), (bob, 1500)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&first, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let before = ledger.clone();

    let second = expense(900, bob, &[(ana, 450), (bob, 450)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&second, category(2), 1);
    })
    .unwrap();
    applied(&mut ledger, |delta| {
        delta.add_transfer(pair(bob, ana), 700, 1);
    })
    .unwrap();
    applied(&mut ledger, |delta| {
        delta.add_expense(&second, category(2), -1);
        delta.add_transfer(pair(bob, ana), 700, -1);
    })
    .unwrap();
    assert_eq!(ledger, before);

    // Removing the first as well empties every table.
    applied(&mut ledger, |delta| {
        delta.add_expense(&first, CategoryId::UNCATEGORISED, -1);
    })
    .unwrap();
    assert_eq!(ledger, Ledger::new(Currency::EUR));
    assert_eq!(ledger.summary().members().len(), 0);
    assert_eq!(ledger.summary().category_totals().len(), 0);
}

/// A transfer moves balances but is not spending: sent and received change,
/// totals and categories do not.
#[test]
fn a_transfer_moves_balances_but_not_spending() {
    let (ana, bob) = (member(1), member(2));
    let mut ledger = Ledger::new(Currency::EUR);
    let lunch = expense(2000, ana, &[(ana, 1000), (bob, 1000)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&lunch, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    applied(&mut ledger, |delta| {
        delta.add_transfer(pair(bob, ana), 1000, 1);
    })
    .unwrap();

    let summary = ledger.summary();
    assert_eq!(summary.total_spent(), eur(2000));
    assert_eq!(summary.member(bob).sent(), eur(1000));
    assert_eq!(summary.member(ana).received(), eur(1000));
    // Bob's repayment settles both of them, pairwise too.
    assert_eq!(summary.member(ana).balance(), Balance::Settled);
    assert_eq!(summary.member(bob).balance(), Balance::Settled);
    assert!(ledger.pairs().is_empty());
}

/// Removing a category would leave a row behind, or give Uncategorised
/// other totals than if the expenses had always been filed there.
#[test]
fn moving_a_category_equals_filing_its_expenses_there_from_the_start() {
    let (ana, bob) = (member(1), member(2));
    let (food, fuel) = (category(1), category(2));
    let lunch = expense(1000, ana, &[(ana, 500), (bob, 500)]);
    let taxi = expense(600, bob, &[(ana, 600)]);
    let mut moved = Ledger::new(Currency::EUR);
    applied(&mut moved, |delta| {
        delta.add_expense(&lunch, food, 1);
        delta.add_expense(&taxi, fuel, 1);
    })
    .unwrap();
    moved.move_category(food, fuel);
    let mut filed = Ledger::new(Currency::EUR);
    applied(&mut filed, |delta| {
        delta.add_expense(&lunch, fuel, 1);
        delta.add_expense(&taxi, fuel, 1);
    })
    .unwrap();
    assert_eq!(moved, filed);
    // A category without totals moves nothing.
    moved.move_category(food, fuel);
    assert_eq!(moved, filed);
}

/// Pairs would be listed, and payments sorted, by recipient before payer.
#[test]
fn member_pairs_order_by_the_first_member_then_the_second() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    assert!(pair(ana, carol) < pair(bob, ana));
    assert!(pair(ana, bob) < pair(ana, carol));
    assert_eq!(pair(ana, bob).cmp(&pair(ana, bob)), Ordering::Equal);
    assert_eq!(pair(ana, bob).reversed(), pair(bob, ana));
}

/// Pairs are keyed from the lower member whichever way the debt runs, so
/// debts in both directions net against each other.
#[test]
fn pair_debts_net_in_both_directions() {
    let (ana, bob) = (member(1), member(2));
    let mut delta = Delta::default();
    // Bob owes Ana 5.00, and Ana owes Bob 3.00.
    delta.add_debt(pair(bob, ana), 500);
    delta.add_debt(pair(ana, bob), 300);
    let mut ledger = Ledger::new(Currency::EUR);
    ledger.apply(&delta).unwrap();
    // Ana, the lower member, owes Bob −2.00: Bob owes Ana 2.00.
    assert_eq!(ledger.pairs().get(&pair(ana, bob)), Some(&-200));
    assert_eq!(ledger.pairs().len(), 1);
}

/// A change that would push any total out of range is refused whole, so the
/// ledger never holds a half-applied entry.
#[test]
fn an_overflowing_change_is_refused_whole() {
    let (ana, bob) = (member(1), member(2));
    let mut ledger = Ledger::new(Currency::EUR);
    let big = expense(i64::MAX, ana, &[(bob, i64::MAX)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&big, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let before = ledger.clone();

    // A second one would overflow the spending total.
    let small = expense(1, bob, &[(ana, 1)]);
    assert_eq!(
        applied(&mut ledger, |delta| delta.add_expense(
            &small,
            CategoryId::UNCATEGORISED,
            1
        )),
        Err(MoneyError::Overflow)
    );
    assert_eq!(ledger, before);
    // So would a single entry below −i64::MAX, which could not be negated.
    let mut empty = Ledger::new(Currency::EUR);
    assert_eq!(
        applied(&mut empty, |delta| delta.add_transfer(
            pair(ana, bob),
            i64::MAX,
            -1
        )),
        Ok(())
    );
    assert_eq!(
        applied(&mut empty, |delta| delta.add_transfer(
            pair(ana, bob),
            1,
            -1
        )),
        Err(MoneyError::Overflow)
    );
}

/// Every value the ledger keeps is range-checked, including those no group
/// entry can push out of range: a check missing on one would let a new kind
/// of change store a wrapped total.
#[test]
fn every_value_is_range_checked() {
    let (ana, bob) = (member(1), member(2));
    let beyond = i128::from(i64::MAX) + 1;
    // Paid, share, sent, received, and net in turn.
    let mut deltas: Vec<Delta> = (0..5)
        .map(|field| {
            let mut change = [0; 5];
            change[field] = beyond;
            Delta {
                members: BTreeMap::from([(ana, change)]),
                ..Delta::default()
            }
        })
        .collect();
    deltas.extend([
        Delta {
            total_spent: beyond,
            ..Delta::default()
        },
        Delta {
            categories: BTreeMap::from([(category(1), beyond)]),
            ..Delta::default()
        },
        Delta {
            member_categories: BTreeMap::from([((ana, CategoryId::UNCATEGORISED), beyond)]),
            ..Delta::default()
        },
        Delta {
            pairs: BTreeMap::from([(pair(ana, bob), beyond)]),
            ..Delta::default()
        },
    ]);
    let mut ledger = Ledger::new(Currency::EUR);
    for delta in &deltas {
        assert_eq!(ledger.apply(delta), Err(MoneyError::Overflow), "{delta:?}");
        assert_eq!(ledger, Ledger::new(Currency::EUR));
    }
}

/// The balance reads the sign of the net: owed money is `GetsBack`, debt is
/// `Owes` with a positive amount, and zero is `Settled`.
#[test]
fn a_balance_reads_the_sign_of_the_net() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let mut ledger = Ledger::new(Currency::EUR);
    let taxi = expense(1200, ana, &[(bob, 1200)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&taxi, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();

    let summary = ledger.summary();
    assert_eq!(summary.member(ana).balance(), Balance::GetsBack(eur(1200)));
    assert_eq!(summary.member(bob).balance(), Balance::Owes(eur(1200)));
    // A member without entries is settled, with zero totals.
    assert_eq!(summary.member(carol).balance(), Balance::Settled);
    assert_eq!(summary.member(carol).paid(), eur(0));
    // The text form the apps may show as is.
    assert_eq!(
        summary.member(ana).balance().to_string(),
        "gets back 12.00 EUR"
    );
    assert_eq!(summary.member(bob).balance().to_string(), "owes 12.00 EUR");
    assert_eq!(Balance::Settled.to_string(), "settled");
}

/// Per-member category totals list only that member's categories,
/// uncategorised first, and never leak another member's rows.
#[test]
fn member_category_totals_list_only_that_member() {
    let (ana, bob) = (member(1), member(2));
    let (food, fuel) = (category(1), category(2));
    let mut ledger = Ledger::new(Currency::EUR);
    for (shares, category) in [
        (expense(1000, ana, &[(ana, 500), (bob, 500)]), food),
        (expense(600, bob, &[(ana, 600)]), fuel),
        (expense(300, ana, &[(ana, 300)]), CategoryId::UNCATEGORISED),
    ] {
        applied(&mut ledger, |delta| delta.add_expense(&shares, category, 1)).unwrap();
    }

    let summary = ledger.summary();
    let rows = |totals: &mut dyn Iterator<Item = CategoryTotal>| {
        totals
            .map(|total| (total.category(), total.total()))
            .collect::<Vec<_>>()
    };
    let ana_rows = rows(&mut summary.member_category_totals(ana));
    assert_eq!(
        ana_rows,
        vec![
            (CategoryId::UNCATEGORISED, eur(300)),
            (food, eur(500)),
            (fuel, eur(600))
        ]
    );
    let bob_rows = rows(&mut summary.member_category_totals(bob));
    assert_eq!(bob_rows, vec![(food, eur(500))]);
    assert_eq!(
        rows(&mut summary.category_totals()),
        vec![
            (CategoryId::UNCATEGORISED, eur(300)),
            (food, eur(1000)),
            (fuel, eur(600))
        ]
    );
}

/// A member's share of one expense is found by member, and an uninvolved
/// member has none.
#[test]
fn expense_shares_look_up_members() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let shares = expense(1000, ana, &[(ana, 400), (bob, 600)]).shares;
    assert_eq!(shares.member(bob).map(MemberShare::share), Some(eur(600)));
    assert_eq!(shares.member(ana).map(MemberShare::paid), Some(eur(1000)));
    assert_eq!(shares.member(ana).map(MemberShare::member), Some(ana));
    assert_eq!(shares.member(carol), None);
    assert_eq!(shares.total(), eur(1000));
    assert_eq!(shares.tip(), eur(0));
    assert_eq!(shares.members().len(), 2);
}

/// A screen listing everyone's totals would attribute them to the wrong
/// member, or a looked-up member's totals would name someone else.
#[test]
fn a_member_summary_names_its_member() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let mut ledger = Ledger::new(Currency::EUR);
    let shares = expense(1000, ana, &[(ana, 500), (bob, 500)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&shares, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let summary = ledger.summary();
    // Every member with totals, in member order.
    assert_eq!(
        summary
            .members()
            .map(|totals| (totals.member(), totals.net()))
            .collect::<Vec<_>>(),
        vec![(ana, eur(500)), (bob, eur(-500))]
    );
    // One member looked up, with totals or without.
    assert_eq!(summary.member(bob).member(), bob);
    assert_eq!(summary.member(carol).member(), carol);
}
