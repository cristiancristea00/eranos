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
cargo build -p eranos                            # the library
cargo build -p eranos --features serde           # with serialisation
cargo test --workspace --all-features            # every suite, doctests included
cargo test -p eranos --lib money::allocation     # one module's unit tests
cargo test -p eranos --test scenarios            # one integration suite
cargo doc -p eranos --all-features --open        # the API documentation
```

The full gate sequence, run before every commit, is in
[AGENTS.md § Gates](../AGENTS.md#gates-before-committing).

## 3. Making a change

### 3.1 A new split method

1. Add the variant to `Split` in `crates/eranos/src/expense/split.rs`; the
   enum is `#[non_exhaustive]`, so this is a `feat`.
2. Give it a constructor next to `Split::equal`, its rules in
   `Split::validate`, and its weights in `Split::divide`. Every division
   must go through `allocate`; a method that cannot be expressed as weights
   needs its own exact division that still adds up to the total. `divide`
   trusts `validate`: an empty split divides into no shares and all-zero
   weights split equally, so `validate` must refuse both.
3. Pin it with a hand-computed case in `expense/split/tests.rs` and in
   `tests/scenarios.rs`, and add it to the random recipes in
   `tests/properties.rs` so the invariants cover it.
4. Document it on the variant and in ARCHITECTURE § 3.

### 3.2 An ISO 4217 amendment

The currency table is `crates/eranos/src/money/currency/iso4217.rs`, one
line per code, sorted by code: `CODE decimal_places "Name";`. SIX Group
publishes the current list at
<https://www.six-group.com/dam/download/financial-information/data-center/iso-currrency/lists/list-one.xml>.

1. Add, change, or remove the lines the amendment names. Keep codes with a
   numeric minor unit only; "N.A." codes are not currencies here.
2. Update the publication date in the file's header comment.
3. `cargo test -p eranos --lib money::currency` checks the order and the
   pinned decimal places.

Removing a code removes a public constant: that is a breaking change
([VERSIONING.md](VERSIONING.md)). A changed number of decimal places
changes stored amounts' meaning and is breaking too.

### 3.3 An IBAN registry update

The lengths and the registry's example IBANs are in
`crates/eranos/src/member/iban/registry.rs`, generated from the SWIFT IBAN
Registry text file (linked from
<https://www.swift.com/standards/data-standards/iban-international-bank-account-number>;
swift.com answers 403 unless the request carries browser-like headers).
Update both tables in the same order and the release number in the header;
the unit tests validate every example against the checksum and its length.
A country added is a `feat`; a length changed can reject stored IBANs and is
breaking.

### 3.4 A new settlement strategy

1. Add the variant to `SettlementStrategy` with a doc comment stating its
   algorithm, cost, guarantee, and a measured timing.
2. Implement it as `pub(super) fn settle(…) -> Vec<Flow>` in a new
   `settlement/<name>.rs`, reading the ledger's balances or pair debts, and
   dispatch to it in `settlement::settle`.
3. Add its cache slot to `Settlements` in `group.rs` (the `match` there is
   exhaustive, so the compiler points at it).
4. Add it to `SettlementStrategy::all()`; the property tests then check that
   its plans clear every balance.
5. Benchmark it in `benches/settlement.rs` and record the timing in
   ARCHITECTURE § 12.

### 3.5 A new field on an entity

1. Add the private field, a getter, and a builder setter; an optional field
   takes `impl Into<Option<T>>` so callers can clear it, a text field trims
   and treats blank as absent.
2. Validate it in the builder's `build()`, or in `Group::check_*` when the
   rule depends on the group.
3. With `serde`: add `#[serde(skip_serializing_if = …)]` for an optional
   field, and the field with `#[serde(default)]` to the type's `*Record`
   struct, so data written before the field existed still loads.
4. Pin the serialised shape in `tests/persistence.rs`.

### 3.6 A new error variant

Error enums are `#[non_exhaustive]`, so a variant is a `feat`. The message is
lowercase, has no trailing full stop, and never repeats the rejected text. Name
the variant in the `# Errors` section of every function that can return it.

### 3.7 A lint exception

Fix the finding when there is any way to. When the code is right and the
lint is not, add `#[expect(clippy::lint_name, reason = "...")]` on the
narrowest item, with a reason that states the invariant. A lint that
contradicts a convention everywhere is removed from `[workspace.lints]` in
`Cargo.toml`, with the reason next to the table.

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
cargo llvm-cov --workspace --all-features --html --open   # HTML report
cargo llvm-cov --workspace --all-features --summary-only  # totals only
cargo mutants -p eranos --file crates/eranos/src/money/allocation.rs
```

Coverage does not count doctests on the stable toolchain, so a function
covered only by its example shows as missed. Mutation testing takes seconds
per mutant; run it one file at a time. Reading both: [TESTING.md](TESTING.md).

## 6. Continuous integration

`.github/workflows/ci.yml` runs on every branch push, on pull requests from
forks, on the merge queue, and on a manual dispatch. A pull request from a
branch of this repository starts no jobs of its own: the push of its head
commit was tested, and those checks show on the pull request.

| Job | What it runs |
| --- | --- |
| Lint | `cargo fmt --check`, clippy for both feature sets, and rustdoc, with warnings denied |
| Test | the library build, every test suite for both feature sets, every example, and each benchmark once in test mode |
| MSRV | the tests with Rust 1.97, the `rust-version` |
| Mobile targets | clippy for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, and `x86_64-linux-android` |
| Dependency audit | `cargo deny check advisories sources`, with the rules in `deny.toml` |
| Workflow lint | actionlint and zizmor, configured in `.github/zizmor.yml` |
| Coverage | the coverage page and its summary, below |
| Test (macOS, Windows) | the tests on `macos-26` and `windows-2025`, on a manual dispatch only |

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
- **Benchmarks take minutes to start**: the release profile's fat LTO;
  `-- --quick` shortens the measurements, not the build.
- **`cargo test --no-default-features` fails where `--all-features`
  passes**: a `use` needed only by serde code sits outside its
  `#[cfg(feature = "serde")]`.
