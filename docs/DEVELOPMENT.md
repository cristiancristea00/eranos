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
  `cargo-llvm-cov` and `cargo-mutants`, neither of which is needed to
  build or test the library, and the bindings' toolchains: JDKs 25 and 21,
  Node 24, LLVM (clang, clang-format, clang-tidy, llvm-profdata, and
  llvm-cov), Doxygen, ktlint, SwiftLint, actionlint, zizmor, cargo-deny,
  the Android command-line tools, cargo-ndk 4.1.2, diplomat-tool 0.16.1,
  and uniffi-bindgen-react-native 0.31.0-5, the last three built from
  their release tags.
- **Paths**: the JDKs, Node 24, and LLVM are keg-only, and the Android
  tools look for their SDK; the gates run the TypeScript package on the
  `node` first on `PATH`, and stop unless it is Node 24, the Kotlin
  Multiplatform package's Gradle on the JDK that `JAVA_HOME_21` names, and
  the C and C++ packages' compilers and linters from the LLVM that
  `LLVM_HOME` names, since Xcode's clang has no clang-tidy:

  ```sh
  export JAVA_HOME="$(brew --prefix openjdk@25)/libexec/openjdk.jdk/Contents/Home"
  export JAVA_HOME_21="$(brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home"
  export ANDROID_HOME="$(brew --prefix)/share/android-commandlinetools"
  export LLVM_HOME="$(brew --prefix llvm)"
  export PATH="$(brew --prefix node@24)/bin:$LLVM_HOME/bin:$PATH"
  ```

- **Android SDK packages**, once, after accepting their licences:

  ```sh
  sdkmanager --licenses
  sdkmanager "platform-tools" "platforms;android-36" "build-tools;36.1.0" "ndk;30.0.16248370"
  ```

- **Xcode**, from the App Store, for the Swift package and every Apple
  target.
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
`.cargo/config.toml`, and the automation in `crates/xtask`. The gates also
write the Swift, Kotlin, and Java sources of the UniFFI glue into
`target/uniffi/<language>`, from the host library in `target/debug`, and
on macOS they build the Swift package's XCFramework from the library's
thirteen Apple targets, which they add with `rustup target add`. On every
machine they build the Kotlin package: the library of the four Android
ABIs through `cargo ndk`, which finds the NDK under `ANDROID_HOME`, then the
Gradle build, whose JVM `JAVA_HOME` names; the gates stop before the first
step when either variable is not set. They build the Java package with
the same JDK, and the TypeScript package on Node 24: the library for
`wasm32-unknown-unknown`, then its TypeScript, which
uniffi-bindgen-react-native generates into the package. Last, they build
the Kotlin Multiplatform package on the JDK 21 that `JAVA_HOME_21` names,
where gobley's plugins build the library for each target and generate its
bindings, and then eranos-capi's static library, from which Diplomat
generates each of the C and C++ packages' headers, whose tests the LLVM
that `LLVM_HOME` names builds under the sanitizers and measures. Then
they build eranos-uniffi's host library again, with `sequential-ids`, into
`target/sequential`, and eranos-kmp's beside it, and run the ports of the
examples: on macOS the Swift ones, on a copy of the Swift package, and on
every machine the Kotlin ones, on the JVM, over the Kotlin package's
bindings and over the Kotlin Multiplatform package's, and the Java ones
([TESTING.md § 2](TESTING.md#2-layout)). A run
from an empty target directory takes about 7 minutes on an Apple M4 Pro,
most of it building the library for every Apple target and every Android
ABI; a warm one takes about 4 minutes in all, 40 seconds of it
clang-tidy's pass over the C and C++ tests and 20 seconds the Swift
examples' two builds and their ports.

```sh
swift test --package-path bindings/swift                        # the Swift tests, once the gates built the framework
swift test --package-path bindings/swift --enable-code-coverage # with coverage; the JSON is at `swift test --show-codecov-path`
swift run --package-path bindings/swift/examples settling_up    # one Swift port, on that framework, whose identifiers are random
(cd bindings/swift && swiftlint lint --strict)                  # SwiftLint, with the package's .swiftlint.yml, the examples included
```

In `bindings/kotlin`, once the gates built the host library and copied
the generated Kotlin; the tests load the library from `target/debug`
unless `-Peranos.library=<directory>` names another:

```sh
./gradlew :eranos:test                        # the Kotlin tests
./gradlew :eranos:koverHtmlReport             # their coverage, in eranos/build/reports/kover/html
./gradlew :eranos:apiDump                     # the API dump, after a change to the layer's API
./gradlew :eranos:dokkaGenerate               # the documentation, in eranos/build/dokka/html
./gradlew :examples:ports                     # the ports and the libraries they run with, in examples/build/ports
java --enable-native-access=ALL-UNNAMED -Dstdout.encoding=UTF-8 -Djna.library.path=../../target/debug -cp 'examples/build/ports/*' eranos.examples.settlingup.MainKt  # one port, whose identifiers are random
ktlint --format --relative '**/*.kt' '**/*.kts' '!generated/**' '!**/build/**'  # the formatter
```

In `bindings/java`, once the gates built the host library and copied the
generated Java; the tests load the library from `target/debug` unless
`-Peranos.library=<directory>` names another:

```sh
./gradlew test                                 # the Java tests
./gradlew jacocoTestReport                     # their coverage, in build/reports/jacoco/test/html
./gradlew javadoc                              # the documentation, in build/docs/javadoc
./gradlew format                               # google-java-format, applied
./gradlew ports                                # the ports and the module they run on, in build/ports
java --enable-native-access=ALL-UNNAMED -Dstdout.encoding=UTF-8 -Djava.library.path=../../target/debug -cp 'build/ports/*' eranos.examples.settlingup.Main  # one port, whose identifiers are random
```

In `bindings/typescript`, once the gates built the WebAssembly library and
generated the TypeScript into `src/internal`; after a change to the glue,
the first two commands write it again:

```sh
cargo build -p eranos-web --target wasm32-unknown-unknown --lib                    # the WebAssembly library
uniffi-bindgen-react-native build wasm2 --config ubrn.config.yaml --no-wasm-build  # the generated TypeScript
npm test                                                                           # the TypeScript tests, under Node
npm run coverage                                                                   # with their coverage, which c8 holds at 100 %
npm run lint                                                                       # ESLint, with the types of both projects
npm run typecheck                                                                  # tsc on the layer and on its tests
npm run docs                                                                       # the documentation, in dist/docs
npm run build                                                                      # the bundle, its declarations, and the module, in dist
npm run format                                                                     # Prettier, applied
```

In `bindings/kmp`, where gobley's plugins build the library and generate
the bindings themselves, on the JDK 21. The package compiles the Kotlin
package's layer from its sources, so a recipe's step for the Kotlin layer
covers both packages:

```sh
export JAVA_HOME="$JAVA_HOME_21"                 # Gradle 9.0 runs on JDK 24 at the latest
./gradlew :eranos:jvmTest                        # the Kotlin tests on the JVM, against gobley's bindings
./gradlew :eranos:assembleDebug                  # the layer's Android library
./gradlew :eranos:compileKotlinIosSimulatorArm64 # the layer for one Apple target
./gradlew :examples:ports                        # the Kotlin ports on gobley's bindings, in examples/build/ports
```

In `bindings/cpp`, once `cargo build -p eranos-capi` wrote the static
library into `target/debug`; `bindings/c` takes the same commands with
`c`, `clang`, `-std=c11`, `tests/tests.c`, and `-pthread`, and on Linux the
libraries that `cargo rustc -p eranos-capi --lib -- --print
native-static-libs` lists in place of `-liconv`:

```sh
diplomat-tool cpp generated --entry ../../crates/eranos-capi/src/lib.rs     # Diplomat's headers, beneath the layer
"$LLVM_HOME/bin/clang++" -std=c++17 -g -fsanitize=address,undefined -Iinclude -isystem generated \
    tests/tests.cpp ../../target/debug/liberanos_capi.a -liconv -o build/tests  # the tests, under the sanitizers
build/tests                                                                 # run them
"$LLVM_HOME/bin/clang-format" -i include/eranos.hpp include/eranos/*.hpp include/eranos/detail/*.hpp tests/tests.cpp  # the formatter, applied
doxygen Doxyfile                                                            # the documentation, in build/docs/html
```

Every internal crate's tests link the facade, which its doc examples use,
so testing one crate builds the whole workspace, and each crate's doctests
are a binary of their own. The split also makes a change
low in the graph rebuild every crate above it. Rebuilds in the dev
profile on an Apple M4 Pro, after touching one file, with the library as
one crate and as thirteen:

| Rebuild after touching                    | One crate | Thirteen crates |
| ----------------------------------------- | --------: | --------------: |
| `cargo build -p eranos`, the group        |    0.65 s |          0.67 s |
| `cargo build -p eranos`, money            |    0.64 s |          1.70 s |
| the touched crate's unit tests, the group |     1.1 s |           1.2 s |
| the touched crate's unit tests, money     |     1.1 s |           2.1 s |
| every test target (`--no-run`), money     |     1.9 s |           6.4 s |

The split buys layers the compiler enforces, crates whose APIs stand on
their own, and smaller units, at those seconds.

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
3. Add its method to `FromSplit`, in variant order, and its arm to
   `from_split`; every implementation, such as the binding core's, then
   fails to build until it handles the new variant, and the mirror of the
   core's `Split` in `crates/eranos-uniffi/src/records.rs` until it
   repeats it.
4. Pin it with a hand-computed case in `eranos-expense/src/split/tests.rs`
   and in `tests/scenarios.rs`, and add it to the random recipes in
   `tests/properties.rs` so the invariants cover it.
5. Document it on the variant and in ARCHITECTURE § 3.

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
   pair debts, and dispatch to it in the crate's `settle_paced`, which both
   `settle` and `settle_async` run. A strategy whose time grows faster than
   its input is an `async fn` that reports its work to a `Pacer`, as
   `minimum.rs` does, so that an asynchronous caller's other tasks run while
   it works.
3. Add its cache slot to `Settlements` beside it (the `match` in
   `Settlements::slot` is exhaustive, so the compiler points at it).
4. Add it to `SettlementStrategy::all()`; the property tests then check that
   its plans clear every balance.
5. Add its method to `FromSettlementStrategy`, in variant order, and its
   arm to `from_settlement_strategy`; the binding core's
   `SettlementStrategy` then fails to build until it has the variant, and
   the mirror in `crates/eranos-uniffi/src/records.rs` until it repeats it.
6. Benchmark it in `benches/settlement.rs` and record the timing in
   ARCHITECTURE § 13.

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
5. Add the field to the entity's record in the binding core
   (`crates/eranos-bindings/src/`), in the same place of the field order,
   read in the record's `from_library` and written in its conversion to the
   library, where an absent optional field clears the stored one; the
   record's round trip in its module's `tests.rs` then covers it. Repeat
   the field in the record's mirror in `crates/eranos-uniffi/src/records.rs`,
   which fails to build until it does, and in the Swift entry type in
   `bindings/swift/Sources/Eranos/Entries/`, with its conversions both ways,
   and the Kotlin entry class in
   `bindings/kotlin/eranos/src/main/kotlin/eranos/`, with its conversions,
   its `copy`, and its equality, hash, and text; `./gradlew :eranos:apiDump`
   then records the new API. The Java entry class in
   `bindings/java/src/main/java/eranos/` gets the field, its accessor, its
   builder's setter, and its place in its conversions, equality, hash, and
   text. The TypeScript entry interface in
   `bindings/typescript/src/entries.ts` gets the field, optional where the
   library's is, and both of its conversions carry it.

### 3.6 A new error variant

Error enums are `#[non_exhaustive]`, so a variant is a `feat`. The variant
joins its crate's enum in that crate's `error.rs`, in alphabetical order;
its message is lowercase, has no trailing full stop, and never repeats the
rejected text. Pin the message in its enum's list in `tests/errors.rs`, and
name the variant in the `# Errors` section of every function that can
return it. Give the enum's conversion trait (`FromGroupError` and the like,
beside the enum) a method for it, in variant order, taking what the
variant holds and the message, and its arm in the provided `from_…`; add
its row to the conversion test in the crate's `error/tests.rs`. The binding
core then fails to build until the module's failure enum in
`crates/eranos-bindings/src/error.rs` (`GroupFailure` for `GroupError`,
and the like) has a case for the variant, in alphabetical order, with what
it holds, and its row in that crate's `error/tests.rs`; and the UniFFI glue
until the failure's mirror in `crates/eranos-uniffi/src/error.rs` repeats
the case. A new error type is a new case of `EranosError`, with a failure
enum of its own. The Swift layer's failure enum in
`bindings/swift/Sources/Eranos/Errors/Failures.swift` gets the case, with
its payload typed, the conversion from the generated one gets its arm, and
`Tests/EranosTests/ErrorTests.swift` its row; so does the Kotlin layer's
failure interface in `bindings/kotlin/eranos/src/main/kotlin/eranos/Failures.kt`,
whose conversion fails to build without the case, and
`src/test/kotlin/eranos/ErrorTest.kt`; the Java layer's failure
interface in `bindings/java/src/main/java/eranos/`, whose conversion in
`EranosException.java` fails to build without the case, and
`src/test/java/eranos/ErrorTest.java`; and the TypeScript layer's failure
union in `bindings/typescript/src/errors.ts`, whose conversion fails to
type-check without the case, and `test/errors.test.ts`. A new error type is
also a registration in the conversion table of `EranosException.java`,
which the Java error test checks.

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

The Kotlin layer follows the same order. A detekt finding is fixed, or
suppressed on the narrowest declaration with `@Suppress("RuleName")`
and the reason in a comment on the same line; a rule that contradicts the
code style everywhere, such as its line length, is set in
`bindings/kotlin/eranos/detekt.yml`, with the reason beside it. In the
Java layer, an Error Prone or lint finding is fixed, or suppressed on the
narrowest declaration with `@SuppressWarnings("CheckName")` and the reason
in a comment above it. In the TypeScript layer, an ESLint finding is fixed,
or disabled for its line with
`// eslint-disable-next-line <rule> -- <reason>` above it; ESLint reports a
directive whose rule no longer fires there, so it goes with the finding.

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

### 3.9 A crossing between crates

When code in one crate needs something another crate holds:

1. Move the logic to the crate that owns its data when it can live there: a
   match on an enum belongs in the enum's crate, since a match on another
   crate's `#[non_exhaustive]` enum needs a wildcard arm that never runs,
   and a rule of the group, such as whether a cost suits its currency, in
   the group. A crate that must tell every variant of another crate's enum
   apart, as the binding core does, implements the enum's conversion trait
   (`FromSplit`, `FromCost`, and the like): one method per variant, in
   variant order, and a provided `from_…` that matches in the enum's own
   crate, so a new variant fails the implementing crate's build.
2. Otherwise make the item a documented public item of its crate. If
   callers need it, re-export it in the facade module of its crate with
   `#[doc(inline)]`, and at the facade's root with `#[doc(no_inline)]`;
   machinery only the crates above use stays out of the facade.
3. A crate never depends on a crate above it. When the code would need
   one, it belongs in that crate, or the crate above passes what it needs.
4. Check that the facade's API changed only by what the change adds: build
   the documentation with `cargo doc --workspace --no-deps --all-features`
   and compare `target/doc/eranos/all.html`, and the method anchors of the
   pages the change touches, with the documentation before it.

### 3.10 A new group method

1. Add it to `Group` in `crates/eranos-group/src/lib.rs`, under the section
   of the field it reads or changes, with its `# Errors`.
2. A method whose time grows with the group, as a currency change's does,
   is written as `<name>_async`: an `async fn` that reports its work to a
   `Pacer`, returned in a `Cooperative`, with the synchronous method as its
   `.complete()`. A change writes nothing to the group before its last
   yield, so a dropped call changes nothing.
3. Mirror it on `SharedGroup` in `crates/eranos-group/src/shared.rs`, in the
   same section and under the same name: a read through `read_with`, a
   change through `write_with`, each returning owned values; a method that
   yields awaits the group's `_async` method while it holds the lock.
4. Compare its result with the group's in the shared group's tests
   (`every_read_returns_what_the_group_returns` or
   `every_change_does_what_the_group_does`), and add its future to
   `every_call_is_send_static_and_small`.
5. Forward it in the binding core: a function over the group in
   `crates/eranos-bindings/src/operations.rs`, which converts the records
   and the answer, then both forms on `GroupHandle` in `group.rs` (the
   synchronous one through `reading` or `writing`, the asynchronous one
   through `reading_async` or `writing_async`) and, for a read, the method
   on `GroupReader` in `reader.rs`. Test the conversion in
   `operations/tests.rs` and the two forms in `group/tests.rs`
   (`every_read_answers_alike_in_both_forms` or the change's
   `*_change_alike_in_both_forms`).
6. Forward both forms from `Group` in `crates/eranos-uniffi/src/objects.rs`,
   and a read also from its `GroupReader`, with the core's documentation;
   nothing fails to build while one is missing, so the methods of each
   object follow the core's, section by section.
7. Add both forms, one name overloaded by `async`, to the Swift layer's
   `Group` in `bindings/swift/Sources/Eranos/Group/Group+<section>.swift`,
   named by the Swift API Design Guidelines, and a read to its
   `GroupReader`; call them in `Tests/EranosTests/GroupTests.swift`.
8. Add both forms to the Kotlin layer's `Group` in
   `bindings/kotlin/eranos/src/main/kotlin/eranos/Group.kt`, the `suspend`
   one under the call's name and its twin with `Blocking` after it, and a
   read to its `GroupReader`; call them in `GroupTest.kt`, and record the
   API with `./gradlew :eranos:apiDump`.
9. Add both forms to the Java layer's `Group` in
   `bindings/java/src/main/java/eranos/Group.java`, the blocking one under
   the call's name and its twin with `Async` after it, and a read to its
   `GroupReader`; call them in `GroupTest.java`.
10. Add both forms to the TypeScript layer's `Group` in
    `bindings/typescript/src/group.ts`, the Promise under the call's name
    and its twin with `Sync` after it, and a read to its `GroupReader` in
    `group_reader.ts`; call them in `test/group.test.ts`.

## 4. Examples and benchmarks

```sh
cargo run -p eranos --example quickstart
cargo run -p eranos --features serde --example persistence
cargo bench -p eranos --features serde --bench '*'  # everything; minutes
cargo bench -p eranos --bench settlement -- --quick  # a fast sanity pass
cargo bench -p eranos --bench ledger -- "replace"    # one benchmark by name
cargo bench -p eranos --features serde --bench '*' -- --save-baseline main
cargo bench -p eranos --features serde --bench '*' -- --baseline main
```

Benchmarks run the release profile (fat LTO), so the first run compiles for
a while. Timings are wall-clock: run them on an idle machine before quoting
them. `--bench '*'` runs the criterion benchmarks alone: the library's own
test harness also runs under `cargo bench` and refuses criterion's options,
such as `--save-baseline`, which keeps a run to compare a change against
with `--baseline`.

## 5. Coverage and mutation testing

```sh
cargo xtask coverage                  # the Rust tests and the binding glue, in one report
cargo llvm-cov report --html --open   # the same report as HTML
cargo llvm-cov report --summary-only  # totals only
cargo mutants --file crates/eranos-money/src/allocation.rs
```

`cargo xtask coverage` needs what the gates need for the Kotlin, Java, C,
and C++ packages, the variables of [§ 1](#1-prerequisites), and on macOS
Xcode for the Swift one. It drops the earlier runs, runs the workspace's
tests under cargo-llvm-cov with every feature and without the default
ones, and then measures the binding glue through the foreign tests: it
builds eranos-uniffi and eranos-capi with cargo-llvm-cov's instrumentation
into its target directory, `target/llvm-cov-target`, so the gates' builds
stay as they are, generates the Swift, Kotlin, and Java sources from that
library, and runs the Swift tests, on a copy of the package there whose
`XCFramework` holds the instrumented shared library, and the Kotlin, Java,
C, and C++ tests on it, each writing the coverage of the glue it runs. The
report comes last; `cargo llvm-cov report` reads the same runs again in
another form. The first run on an Apple M4 Pro took about 9 minutes, and a
run after it takes about 2.

Coverage merges the two Rust runs because each compiles code the other
leaves out: the serde code, and one of the two identifier generators. A
single `--all-features` run reports the clock-based generator as absent
rather than missed, so its total would still read 100 %. The Rust runs
leave out the glue crates, which have no Rust tests: the glue forwards
every call to the binding core, which the Rust tests cover. `eranos-web`
compiles the glue's sources for WebAssembly, which stable Rust cannot
measure, and `eranos-kmp` compiles them again for gobley, which builds its
own library, so the report has neither. CI's coverage job runs the Rust
part alone, as these commands do:

```sh
cargo llvm-cov clean --workspace      # drop earlier runs
# Sequential IDs and serde, then clock-based IDs, without the glue crates:
cargo llvm-cov --no-report --workspace --all-features \
    --exclude eranos-capi --exclude eranos-kmp --exclude eranos-uniffi \
    --exclude eranos-uniffi-bindgen --exclude eranos-web
cargo llvm-cov --no-report --workspace --no-default-features \
    --exclude eranos-capi --exclude eranos-kmp --exclude eranos-uniffi \
    --exclude eranos-uniffi-bindgen --exclude eranos-web
cargo llvm-cov report --summary-only
```

Coverage does not count doctests on the stable toolchain, so a function
covered only by its example shows as missed. Mutation testing takes
seconds per mutant; run it one file at a time. Reading both:
[TESTING.md](TESTING.md).

## 6. Continuous integration

`.github/workflows/ci.yml` runs on every branch push, on pull requests from
forks, on the merge queue, and on a manual dispatch. A pull request from a
branch of this repository starts no jobs of its own: the push of its head
commit was tested, and those checks show on the pull request.

| Job                     | What it runs                                                                                                                                                                                                                                |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Lint                    | `cargo fmt --check`, clippy for both feature sets, and rustdoc, with warnings denied, and the check that no package turns on a feature of async-lock                                                                                        |
| Test                    | the library build, every test suite for both feature sets, every example, and each benchmark once in test mode                                                                                                                              |
| MSRV                    | the tests with Rust 1.97, the `rust-version`                                                                                                                                                                                                |
| Mobile and wasm targets | clippy for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android`, and `wasm32-wasip1`, which compiles the library's wasm code as the browser does, and for `eranos-web` on `wasm32-unknown-unknown` |
| Dependency audit        | `cargo deny check advisories sources`, with the rules in `deny.toml`                                                                                                                                                                        |
| Workflow lint           | actionlint and zizmor, configured in `.github/zizmor.yml`                                                                                                                                                                                   |
| Coverage                | both feature sets, merged into the coverage page and its summary, below                                                                                                                                                                     |
| Test (macOS, Windows)   | the tests on `macos-26` and `windows-2025`, on a manual dispatch only                                                                                                                                                                       |
| Bindings on Linux       | every gate, with the toolchains of the Kotlin, Java, web, Kotlin Multiplatform, C, and C++ packages, on a dispatch that asks for the bindings only                                                                                          |
| Bindings on macOS       | every gate, the Swift package's included, and `cargo xtask coverage`, whose report it uploads and summarises, on a dispatch that asks for the bindings only                                                                                 |

The repository is private on the GitHub Free plan: 2,000 Actions minutes a
month, with macOS minutes counting about ten times and Windows minutes about
twice. Every job is billed in whole minutes, so a push costs about 7 (10
while the caches are cold) and a run with macOS and Windows about 25. Those
two jobs run only when dispatched from the Actions tab or with
`gh workflow run ci.yml --ref main`, and the two binding jobs only on a
dispatch that asks for them, by the Actions tab's checkbox or with
`gh workflow run ci.yml --ref main -f bindings=true`; they install every
binding's toolchain and run every gate. When the minutes run out, runs stop
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
- **`UnsatisfiedLinkError: Unable to load library 'eranos_uniffi'`**: the
  Kotlin tests found no host library; `cargo build --package
  eranos-uniffi` writes it into `target/debug`.
- **`ANDROID_HOME is not set`**, **`JAVA_HOME_21 is not set`**, or
  **`LLVM_HOME is not set`** from the gates, or `SDK location not found`
  from Gradle: export the variables of [§ 1](#1-prerequisites).
- **`the C tests run … lines and … functions of the C layer`** from the
  gates: a function of the layer that no test calls, or a branch of one
  that none takes; `llvm-cov show build/tests -instr-profile=build/tests.profdata include`,
  in the package's directory, marks each unrun line.
- **`the Swift port of … prints … at line …, where the Rust example
  prints …`** from the gates, or the Kotlin, the Kotlin Multiplatform, or
  the Java port's: the port and its example first differ there;
  `swift run --package-path target/sequential/swift/examples <example>`,
  or the Kotlin or the Java command of [§ 2](#2-building-and-testing)
  with `target/sequential/debug` as the library's path, the Kotlin one
  from `bindings/kotlin` or, with the JDK 21's `java`, from `bindings/kmp`,
  prints the port's whole report on the library whose identifiers count
  up, to diff with `cargo run -p eranos --all-features --example <example>`.
- **`the gates run on Node 24`**: another Node is first on `PATH`; put
  Node 24 before it, as [§ 1](#1-prerequisites) does.
- **`Cannot find module '…/src/internal/eranos.js'`** from the
  TypeScript tests: the generated TypeScript is missing; the gates, or the first two
  commands of the TypeScript package in [§ 2](#2-building-and-testing),
  write it.
- **Benchmarks take minutes to start**: the release profile's fat LTO;
  `-- --quick` shortens the measurements, not the build.
- **`cargo test --no-default-features` fails where `--all-features`
  passes**: a `use` needed only by serde code sits outside its
  `#[cfg(feature = "serde")]`.
