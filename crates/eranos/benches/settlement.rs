//! Benchmarks of the three settlement strategies against groups of growing
//! size. The timings documented on `SettlementStrategy` come from here; run
//! on an idle machine with `cargo bench -p eranos --bench settlement`.
#![expect(
    clippy::panic,
    reason = "a benchmark whose fixture cannot be built has nothing to measure"
)]

use std::hint::black_box;
use std::iter;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use eranos::{
    CalendarDate, Currency, Expense, Group, Member, MemberId, Money, Settlement,
    SettlementStrategy, Split,
};

/// A deterministic pseudo-random sequence (xorshift64), so every run
/// benchmarks the same groups.
struct Draws(u64);

impl Draws {
    const fn next(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }
}

/// A group of `size` members where each member paid one expense shared by
/// everyone: `size` open balances with, almost surely, no zero-sum subset
/// smaller than the whole group, which is the exact solver's worst case.
fn group(size: usize) -> Group {
    let mut group = Group::builder(Currency::EUR, "Benchmark")
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
    // A member of their own for each slot, since a group holds each member
    // once.
    let members: Vec<MemberId> = iter::repeat_with(|| {
        let member = Member::builder("Member")
            .build()
            .unwrap_or_else(|error| panic!("{error}"));
        group
            .add_member(member)
            .unwrap_or_else(|error| panic!("{error}"))
    })
    .take(size)
    .collect();
    let mut draws = Draws(0x9e37_79b9_7f4a_7c15);
    let day = CalendarDate::new(23, 9, 2026).unwrap_or_else(|error| panic!("{error}"));
    for &payer in &members {
        let minor_units = 1 + i64::try_from(draws.next(1_000_000)).unwrap_or(0);
        let expense = Expense::builder(
            Money::from_minor_units(Currency::EUR, minor_units),
            day,
            "Expense",
            payer,
            Split::equal(members.iter().copied()),
        )
        .build()
        .unwrap_or_else(|error| panic!("{error}"));
        group
            .add_expense(expense)
            .unwrap_or_else(|error| panic!("{error}"));
    }
    group
}

fn strategies(criterion: &mut Criterion) {
    let cases = [
        (
            SettlementStrategy::MinimumTransfers,
            vec![4, 8, 12, 16, 20, 24],
        ),
        (SettlementStrategy::Greedy, vec![4, 20, 100, 1_000]),
        (SettlementStrategy::Direct, vec![4, 20, 100]),
    ];
    for (strategy, sizes) in cases {
        let mut benchmark = criterion.benchmark_group(format!("settlement/{strategy:?}"));
        benchmark.sample_size(10);
        for size in sizes {
            let template = group(size);
            benchmark.bench_with_input(
                BenchmarkId::from_parameter(size),
                &template,
                |bencher, template| {
                    // Cloning gives every iteration an empty settlement cache;
                    // the clones are dropped outside the timed section.
                    bencher.iter_batched_ref(
                        || template.clone(),
                        |group| black_box(group.settlement(strategy).map(Settlement::len).ok()),
                        BatchSize::LargeInput,
                    );
                },
            );
        }
        benchmark.finish();
    }
}

criterion_group!(benches, strategies);
criterion_main!(benches);
