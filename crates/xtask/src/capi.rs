//! The gates of the C and C++ packages: eranos-capi's static library, the C
//! and C++ headers that Diplomat generates from it beneath each package's
//! hand-written layer, and each package's tests, built with warnings as errors
//! and run under `AddressSanitizer` and `UndefinedBehaviorSanitizer`, with their
//! coverage of the layer, format, lint, and documentation; and the ports of
//! the examples, built the same way on a library whose identifiers count up.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Failure;
use crate::gates::Metadata;
use crate::plan::{Action, Phase};
use crate::process::{Runner, Step, WORKSPACE};

/// The environment variable that names the LLVM installation whose tools the
/// gates run: its clang builds the tests with the sanitizers and with
/// coverage, and its llvm-profdata, llvm-cov, clang-format, and clang-tidy
/// read and check them. Homebrew's `llvm` is keg-only, so its tools are not
/// on `PATH`, and Xcode's clang comes without the linters.
pub(crate) const VARIABLE: &str = "LLVM_HOME";

/// The libraries eranos-capi's static library needs besides the C library,
/// as `rustc --print native-static-libs` lists them for the host.
const SYSTEM_LIBRARIES: &[&str] = if cfg!(target_os = "macos") {
    &["-liconv"]
} else {
    &["-lgcc_s", "-lutil", "-lrt", "-lpthread", "-lm", "-ldl"]
};

/// One of the two packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Package {
    /// The package's name in the gates' headings: C or C++.
    pub(crate) name: &'static str,
    /// The package's directory, from the workspace root.
    pub(crate) directory: &'static str,
    /// The language Diplomat generates the package's headers in.
    language: &'static str,
    /// The compiler, in LLVM's `bin` directory.
    compiler: &'static str,
    /// The flags of the language the package's code keeps to: its standard,
    /// and for C++ no exceptions, so that a `throw` or a `try` in a header
    /// fails the build.
    dialect: &'static [&'static str],
    /// The tests' source, from the package's directory.
    tests: &'static str,
    /// The hand-written headers, from the package's directory: the layer
    /// the tests cover and the formatter checks.
    pub(crate) headers: &'static [&'static str],
    /// The ports of the examples, from the package's directory: the report
    /// module they share, its header and its source, and one program per
    /// example, which the formatter and the linter check.
    pub(crate) examples: &'static [&'static str],
}

/// The C package: the hand-written C header over Diplomat's C API.
pub(crate) const C_PACKAGE: Package = Package {
    name: "C",
    directory: "bindings/c",
    language: "c",
    compiler: "clang",
    dialect: &["-std=c23"],
    tests: "tests/tests.c",
    headers: &[
        "include/eranos.h",
        "include/eranos/calendar.h",
        "include/eranos/category.h",
        "include/eranos/contact.h",
        "include/eranos/currency_code.h",
        "include/eranos/detail.h",
        "include/eranos/error.h",
        "include/eranos/expense.h",
        "include/eranos/failures.h",
        "include/eranos/expense_group.h",
        "include/eranos/expense_group_reader.h",
        "include/eranos/expense_group_writer.h",
        "include/eranos/identifiers.h",
        "include/eranos/members.h",
        "include/eranos/money.h",
        "include/eranos/ratios.h",
        "include/eranos/split.h",
        "include/eranos/text.h",
        "include/eranos/totals.h",
        "include/eranos/transfer.h",
    ],
    examples: &[
        "examples/grand_tour.c",
        "examples/members_and_accounts.c",
        "examples/multi_currency.c",
        "examples/persistence.c",
        "examples/quickstart.c",
        "examples/report.c",
        "examples/report.h",
        "examples/restaurant_bill.c",
        "examples/settling_up.c",
        "examples/split_methods.c",
    ],
};

/// The C++ package: the hand-written C++ layer over Diplomat's C++ classes.
pub(crate) const CPP_PACKAGE: Package = Package {
    name: "C++",
    directory: "bindings/cpp",
    language: "cpp",
    compiler: "clang++",
    dialect: &["-std=c++23", "-fno-exceptions"],
    tests: "tests/tests.cpp",
    headers: &[
        "include/eranos.hpp",
        "include/eranos/calendar.hpp",
        "include/eranos/category.hpp",
        "include/eranos/contact.hpp",
        "include/eranos/currency_code.hpp",
        "include/eranos/detail/conversions.hpp",
        "include/eranos/detail/declarations.hpp",
        "include/eranos/error.hpp",
        "include/eranos/expense.hpp",
        "include/eranos/failures.hpp",
        "include/eranos/expense_group.hpp",
        "include/eranos/expense_group_reader.hpp",
        "include/eranos/expense_group_writer.hpp",
        "include/eranos/identifiers.hpp",
        "include/eranos/members.hpp",
        "include/eranos/money.hpp",
        "include/eranos/ratios.hpp",
        "include/eranos/split.hpp",
        "include/eranos/totals.hpp",
        "include/eranos/transfer.hpp",
    ],
    examples: &[
        "examples/grand_tour.cpp",
        "examples/members_and_accounts.cpp",
        "examples/multi_currency.cpp",
        "examples/persistence.cpp",
        "examples/quickstart.cpp",
        "examples/report.cpp",
        "examples/report.hpp",
        "examples/restaurant_bill.cpp",
        "examples/settling_up.cpp",
        "examples/split_methods.cpp",
    ],
};

/// The C and C++ packages' gates, in order: the library, then for each
/// package its headers in a cleared `generated` directory, its tests in a
/// new `build` directory, their coverage of the hand-written layer, which
/// must be whole, and its checks.
///
/// # Errors
///
/// [`Failure::Variable`] when [`VARIABLE`] is not set.
pub(crate) fn actions(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    let llvm = runner
        .variable(VARIABLE)
        .ok_or(Failure::Variable(VARIABLE))?;
    let mut actions = vec![Action::run(Phase::Prepare, library())];
    for package in [C_PACKAGE, CPP_PACKAGE] {
        for name in ["generated", "build"] {
            actions.push(Action::clear(Phase::Prepare, directory(package, name)));
        }
        actions.push(Action::create(Phase::Prepare, directory(package, "build")));
        actions.push(Action::run(Phase::Prepare, headers(package)));
        actions.extend(Action::runs(Phase::Test, tests(package, &llvm, metadata)));
        actions.push(Action::read(
            Phase::Test,
            coverage(package, &llvm),
            move |summary| check_coverage(package, &summary).map(|()| Vec::new()),
        ));
        actions.extend(checks(package, &llvm));
    }
    Ok(actions)
}

/// The step that builds eranos-capi's static library, which the tests link.
#[must_use]
pub(crate) fn library() -> Step {
    Step::cargo(
        &["build", "--package", "eranos-capi"],
        "the static library of eranos-capi",
    )
}

/// The step that builds eranos-capi's static library with identifiers that
/// count up, into `sequential`'s target directory, which the ports link.
#[must_use]
pub(crate) fn sequential_library(sequential: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-capi",
            "--features",
            "sequential-ids",
        ],
        "the static library of eranos-capi, with identifiers that count up",
    )
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// The directory `name` of `package`: `generated`, where Diplomat's headers
/// go, and `build`, where the tests, their coverage, and the documentation
/// go; the gates clear both first, so nothing an earlier run left behind is
/// read.
#[must_use]
pub(crate) fn directory(package: Package, name: &str) -> PathBuf {
    Path::new(WORKSPACE).join(package.directory).join(name)
}

/// The step that generates `package`'s headers from the library's bridge
/// into its `generated` directory, which the package includes as system
/// headers, so no warning of the generated code fails a build.
#[must_use]
pub(crate) fn headers(package: Package) -> Step {
    Step::new(
        &[
            package.language,
            "generated",
            "--entry",
            "../../crates/eranos-capi/src/lib.rs",
        ],
        &format!("the headers Diplomat generates for {}", package.name),
        "diplomat-tool",
    )
    .in_directory(package.directory)
}

/// The flags of every build of `package` under the sanitizers, the tests'
/// and the ports': its dialect, warnings as errors, debugging information,
/// and `AddressSanitizer` and `UndefinedBehaviorSanitizer`, whose every
/// finding ends the run.
fn sanitised(package: Package) -> Vec<&'static str> {
    let mut flags = package.dialect.to_vec();
    flags.extend([
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
        "-g",
        "-fsanitize=address,undefined",
        "-fno-sanitize-recover=all",
        "-fno-omit-frame-pointer",
    ]);
    flags
}

/// The steps that build `package`'s tests with LLVM's compiler at `llvm`,
/// under the sanitizers and with coverage, linked with the static library in
/// `metadata`'s target directory; run them; and merge their profile for
/// llvm-cov.
#[must_use]
pub(crate) fn tests(package: Package, llvm: &str, metadata: &Metadata) -> Vec<Step> {
    let library = format!("{}/debug/liberanos_capi.a", metadata.target_directory);
    let mut arguments = sanitised(package);
    arguments.extend([
        "-fprofile-instr-generate",
        "-fcoverage-mapping",
        "-pthread",
        "-Iinclude",
        "-isystem",
        "generated",
        package.tests,
        &library,
    ]);
    arguments.extend(SYSTEM_LIBRARIES);
    arguments.extend(["-o", "build/tests"]);
    let program = format!("{WORKSPACE}/{}/build/tests", package.directory);
    vec![
        Step::new(
            &arguments,
            &format!("the {} tests' build", package.name),
            &format!("{llvm}/bin/{}", package.compiler),
        )
        .in_directory(package.directory),
        Step::new(&[], &format!("the {} tests", package.name), &program)
            .in_directory(package.directory)
            .with_variable("LLVM_PROFILE_FILE", "build/tests.profraw"),
        Step::new(
            &[
                "merge",
                "-sparse",
                "build/tests.profraw",
                "-o",
                "build/tests.profdata",
            ],
            &format!("the {} tests' profile", package.name),
            &format!("{llvm}/bin/llvm-profdata"),
        )
        .in_directory(package.directory),
    ]
}

/// Where [`measured_tests`] builds `package`'s tests: a directory of their
/// own in `metadata`'s target directory, beside the library they load.
#[must_use]
pub(crate) fn measured_directory(package: Package, metadata: &Metadata) -> PathBuf {
    PathBuf::from(format!(
        "{}/{}",
        metadata.target_directory, package.language
    ))
}

/// The steps that build `package`'s tests with LLVM's compiler at `llvm`,
/// linked with eranos-capi's shared library in `metadata`'s target
/// directory, which holds its own profiling runtime, and run them. They
/// build without the sanitizers and without the layer's own coverage, whose
/// runtime, clang's, would take over from the library's, whose profile
/// cargo-llvm-cov reads.
#[must_use]
pub(crate) fn measured_tests(package: Package, llvm: &str, metadata: &Metadata) -> Vec<Step> {
    let libraries = format!("{}/debug", metadata.target_directory);
    let library = format!("{libraries}/{DLL_PREFIX}eranos_capi{DLL_SUFFIX}");
    let search = format!("-Wl,-rpath,{libraries}");
    let program = format!("{}/tests", measured_directory(package, metadata).display());
    let mut arguments = package.dialect.to_vec();
    arguments.extend([
        "-g",
        "-pthread",
        "-Iinclude",
        "-isystem",
        "generated",
        package.tests,
        &library,
        &search,
        "-o",
        &program,
    ]);
    vec![
        Step::new(
            &arguments,
            &format!(
                "the {} tests' build, on the instrumented library",
                package.name
            ),
            &format!("{llvm}/bin/{}", package.compiler),
        )
        .in_directory(package.directory),
        Step::new(&[], &format!("the {} tests", package.name), &program)
            .in_directory(package.directory),
    ]
}

/// Where the ports of `package` are built: a directory of their own in
/// `sequential`'s target directory, beside the library they link.
#[must_use]
pub(crate) fn ports_directory(package: Package, sequential: &Metadata) -> PathBuf {
    PathBuf::from(format!(
        "{}/{}",
        sequential.target_directory, package.language
    ))
}

/// The step that builds the port of `example` in `package`'s language, its
/// program and the report module, with LLVM's compiler at `llvm`, under the
/// sanitizers as the tests are, on the headers that the package's gates
/// generated, linked with the static library in `sequential`'s target
/// directory, whose identifiers count up.
#[must_use]
pub(crate) fn port_build(
    package: Package,
    llvm: &str,
    sequential: &Metadata,
    example: &str,
) -> Step {
    let library = format!("{}/debug/liberanos_capi.a", sequential.target_directory);
    let report = format!("examples/report.{}", package.language);
    let program = format!("examples/{example}.{}", package.language);
    let output = format!(
        "{}/{example}",
        ports_directory(package, sequential).display()
    );
    let mut arguments = sanitised(package);
    arguments.extend([
        "-pthread",
        "-Iinclude",
        "-isystem",
        "generated",
        &report,
        &program,
        &library,
    ]);
    arguments.extend(SYSTEM_LIBRARIES);
    arguments.extend(["-o", &output]);
    Step::new(
        &arguments,
        &format!("the build of the {} port of {example}", package.name),
        &format!("{llvm}/bin/{}", package.compiler),
    )
    .in_directory(package.directory)
}

/// The step that runs the port of `example` in `package`'s language, with
/// `LeakSanitizer`, which `AddressSanitizer` runs by default on Linux alone,
/// so that a port that leaks fails on macOS too.
#[must_use]
pub(crate) fn port(package: Package, example: &str, sequential: &Metadata) -> Step {
    crate::ports::port(
        package.name,
        example,
        &ports_directory(package, sequential).to_string_lossy(),
    )
    .with_variable("ASAN_OPTIONS", "detect_leaks=1")
}

/// The step that prints, as llvm-cov's JSON summary, what `package`'s tests
/// cover of its hand-written headers, the files under `include`.
#[must_use]
pub(crate) fn coverage(package: Package, llvm: &str) -> Step {
    Step::new(
        &[
            "export",
            "-summary-only",
            "build/tests",
            "-instr-profile=build/tests.profdata",
            "include",
        ],
        &format!("the {} tests' coverage", package.name),
        &format!("{llvm}/bin/llvm-cov"),
    )
    .in_directory(package.directory)
}

/// The totals of llvm-cov's JSON summary, the part [`check_coverage`] reads:
/// one export, since the gates export one program's coverage.
#[derive(Debug, Deserialize)]
struct Summary {
    data: [Export; 1],
}

/// One export of the summary: its totals over every file.
#[derive(Debug, Deserialize)]
struct Export {
    totals: Totals,
}

/// What the tests ran of the files, by kind of code.
#[derive(Debug, Deserialize)]
struct Totals {
    functions: Count,
    lines: Count,
}

/// How many of a kind of code there are, and how many ran.
#[derive(Debug, Deserialize)]
struct Count {
    count: u64,
    covered: u64,
}

/// Succeeds when `summary`, what [`coverage`] printed, shows every line and
/// every function of `package`'s layer run by its tests.
///
/// # Errors
///
/// [`Failure::Coverage`] with what ran of each, and
/// [`Failure::CoverageSummary`] when `summary` is not llvm-cov's JSON.
pub(crate) fn check_coverage(package: Package, summary: &str) -> Result<(), Failure> {
    let Summary {
        data: [Export { totals }],
    } = serde_json::from_str(summary).map_err(Failure::CoverageSummary)?;
    if totals.lines.count == totals.lines.covered
        && totals.functions.count == totals.functions.covered
    {
        Ok(())
    } else {
        Err(Failure::Coverage {
            functions: format!("{} of {}", totals.functions.covered, totals.functions.count),
            lines: format!("{} of {}", totals.lines.covered, totals.lines.count),
            package: package.name,
        })
    }
}

/// The files of `package` that clang-format checks and applies to: the
/// layer's headers, the tests, and the ports.
fn formatted(package: Package) -> Vec<&'static str> {
    let mut sources = package.headers.to_vec();
    sources.push(package.tests);
    sources.extend(package.examples);
    sources
}

/// The C and C++ packages' formatter, applied: clang-format, of the LLVM at
/// `llvm`, over the files each package's check reads.
#[must_use]
pub(crate) fn formats(llvm: &str) -> Vec<Action> {
    [C_PACKAGE, CPP_PACKAGE]
        .map(|package| {
            let mut arguments = vec!["-i"];
            arguments.extend(formatted(package));
            Action::run(
                Phase::Format,
                Step::new(
                    &arguments,
                    &format!("the {} format, applied", package.name),
                    &format!("{llvm}/bin/clang-format"),
                )
                .in_directory(package.directory),
            )
        })
        .into()
}

/// The steps that check `package`'s format with clang-format, lint its tests,
/// its ports, and every header they include with clang-tidy, and build its
/// documentation with Doxygen, each with warnings as errors.
#[must_use]
pub(crate) fn checks(package: Package, llvm: &str) -> Vec<Action> {
    let mut sources = vec!["--dry-run", "--Werror"];
    sources.extend(formatted(package));
    let extension = format!(".{}", package.language);
    let mut programs = vec!["--quiet", package.tests];
    programs.extend(
        package
            .examples
            .iter()
            .filter(|example| example.ends_with(&extension)),
    );
    programs.push("--");
    programs.extend(package.dialect);
    programs.extend(["-Iinclude", "-isystem", "generated"]);
    vec![
        Action::run(
            Phase::Lint,
            Step::new(
                &sources,
                &format!("the {} format", package.name),
                &format!("{llvm}/bin/clang-format"),
            )
            .in_directory(package.directory),
        ),
        Action::run(
            Phase::Lint,
            Step::new(
                &programs,
                &format!("the {} lint", package.name),
                &format!("{llvm}/bin/clang-tidy"),
            )
            .in_directory(package.directory),
        ),
        Action::run(
            Phase::Docs,
            Step::new(
                &["Doxyfile"],
                &format!("the {} documentation", package.name),
                "doxygen",
            )
            .in_directory(package.directory),
        ),
    ]
}

/// The step that names the shared library at `library` by the run path, so
/// an app that links it finds it through its own: macOS records the name a
/// library gives itself, which cargo makes the path it built it at.
#[cfg(target_os = "macos")]
fn install_name(library: &str) -> Vec<Step> {
    vec![Step::new(
        &[
            "-id",
            &format!("@rpath/{DLL_PREFIX}eranos_capi{DLL_SUFFIX}"),
            library,
        ],
        "the shared library's name",
        "install_name_tool",
    )]
}

/// No step: elsewhere, what links a library records the file name alone,
/// which the run path finds.
#[cfg(not(target_os = "macos"))]
const fn install_name(_library: &str) -> Vec<Step> {
    Vec::new()
}

/// The actions that build the C and C++ bundle an app consumes into its
/// directory of the packages, cleared first: eranos-capi's static and shared
/// libraries in release, in `lib`, and each package's hand-written headers
/// and the headers Diplomat generates beneath them, in `c` and `cpp`, laid
/// out as the package lays them, so an app builds with the flags the tests
/// build with.
#[must_use]
pub(crate) fn release(metadata: &Metadata) -> Vec<Action> {
    let packaged = crate::gates::packaged(metadata, "c");
    let bundle = packaged.to_string_lossy().into_owned();
    let libraries = format!("{bundle}/lib");
    let release = format!("{}/release", metadata.target_directory);
    let mut actions = vec![
        Action::clear(Phase::Package, packaged),
        Action::run(
            Phase::Package,
            Step::cargo(
                &["build", "--release", "--package", "eranos-capi"],
                "the libraries of eranos-capi, in release",
            ),
        ),
        Action::run(
            Phase::Package,
            Step::new(&["-p", &libraries], "the libraries' directory", "mkdir"),
        ),
        Action::run(
            Phase::Package,
            Step::new(
                &[
                    &format!("{release}/liberanos_capi.a"),
                    &format!("{release}/{DLL_PREFIX}eranos_capi{DLL_SUFFIX}"),
                    &libraries,
                ],
                "the libraries of eranos-capi",
                "cp",
            ),
        ),
    ];
    actions.extend(Action::runs(
        Phase::Package,
        install_name(&format!("{libraries}/{DLL_PREFIX}eranos_capi{DLL_SUFFIX}")),
    ));
    for package in [C_PACKAGE, CPP_PACKAGE] {
        let destination = format!("{bundle}/{}", package.language);
        actions.push(Action::clear(
            Phase::Package,
            directory(package, "generated"),
        ));
        actions.push(Action::run(Phase::Package, headers(package)));
        actions.extend(Action::runs(
            Phase::Package,
            vec![
                Step::new(
                    &["-p", &destination],
                    &format!("the {} headers' directory", package.name),
                    "mkdir",
                ),
                Step::new(
                    &[
                        "-R",
                        &format!("{}/include", package.directory),
                        &format!("{}/generated", package.directory),
                        &destination,
                    ],
                    &format!("the {} headers", package.name),
                    "cp",
                ),
            ],
        ));
    }
    actions
}

#[cfg(test)]
mod tests;
