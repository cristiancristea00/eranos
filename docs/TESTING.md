# Testing

What the test suites check, where they live, and how to write one.
[AGENTS.md](../AGENTS.md) holds the gates, [DEVELOPMENT.md](DEVELOPMENT.md)
every command.

## Contents

1. [What the suite believes](#1-what-the-suite-believes)
2. [Layout](#2-layout)
3. [The invariants](#3-the-invariants)
4. [Writing a test here](#4-writing-a-test-here)
5. [Coverage](#5-coverage)
6. [Mutation testing](#6-mutation-testing)
7. [Benchmarks](#7-benchmarks)
8. [What to run for a given change](#8-what-to-run-for-a-given-change)

## 1. What the suite believes

- **Money must add up.** Every test that divides an amount checks the parts
  against the whole; the properties check it for random inputs.
- **Numbers users see are pinned.** Hand-computed splits, the rounding
  rule, the FNV-1a tie rotation, and the serialised shapes are asserted
  literally, so a change to any of them fails a test before it changes what
  a group owes (see [VERSIONING.md](VERSIONING.md)).
- **Incremental equals rebuilt.** The group keeps its totals incrementally;
  a property test compares them with a full rebuild after random changes.
- **A refused change changes nothing.** Every error path of a group
  mutation is asserted to leave the group, and its totals, exactly as they
  were.
- **The public API is tested as a caller uses it.** Integration tests and
  examples use only `eranos::…`; unit tests reach private items.

## 2. Layout

| Suite       | Location                                | Reaches                               | Runs with                                                |
| ----------- | --------------------------------------- | ------------------------------------- | -------------------------------------------------------- |
| Unit tests  | `crates/<crate>/src/<module>/tests.rs`  | private items (`use super::*`)        | `cargo test --workspace --lib`                           |
| Scenarios   | `crates/eranos/tests/scenarios.rs`      | public API                            | `cargo test -p eranos --test scenarios`                  |
| Properties  | `crates/eranos/tests/properties.rs`     | public API                            | `cargo test -p eranos --test properties`                 |
| Persistence | `crates/eranos/tests/persistence.rs`    | public API, feature `serde`           | `cargo test -p eranos --all-features --test persistence` |
| Errors      | `crates/eranos/tests/errors.rs`         | public API: every error message       | `cargo test -p eranos --test errors`                     |
| API         | `crates/eranos/tests/api.rs`            | public API: every type `Send + Sync`  | `cargo test -p eranos --test api`                        |
| Doctests    | every `# Examples` block and the README | public API                            | `cargo test --workspace --doc`                           |
| Examples    | `crates/eranos/examples/*.rs`           | public API; they assert their numbers | `cargo run -p eranos --example <name>`                   |
| Benchmarks  | `crates/eranos/benches/*.rs`            | public API                            | `cargo bench -p eranos --features serde --bench '*'`     |

Unit tests live in a sibling file: eranos-money's `allocation.rs` ends with
`#[cfg(test)] mod tests;` and its tests are in `allocation/tests.rs`.
Visibility is never widened to make something testable.

The README is compiled as a doctest through a `#[cfg(doctest)]` item in
`lib.rs`, so its example cannot drift from the API.

## 3. The invariants

Property tests (proptest) carry the invariants; each has a hand-computed
companion that pins concrete numbers.

| Invariant                                                                                               | Where                                                                                                    |
| ------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| an allocation's parts add up to its total, each its exact share rounded down or up                      | `eranos-money/src/allocation/tests.rs`                                                                   |
| paid and consumed shares of an expense each add up to its total, in any currency                        | `eranos-expense/src/resolution/tests.rs`                                                                 |
| the pairwise debts of an expense net to each member's paid minus consumed                               | `eranos-expense/src/resolution/tests.rs`                                                                 |
| the exact solver finds the true minimum (checked against an exhaustive search)                          | `eranos-settlement/src/minimum/tests.rs`                                                                 |
| every plan clears every balance, pays only from debtors to creditors, and never overpays                | `eranos-settlement/src/minimum/tests.rs`, `eranos-settlement/src/greedy/tests.rs`, `tests/properties.rs` |
| greedy needs at most n − 1 payments; the exact plan never more than greedy                              | `eranos-settlement/src/greedy/tests.rs`, `tests/properties.rs`                                           |
| incremental totals equal a full rebuild after any sequence of changes; a refused change changes nothing | `eranos-group/src/tests.rs`                                                                              |
| paid, consumed, and spent agree; balances sum to zero; category totals add up                           | `tests/properties.rs`                                                                                    |
| the order entries arrive in does not change the group, its totals, or its plans                         | `tests/properties.rs`                                                                                    |
| removing an expense and adding it back restores the totals exactly                                      | `tests/properties.rs`                                                                                    |
| a group round-trips through JSON unchanged                                                              | `tests/persistence.rs`                                                                                   |

## 4. Writing a test here

### The doc comments are not optional

Every test module starts with a `//!` doc saying what its tests protect,
and every test has a `///` doc stating what breaks in production if it
regresses, not what the code does. Inside a test, each group of assertions
gets a `//` line naming the case.

### Naming

Names are claims in snake_case, never `test_*`:

| Shape                    | Use                        | Examples                                                                                        |
| ------------------------ | -------------------------- | ----------------------------------------------------------------------------------------------- |
| `a_*` / `an_*` / `the_*` | a declarative claim        | `a_single_payer_is_owed_every_other_share`, `the_rotation_is_fnv1a_over_the_entry_and_the_pass` |
| `every_*`                | an exhaustive guard        | `every_registry_example_validates`, `every_strategy_pays_in_the_group_currency_in_order`        |
| `*_round_trips*`         | encode and decode symmetry | `a_group_round_trips_through_json`                                                              |
| a negative boundary      | the other half of a claim  | `a_zero_weight_never_receives_a_leftover_unit`, `a_tampered_file_is_refused`                    |

### Helpers and fixtures

Helpers are plain functions at the top of the test file (`member(1)`,
`eur("12.00")`, `day()`). An entity gets a time-based ID when it is built;
the tie rotation depends on an expense's ID, so a unit test that pins
which member receives a leftover cent builds it with the test-only
`ExpenseBuilder::with_id`. An integration test sees only the public API:
it builds its entries once and clones them, and a base group, wherever
two groups must be equal.

### Lints in tests

Tests are held to the same strict clippy lints as the library. The test
exceptions in `clippy.toml` allow unwrapping, expecting, panicking, and indexing inside `#[test]`
functions and `#[cfg(test)]` modules; integration-test helpers outside
those declare the same exception once, at the top of their file, with
`#![expect(..., reason = "...")]`.

### What not to do

- No `#[ignore]`: a test that does not run protects nothing.
- No `#[should_panic]`: the library returns errors; assert the error.
- No snapshot tests: accepting a wrong change becomes one command away.
- No floating point anywhere near money, in tests either.

## 5. Coverage

Coverage merges a run with every feature and one without the default
features ([DEVELOPMENT.md § 5](DEVELOPMENT.md#5-coverage-and-mutation-testing)
has the commands), because each compiles code the other leaves out: the
serde code, and either the clock-based identifier generator or the
`sequential-ids` counter. It measures lines and regions for the unit,
integration, and property tests. It does not count
doctests on the stable toolchain, so an item covered only by its example
shows as missed. Coverage is a diagnostic, never a gate; a gap is a reading
list: either a missing test or dead code. CI measures the same way on every
push and publishes the result as a page:
[DEVELOPMENT.md § 6](DEVELOPMENT.md#6-continuous-integration).

The suites cover every line, region, and function. Keeping it that way
follows three rules:

- **A reachable branch gets a test**, however extreme its input: an amount
  near `i64::MAX`, a crafted file, 63 balances for the exact solver's
  out-of-memory path. The ledger's range checks are tested directly with
  hand-built deltas, because no group entry can push some of them out of
  range.
- **A branch that cannot run is removed, not kept as a fallback.** A
  function whose failure is impossible becomes infallible (`allocate`), and
  a loop ends on its own condition instead of through a fallback that never
  runs (the exact solver's walk back). A lossless integer conversion falls
  back with `unwrap_or_default()` and a comment saying why it cannot fail.
- **Generic functions stay small.** A generic function is measured per
  instantiation and the report keeps the best single one, so logic that
  every instantiation shares lives in a non-generic function (the replay
  behind `Deserialize for Group`).

## 6. Mutation testing

`cargo mutants --file <file>` changes the code one operator at a time and
runs every crate's tests against each change; a mutant that survives is a
change no test noticed. It complements coverage: coverage says a line ran,
mutation testing says a test would have noticed it was wrong. It is an
occasional audit, run one file at a time (`.cargo/mutants.toml` sets the
workspace's tests, every feature, and a timeout floor); the arithmetic in
`eranos-money/src/allocation.rs`, `eranos-expense/src/resolution.rs`, and
`eranos-settlement/src/minimum.rs` is where survivors matter most. Every feature
includes `sequential-ids`, so an audit of `eranos-core` also runs once with
`--no-default-features` to reach the clock-based generator.

## 7. Benchmarks

`benches/settlement.rs` times every strategy on groups of growing size (the
exact solver's worst case: every member pays one expense everyone shares),
`benches/ledger.rs` times adding, replacing, and removing one expense in a
group of 10 000, and `benches/group.rs` times a currency change and loading
a saved group (feature `serde`) with 1 000 and 10 000 expenses, the
operations that touch every entry. The timings quoted on
`SettlementStrategy` and in
[ARCHITECTURE § 12](ARCHITECTURE.md#12-key-numbers) come from them; update
both together.

## 8. What to run for a given change

| Change                            | Run                                                                                                                                |
| --------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| anything                          | the gates in [AGENTS.md](../AGENTS.md#gates-before-committing)                                                                     |
| rounding, allocation, conversion  | the gates, then `cargo mutants --file crates/eranos-money/src/allocation.rs`                                                       |
| a settlement strategy             | the gates, then `cargo bench -p eranos --bench settlement`                                                                         |
| the ledger or group mutations     | the gates, then `cargo bench -p eranos --bench ledger`                                                                             |
| a currency change or loading      | the gates, then `cargo bench -p eranos --features serde --bench group`                                                             |
| serialised shapes                 | `cargo test -p eranos --all-features --test persistence`, and update [VERSIONING.md](VERSIONING.md) if older files stop loading    |
| a libphonenumber metadata release | the gates, then `cargo test -p eranos-contact --lib` ([DEVELOPMENT.md § 3.8](DEVELOPMENT.md#38-a-libphonenumber-metadata-release)) |
| docs or README                    | `cargo test --workspace --doc` and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`                     |
