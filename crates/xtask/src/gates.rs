//! The commit gates, in the order AGENTS.md lists them.

use serde::Deserialize;

use crate::Failure;
use crate::process::{Runner, Step};

/// The part of `cargo metadata`'s JSON the gates read.
#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

/// A workspace package, with its targets.
#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    targets: Vec<Target>,
}

/// A target of a package: a library, a binary, a test, an example, a bench.
#[derive(Debug, Deserialize)]
struct Target {
    kind: Vec<String>,
    name: String,
}

/// Runs every gate, stopping at the first that fails.
///
/// # Errors
///
/// The failure of the first gate that fails, and [`Failure::Metadata`] when
/// the list of examples cannot be read.
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    for step in checks() {
        runner.run(&step)?;
    }
    check_lock_features(&runner.output(&lock_features())?)?;
    let metadata = runner.output(&metadata())?;
    for step in examples(&metadata)? {
        runner.run(&step)?;
    }
    println!("\nevery gate passed");
    Ok(())
}

/// The gates before the examples: formatting, clippy and the tests with
/// every feature and without the default ones, and the documentation.
#[must_use]
pub(crate) fn checks() -> Vec<Step> {
    vec![
        Step::cargo(&["fmt", "--all", "--check"], "formatting"),
        Step::cargo(
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            "clippy with every feature",
        ),
        Step::cargo(
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--no-default-features",
                "--",
                "-D",
                "warnings",
            ],
            "clippy without the default features",
        ),
        Step::cargo(
            &["test", "--workspace", "--all-features"],
            "tests with every feature",
        ),
        Step::cargo(
            &["test", "--workspace", "--no-default-features"],
            "tests without the default features",
        ),
        Step::cargo(
            &["doc", "--workspace", "--no-deps", "--all-features"],
            "documentation",
        )
        .with_variable("RUSTDOCFLAGS", "-D warnings"),
    ]
}

/// The step that prints async-lock's dependents with the features they turn
/// on in it, each as a line `async-lock feature "<name>"`.
#[must_use]
pub(crate) fn lock_features() -> Step {
    Step::cargo(
        &[
            "tree",
            "--workspace",
            "--all-features",
            "--edges",
            "normal,features",
            "--invert",
            "async-lock",
            "--prefix",
            "none",
        ],
        "async-lock's features",
    )
}

/// Succeeds when `tree`, what [`lock_features`] printed, shows no feature
/// of async-lock turned on. With `std`, the lock gains blocking methods that
/// read the clock and park threads through a per-thread parker, which the
/// library keeps out.
///
/// # Errors
///
/// [`Failure::LockFeatures`] with the features that are on.
pub(crate) fn check_lock_features(tree: &str) -> Result<(), Failure> {
    let mut features: Vec<&str> = tree
        .lines()
        .filter_map(|line| line.strip_prefix("async-lock feature "))
        .map(|feature| feature.trim_end_matches(" (*)"))
        .collect();
    features.sort_unstable();
    features.dedup();
    if features.is_empty() {
        Ok(())
    } else {
        Err(Failure::LockFeatures(features.join(", ")))
    }
}

/// The step that lists the workspace's packages and their targets.
#[must_use]
pub(crate) fn metadata() -> Step {
    Step::cargo(
        &["metadata", "--no-deps", "--format-version", "1"],
        "the list of examples",
    )
}

/// One step per example of every workspace package, in the order `metadata`,
/// the JSON `cargo metadata` prints, lists them. Every example asserts its
/// key numbers, so running one is a check.
///
/// # Errors
///
/// [`Failure::Metadata`] when `metadata` is not the JSON of `cargo metadata`.
pub(crate) fn examples(metadata: &str) -> Result<Vec<Step>, Failure> {
    let metadata: Metadata = serde_json::from_str(metadata).map_err(Failure::Metadata)?;
    Ok(metadata
        .packages
        .iter()
        .flat_map(|package| {
            package
                .targets
                .iter()
                .filter(|target| target.kind == ["example"])
                .map(|target| {
                    Step::cargo(
                        &[
                            "run",
                            "--quiet",
                            "--package",
                            &package.name,
                            "--all-features",
                            "--example",
                            &target.name,
                        ],
                        &format!("example {}", target.name),
                    )
                })
        })
        .collect())
}

#[cfg(test)]
mod tests;
