//! Benchmarks of the incremental totals: adding, replacing, and removing one
//! expense in a group that already has 10 000, which the documentation
//! states takes time proportional to the expense's size, not the group's.
//! Run with `cargo bench -p eranos --bench ledger`.
#![forbid(unsafe_code)]
#![expect(
    clippy::panic,
    reason = "a benchmark whose fixture cannot be built has nothing to measure"
)]

use std::hint::black_box;
use std::iter;

use criterion::{Criterion, criterion_group, criterion_main};
use eranos::{
    CalendarDate, CurrencyCode, Expense, ExpenseGroup, Member, MemberId, MemberParts, Money, Parts,
    SettlementStrategy, Split,
};

const EXISTING: i64 = 10_000;

/// `members` sharing by parts 1, 2, 3, and so on.
fn parts(members: &[MemberId]) -> Split {
    Split::parts(members.iter().zip(1_u32..).map(|(&member, weight)| {
        MemberParts::new(member, Parts::from_whole(weight).unwrap_or(Parts::ONE))
    }))
}

fn expense(minor_units: i64, payer: MemberId, members: &[MemberId]) -> Expense {
    let day = CalendarDate::new(2026, 9, 23).unwrap_or_else(|error| panic!("{error}"));
    Expense::builder(
        "Expense",
        Money::from_minor_units(minor_units, CurrencyCode::EUR),
        payer,
        parts(members),
        day,
    )
    .build()
    .unwrap_or_else(|error| panic!("{error}"))
}

/// A member of their own, since a group holds each member once.
fn member() -> Member {
    Member::builder("Member")
        .build()
        .unwrap_or_else(|error| panic!("{error}"))
}

/// A group and its members, in the order they joined.
struct Household {
    group: ExpenseGroup,
    members: Vec<MemberId>,
}

/// A household of six with `EXISTING` expenses, each paid by the next member
/// in turn.
fn household() -> Household {
    let mut group = ExpenseGroup::builder("Benchmark", CurrencyCode::EUR)
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    let members: Vec<MemberId> = iter::repeat_with(|| {
        group
            .add_member(member())
            .unwrap_or_else(|error| panic!("{error}"))
    })
    .take(6)
    .collect();
    for (index, &payer) in (0..EXISTING).zip(members.iter().cycle()) {
        let minor_units = 100 + (index % 9_973) * 7;
        group
            .add_expense(expense(minor_units, payer, &members))
            .unwrap_or_else(|error| panic!("{error}"));
    }
    Household { group, members }
}

fn mutations(criterion: &mut Criterion) {
    let Household { mut group, members } = household();
    let (Some(&first), Some(&second), Some(&reader)) =
        (members.first(), members.get(1), members.get(2))
    else {
        panic!("the household has six members");
    };
    let added = expense(12_345, first, &members);
    // A new version of `added`, so it replaces `added` under its identifier.
    let replacement = added
        .to_builder()
        .cost(Money::from_minor_units(54_321, CurrencyCode::EUR))
        .paid_by(second)
        .split(parts(&[first, second, reader]))
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    // Computed once, so every mutation below also pays for dropping the
    // cached settlement.
    group
        .settlement(SettlementStrategy::Greedy)
        .unwrap_or_else(|error| panic!("{error}"));

    criterion.bench_function("ledger/add and remove an expense", |bencher| {
        bencher.iter(|| {
            let id = group
                .add_expense(black_box(added.clone()))
                .unwrap_or_else(|error| panic!("{error}"));
            black_box(group.remove_expense(id).ok());
        });
    });
    group
        .add_expense(added.clone())
        .unwrap_or_else(|error| panic!("{error}"));
    criterion.bench_function("ledger/replace an expense", |bencher| {
        bencher.iter(|| {
            black_box(group.replace_expense(black_box(replacement.clone())).ok());
            black_box(group.replace_expense(black_box(added.clone())).ok());
        });
    });
    criterion.bench_function("ledger/read a member's totals", |bencher| {
        bencher.iter(|| black_box(group.summary().member(black_box(reader)).balance()));
    });
}

criterion_group!(benches, mutations);
criterion_main!(benches);
