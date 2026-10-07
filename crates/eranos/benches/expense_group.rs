//! Benchmarks of the operations that touch every entry of a group: a
//! currency change, which resolves every entry again, and loading a saved
//! group (feature `serde`), which replays every entry through the group's
//! checks. Run on an idle machine with
//! `cargo bench -p eranos --all-features --bench group`.
#![forbid(unsafe_code)]
#![expect(
    clippy::panic,
    reason = "a benchmark whose fixture cannot be built has nothing to measure"
)]

use std::hint::black_box;
use std::iter;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use eranos::{
    CalendarDate, Currency, CurrencyRate, ExchangeRate, Expense, ExpenseGroup, Member, MemberId,
    Money, Split, Transfer,
};

/// The numbers of expenses measured: a busy year of a household, and ten.
const SIZES: [usize; 2] = [1_000, 10_000];

/// A group of six with `size` expenses shared by everyone, every tenth of
/// them in RON at a fixed rate, and a transfer for every tenth expense.
fn group(size: usize) -> ExpenseGroup {
    let mut group = ExpenseGroup::builder("Benchmark", Currency::EUR)
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    let members: Vec<MemberId> = iter::repeat_with(|| {
        let member = Member::builder("Member")
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        group
            .add_member(member)
            .unwrap_or_else(|error| panic!("{error}"))
    })
    .take(6)
    .collect();
    let day = CalendarDate::new(2026, 9, 23).unwrap_or_else(|error| panic!("{error}"));
    let rate = ExchangeRate::parse("0.2").unwrap_or_else(|error| panic!("{error}"));
    for (index, &payer) in (0_i64..).zip(members.iter().cycle()).take(size) {
        let minor_units = 100 + (index % 9_973) * 7;
        let cost = if 0 == index % 10 {
            Money::from_minor_units(minor_units, Currency::RON).at(rate)
        } else {
            Money::from_minor_units(minor_units, Currency::EUR).into()
        };
        let expense = Expense::builder("Expense", cost, payer, Split::equal(members.clone()), day)
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        group
            .add_expense(expense)
            .unwrap_or_else(|error| panic!("{error}"));
        if 5 == index % 10 {
            let recipient = members
                .get(usize::try_from(index).unwrap_or_default() % 5)
                .copied()
                .unwrap_or(payer);
            if recipient != payer {
                let transfer = Transfer::builder(
                    payer,
                    recipient,
                    Money::from_minor_units(minor_units / 2, Currency::EUR),
                    day,
                )
                .build()
                .unwrap_or_else(|error| panic!("{error}"));
                group
                    .add_transfer(transfer)
                    .unwrap_or_else(|error| panic!("{error}"));
            }
        }
    }
    group
}

fn currency_changes(criterion: &mut Criterion) {
    let rates = [CurrencyRate::new(
        Currency::EUR,
        ExchangeRate::parse("4.97").unwrap_or_else(|error| panic!("{error}")),
    )];
    let mut benchmark = criterion.benchmark_group("expense_group/change currency");
    benchmark.sample_size(10);
    for size in SIZES {
        let template = group(size);
        benchmark.bench_with_input(
            BenchmarkId::from_parameter(size),
            &template,
            |bencher, template| {
                // A fresh copy for every iteration, dropped outside the timed
                // section.
                bencher.iter_batched_ref(
                    || template.clone(),
                    |group| black_box(group.change_currency(Currency::RON, &rates).ok()),
                    BatchSize::LargeInput,
                );
            },
        );
    }
    benchmark.finish();
}

#[cfg(feature = "serde")]
fn loading(criterion: &mut Criterion) {
    let mut benchmark = criterion.benchmark_group("expense_group/load");
    benchmark.sample_size(10);
    for size in SIZES {
        let saved = serde_json::to_string(&group(size)).unwrap_or_else(|error| panic!("{error}"));
        benchmark.bench_with_input(
            BenchmarkId::from_parameter(size),
            &saved,
            |bencher, saved| {
                bencher.iter(|| {
                    black_box(
                        serde_json::from_str::<ExpenseGroup>(saved)
                            .map(|group| group.expenses().len())
                            .ok(),
                    )
                });
            },
        );
    }
    benchmark.finish();
}

#[cfg(feature = "serde")]
criterion_group!(benches, currency_changes, loading);
#[cfg(not(feature = "serde"))]
criterion_group!(benches, currency_changes);
criterion_main!(benches);
