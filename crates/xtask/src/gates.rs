//! The commit gates, in the order AGENTS.md lists them.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Failure;
use crate::process::{Runner, Step};

/// The part of `cargo metadata`'s JSON the gates read.
#[derive(Debug, Deserialize)]
pub(crate) struct Metadata {
    packages: Vec<Package>,
    pub(crate) target_directory: String,
}

impl Metadata {
    /// The metadata in `json`, what `cargo metadata` printed.
    ///
    /// # Errors
    ///
    /// [`Failure::Metadata`] when `json` is not the JSON of `cargo
    /// metadata`.
    pub(crate) fn parse(json: &str) -> Result<Self, Failure> {
        serde_json::from_str(json).map_err(Failure::Metadata)
    }
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
    let metadata = Metadata::parse(&runner.output(&metadata())?)?;
    for step in examples(&metadata) {
        runner.run(&step)?;
    }
    runner.run(&host_library())?;
    runner.clear(&generated(&metadata))?;
    for step in generators(&metadata) {
        runner.run(&step)?;
    }
    if runner.builds_apple_platforms() {
        crate::swift::run(runner, &metadata)?;
    }
    crate::kotlin::run(runner, &metadata)?;
    crate::java::run(runner, &metadata)?;
    crate::web::run(runner)?;
    crate::kmp::run(runner)?;
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

/// One step per example of every workspace package, in the order `cargo
/// metadata` lists them. Every example asserts its key numbers, so running
/// one is a check.
#[must_use]
pub(crate) fn examples(metadata: &Metadata) -> Vec<Step> {
    metadata
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
        .collect()
}

/// The step that builds eranos-uniffi's library for the host, which the
/// generators read.
#[must_use]
pub(crate) fn host_library() -> Step {
    Step::cargo(
        &["build", "--package", "eranos-uniffi"],
        "the host library of eranos-uniffi",
    )
}

/// Where the generated sources go, one directory per language, under
/// cargo's target directory; the gates clear it first, so a type the
/// library no longer has leaves no file behind.
#[must_use]
pub(crate) fn generated(metadata: &Metadata) -> PathBuf {
    Path::new(&metadata.target_directory).join("uniffi")
}

/// The steps that generate the Swift, Kotlin, and Java sources from the host
/// library of eranos-uniffi, each with the generator of eranos-uniffi-bindgen
/// that the library's `uniffi` release builds, so a type a generator cannot
/// read stops the gates. Their paths are written with `/`, which the programs
/// of every host accept, so a step reads alike on every host.
#[must_use]
pub(crate) fn generators(metadata: &Metadata) -> Vec<Step> {
    let library = format!(
        "{}/debug/{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}",
        metadata.target_directory
    );
    let directory = |language: &str| format!("{}/uniffi/{language}", metadata.target_directory);
    let generator = |bin: &str, arguments: &[&str], name: &str| {
        let mut command = vec![
            "run",
            "--quiet",
            "--package",
            "eranos-uniffi-bindgen",
            "--bin",
            bin,
            "--",
            "generate",
        ];
        command.extend(arguments);
        Step::cargo(&command, name)
    };
    let (swift, kotlin, java) = (directory("swift"), directory("kotlin"), directory("java"));
    vec![
        generator(
            "uniffi-bindgen",
            &[
                "--library",
                &library,
                "--language",
                "swift",
                "--out-dir",
                &swift,
                "--no-format",
            ],
            "the Swift sources",
        ),
        generator(
            "uniffi-bindgen",
            &[
                "--library",
                &library,
                "--language",
                "kotlin",
                "--out-dir",
                &kotlin,
                "--no-format",
            ],
            "the Kotlin sources",
        ),
        generator(
            "uniffi-bindgen-java",
            &["--out-dir", &java, "--no-format", &library],
            "the Java sources",
        ),
    ]
}

#[cfg(test)]
mod tests;
