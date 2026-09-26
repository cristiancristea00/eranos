//! The commit gates, in the order AGENTS.md lists them.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::Failure;
use crate::plan;
use crate::ports::Reference;
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

    /// The metadata of the target directory `name` inside this one, such as
    /// the one cargo-llvm-cov builds in, without the packages: the steps that
    /// read a target directory read it from there.
    #[must_use]
    pub(crate) fn inner(&self, name: &str) -> Self {
        Self {
            packages: Vec::new(),
            target_directory: format!("{}/{name}", self.target_directory),
        }
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
    // Every package's variables are checked here, before the examples run,
    // so a missing one stops the gates before minutes of gates that pass.
    let packages = [
        crate::kotlin::actions(runner, &metadata),
        crate::java::actions(runner, &metadata),
        Ok(crate::web::actions()),
        crate::kmp::actions(runner),
        crate::capi::actions(runner, &metadata),
        Ok(crate::python::actions(&metadata)),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    let mut references = Vec::new();
    for example in examples(&metadata) {
        let report = runner.output(&example.step)?;
        if "eranos" == example.package {
            references.push(Reference {
                example: example.name,
                report,
            });
        }
    }
    runner.run(&host_library())?;
    runner.clear(&generated(&metadata))?;
    for step in generators(&metadata) {
        runner.run(&step)?;
    }
    if runner.builds_apple_platforms() {
        plan::execute(runner, crate::swift::actions(&metadata), |_| true)?;
    }
    for actions in packages {
        plan::execute(runner, actions, |_| true)?;
    }
    crate::ports::run(runner, &metadata, &references)?;
    println!("\nevery gate passed");
    Ok(())
}

/// The mobile targets the libraries are linted for: an iOS device and
/// simulator and two Android ABIs, whose standard libraries type-check the
/// `target_os` code of the library and its dependencies without an SDK.
const MOBILE_TARGETS: [&str; 4] = [
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "aarch64-linux-android",
    "x86_64-linux-android",
];

/// The wasm target of the WebAssembly System Interface, which compiles the
/// library's `target_family = "wasm"` code as the browser's target does,
/// with the clock and the random source that the browser's has only through
/// uuid's `js` feature.
const WASI_TARGET: &str = "wasm32-wasip1";

/// The gates before the examples: [`host_lints`], [`target_lints`], and
/// [`tests_and_documentation`], in that order.
#[must_use]
pub(crate) fn checks() -> Vec<Step> {
    [host_lints(), target_lints(), tests_and_documentation()].concat()
}

/// The gates that lint the workspace for the host: formatting, and clippy on
/// every target with every feature and without the default ones.
#[must_use]
pub(crate) fn host_lints() -> Vec<Step> {
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
    ]
}

/// The gate that lints the libraries for the mobile and wasm targets, with
/// every feature and without the default ones, as CI's job of that name
/// does: clippy type-checks each target's code against its standard library
/// alone, so no SDK, NDK, or linker is needed. eranos-capi is left out on
/// WASI: its worker thread drives each asynchronous C call with `block_on`,
/// which is not compiled for wasm, where the one thread would wait with no
/// other to wake it. The browser's target lints eranos-web alone, which
/// turns on uuid's `js` feature, without which that target has no clock or
/// random source.
#[must_use]
pub(crate) fn target_lints() -> Vec<Step> {
    let clippy = |scope: &[&str], targets: &[&str], name: &str| {
        [
            ("--all-features", "with every feature"),
            ("--no-default-features", "without the default features"),
        ]
        .map(|(features, set)| {
            let mut arguments = vec!["clippy"];
            arguments.extend(scope);
            arguments.extend(["--lib", features]);
            for target in targets {
                arguments.extend(["--target", target]);
            }
            arguments.extend(["--", "-D", "warnings"]);
            Step::cargo(&arguments, &format!("clippy for {name} {set}"))
        })
    };
    let mut add = vec!["target", "add"];
    add.extend(MOBILE_TARGETS);
    add.extend([WASI_TARGET, crate::web::TARGET]);
    let mut steps = vec![Step::new(&add, "the mobile and wasm targets", "rustup")];
    steps.extend(clippy(
        &["--workspace"],
        &MOBILE_TARGETS,
        "the mobile targets",
    ));
    steps.extend(clippy(
        &["--workspace", "--exclude", "eranos-capi"],
        &[WASI_TARGET],
        "WASI",
    ));
    steps.extend(clippy(
        &["--package", "eranos-web"],
        &[crate::web::TARGET],
        "the browser",
    ));
    steps
}

/// The gates after the lints: the tests with every feature and without the
/// default ones, and the documentation.
#[must_use]
pub(crate) fn tests_and_documentation() -> Vec<Step> {
    vec![
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

/// The step that lists the workspace's packages and their targets, and
/// names cargo's target directory.
#[must_use]
pub(crate) fn metadata() -> Step {
    Step::cargo(
        &["metadata", "--no-deps", "--format-version", "1"],
        "the workspace's packages",
    )
}

/// An example of a workspace package, and the step that runs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Example {
    /// The example's name.
    pub(crate) name: String,
    /// The package whose example it is.
    pub(crate) package: String,
    /// The step that runs it, with every feature.
    pub(crate) step: Step,
}

/// Every example of every workspace package, in the order `cargo metadata`
/// lists them. Every example asserts its key numbers, so running one is a
/// check.
#[must_use]
pub(crate) fn examples(metadata: &Metadata) -> Vec<Example> {
    metadata
        .packages
        .iter()
        .flat_map(|package| {
            package
                .targets
                .iter()
                .filter(|target| target.kind == ["example"])
                .map(|target| Example {
                    name: target.name.clone(),
                    package: package.name.clone(),
                    step: Step::cargo(
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
                    ),
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

/// The steps that generate the Swift, Kotlin, Java, and Python sources from
/// the host library of eranos-uniffi, each with the generator of
/// eranos-uniffi-bindgen that the library's `uniffi` release builds, so a
/// type a generator cannot read stops the gates. Their paths are written
/// with `/`, which the programs of every host accept, so a step reads alike
/// on every host.
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
    let (swift, kotlin, java, python) = (
        directory("swift"),
        directory("kotlin"),
        directory("java"),
        directory("python"),
    );
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
        generator(
            "uniffi-bindgen",
            &[
                "--library",
                &library,
                "--language",
                "python",
                "--out-dir",
                &python,
                "--no-format",
            ],
            "the Python sources",
        ),
    ]
}

#[cfg(test)]
mod tests;
