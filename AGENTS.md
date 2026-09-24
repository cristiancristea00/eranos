# eranos — contributor notes

Working notes for contributors and coding agents. `README.md` says what the
library does, `docs/ARCHITECTURE.md` how it is built, `docs/DEVELOPMENT.md`
holds every command and change recipe, `docs/TESTING.md` the test suites, and
`docs/VERSIONING.md` what the version number promises. This file holds the
conventions and the gates. `CLAUDE.md` and `GEMINI.md` are symlinks to it.

## Commit messages

All commits follow [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```text
<type>(<optional scope>)<!>: <lowercase description, imperative, ≤72 chars>

<optional body: what and why>

<optional footers, e.g. BREAKING CHANGE:>
Signed-off-by: <committer name> <<committer email>>
<optional Co-Authored-By: footers>
```

- Types in use: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`, `ci`, `perf`, `style`.
- Scopes in use: `core`, `money`, `member`, `expense`, `transfer`, `group`,
  `ledger`, `settlement`, `serde`, `docs`, `examples`, `build`, `xtask`.
- A breaking change appends `!` after the type or scope and adds a `BREAKING CHANGE:` footer.
- One commit per concern. Bodies are prose in full sentences, not bullet dumps.
- **Every commit is signed off**: a `Signed-off-by:` trailer with the
  committer's name and email, as `git commit -s` writes it.
- **The version moves with the commit**: edit `[workspace.package] version`
  in `Cargo.toml` in the same commit as the change it describes. What bumps
  what, including why a rounding change is breaking:
  [docs/VERSIONING.md](docs/VERSIONING.md).
- Never commit or push without being asked.

## Gates before committing

```sh
cargo xtask gates
```

It runs these, in this order, and stops at the first that fails:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`
4. `cargo test --workspace --all-features` (unit, integration, and doc
   tests, the README's included)
5. `cargo test --workspace --no-default-features`
6. `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features`
7. `cargo run --quiet --package <package> --all-features --example <name>`
   for every example of every package, as `cargo metadata` lists them

All of them pass before a commit; clippy findings are fixed, not silenced.
The examples assert their key numbers, so running them is a check, not a
demonstration. A test in `crates/xtask` reads this list, so the command and
the list cannot drift apart. The VS Code task `gates` runs `cargo xtask
gates`, and CI runs the same steps as its jobs on every push
([docs/DEVELOPMENT.md § 6](docs/DEVELOPMENT.md#6-continuous-integration)).

Clippy is strict on every target (library, tests, examples, benches): the
pedantic, nursery, and cargo groups plus the restriction lints listed, each
with its reason, in `[workspace.lints]` of `Cargo.toml`. `clippy.toml`
holds two kinds of exception: what a test needs to fail loudly (unwrapping,
expecting, panicking, and indexing inside test functions), and, in
`allowed-duplicate-crates`, a crate that a dependency's graph lists in two
versions, each with the upstream issue that removes it. Any other exception
is an `#[expect(lint, reason = "...")]` at the narrowest scope (a whole file
only for test helpers and benchmark fixtures, which panic to report
failure); `#[allow]` is itself a lint error.

## Conventions

- **Variable names are never abbreviated**: `error`, `amount`, `currency`,
  `member`, `expense`, `settlement`, `participants`. Only acronyms and
  initialisms stay short: id, iban, iso, uuid, utc, ffi, fnv, bcp, json, url,
  api, io, os, cpu, ui; `max` and `min` count as dictionary words. The rule
  covers locals, parameters, closure bindings, test helpers, and every
  serialised field name.
- **Money is never a float.** Amounts are `i64` minor units; every division
  goes through `money::allocation::allocate`, every conversion through
  `ExchangeRate::convert`, and every percentage through
  `allocation::apply_basis_points`, so parts always add up and rounding is
  the documented rule (largest remainder; half away from zero).
- **Library code does not panic.** `unwrap` and `expect` warn everywhere
  but in tests and in constant initialisers, where a failure stops the
  build rather than a running app; arithmetic on user amounts is checked
  and becomes an error. An invariant that makes a failure impossible is
  written down, next to the code, as a comment or an
  `#[expect(..., reason = "...")]`.
- **The library reads no clock, touches no file, and holds no global
  state**, with three exceptions. ID generation reads the clock and the OS
  random source (UUID v7), under one private lock in `id.rs` that keeps the
  IDs of concurrent threads in creation order: an entity gets its ID when
  it is built, and loading a saved group restores the IDs through serde,
  never through a public setter. With the `sequential-ids` feature, which
  is for examples and tests, generation reads neither and takes the next
  value of one counter in `id.rs` instead. Phone-number validation reads
  rlibphonenumber's `PHONE_NUMBER_UTIL`, a `LazyLock` that decodes the
  embedded libphonenumber metadata on first use and is never written after;
  the invariant that rules out its panic is stated at the call in
  `member/contact.rs`.
- **A group changes in one way**: validate the entry, resolve it, build a
  `Delta`, apply it (range-checked before anything is written), then drop the
  cached settlements. The one other writer is `Ledger::move_category`, with
  which removing a category moves its totals to Uncategorised without
  touching a balance. So the incremental totals always equal a full rebuild;
  the group's property test checks it.
- **The public API grows additively.** Fields are private; entities are
  built with builders and edited with `to_builder`; public enums are
  `#[non_exhaustive]`; getters have no `get_` prefix; errors are
  `#[non_exhaustive]` thiserror enums, one per module, whose messages are
  lowercase, carry no trailing full stop, and never echo the text they
  rejected (inputs include account numbers). No error wraps another and
  none converts into another: a function reports its own module's failure,
  and a caller that mixes modules uses `Box<dyn Error>` or its own type.
  Every public type is `Send` and `Sync`, and a new one joins the list in
  `tests/api.rs` that asserts it.
- **A builder starts complete.** A type with a builder names its
  constructors `builder` or a variant of it (`itemised_builder`,
  `builder_with_payers`, `builder_with_split`), and `to_builder` starts an
  edit; a type without one, such as `Category` or `Account`, has `new`.
  Every required value is a constructor argument, so `build` never reports
  a missing one, and every setter replaces what was set before: the last
  call wins.
- **Fields follow one order.** Named fields, in structs and in enum
  variants, start with `format` (the saved group's version) and `id`, then
  list the required fields, then the optional ones, each alphabetically. A
  field is optional when the caller may leave it out: an `Option`, or a
  field with a default, such as a quantity of 1 or no accounts. Generated
  and derived fields, such as an expense's total, count as required. Struct
  literals, struct patterns, serialised keys, and the arguments of a
  constructor that takes fields follow the same order: `CalendarDate::new(day,
  month, year)`, `Money::from_minor_units(currency, minor_units)`,
  `Expense::builder(cost, date, name, payer, split)`; a `parse` takes its
  text first. No field order carries meaning: `Currency`, `CalendarDate`,
  and `TimeOfDay` write their ordering out by hand, and a table row in the examples
  fixes its columns with `#[tabled(order = N)]`.
- **Accessors follow their fields.** An `impl` starts with its
  constructors and associated constants, then lists the getters and
  setters in field order: for each field, the method named after it, then
  its other readers, then its setters. A method derived from one field
  sits beside it (`tip_amount` after `tip`, `paid_by` before `payers`),
  and methods that read several fields, such as `build`, come last.
  `Group` heads each field's methods with a section comment.
- **Variants are alphabetical.** Enum variants are declared in
  alphabetical order, and so are the arms of a `match` over them and the
  alternatives of an or-pattern; a catch-all arm stays last. A list of
  every variant, such as `SettlementStrategy::all`, follows the
  declaration.
- **Serialised data outlives releases.** A new field gets
  `#[serde(default)]`; a type with invariants deserialises through its
  validating constructor (`serde(try_from = …)`); a group loads through
  `SavedGroup::into_group`, which replays its entries through the `add_*`
  methods; a change older readers cannot accept bumps the `FORMAT` constant
  in `group.rs`.
- **Doc comments follow rustdoc conventions**: a one-sentence summary, then
  `# Errors`, `# Panics`, and `# Examples` sections where they apply.
  Examples use `?`, never `unwrap`, and compile with warnings denied. Types
  are referenced with intra-doc links (`` [`Group`] ``); values and snippets
  keep single backticks.
- **Every fence in README.md has a language tag** (`rust`, `sh`, `text`,
  `mermaid`, `toml`): the README is compiled as a doctest, and an untagged
  fence would be compiled as Rust.
- **Every Mermaid diagram ships with an ASCII twin** in a
  `<details><summary>ASCII version (for renderers without Mermaid)</summary>`
  block directly beneath it. Box-drawing characters (`┌─┐│└┘`), ≤80 columns,
  inside a plain ```text fence so monospace alignment survives.
- **Comments state constraints, never history.** A comment says what the
  code cannot say about itself, written as if the code had always been this
  way: no "now", "used to", "no longer", "before this existed"; no narrating
  what a change did. An incident becomes a conditional ("without this, X
  would…"). Regression witnesses in tests are the exception: the recorded
  failure is the constraint.
- **Prose says what it means.** Comments, docs, and error messages use the
  literal phrase when one exists: no metaphor, no ALL-CAPS emphasis inside a
  sentence, no reassurance ("cheap", "for free"), no signposting ("worth
  knowing"). A sentence stays only if it carries a default, a rejected
  alternative, a failure the code prevents, a platform constraint, a
  command, a cross-reference, or a format trap. One canonical home per topic
  in the docs; elsewhere one sentence and a relative link.
- **British English with the Oxford comma.** Prose, commit messages, and
  every identifier we own use British spelling with -ise (organise,
  serialise, behaviour, licence, artefact) and British words (full stop, per
  cent, towards); a list of three or more items takes a comma before its
  final "and" or "or". Names that belong to other code keep their spelling
  inside code spans: `Serialize`, `skip_serializing_if`,
  `CARGO_TERM_COLOR`, `actions/upload-artifact`.
- **Tests are claims.** Unit tests live in `<module>/tests.rs` (declared
  with `#[cfg(test)] mod tests;` at the bottom of the module); every test
  module has a `//!` doc and every test a `///` doc stating what breaks in
  production if it regresses; names read as claims
  (`an_archived_member_keeps_history_but_joins_no_new_expense`), never
  `test_*`. No `#[ignore]`, no `#[should_panic]`, no snapshot tests. Details:
  [docs/TESTING.md](docs/TESTING.md).

### Deliberate choices

- Doctests and rustdoc sections are required: the documentation is the
  library's interface, and a doctest keeps every example compiling.
- Types in docs are intra-doc links, so a renamed type breaks the docs build
  instead of leaving a stale name.
- Every directory module uses the `foo.rs` + `foo/` form, never `mod.rs`
  (`clippy::mod_module_files` enforces it).
- `.cargo/config.toml` holds only the `xtask` alias: a build flag there,
  such as `target-cpu=native`, would make the mobile and web builds
  non-portable.

## Architecture in one paragraph

One library crate, `crates/eranos`, with binding crates (`eranos-ffi` for
UniFFI, `eranos-wasm`) planned as siblings. A `Group` owns its members,
categories, expenses, and transfers in ID-keyed maps. Each entry is resolved
into group-currency shares when it is added or replaced (`resolution.rs`:
exact splits, tips, conversion, and the pairwise debts behind the direct
settlement) and applied to the `Ledger` as a range-checked `Delta`; the
`Summary` is part of the ledger and is updated by every change. Only
`change_currency` resolves every entry again and rebuilds the ledger.
Settlements are computed from the ledger on request, by the strategy the
caller names, and cached per strategy in `OnceLock`s that every
balance-changing mutation clears. Diagrams, rules, and numbers:
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Verification habits

- A change to rounding, allocation order, the tie rotation, or conversion
  changes stored shares and must fail a pinned test first: the rotation
  hash and several hand-computed splits are pinned on purpose. See
  [docs/VERSIONING.md](docs/VERSIONING.md) before changing any of them.
- `cargo test --no-default-features` compiles out the serde code and
  generates identifiers from the clock, where `--all-features` counts them;
  run both feature sets before claiming a change is tested.
- Coverage is a diagnostic, never a gate: a run with every feature and one
  without the default features, merged into one report
  ([docs/DEVELOPMENT.md § 5](docs/DEVELOPMENT.md#5-coverage-and-mutation-testing)),
  because each compiles an identifier generator the other leaves out. It
  does not count doctests on the stable toolchain. Every line, region, and
  function is covered; a new gap is either a missing test or a branch that
  cannot run, which is restructured away rather than kept
  ([docs/TESTING.md](docs/TESTING.md#5-coverage)).
  `cargo mutants -p eranos --file <file>` is the occasional companion
  audit: coverage says a line ran, mutation testing says a test would have
  noticed it was wrong.
- Benchmarks are wall-clock timings: run `cargo bench -p eranos` on an idle
  machine, and update the timings quoted on `SettlementStrategy` and in
  docs/ARCHITECTURE.md together.
