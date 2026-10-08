//! The coverage of the Rust tests and of the binding glue, in one report:
//! cargo-llvm-cov's runs of the workspace's tests with every feature and
//! without the default ones, then eranos-uniffi and eranos-capi built with
//! cargo-llvm-cov's instrumentation, whose shared libraries the Swift tests
//! (where the machine builds for Apple platforms), the Kotlin, Java, C,
//! C++, and Python tests load, each writing the coverage of the glue it
//! runs. The glue
//! has no Rust tests: it forwards every call to the binding core, which the
//! Rust tests measure. eranos-web compiles the glue's sources again, for
//! WebAssembly, which stable Rust cannot measure, so the report leaves it
//! out.

use std::path::PathBuf;

use crate::Failure;
use crate::gates::{self, Metadata};
use crate::process::{Runner, Step, Variable};
use crate::{capi, java, kotlin, python, swift};

/// The crates the Rust runs leave out: the glue and the generators, which
/// have no Rust tests, and the glue's proc macros, which run inside the
/// compiler, only when a crate that invokes them compiles, so a cached
/// build would leave them unrun; eranos-glue's tests measure what they
/// call.
pub(crate) const GLUE: [&str; 5] = [
    "eranos-capi",
    "eranos-glue-macros",
    "eranos-uniffi",
    "eranos-uniffi-bindgen",
    "eranos-web",
];

/// cargo-llvm-cov's target directory, inside cargo's, where the Rust runs
/// and the instrumented glue build, so the gates' builds stay as they are.
pub(crate) const DIRECTORY: &str = "llvm-cov-target";

/// Measures the Rust tests and the glue, and prints the report, stopping at
/// the first step that fails.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable the Kotlin or the C and C++ tests
/// need that is not set, before any step runs; the failure of the first step
/// that fails, of reading cargo's metadata, of clearing an earlier copy or
/// generated source, and of creating the directories the C and C++ tests
/// build in.
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    if let Some(name) = kotlin::VARIABLES
        .into_iter()
        .find(|name| runner.variable(name).is_none())
    {
        return Err(Failure::Variable(name));
    }
    let llvm = runner
        .variable(capi::VARIABLE)
        .ok_or(Failure::Variable(capi::VARIABLE))?;
    let metadata = Metadata::parse(&runner.output(&gates::metadata())?)?;
    let measured = metadata.inner(DIRECTORY);
    let environment = environment(&runner.output(&show_environment(&measured))?);
    let apple = runner.builds_apple_platforms();
    for directory in cleared(&measured, apple) {
        runner.clear(&directory)?;
    }
    for directory in created(&measured) {
        runner.create(&directory)?;
    }
    for step in steps(&measured, &llvm, &environment, apple) {
        runner.run(&step)?;
    }
    Ok(())
}

/// What the coverage clears before it runs anything, so nothing an earlier
/// run left behind is read: the sources generated from `measured`'s
/// instrumented library, where `apple` the Swift package's copy but its
/// build, the generated sources the Kotlin, Java, C, and C++ packages
/// compile, and the Python package's copy but its environment.
#[must_use]
pub(crate) fn cleared(measured: &Metadata, apple: bool) -> Vec<PathBuf> {
    let mut directories = vec![gates::generated(measured)];
    if apple {
        directories.extend(swift::copied_directories(measured));
    }
    directories.extend([
        kotlin::generated_sources(),
        java::generated_sources(),
        capi::directory(capi::C_PACKAGE, "generated"),
        capi::directory(capi::CPP_PACKAGE, "generated"),
    ]);
    directories.extend(python::copied_directories(measured));
    directories
}

/// The directories the C and C++ tests build in, in `measured`'s target
/// directory.
#[must_use]
pub(crate) fn created(measured: &Metadata) -> Vec<PathBuf> {
    [capi::C_PACKAGE, capi::CPP_PACKAGE]
        .map(|package| capi::measured_directory(package, measured))
        .to_vec()
}

/// Every step, in order: the Rust runs, the instrumented glue, the sources
/// generated from it, the Swift tests where `apple`, the Kotlin, Java, C,
/// C++, and Python tests, each suite in `environment`, which writes its
/// coverage where the report reads it, and the report.
#[must_use]
pub(crate) fn steps(
    measured: &Metadata,
    llvm: &str,
    environment: &[Variable],
    apple: bool,
) -> Vec<Step> {
    let measuring = |steps: Vec<Step>| {
        steps
            .into_iter()
            .map(|step| step.with_environment(environment))
            .collect::<Vec<_>>()
    };
    let mut steps = rust();
    steps.push(libraries(measured).with_environment(environment));
    steps.extend(gates::generators(measured));
    if apple {
        steps.extend(measuring(swift::copy(measured)));
        steps.extend(measuring(vec![swift::copy_tests(measured)]));
    }
    steps.extend(kotlin::sources(measured));
    steps.extend(measuring(vec![kotlin::tests(measured)]));
    steps.extend(java::sources(measured));
    steps.extend(measuring(vec![java::tests(measured)]));
    for package in [capi::C_PACKAGE, capi::CPP_PACKAGE] {
        steps.push(capi::headers(package));
        steps.extend(measuring(capi::measured_tests(package, llvm, measured)));
    }
    steps.extend(python::copy(measured));
    steps.extend(measuring(vec![python::copy_tests(measured)]));
    steps.push(report());
    steps
}

/// The steps that drop the earlier runs' coverage, then run the workspace's
/// tests under cargo-llvm-cov with every feature and without the default
/// ones, leaving out [`GLUE`]: each feature set compiles code the other
/// leaves out, the serde code and one of the two identifier generators.
#[must_use]
pub(crate) fn rust() -> Vec<Step> {
    let tests = |features: &str, name: &str| {
        let mut arguments = vec!["llvm-cov", "--no-report", "--workspace", features];
        for package in GLUE {
            arguments.extend(["--exclude", package]);
        }
        Step::cargo(&arguments, name)
    };
    vec![
        Step::cargo(
            &["llvm-cov", "clean", "--workspace"],
            "the earlier runs' coverage",
        ),
        tests("--all-features", "the Rust tests with every feature"),
        tests(
            "--no-default-features",
            "the Rust tests without the default features",
        ),
    ]
}

/// The step that prints, as exports a shell reads, the environment in which
/// a build of cargo's is instrumented as cargo-llvm-cov's own are, into
/// `measured`'s target directory, and in which a program writes its
/// coverage there.
#[must_use]
pub(crate) fn show_environment(measured: &Metadata) -> Step {
    Step::cargo(
        &["llvm-cov", "show-env", "--sh"],
        "cargo-llvm-cov's environment",
    )
    .with_variable("CARGO_TARGET_DIR", &measured.target_directory)
}

/// The variables `exports` sets, what [`show_environment`] printed: a line
/// `export NAME=value` each, a value that a shell would split quoted as
/// `'value'`, and a quote inside it written `'\''`. A line of another shape
/// sets nothing.
#[must_use]
pub(crate) fn environment(exports: &str) -> Vec<Variable> {
    exports
        .lines()
        .filter_map(|line| {
            let (name, written) = line.strip_prefix("export ")?.split_once('=')?;
            let value = match written.strip_prefix('\'') {
                Some(quoted) => quoted.strip_suffix('\'')?.replace("'\\''", "'"),
                None => written.to_owned(),
            };
            Some(Variable::new(name, &value))
        })
        .collect()
}

/// The step that builds eranos-uniffi's and eranos-capi's libraries for the
/// host into `measured`'s target directory, which the environment of
/// [`show_environment`] instruments.
#[must_use]
pub(crate) fn libraries(measured: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-uniffi",
            "--package",
            "eranos-capi",
        ],
        "the glue, instrumented",
    )
    .with_variable("CARGO_TARGET_DIR", &measured.target_directory)
}

/// The files of the glue's proc macros, which the report skips: the runs
/// build them instrumented, as a dependency of the crates they test, but the
/// macros run inside the compiler only when a crate that invokes them
/// compiles, so a report that kept them would read them as run or unrun as
/// the build cache decides.
pub(crate) const MACROS: &str = "eranos-glue-macros/";

/// The step that prints the report of every run's coverage, a line per file
/// with its regions, functions, and lines, the files of [`MACROS`] left out.
#[must_use]
pub(crate) fn report() -> Step {
    Step::cargo(
        &["llvm-cov", "report", "--ignore-filename-regex", MACROS],
        "the report",
    )
}

#[cfg(test)]
mod tests;
