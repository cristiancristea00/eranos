//! The ports of the examples: every example of eranos prints a report, run
//! with every feature, which its port in each language prints byte for byte
//! on a build of the glue whose identifiers count up from one as the Rust
//! example's do. The ports run where their language builds: the Swift ones
//! where the machine builds for Apple platforms, and the Kotlin ones on
//! every machine, on the JVM that `JAVA_HOME` names.

use std::path::PathBuf;

use crate::Failure;
use crate::gates::{self, Metadata};
use crate::process::{Runner, Step};
use crate::{kotlin, swift};

/// The target directory, inside cargo's, of the glue whose identifiers
/// count up: a directory of its own, so the gates' builds stay as they are.
pub(crate) const DIRECTORY: &str = "sequential";

/// What a Rust example printed, which each of its ports prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reference {
    /// The example's name.
    pub(crate) example: String,
    /// Its standard output.
    pub(crate) report: String,
}

/// Builds the glue whose identifiers count up, runs every port of every
/// example on it, and compares each report with its example's `references`,
/// stopping at the first that fails.
///
/// # Errors
///
/// [`Failure::Variable`] when the JDK's variable is not set, before any step
/// runs; the failure of the first step that fails, of clearing an earlier
/// copy or generated source, and [`Failure::Port`] for a port whose report
/// is not its example's.
pub(crate) fn run(
    runner: &mut impl Runner,
    metadata: &Metadata,
    references: &[Reference],
) -> Result<(), Failure> {
    let jdk = runner
        .variable(kotlin::JDK)
        .ok_or(Failure::Variable(kotlin::JDK))?;
    let sequential = metadata.inner(DIRECTORY);
    let apple = runner.builds_apple_platforms();
    runner.run(&library(&sequential))?;
    for directory in cleared(&sequential, apple) {
        runner.clear(&directory)?;
    }
    for step in gates::generators(&sequential) {
        runner.run(&step)?;
    }
    if apple {
        for step in swift::copy(&sequential)
            .into_iter()
            .chain(swift::examples(&sequential))
        {
            runner.run(&step)?;
        }
        let programs = runner.output(&swift::examples_directory(&sequential))?;
        for reference in references {
            let port = port("Swift", &reference.example, programs.trim());
            compare("Swift", reference, &runner.output(&port)?)?;
        }
    }
    runner.run(&kotlin::ports(metadata))?;
    for reference in references {
        let port = kotlin::port(&reference.example, &jdk, &sequential);
        compare("Kotlin", reference, &runner.output(&port)?)?;
    }
    Ok(())
}

/// What the ports clear before they build on the library, so nothing an
/// earlier run left behind is read: the sources generated from
/// `sequential`'s library and, where `apple`, the Swift package's copy but
/// its build.
#[must_use]
pub(crate) fn cleared(sequential: &Metadata, apple: bool) -> Vec<PathBuf> {
    let mut directories = vec![gates::generated(sequential)];
    if apple {
        directories.extend(swift::copied_directories(sequential));
    }
    directories
}

/// The step that builds eranos-uniffi's library for the host, with
/// identifiers that count up, into `sequential`'s target directory.
#[must_use]
pub(crate) fn library(sequential: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-uniffi",
            "--features",
            "sequential-ids",
        ],
        "the host library of eranos-uniffi, with identifiers that count up",
    )
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// The step that runs the port of `example` in `language`, a program in
/// the directory `programs`.
#[must_use]
pub(crate) fn port(language: &str, example: &str, programs: &str) -> Step {
    Step::new(
        &[],
        &format!("the {language} port of {example}"),
        &format!("{programs}/{example}"),
    )
}

/// Succeeds when `report`, what a port in `language` printed, is the
/// `reference`'s report byte for byte.
///
/// # Errors
///
/// [`Failure::Port`] with the first line that differs and both versions of
/// it, the end of a report standing for a line it lacks.
pub(crate) fn compare(
    language: &'static str,
    reference: &Reference,
    report: &str,
) -> Result<(), Failure> {
    if reference.report == report {
        return Ok(());
    }
    let expected: Vec<&str> = reference.report.split('\n').collect();
    let found: Vec<&str> = report.split('\n').collect();
    let index = expected
        .iter()
        .zip(&found)
        .position(|(expected, found)| expected != found)
        .unwrap_or_else(|| expected.len().min(found.len()));
    let line = |lines: &[&str]| {
        lines.get(index).map_or_else(
            || "the end of the report".to_owned(),
            |line| format!("{line:?}"),
        )
    };
    Err(Failure::Port {
        example: reference.example.clone(),
        expected: line(&expected),
        found: line(&found),
        language,
        line: index + 1,
    })
}

#[cfg(test)]
mod tests;
