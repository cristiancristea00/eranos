# Versioning

eranos follows [Semantic Versioning 2.0.0](https://semver.org/) as a
library: the version tells the apps and the binding crates whether an
upgrade can change their code or their users' numbers.
[AGENTS.md](../AGENTS.md) says when the number moves (with the commit that
changes it).

## 1. The public contract

| Surface                  | What is covered                                                                                                                                                                                         |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The Rust API             | every public item reachable from `eranos::`, its signature, and its documented behaviour, whichever internal crate holds it                                                                             |
| The serialised form      | the JSON shapes of [ARCHITECTURE § 9](ARCHITECTURE.md#9-the-serialised-form) and the `FORMAT` number                                                                                                    |
| The numbers              | the shares, totals, and settlements computed from the same entries: rounding, the tie rotation, conversion, and payment order                                                                           |
| The minimum Rust version | `rust-version` in `Cargo.toml`                                                                                                                                                                          |
| The concurrency          | which lock each `SharedExpenseGroup` call takes, the order in which waiting calls get it, and that a call dropped before it completes changes nothing                                                   |
| The floors               | what each binding's package runs on, as the first table of [BINDINGS.md](BINDINGS.md) lists it: a platform, a JDK, Node, or Python release, or a language standard such as C23; raising one is breaking |

The numbers are part of the contract because groups are shared: two devices
on different releases must compute the same cent for the same expense. A
change to the allocation order, the FNV-1a tie rotation, the rounding rule,
or a strategy's payment choice changes what users see without any API
change, and counts as breaking. The rotation's seed is the text of the
expense's identifier, byte for byte, and identifiers compare the same way,
so two spellings of one UUID, such as uppercase and lowercase hexadecimal
digits or the braced form, are two identifiers, which split and sort
differently.

The conversion of a double into an amount, a percentage, parts, a
quantity, or a rate, by `from_f64`, is part of the numbers too: it rounds
the double's shortest decimal form as std's `Display` writes it, which the
tests pin, so a change to the rounding counts as breaking, and a toolchain
that wrote another form would fail the tests rather than change a stored
cent ([ARCHITECTURE § 4](ARCHITECTURE.md#4-money-and-rounding)). So is an
item's line total, its unit price times its quantity rounded half away
from zero, which the shares of an itemised expense divide.

The internal crates (`eranos-core` to `eranos-expense-group`) are outside
the contract on their own: they are not published, they take the workspace's
version, and callers depend on the facade. A re-exported type is covered
through its `eranos::` path, so moving it to another internal crate changes
nothing the contract covers. What the facade does not re-export moves
freely: the machinery the crates above use (such as `eranos_core::text`,
`eranos_money::allocation`, `eranos_ledger::Ledger`, and
`eranos_expense::resolve_expense`), the internal crates' own paths, private
modules, and the internal layout of `ExpenseGroup`. So does
`eranos-bindings`, the core the bindings forward to: a binding's contract is
the API of its language's package. For C, whose header includes Diplomat's,
that is what `eranos.h` and its parts declare, not the names Diplomat
generates, which change with the bridge. The Kotlin package records its API
in `bindings/kotlin/eranos/api/eranos.api`, which the gates check the layer
against, so a change to the API shows in the commit that makes it; the
Kotlin Multiplatform package compiles the same layer, so the dump records
its API too.

## 2. What moves the number

Before 1.0.0, the minor version plays the role of the major one:

| Change                                                                                                                                                                      | Before 1.0 | From 1.0  |
| --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | --------- |
| breaking (`!` commit): removed or changed API, a new variant in a matchable position, a serialised shape older releases cannot read, different numbers for the same entries | 0.**y**.0  | **x**.0.0 |
| `feat`: new API, a new error variant (every enum is `#[non_exhaustive]`), a new optional serialised field                                                                   | 0.y.**z**  | x.**y**.0 |
| `fix` that restores documented behaviour                                                                                                                                    | 0.y.**z**  | x.y.**z** |
| `docs`, `test`, `style`, `chore`, `refactor`, `ci`, `build`, `perf` without visible change                                                                                  | none       | none      |

A `fix` that changes numbers users already saw is still breaking: record it
as `fix!` with a `BREAKING CHANGE:` footer naming what changes.

A dependency whose data decides which values are valid (`iso4217-static`)
is pinned exactly (`=x.y.z`), so its data changes only with an eranos
release. Saved groups load through the same checks as input, so moving the
pin is a `feat` when it only adds values and breaking when it removes one
or changes what one means, such as a withdrawn currency or a changed minor
unit. `rlibphonenumber` is pinned the same way, and every move of its pin is
breaking: its libphonenumber metadata can withdraw a range, and then groups
holding such a number stop loading, while an older device refuses numbers
in a new range, and the grouping `PhoneNumber::international` returns
follows the metadata too
([DEVELOPMENT § 3.8](DEVELOPMENT.md#38-a-libphonenumber-metadata-release)).
An IBAN registry release that changes a length or a structure is breaking
for the same reason.

## 3. Bumping, step by step

1. Edit `[workspace.package] version` in `Cargo.toml`, and the version of
   the TypeScript package's `package.json` and of the Python package's
   `pyproject.toml`, which a test of xtask holds to it.
2. Build without `--locked` so `Cargo.lock` refreshes, and refresh the two
   packages' lock files: `npm install --package-lock-only` in
   `bindings/typescript` and `uv lock` in `bindings/python`.
3. Commit the bump together with the change it describes.
4. Tag it when asked: `git tag -s vX.Y.Z -m 'eranos vX.Y.Z'`.

## 4. Deprecating and removing

Mark the old item `#[deprecated(since = "…", note = "use … instead")]` in a
minor release and remove it in the next breaking one; the footer of the
breaking commit names what disappeared and its replacement. Before 1.0, a
breaking release may remove an item without a deprecating minor release
when its replacement has another shape, and its `BREAKING CHANGE:` footer
names the replacement. A serialised
shape is never removed while `FORMAT` stays the same: reading support for
an old format stays until a breaking release says otherwise.

## 5. What the version does not promise

- **Performance.** Timings are documented, not guaranteed.
- **Where a call yields.** The work between two yields
  ([ARCHITECTURE § 12](ARCHITECTURE.md#12-async-and-concurrency)), and so
  the number of polls a call takes, may change in any release.
- **`Debug` output.** Its format may change in any release.
- **Error messages.** The `Display` text of errors may be reworded; match on
  the variants.
- **Sequential identifiers.** The identifiers the `sequential-ids` feature
  generates may change in any release; the feature is for examples and
  tests.
- **Publishing.** The crate is `publish = false`: the apps consume it from
  this repository.

## 6. The minimum Rust version

`rust-version` is the oldest toolchain the workspace supports. CI builds,
lints, and tests with the latest stable release, and no job checks an
older one, so `rust-version` moves up whenever the code needs a newer
release. Raising it is a `build` commit and bumps the minor version,
because it can break a consumer's build.
