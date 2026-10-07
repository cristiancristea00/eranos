//! Tests for the running totals. The summary is what the apps show as the
//! group's spending and everyone's balance, and it is never recomputed: a
//! delta applied twice, a row left behind at zero, a pair keyed the wrong
//! way round, or a half-applied change after an overflow would leave the
//! numbers permanently wrong. A balance converted through `FromBalance`
//! keeps its standing.

use std::cmp::Ordering;

use super::*;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

fn category(number: u8) -> CategoryId {
    CategoryId::from_bytes([number; 16])
}

const fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

/// The pair of `from` owing or paying `to`.
const fn pair(from: MemberId, to: MemberId) -> MemberPair {
    MemberPair { from, to }
}

/// What one participant of an expense consumed, in minor units.
#[derive(Debug, Clone, Copy)]
struct Share {
    member: MemberId,
    amount: i64,
}

/// The share of `member`: `amount` minor units.
const fn share(member: MemberId, amount: i64) -> Share {
    Share { member, amount }
}

/// The key of `member`'s spending in `category`.
const fn member_category(member: MemberId, category: CategoryId) -> MemberCategory {
    MemberCategory { member, category }
}

/// The spending of `total` minor units in `category`.
const fn category_total(category: CategoryId, total: i64) -> CategoryTotal {
    CategoryTotal {
        category,
        total: eur(total),
    }
}

/// An expense of `total` paid by `payer` and consumed by `participants`,
/// with the payer owed every other share.
fn expense(total: i64, payer: MemberId, participants: &[Share]) -> ResolvedExpense {
    let mut members: BTreeMap<MemberId, MemberShare> = BTreeMap::new();
    members.insert(payer, MemberShare::new(payer, eur(total), eur(0)));
    for participant in participants {
        members
            .entry(participant.member)
            .or_insert_with(|| MemberShare::new(participant.member, eur(0), eur(0)))
            .share = eur(participant.amount);
    }
    let debts = participants
        .iter()
        .filter(|participant| participant.member != payer)
        .map(|debtor| Debt {
            members: pair(debtor.member, payer),
            amount: debtor.amount,
        })
        .collect();
    ResolvedExpense::new(members.into_values().collect(), eur(total), eur(0), debts)
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let dinner = expense(3000, ana, &[share(ana, 1000), share(bob, 2000)]);
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let first = expense(3000, ana, &[share(ana, 1500), share(bob, 1500)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&first, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let before = ledger.clone();

    let second = expense(900, bob, &[share(ana, 450), share(bob, 450)]);
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
    assert_eq!(ledger, Ledger::new(CurrencyCode::EUR));
    assert_eq!(ledger.summary().members().len(), 0);
    assert_eq!(ledger.summary().category_totals().len(), 0);
}

/// A transfer moves balances but is not spending: sent and received change,
/// totals and categories do not.
#[test]
fn a_transfer_moves_balances_but_not_spending() {
    let (ana, bob) = (member(1), member(2));
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let lunch = expense(2000, ana, &[share(ana, 1000), share(bob, 1000)]);
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
    let lunch = expense(1000, ana, &[share(ana, 500), share(bob, 500)]);
    let taxi = expense(600, bob, &[share(ana, 600)]);
    let mut moved = Ledger::new(CurrencyCode::EUR);
    applied(&mut moved, |delta| {
        delta.add_expense(&lunch, food, 1);
        delta.add_expense(&taxi, fuel, 1);
    })
    .unwrap();
    moved.move_category(food, fuel);
    let mut filed = Ledger::new(CurrencyCode::EUR);
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

/// A member's spending would be listed with gaps, or with another member's
/// rows among it: the rows of one member must lie next to each other, in
/// category order.
#[test]
fn member_category_keys_order_by_member_then_category() {
    let (ana, bob) = (member(1), member(2));
    let (food, fuel) = (category(1), category(2));
    assert!(member_category(ana, fuel) < member_category(bob, CategoryId::UNCATEGORISED));
    assert!(member_category(ana, food) < member_category(ana, fuel));
    assert_eq!(
        member_category(ana, food).cmp(&member_category(ana, food)),
        Ordering::Equal
    );
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let big = expense(i64::MAX, ana, &[share(bob, i64::MAX)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&big, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let before = ledger.clone();

    // A second one would overflow the spending total.
    let small = expense(1, bob, &[share(ana, 1)]);
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
    let mut empty = Ledger::new(CurrencyCode::EUR);
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
    let changes = [
        TotalsChange {
            paid: beyond,
            ..TotalsChange::default()
        },
        TotalsChange {
            share: beyond,
            ..TotalsChange::default()
        },
        TotalsChange {
            sent: beyond,
            ..TotalsChange::default()
        },
        TotalsChange {
            received: beyond,
            ..TotalsChange::default()
        },
        TotalsChange {
            net: beyond,
            ..TotalsChange::default()
        },
    ];
    let mut deltas = vec![Delta {
        total_spent: beyond,
        ..Delta::default()
    }];
    deltas.extend(changes.into_iter().map(|change| Delta {
        members: BTreeMap::from([(ana, change)]),
        ..Delta::default()
    }));
    deltas.extend([
        Delta {
            categories: BTreeMap::from([(category(1), beyond)]),
            ..Delta::default()
        },
        Delta {
            member_categories: BTreeMap::from([(
                member_category(ana, CategoryId::UNCATEGORISED),
                beyond,
            )]),
            ..Delta::default()
        },
        Delta {
            pairs: BTreeMap::from([(pair(ana, bob), beyond)]),
            ..Delta::default()
        },
    ]);
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    for delta in &deltas {
        assert_eq!(ledger.apply(delta), Err(MoneyError::Overflow), "{delta:?}");
        assert_eq!(ledger, Ledger::new(CurrencyCode::EUR));
    }
}

/// The balance reads the sign of the net: debt is `Owes` with a positive
/// amount, zero is `Settled`, and owed money is `GetsBack`.
#[test]
fn a_balance_reads_the_sign_of_the_net() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let taxi = expense(1200, ana, &[share(bob, 1200)]);
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let lunch = expense(1000, ana, &[share(ana, 500), share(bob, 500)]);
    applied(&mut ledger, |delta| delta.add_expense(&lunch, food, 1)).unwrap();
    let taxi = expense(600, bob, &[share(ana, 600)]);
    applied(&mut ledger, |delta| delta.add_expense(&taxi, fuel, 1)).unwrap();
    let snack = expense(300, ana, &[share(ana, 300)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&snack, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();

    let summary = ledger.summary();
    assert_eq!(
        summary.member_category_totals(ana).collect::<Vec<_>>(),
        [
            category_total(CategoryId::UNCATEGORISED, 300),
            category_total(food, 500),
            category_total(fuel, 600),
        ]
    );
    assert_eq!(
        summary.member_category_totals(bob).collect::<Vec<_>>(),
        [category_total(food, 500)]
    );
    assert_eq!(
        summary.category_totals().collect::<Vec<_>>(),
        [
            category_total(CategoryId::UNCATEGORISED, 300),
            category_total(food, 1000),
            category_total(fuel, 600),
        ]
    );
}

/// A member whose rows lie between other members' would have their list cut
/// short, or padded with a neighbour's rows.
#[test]
fn member_category_totals_list_a_member_between_others_whole() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let (food, fuel) = (category(1), category(2));
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let everyone = [share(ana, 200), share(bob, 200), share(carol, 200)];
    applied(&mut ledger, |delta| {
        delta.add_expense(&expense(600, ana, &everyone), CategoryId::UNCATEGORISED, 1);
        delta.add_expense(&expense(400, bob, &[share(bob, 400)]), food, 1);
        delta.add_expense(&expense(600, carol, &everyone), fuel, 1);
    })
    .unwrap();

    let summary = ledger.summary();
    assert_eq!(
        summary.member_category_totals(bob).collect::<Vec<_>>(),
        [
            category_total(CategoryId::UNCATEGORISED, 200),
            category_total(food, 400),
            category_total(fuel, 200),
        ]
    );
    // The members before and after keep their own rows.
    for neighbour in [ana, carol] {
        assert_eq!(
            summary
                .member_category_totals(neighbour)
                .collect::<Vec<_>>(),
            [
                category_total(CategoryId::UNCATEGORISED, 200),
                category_total(fuel, 200),
            ]
        );
    }
}

/// A member's share of one expense is found by member, and an uninvolved
/// member has none.
#[test]
fn expense_shares_look_up_members() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let shares = expense(1000, ana, &[share(ana, 400), share(bob, 600)]).shares;
    assert_eq!(shares.member(bob).map(MemberShare::share), Some(eur(600)));
    assert_eq!(shares.member(ana).map(MemberShare::paid), Some(eur(1000)));
    assert_eq!(shares.member(ana).map(MemberShare::member), Some(ana));
    assert_eq!(shares.member(carol), None);
    assert_eq!(shares.total(), eur(1000));
    assert_eq!(shares.tip(), eur(0));
    assert_eq!(shares.members().len(), 2);
}

/// A member's net total as a screen listing everyone's totals shows it.
#[derive(Debug, PartialEq, Eq)]
struct MemberNet {
    member: MemberId,
    net: Money,
}

/// A screen listing everyone's totals would attribute them to the wrong
/// member, or a looked-up member's totals would name someone else.
#[test]
fn a_member_summary_names_its_member() {
    let (ana, bob, carol) = (member(1), member(2), member(3));
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    let shares = expense(1000, ana, &[share(ana, 500), share(bob, 500)]);
    applied(&mut ledger, |delta| {
        delta.add_expense(&shares, CategoryId::UNCATEGORISED, 1);
    })
    .unwrap();
    let summary = ledger.summary();
    // Every member with totals, in member order.
    assert_eq!(
        summary
            .members()
            .map(|totals| MemberNet {
                member: totals.member(),
                net: totals.net(),
            })
            .collect::<Vec<_>>(),
        [
            MemberNet {
                member: ana,
                net: eur(500),
            },
            MemberNet {
                member: bob,
                net: eur(-500),
            },
        ]
    );
    // One member looked up, with totals or without.
    assert_eq!(summary.member(bob).member(), bob);
    assert_eq!(summary.member(carol).member(), carol);
}

/// A balance as a crate outside this one converts it.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedBalance {
    Owes(Money),
    Settled,
    GetsBack(Money),
}

impl FromBalance for ConvertedBalance {
    fn owes(amount: Money) -> Self {
        Self::Owes(amount)
    }

    fn settled() -> Self {
        Self::Settled
    }

    fn gets_back(amount: Money) -> Self {
        Self::GetsBack(amount)
    }
}

/// A conversion that called another variant's method would tell a member
/// who is owed money that they owe it.
#[test]
fn a_balance_converts_through_the_method_of_its_variant() {
    assert_eq!(
        ConvertedBalance::from_balance(Balance::Owes(eur(300))),
        ConvertedBalance::Owes(eur(300))
    );
    assert_eq!(
        ConvertedBalance::from_balance(Balance::Settled),
        ConvertedBalance::Settled
    );
    assert_eq!(
        ConvertedBalance::from_balance(Balance::GetsBack(eur(1_200))),
        ConvertedBalance::GetsBack(eur(1_200))
    );
}
