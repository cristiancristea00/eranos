//! The totals reach the apps as records. A balance on the wrong side, a
//! total of another member or category, or an expense's shares with a
//! member dropped would be what the apps show.

use eranos::{CalendarDate, CurrencyCode, Expense, ExpenseGroup, Member, Split, Transfer};

use super::*;
use crate::CurrencyDetails;

fn euros(minor_units: i64) -> eranos::Money {
    eranos::Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn record(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: CurrencyDetails::from_library(CurrencyCode::EUR),
    }
}

/// A balance would reach the apps on the wrong side, or as settled when it
/// is not.
#[test]
fn every_balance_converts_to_its_side() {
    assert_eq!(
        Balance::from_balance(library::Balance::GetsBack(euros(1_200))),
        Balance::GetsBack {
            amount: record(1_200)
        }
    );
    assert_eq!(
        Balance::from_balance(library::Balance::Owes(euros(300))),
        Balance::Owes {
            amount: record(300)
        }
    );
    assert_eq!(
        Balance::from_balance(library::Balance::Settled),
        Balance::Settled
    );
}

/// A lookup in the summary that found another member's or another category's
/// totals, or failed for one without entries, would show the C apps numbers
/// the group does not hold.
#[test]
fn a_lookup_in_the_summary_finds_its_totals_and_zero_for_the_rest() {
    let totals = |member: &str| MemberSummary {
        member: member.to_owned(),
        paid: record(9_000),
        share: record(4_500),
        sent: record(0),
        received: record(1_000),
        net: record(3_500),
        balance: Balance::GetsBack {
            amount: record(3_500),
        },
    };
    let rent = |category: &str| CategoryTotal {
        category: category.to_owned(),
        total: record(4_500),
    };
    let summary = Summary {
        total_spent: record(9_000),
        members: vec![totals("ana")],
        category_totals: vec![rent("home")],
        member_category_totals: vec![MemberCategoryTotals {
            member: "ana".to_owned(),
            totals: vec![rent("home")],
        }],
    };
    assert_eq!(summary.member("ana"), totals("ana"));
    assert_eq!(
        summary.member("bob"),
        MemberSummary {
            member: "bob".to_owned(),
            paid: record(0),
            share: record(0),
            sent: record(0),
            received: record(0),
            net: record(0),
            balance: Balance::Settled,
        }
    );
    assert_eq!(summary.category_total("home"), record(4_500));
    assert_eq!(summary.category_total("food"), record(0));
    assert_eq!(summary.member_category_total("ana", "home"), record(4_500));
    assert_eq!(summary.member_category_total("ana", "food"), record(0));
    assert_eq!(summary.member_category_total("bob", "home"), record(0));
}

/// The summary and an expense's shares would reach the apps with another
/// member's or another category's numbers.
#[test]
fn the_summary_and_the_shares_keep_every_number() {
    let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR)
        .build()
        .unwrap();
    let ana = group
        .add_member(Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(Member::builder("Bob").build().unwrap())
        .unwrap();
    let day = CalendarDate::new(2026, 10, 1).unwrap();
    let rent = Expense::builder("Rent", euros(9_000), ana, Split::equal([ana, bob]), day)
        .build()
        .unwrap();
    let rent = group.add_expense(rent).unwrap();
    group
        .add_transfer(
            Transfer::builder(bob, ana, euros(1_000), day)
                .build()
                .unwrap(),
        )
        .unwrap();

    let summary = Summary::from_library(group.summary());
    assert_eq!(summary.total_spent, record(9_000));
    let uncategorised = eranos::CategoryId::UNCATEGORISED.to_string();
    assert_eq!(
        summary.category_totals,
        [CategoryTotal {
            category: uncategorised.clone(),
            total: record(9_000),
        }]
    );
    assert_eq!(
        summary.members,
        [
            MemberSummary {
                member: ana.to_string(),
                paid: record(9_000),
                share: record(4_500),
                sent: record(0),
                received: record(1_000),
                net: record(3_500),
                balance: Balance::GetsBack {
                    amount: record(3_500)
                },
            },
            MemberSummary {
                member: bob.to_string(),
                paid: record(0),
                share: record(4_500),
                sent: record(1_000),
                received: record(0),
                net: record(-3_500),
                balance: Balance::Owes {
                    amount: record(3_500)
                },
            },
        ]
    );
    assert_eq!(
        summary.member_category_totals,
        [ana, bob].map(|member| MemberCategoryTotals {
            member: member.to_string(),
            totals: vec![CategoryTotal {
                category: uncategorised.clone(),
                total: record(4_500),
            }],
        })
    );

    let shares = ExpenseShares::from_library(group.expense_shares(rent).unwrap());
    assert_eq!(
        shares,
        ExpenseShares {
            members: vec![
                MemberShare {
                    member: ana.to_string(),
                    paid: record(9_000),
                    share: record(4_500),
                },
                MemberShare {
                    member: bob.to_string(),
                    paid: record(0),
                    share: record(4_500),
                },
            ],
            total: record(9_000),
            tip: record(0),
        }
    );
}
