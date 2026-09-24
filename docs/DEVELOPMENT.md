# Development

Every command and change recipe. The conventions and gates are in
[AGENTS.md](../AGENTS.md), the design in [ARCHITECTURE.md](ARCHITECTURE.md),
and the test suites in [TESTING.md](TESTING.md).

## Contents

1. [Prerequisites](#1-prerequisites)
2. [Building and testing](#2-building-and-testing)
3. [Making a change](#3-making-a-change)
4. [Examples and benchmarks](#4-examples-and-benchmarks)
5. [Coverage and mutation testing](#5-coverage-and-mutation-testing)
6. [Continuous integration](#6-continuous-integration)
7. [Troubleshooting](#7-troubleshooting)

## 1. Prerequisites

- **rustup**, from [rustup.rs](https://rustup.rs). `rust-toolchain.toml` pins
  the stable channel with `rustfmt`, `clippy`, and `llvm-tools` (for
  coverage); the first `cargo` command in the repository installs them.
- **Homebrew tools**: `brew bundle --file Brewfile.develop` installs
  `cargo-llvm-cov` and `cargo-mutants`. Neither is needed to build or test.
- **VS Code** (optional): the workspace recommends `rust-analyzer`, CodeLLDB,
  and the Markdown and TOML extensions; the `gates` task runs every gate.

## 2. Building and testing

```sh
cargo xtask gates                                # every commit gate, in order
cargo build -p eranos                            # the library
cargo build -p eranos --features serde           # with serialisation
cargo test --workspace --all-features            # every suite, doctests included
cargo test -p eranos-money --lib allocation      # one module's unit tests
cargo test -p eranos --test scenarios            # one integration suite
cargo doc -p eranos --all-features --open        # the API documentation
```

`cargo xtask gates` runs the full gate sequence, listed in
[AGENTS.md § Gates](../AGENTS.md#gates-before-committing), before every
commit; `cargo xtask help` lists the commands. The alias lives in
`.cargo/config.toml`, and the automation in `crates/xtask`.

## 3. Making a change

### 3.1 A new split method

1. Add the variant to `Split` in `crates/eranos-expense/src/split.rs`; the
   enum is `#[non_exhaustive]`, so this is a `feat`.
2. Give it a constructor next to `Split::equal`, its rules in
   `Split::validate`, and its weights in `Split::divide`. Every division
   must go through `allocate`; a method that cannot be expressed as weights
   needs its own exact division that still adds up to the total. `divide`
   trusts `validate`: an empty split divides into no shares and all-zero
   weights split equally, so `validate` must refuse both.
3. Pin it with a hand-computed case in `eranos-expense/src/split/tests.rs`
   and in `tests/scenarios.rs`, and add it to the random recipes in
   `tests/properties.rs` so the invariants cover it.
4. Document it on the variant and in ARCHITECTURE § 3.

### 3.2 An ISO 4217 amendment

The codes, minor units, and names come from `iso4217-static`, pinned
exactly in `Cargo.toml` and generated from the list SIX Group publishes at
<https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml>.
`crates/eranos-money/src/currency/iso4217.rs` lists, in code order, the
codes that get a constant.

1. Move the exact pin to the release that carries the amendment, and build
   without `--locked` so `Cargo.lock` follows.
2. `cargo test -p eranos-money --lib currency`:
   `all_matches_every_iso4217_static_code_with_a_minor_unit` names the codes
   to add to the list, `decimal_places_follow_iso_4217` names a changed
   minor unit, and a code the release dropped fails the build at its
   constant.
3. Update the list, `CURRENCY_COUNT`, the pinned decimal places, and the
   publication date in [ARCHITECTURE.md § 4](ARCHITECTURE.md#4-money-and-rounding).

An added code is a `feat`. A removed code removes a public constant, and a
changed number of decimal places changes the meaning of stored amounts;
both are breaking ([VERSIONING.md](VERSIONING.md)), and saved groups in a
removed currency stop loading.

### 3.3 An IBAN registry update

The countries, their IBAN lengths and BBAN structures, and the registry's
example IBANs are in `crates/eranos-account/src/iban/registry.rs`, generated
from the SWIFT IBAN Registry text file (linked from
<https://www.swift.com/standards/data-standards/iban-international-bank-account-number>;
swift.com answers 403 unless the request carries browser-like headers).

The file is tab-separated and transposed: one row per data element, one
column per country after the label. Row 3 holds the country code, row 9 the
BBAN structure, row 20 the IBAN length, and row 22 the example in electronic
format; rows 27 onwards hold contact details in multi-line quoted cells, so
read only the first 26 rows. The lines end in CRLF, and without stripping
the carriage return the last country's cells keep it.

1. Regenerate `COUNTRIES` and `EXAMPLES`, in code order, and update the
   release number in the header.
2. Check each changed structure against the PDF of the same release. The
   text file has had errors: release 98 left out PK, release 99 gave IQ's
   positions as spreadsheet dates, and release 103 gives CZ one BBAN
   structure (`4!n16!n`, used here) and a differently split IBAN structure.
3. `cargo test -p eranos-account --lib iban` validates every example
   against its length, its structure, and the check digits, and checks that
   every structure adds up to its country's length.

A country added is a `feat`. A length or a structure changed can refuse
stored IBANs, and saved groups that hold one stop loading, so it is
breaking.

### 3.4 A new settlement strategy

1. Add the variant to `SettlementStrategy` with a doc comment stating its
   algorithm, cost, guarantee, and a measured timing.
2. Implement it as `pub(super) fn settle(…) -> Vec<Flow>` in a new
   `crates/eranos-settlement/src/<name>.rs`, reading the ledger's balances or
   pair debts, and dispatch to it in the crate's `settle`.
3. Add its cache slot to `Settlements` beside it (the `match` in
   `Settlements::slot` is exhaustive, so the compiler points at it).
4. Add it to `SettlementStrategy::all()`; the property tests then check that
   its plans clear every balance.
5. Benchmark it in `benches/settlement.rs` and record the timing in
   ARCHITECTURE § 12.

### 3.5 A new field on an entity

1. Add the private field, a getter, and a builder setter; an optional field
   takes `impl Into<Option<T>>` so callers can clear it, a field with a
   default takes the value itself and its default clears it (a category, a
   tip), and a text field trims and treats blank as absent. The field goes
   where the field order of [AGENTS.md](../AGENTS.md) puts it, in the
   builder and the `*Record` struct too, and its getter and setter go where
   the accessor order puts them.
2. Validate it in the builder's `build()`, or in `Group::check_*` when the
   rule depends on the group.
3. With `serde`: add `#[serde(skip_serializing_if = …)]` for an optional
   field, and the field with `#[serde(default)]` to the type's `*Record`
   struct, so data written before the field existed still loads.
4. Pin the serialised shape in `tests/persistence.rs`.

### 3.6 A new error variant

Error enums are `#[non_exhaustive]`, so a variant is a `feat`. The variant
joins its crate's enum in that crate's `error.rs`, in alphabetical order;
its message is lowercase, has no trailing full stop, and never repeats the
rejected text.
Pin the message in its enum's list in `tests/errors.rs`, and name the variant
in the `# Errors` section of every function that can return it.

### 3.7 A lint exception

Fix the finding when there is any way to. When the code is right and the
lint is not, add `#[expect(clippy::lint_name, reason = "...")]` on the
narrowest item, with a reason that states the invariant. A lint that
contradicts a convention everywhere is removed from `[workspace.lints]` in
`Cargo.toml`, with the reason next to the table. A duplicate crate that
`multiple_crate_versions` reports, and that only a dependency's release can
remove, goes into `allowed-duplicate-crates` in `clippy.toml`, with the
dependency, the cause, and the upstream issue; it comes out when that
release lands.

### 3.8 A libphonenumber metadata release

`rlibphonenumber` embeds libphonenumber's numbering plans and is pinned
exactly in `Cargo.toml`. It follows each libphonenumber release within days
(<https://github.com/vloldik/rlibphonenumber/releases>); Dependabot opens
only security updates, so check for a new one by hand.

1. Move the exact pin, run `cargo update -p rlibphonenumber`, and read the
   `Cargo.lock` diff: a new crate in two versions is a decision for
   [§ 3.7](#37-a-lint-exception), not a routine update.
2. Run the gates; `every_regions_example_numbers_are_accepted` compiles
   every region's patterns under `lite`, where a pattern the regex engine
   cannot compile shows up.
3. Raise the minor version and commit as `build(member)!:`, with a
   `BREAKING CHANGE:` footer naming the metadata versions: a withdrawn range
   refuses stored numbers ([VERSIONING.md](VERSIONING.md)).

## 4. Examples and benchmarks

```sh
cargo run -p eranos --example quickstart
cargo run -p eranos --features serde --example persistence
cargo bench -p eranos                               # everything; minutes
cargo bench -p eranos --bench settlement -- --quick # a fast sanity pass
cargo bench -p eranos --bench ledger -- "replace"   # one benchmark by name
```

Benchmarks run the release profile (fat LTO), so the first run compiles for
a while. Timings are wall-clock: run them on an idle machine before quoting
them.

## 5. Coverage and mutation testing

```sh
cargo llvm-cov clean --workspace                              # drop earlier runs
cargo llvm-cov --no-report --workspace --all-features         # sequential IDs, serde
cargo llvm-cov --no-report --workspace --no-default-features  # clock-based IDs
cargo llvm-cov report --html --open                           # HTML report
cargo llvm-cov report --summary-only                          # totals only
cargo mutants --file crates/eranos-money/src/allocation.rs
```

Coverage merges the two runs because each compiles code the other leaves
out: the serde code, and one of the two identifier generators. A single
`--all-features` run reports the clock-based generator as absent rather
than missed, so its total would still read 100 %.

Coverage does not count doctests on the stable toolchain, so a function
covered only by its example shows as missed. Mutation testing takes seconds
per mutant; run it one file at a time. Reading both: [TESTING.md](TESTING.md).

## 6. Continuous integration

`.github/workflows/ci.yml` runs on every branch push, on pull requests from
forks, on the merge queue, and on a manual dispatch. A pull request from a
branch of this repository starts no jobs of its own: the push of its head
commit was tested, and those checks show on the pull request.

| Job                   | What it runs                                                                                                   |
| --------------------- | -------------------------------------------------------------------------------------------------------------- |
| Lint                  | `cargo fmt --check`, clippy for both feature sets, and rustdoc, with warnings denied                           |
| Test                  | the library build, every test suite for both feature sets, every example, and each benchmark once in test mode |
| MSRV                  | the tests with Rust 1.97, the `rust-version`                                                                   |
| Mobile targets        | clippy for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, and `x86_64-linux-android`   |
| Dependency audit      | `cargo deny check advisories sources`, with the rules in `deny.toml`                                           |
| Workflow lint         | actionlint and zizmor, configured in `.github/zizmor.yml`                                                      |
| Coverage              | both feature sets, merged into the coverage page and its summary, below                                        |
| Test (macOS, Windows) | the tests on `macos-26` and `windows-2025`, on a manual dispatch only                                          |

The repository is private on the GitHub Free plan: 2,000 Actions minutes a
month, with macOS minutes counting about ten times and Windows minutes about
twice. Every job is billed in whole minutes, so a push costs about 7 (10
while the caches are cold) and a run with macOS and Windows about 25. Those
two jobs run only when dispatched from the Actions tab or with
`gh workflow run ci.yml --ref main`. When the minutes run out, runs stop
until the next month; nothing is billed. The plan has no branch protection,
so a failing run blocks nothing: it is a report.

- **The coverage page.** The Coverage job uploads `eranos-coverage.html`,
  the annotated source with line counts, as one file that opens in the
  browser for anyone signed in with access to the repository. The run's
  summary links to it and adds a table of regions, functions, and lines per
  file from cargo-llvm-cov. Pages are kept for 7 days.
- **Dependabot** opens security-update pull requests only
  (`.github/dependabot.yml`). They need Dependabot alerts and Dependabot
  security updates switched on under Settings, Advanced Security. Its
  commits carry no `Signed-off-by`, so squash-merge them with a signed-off
  message.
- **Changing the workflow.** Actions are referenced by release tag. Before
  pushing a change, run `actionlint` and `zizmor --offline .github/` (both
  are in Homebrew); CI runs the same two checks.

## 7. Troubleshooting

- **`the README code blocks fail as doctests`**: every fence in README.md
  needs a language tag; an untagged fence is compiled as Rust.
- **`unfulfilled lint expectation`**: an `#[expect]` whose lint no longer
  fires; delete it, because the reason it states is gone.
- **`multiple versions for dependency`**: a new dependency brings a second
  version of a crate; see [§ 3.7](#37-a-lint-exception).
- **Benchmarks take minutes to start**: the release profile's fat LTO;
  `-- --quick` shortens the measurements, not the build.
- **`cargo test --no-default-features` fails where `--all-features`
  passes**: a `use` needed only by serde code sits outside its
  `#[cfg(feature = "serde")]`.
