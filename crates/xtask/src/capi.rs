//! The gates of the C and C++ packages: eranos-capi's static library, the C
//! and C++ headers that Diplomat generates from it beneath each package's
//! hand-written layer, and each package's tests, built with warnings as errors
//! and run under `AddressSanitizer` and `UndefinedBehaviorSanitizer`, with their
//! coverage of the layer, format, lint, and documentation.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Failure;
use crate::gates::Metadata;
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
    /// The standard the package's code keeps to.
    standard: &'static str,
    /// The tests' source, from the package's directory.
    tests: &'static str,
    /// The hand-written headers, from the package's directory: the layer
    /// the tests cover and the formatter checks.
    pub(crate) headers: &'static [&'static str],
}

/// The C package: the hand-written C header over Diplomat's C API.
pub(crate) const C_PACKAGE: Package = Package {
    name: "C",
    directory: "bindings/c",
    language: "c",
    compiler: "clang",
    standard: "-std=c11",
    tests: "tests/tests.c",
    headers: &[
        "include/eranos.h",
        "include/eranos/calendar.h",
        "include/eranos/category.h",
        "include/eranos/contact.h",
        "include/eranos/currency.h",
        "include/eranos/detail.h",
        "include/eranos/error.h",
        "include/eranos/expense.h",
        "include/eranos/failures.h",
        "include/eranos/group.h",
        "include/eranos/group_reader.h",
        "include/eranos/identifiers.h",
        "include/eranos/members.h",
        "include/eranos/money.h",
        "include/eranos/ratios.h",
        "include/eranos/split.h",
        "include/eranos/text.h",
        "include/eranos/totals.h",
        "include/eranos/transfer.h",
    ],
};

/// The C++ package: the hand-written C++ layer over Diplomat's C++ classes.
pub(crate) const CPP_PACKAGE: Package = Package {
    name: "C++",
    directory: "bindings/cpp",
    language: "cpp",
    compiler: "clang++",
    standard: "-std=c++17",
    tests: "tests/tests.cpp",
    headers: &[
        "include/eranos.hpp",
        "include/eranos/calendar.hpp",
        "include/eranos/category.hpp",
        "include/eranos/contact.hpp",
        "include/eranos/currency.hpp",
        "include/eranos/detail/conversions.hpp",
        "include/eranos/detail/declarations.hpp",
        "include/eranos/error.hpp",
        "include/eranos/expense.hpp",
        "include/eranos/failures.hpp",
        "include/eranos/group.hpp",
        "include/eranos/group_reader.hpp",
        "include/eranos/identifiers.hpp",
        "include/eranos/members.hpp",
        "include/eranos/money.hpp",
        "include/eranos/ratios.hpp",
        "include/eranos/split.hpp",
        "include/eranos/totals.hpp",
        "include/eranos/transfer.hpp",
    ],
};

/// Runs the C and C++ packages' gates, stopping at the first that fails.
///
/// # Errors
///
/// [`Failure::Variable`] when [`VARIABLE`] is not set, before any step runs;
/// the failure of clearing or creating a package's directories, of the first
/// step that fails, and [`Failure::Coverage`] when a package's tests leave a
/// line or a function of its layer unrun.
pub(crate) fn run(runner: &mut impl Runner, metadata: &Metadata) -> Result<(), Failure> {
    let llvm = runner
        .variable(VARIABLE)
        .ok_or(Failure::Variable(VARIABLE))?;
    runner.run(&library())?;
    for package in [C_PACKAGE, CPP_PACKAGE] {
        for name in ["generated", "build"] {
            runner.clear(&directory(package, name))?;
        }
        runner.create(&directory(package, "build"))?;
        runner.run(&headers(package))?;
        for step in tests(package, &llvm, metadata) {
            runner.run(&step)?;
        }
        check_coverage(package, &runner.output(&coverage(package, &llvm))?)?;
        for step in checks(package, &llvm) {
            runner.run(&step)?;
        }
    }
    Ok(())
}

/// The step that builds eranos-capi's static library, which the tests link.
#[must_use]
pub(crate) fn library() -> Step {
    Step::cargo(
        &["build", "--package", "eranos-capi"],
        "the static library of eranos-capi",
    )
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

/// The steps that build `package`'s tests with LLVM's compiler at `llvm`,
/// with warnings as errors, `AddressSanitizer` and `UndefinedBehaviorSanitizer`,
/// whose every finding ends the run, and coverage, linked with the static
/// library in `metadata`'s target directory; run them; and merge their
/// profile for llvm-cov.
#[must_use]
pub(crate) fn tests(package: Package, llvm: &str, metadata: &Metadata) -> Vec<Step> {
    let library = format!("{}/debug/liberanos_capi.a", metadata.target_directory);
    let mut arguments = vec![
        package.standard,
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-Werror",
        "-g",
        "-fsanitize=address,undefined",
        "-fno-sanitize-recover=all",
        "-fno-omit-frame-pointer",
        "-fprofile-instr-generate",
        "-fcoverage-mapping",
        "-pthread",
        "-Iinclude",
        "-isystem",
        "generated",
        package.tests,
        &library,
    ];
    arguments.extend(SYSTEM_LIBRARIES);
    arguments.extend(["-o", "build/tests"]);
    let program = directory(package, "build").join("tests");
    vec![
        Step::new(
            &arguments,
            &format!("the {} tests' build", package.name),
            &format!("{llvm}/bin/{}", package.compiler),
        )
        .in_directory(package.directory),
        Step::new(
            &[],
            &format!("the {} tests", package.name),
            &program.to_string_lossy(),
        )
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

/// The steps that check `package`'s format with clang-format, lint its tests
/// and every header they include with clang-tidy, and build its
/// documentation with Doxygen, each with warnings as errors.
#[must_use]
pub(crate) fn checks(package: Package, llvm: &str) -> Vec<Step> {
    let mut sources = vec!["--dry-run", "--Werror"];
    sources.extend(package.headers);
    sources.push(package.tests);
    vec![
        Step::new(
            &sources,
            &format!("the {} format", package.name),
            &format!("{llvm}/bin/clang-format"),
        )
        .in_directory(package.directory),
        Step::new(
            &[
                "--quiet",
                package.tests,
                "--",
                package.standard,
                "-Iinclude",
                "-isystem",
                "generated",
            ],
            &format!("the {} lint", package.name),
            &format!("{llvm}/bin/clang-tidy"),
        )
        .in_directory(package.directory),
        Step::new(
            &["Doxyfile"],
            &format!("the {} documentation", package.name),
            "doxygen",
        )
        .in_directory(package.directory),
    ]
}

#[cfg(test)]
mod tests;
