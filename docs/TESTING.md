# Testing

What the test suites check, where they live, and how to write one.
[AGENTS.md](../AGENTS.md) holds the gates, [DEVELOPMENT.md](DEVELOPMENT.md)
every command.

## Contents

- [Testing](#testing)
  - [Contents](#contents)
  - [1. What the suite believes](#1-what-the-suite-believes)
  - [2. Layout](#2-layout)
  - [3. The invariants](#3-the-invariants)
  - [4. Writing a test here](#4-writing-a-test-here)
    - [The doc comments are not optional](#the-doc-comments-are-not-optional)
    - [Naming](#naming)
    - [Helpers and fixtures](#helpers-and-fixtures)
    - [Lints in tests](#lints-in-tests)
    - [Asynchronous code](#asynchronous-code)
    - [What not to do](#what-not-to-do)
  - [5. Coverage](#5-coverage)
  - [6. Mutation testing](#6-mutation-testing)
  - [7. Benchmarks](#7-benchmarks)
  - [8. What to run for a given change](#8-what-to-run-for-a-given-change)

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
- **A dropped call changes nothing.** Each asynchronous call is dropped at
  its yields in a test that finds the group, its totals, its cache, and its
  lock as they were; the binding core's calls are cancelled the same way,
  while they wait for the lock and at their yields.
- **The public API is tested as a caller uses it.** Integration tests and
  examples use only `eranos::…`; unit tests reach private items.

## 2. Layout

| Suite                | Location                                                    | Reaches                                                    | Runs with                                                |
| -------------------- | ----------------------------------------------------------- | ---------------------------------------------------------- | -------------------------------------------------------- |
| Unit tests           | `crates/<crate>/src/<module>/tests.rs`                      | private items (`use super::*`)                             | `cargo test --workspace --lib`                           |
| Scenarios            | `crates/eranos/tests/scenarios.rs`                          | public API                                                 | `cargo test -p eranos --test scenarios`                  |
| Properties           | `crates/eranos/tests/properties.rs`                         | public API                                                 | `cargo test -p eranos --test properties`                 |
| Persistence          | `crates/eranos/tests/persistence.rs`                        | public API, feature `serde`                                | `cargo test -p eranos --all-features --test persistence` |
| Errors               | `crates/eranos/tests/errors.rs`                             | public API: every error message                            | `cargo test -p eranos --test errors`                     |
| API                  | `crates/eranos/tests/api.rs`                                | public API: every type `Send + Sync`                       | `cargo test -p eranos --test api`                        |
| Doctests             | every `# Examples` block and the README                     | public API                                                 | `cargo test --workspace --doc`                           |
| Examples             | `crates/eranos/examples/*.rs`                               | public API; they assert their numbers                      | `cargo run -p eranos --example <name>`                   |
| Benchmarks           | `crates/eranos/benches/*.rs`                                | public API                                                 | `cargo bench -p eranos --features serde --bench '*'`     |
| Swift                | `bindings/swift/Tests/EranosTests/*.swift`                  | the `Eranos` module, and the generated code's values       | `swift test --package-path bindings/swift`               |
| Kotlin               | `bindings/kotlin/eranos/src/test/kotlin/`                   | the `eranos` package, and the generated code's values      | `./gradlew :eranos:test`, in `bindings/kotlin`           |
| Java                 | `bindings/java/src/test/java/eranos/`                       | the `eranos` package, and the generated code's values      | `./gradlew test`, in `bindings/java`                     |
| TypeScript           | `bindings/typescript/test/*.test.ts`                        | the package's entry point, and the generated code's values | `npm test`, in `bindings/typescript`                     |
| Kotlin Multiplatform | the Kotlin suite, `bindings/kotlin/eranos/src/test/kotlin/` | the `eranos` package over gobley's bindings, on the JVM    | `./gradlew :eranos:jvmTest`, in `bindings/kmp`           |
| C                    | `bindings/c/tests/tests.c`                                  | the `eranos.h` layer, and Diplomat's C API beneath it      | `build/tests`, in `bindings/c`, once built               |
| C++                  | `bindings/cpp/tests/tests.cpp`                              | the `eranos.hpp` layer, and Diplomat's classes beneath it  | `build/tests`, in `bindings/cpp`, once built             |
| Python               | `bindings/python/tests/test_*.py`                           | the `eranos` package, and the generated module's values    | `uv run pytest`, in `bindings/python`                    |
| Ports                | `bindings/<language>/examples/`                             | every example's report, byte for byte, through the layer   | `cargo xtask gates`, gate 19                             |

Unit tests live in a sibling file: eranos-money's `allocation.rs` ends with
`#[cfg(test)] mod tests;` and its tests are in `allocation/tests.rs`.
Visibility is never widened to make something testable.

The README is compiled as a doctest through a `#[cfg(doctest)]` item in
`lib.rs`, so its example cannot drift from the API.

Every example of `crates/eranos/examples` has a port in each binding's
`examples` package that follows it section by section, through the
language's hand-written layer alone, and prints its report byte for byte.
Every text of a report is one the library writes
([BINDINGS.md § 5](BINDINGS.md#5-the-text-a-binding-shows)), so a port that
prints another byte is a layer that shows a value otherwise, or a port that
no longer follows its example. A port makes an entry with its language's
generating form where its example calls a constructor with the plain
name, and with the form that takes the identifier where its example calls
a `with_id` twin, so it generates identifiers in the example's order.
Gate 9 keeps each example's report; gate 19 runs every port on a build of
the glue with `sequential-ids` in `target/sequential`, whose identifiers
count up as the example's do, and stops at the first line that differs,
printing both versions of it. The
Swift ports run on macOS, on a copy of the Swift package over the host
library, since the identifiers of the package's own XCFramework are random;
the Kotlin and Java ports run on every machine, on the JVM, loading that
library from `target/sequential/debug`, and the Kotlin ones again on the
Kotlin Multiplatform package's JVM target, over gobley's bindings, where JNA
loads eranos-kmp's library, built beside it, before the one gobley embeds.
The TypeScript ports run on Node, in a copy of the TypeScript package whose
generated TypeScript comes from eranos-web built with the feature for
WebAssembly, since wasm-bindgen's glue follows the module it is written for.
The C and C++ ports build as their packages' tests do, under the
sanitizers, on eranos-capi's static library built with the feature, and run
with `LeakSanitizer`, which `AddressSanitizer` runs by default on Linux
alone, so a port that leaks fails on macOS too. The Python ports run in a
copy of the Python package in `target/sequential/python`, over the module
generated from eranos-uniffi built with the feature and a link to that
library, with `PYTHONUTF8=1`, so they write UTF-8 whatever the locale. A
test in `crates/xtask` holds every example to a port in every language the
ports cover.

## 3. The invariants

Property tests (proptest) carry the invariants; each has a hand-computed
companion that pins concrete numbers.

| Invariant                                                                                                                                                     | Where                                                                                                    |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| identifiers order as the bytes of their text, and lowercase hyphenated UUID text as the UUID's bytes, so generated identifiers keep creation order            | `eranos-core/src/tests.rs`                                                                               |
| an allocation's parts add up to its total, each its exact share rounded down or up                                                                            | `eranos-money/src/allocation/tests.rs`                                                                   |
| an item's line total is its unit price times its quantity: exact for a whole quantity, rounded half away from zero for a fractional one                       | `eranos-money/src/allocation/tests.rs`, `eranos-expense/src/expense_item/tests.rs`                       |
| an item's discount takes off between nothing and its whole line, and the line total and the discount add up to the line total before it                       | `eranos-expense/src/expense_item/tests.rs`                                                               |
| paid and consumed shares of an expense each add up to its total, in any currency                                                                              | `eranos-expense/src/resolution/tests.rs`                                                                 |
| the pairwise debts of an expense net to each member's paid minus consumed                                                                                     | `eranos-expense/src/resolution/tests.rs`                                                                 |
| an equal tip goes to every participant with a share of the amount, as evenly as the minor unit allows, and to no other                                        | `eranos-expense/src/resolution/tests.rs`                                                                 |
| the exact solver finds the true minimum (checked against an exhaustive search)                                                                                | `eranos-settlement/src/minimum/tests.rs`                                                                 |
| every plan clears every balance, pays only from debtors to creditors, and never overpays                                                                      | `eranos-settlement/src/minimum/tests.rs`, `eranos-settlement/src/greedy/tests.rs`, `tests/properties.rs` |
| greedy needs at most n − 1 payments, never more than the largest-first plan alone, and at most 3/2 of the exact plan's; the exact plan never more than greedy | `eranos-settlement/src/greedy/tests.rs`, `tests/properties.rs`                                           |
| the adaptive plan is the exact one while at most its limit of balances is left after pairing and greedy's beyond, named by the strategy that made it          | `eranos-settlement/src/adaptive/tests.rs`, `tests/properties.rs`                                         |
| no two balances left after pairing exact opposites are opposites, and every pair owes exactly what it is owed                                                 | `eranos-settlement/src/opposites/tests.rs`                                                               |
| incremental totals equal a full rebuild after any sequence of changes; a refused change changes nothing                                                       | `eranos-expense-group/src/tests.rs`                                                                      |
| paid, consumed, and spent agree; balances sum to zero; category totals add up                                                                                 | `tests/properties.rs`                                                                                    |
| the order entries arrive in does not change the group, its totals, or its plans                                                                               | `tests/properties.rs`                                                                                    |
| a plan or a currency change awaited in an executor equals the one made on the caller's thread                                                                 | `tests/properties.rs`                                                                                    |
| removing an expense and adding it back restores the totals exactly                                                                                            | `tests/properties.rs`                                                                                    |
| a group round-trips through JSON unchanged                                                                                                                    | `tests/persistence.rs`                                                                                   |

## 4. Writing a test here

### The doc comments are not optional

Every test module starts with a `//!` doc saying what its tests protect,
and every test has a `///` doc stating what breaks in production if it
regresses, not what the code does; in Python, the module's docstring and
the test's say the same. Inside a test, each group of assertions gets a
`//` line naming the case.

### Naming

Names are claims in snake_case, never `test_*`, but in the Python package,
where pytest collects a test by that prefix and the claim follows it
(`test_a_member_in_use_is_not_removed`):

| Shape                    | Use                        | Examples                                                                                        |
| ------------------------ | -------------------------- | ----------------------------------------------------------------------------------------------- |
| `a_*` / `an_*` / `the_*` | a declarative claim        | `a_single_payer_is_owed_every_other_share`, `the_rotation_is_fnv1a_over_the_entry_and_the_pass` |
| `every_*`                | an exhaustive guard        | `every_registry_example_validates`, `every_strategy_pays_in_the_group_currency_in_order`        |
| `*_round_trips*`         | encode and decode symmetry | `a_group_round_trips_through_json`                                                              |
| a negative boundary      | the other half of a claim  | `a_zero_weight_never_receives_a_leftover_unit`, `a_tampered_file_is_refused`                    |

### Helpers and fixtures

Helpers are plain functions at the top of the test file (`member(1)`,
`eur("12.00")`, `day()`). A test names members, categories, and entries by
identifiers it builds from text with `new`, such as `member-001` or `ana`,
whose byte order is the order the test means. An entity's constructor
with the plain name generates its ID, and its `with_id` twin takes one
built with `new`; the tie rotation hashes the text of an expense's ID, so a
test that pins which member receives a leftover cent gives the expense an
ID built with `new`, through `Expense::builder_with_id` or
`ExpenseBuilder::id`. An integration test sees only the public API: it
builds its entries once and clones them, and a base group, wherever two
groups must be equal.

### Lints in tests

Tests are held to the same strict clippy lints as the library. The test
exceptions in `clippy.toml` allow unwrapping, expecting, panicking, and indexing inside `#[test]`
functions and `#[cfg(test)]` modules; integration-test helpers outside
those declare the same exception once, at the top of their file, with
`#![expect(..., reason = "...")]`.

### Asynchronous code

A test polls a future by hand: with `Waker::noop()` when only the pending
polls count, and with a counting waker when a wake-up must happen, as in
`eranos-expense-group/src/shared/tests.rs`. It pins where a computation yields
(the exact solver's table of 16 balances fills in one poll and that of 17
in two; a currency change of 2 100 entries yields twice) and drops the
future at each yield. A test that needs another thread gives it 50 ms and
bounds the number of polls instead of timing anything: a driver that spun
would poll thousands of times.

### What not to do

- No `#[ignore]`: a test that does not run protects nothing.
- No `#[should_panic]`: the library returns errors; assert the error.
- No snapshot tests: accepting a wrong change becomes one command away.
- No floating point anywhere near money, in tests either: a float appears
  only as the input of a `from_f64` constructor, whose tests pin the
  conversion.

## 5. Coverage

Coverage merges a run with every feature and one without the default
features ([DEVELOPMENT.md § 5](DEVELOPMENT.md#5-coverage-and-mutation-testing)
has the commands), because each compiles code the other leaves out: the
serde code, and either the clock-based identifier generator or the
`sequential-ids` counter. It measures lines and regions for the unit,
integration, and property tests of every crate but the glue crates and the
generators, which have no Rust tests: the glue forwards every call to the
binding core. eranos-glue's own tests compare the code it writes with the
code expected and measure it, and eranos-glue-macros, whose macros run
inside the compiler only when a crate that invokes them compiles, is left
out of the runs and of the report, which skips its files: the runs build it
instrumented all the same, and a warm build would read its macros as unrun.
rustc maps no coverage for the code a proc macro writes, so the group
handle's and the read handle's forms are not in the report either; the
operations and the locks they call are, and the core's tests call every
form. It does not count doctests on the stable toolchain, so an
item covered only by its example shows as missed. Coverage is a diagnostic,
never a gate; a gap is a reading list: either a missing test or dead code.
CI measures the Rust tests the same way on every push and publishes the
result as a page: [DEVELOPMENT.md § 6](DEVELOPMENT.md#6-continuous-integration).

The glue is measured through the foreign tests instead:
`cargo xtask coverage` builds eranos-uniffi and eranos-capi with the
instrumentation of the Rust runs, runs the Swift, Kotlin, Java, Python, C,
and C++ tests on that build, and merges what they ran into the same report.
The Swift, Kotlin, Java, and Python tests run every line, region, and
function of eranos-uniffi's hand-written sources, and the C and C++ tests
every line and function of eranos-capi's bridge, whose generated modules
are files like any other: eranos-glue's tests measure the generator that
writes them, and a test of xtask holds the committed files to what it
writes. The objects' methods and the copies of the core's items that the
glue's macros stamp are not in the report, as the core's forms are not. One
kind of the bridge's regions stays unrun: the check that a text parameter
is UTF-8, which only a call given other bytes takes and which would need a
test for each function. The synchronous form of a call that the core marks
as one that cannot fail has no failure branch to leave unrun: the core
refuses such a call only on wasm, for which the bridge is not built. Only
the Swift layer passes the UniFFI glue a `Cancellation`, so a run off
macOS, where the Swift tests do not run, reports its two methods unrun.
eranos-web and eranos-kmp compile eranos-uniffi's sources again and are
not measured: stable Rust does not measure WebAssembly, and gobley builds
its own library.

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
  behind `Deserialize for ExpenseGroup`).

The Swift layer is held to the same rules by its own tests: `swift test
--enable-code-coverage` reports every line, region, and function of the
`Eranos` module, and the generated module is not measured. The layer
holds no fallback for a value it prints: a percentage, parts, a date, and
a time print through the core's formatters, which cannot fail; an amount
prints from its currency's details; and a phone number and an IBAN print
the text the core derived for them, which they hold.

The Kotlin layer's tests hold it to every line and every method, which
Kover measures and `koverVerify` enforces; the generated module is not
measured. Branches are not held to 100 %: every class writes its
equality out as a chain of `&&`, a branch per field, and the tests
compare each class with an equal value and with one that differs, not
with one per field. That comparison, `assertValue`, checks a class's
equality, hash, and text together, and the error test converts every
failure the generated code can report.

The Java layer's tests hold it to every instruction, line, and method,
which JaCoCo measures and `jacocoTestCoverageVerification` enforces, with
the generated source set left out; branches are not held to 100 %, for
the reason the Kotlin layer's are not. Records write their equality
themselves, and JaCoCo leaves those methods out; the classes that write
theirs out go through the same `assertValue`.

The Kotlin Multiplatform package runs the Kotlin layer's tests on its JVM
target, against gobley's bindings, and measures nothing: its layer is the
Kotlin package's, whose coverage Kover holds.

The TypeScript layer's tests hold it to every line, branch, function, and
statement, which c8 reads from Node's coverage and enforces through
`.c8rc.json`; the generated code in `src/internal` is not measured. Its
branches are held too, since its entries are interfaces that the tests
compare whole, with no equality written out field by field. The tests
import what callers use through the package's entry point, `src/index.ts`,
and reach an internal conversion, such as that of every failure the
generated code can report, through its own module.

The Python layer's tests hold it to every line and branch, which
coverage.py measures and enforces through `fail_under = 100` in
`pyproject.toml`; the generated module is not measured. The last arm of an
exhaustive `match`, `case _: assert_never(value)`, which ty proves no value
reaches, is left out through `exclude_also`. A conversion from the
generated classes goes through a table keyed by the generated class, so no
`match` over them needs an arm that never runs, and
`tests/test_conversions.py` checks each table's keys against the cases the
generated module writes and converts every failure it can report.

The C and C++ layers are headers of inline functions, which the tests'
programs compile; llvm-cov reports an inline function that no test calls
as unrun, so a function a layer defines and nothing reaches shows. Their
tests run under `AddressSanitizer` and `UndefinedBehaviorSanitizer`, whose
first finding ends the run, and the gates hold each layer to every line
and every function of its headers; the headers Diplomat generates are not
measured. The C++ layer is also held to every region by its tests, since a
call's failure branches in three templates that every call shares:
`expectedOf`, which reads a synchronous call's result, `launch`, which
starts an asynchronous call or fails its future when an argument does not
convert, and `complete`, which reads an asynchronous call's outcome;
everything else chains through `std::expected`'s `and_then` and
`transform`, whose own branches are the standard library's. The C layer is
held to every region too: a synchronous call that the library refuses only
on wasm, where the C API does not run, returns what it reads without a
branch on its result, so each branch of the layer is one that a call can
take, and its tests take every one. Both suites follow the rules of
[§ 4](#4-writing-a-test-here) in their language: a test is a function named
for its claim, in camelCase, whose comment says what an app loses if it
regresses, and the program prints every claim that fails.

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
group of 10 000, and `benches/expense_group.rs` times a currency change and
loading a saved group (feature `serde`) with 1 000 and 10 000 expenses, the
operations that touch every entry. The timings of [ARCHITECTURE §
13](ARCHITECTURE.md#13-key-numbers) come from them; no comment or doc
comment quotes one.

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
| an example's report               | `cargo xtask examples <example>`, then the gates, whose gate 19 compares every port with it: change each port with the example     |
| a record or a call of the core    | `cargo xtask generate`, then the gates: until it runs, `the_committed_bridge_is_what_the_generator_writes` fails                   |
| docs or README                    | `cargo test --workspace --doc` and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`                     |
