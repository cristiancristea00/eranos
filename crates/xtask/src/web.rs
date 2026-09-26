//! The gates of the TypeScript package: the library for the WebAssembly
//! target of browsers and Node, the generated TypeScript in the package, and
//! the package's format, lint, type checks, tests and their coverage,
//! documentation, and bundle; and the steps that run the ports of the
//! examples in a copy of the package over the library whose identifiers
//! count up.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::plan::{Action, Phase};
use crate::process::{Step, WORKSPACE};

/// The TypeScript project, from the workspace root.
const PROJECT: &str = "bindings/typescript";

/// The WebAssembly target of browsers and Node, which has no operating
/// system.
pub(crate) const TARGET: &str = "wasm32-unknown-unknown";

/// How `node --version` starts for the Node the gates run on: the oldest
/// release line the package supports, so a call that only a later Node has
/// fails its tests.
const NODE: &str = "v24.";

/// The TypeScript package's gates, in order: Node's version, which must be
/// Node 24's before anything else runs, then the WebAssembly library, the
/// package's dependencies, and its generated TypeScript, in cleared
/// directories, then the package's checks and its package.
#[must_use]
pub(crate) fn actions() -> Vec<Action> {
    vec![Action::read(Phase::Prepare, node(), |version| {
        if !version.starts_with(NODE) {
            return Err(Failure::Node(version.trim().to_owned()));
        }
        let mut actions = vec![
            Action::clear(Phase::Prepare, generated_sources()),
            Action::clear(Phase::Prepare, bundle()),
        ];
        actions.extend(Action::runs(Phase::Prepare, library()));
        actions.push(Action::run(Phase::Prepare, install()));
        actions.push(Action::run(Phase::Prepare, generator()));
        actions.extend(checks());
        actions.extend(package());
        Ok(actions)
    })]
}

/// The step that prints the version of the `node` on `PATH`.
#[must_use]
pub(crate) fn node() -> Step {
    Step::new(&["--version"], "the Node version", "node")
}

/// The steps that add the WebAssembly target and build the library for it,
/// which the generator reads and the package loads.
#[must_use]
pub(crate) fn library() -> Vec<Step> {
    vec![
        Step::new(
            &["target", "add", TARGET],
            "the WebAssembly target",
            "rustup",
        ),
        Step::cargo(
            &[
                "build",
                "--package",
                "eranos-web",
                "--target",
                TARGET,
                "--lib",
            ],
            "the WebAssembly library",
        ),
    ]
}

/// The step that installs the package's dependencies exactly as its lock
/// file lists them, and fails when the lock file and `package.json`
/// disagree.
#[must_use]
pub(crate) fn install() -> Step {
    Step::new(&["ci"], "the TypeScript dependencies", "npm").in_directory(PROJECT)
}

/// Where the package keeps the generated TypeScript, the WebAssembly module,
/// and wasm-bindgen's glue; the gates clear it first, so a type the library
/// no longer has leaves no file behind.
#[must_use]
pub(crate) fn generated_sources() -> PathBuf {
    Path::new(WORKSPACE)
        .join(PROJECT)
        .join("src")
        .join("internal")
}

/// The step that generates the TypeScript from the library just built,
/// with the generator's command-line tool, which `Brewfile.develop` installs
/// from its release tag with the tag's lock file: its npm package builds the
/// tool on first use without one. `--no-wasm-build` makes it read the
/// library the step before built, rather than build it again with arguments
/// of its own.
#[must_use]
pub(crate) fn generator() -> Step {
    Step::new(
        &[
            "build",
            "wasm2",
            "--config",
            "ubrn.config.yaml",
            "--no-wasm-build",
        ],
        "the generated TypeScript",
        "uniffi-bindgen-react-native",
    )
    .in_directory(PROJECT)
}

/// The steps that check the package's format, lint it, type-check the layer
/// and its tests, and run the tests with their coverage, each with warnings
/// as errors.
#[must_use]
pub(crate) fn checks() -> Vec<Action> {
    [
        (Phase::Lint, "format:check", "Prettier"),
        (Phase::Lint, "lint", "ESLint"),
        (Phase::Lint, "typecheck", "the TypeScript type checks"),
        (
            Phase::Test,
            "coverage",
            "the TypeScript tests and their coverage",
        ),
    ]
    .into_iter()
    .map(|(phase, script, name)| Action::run(phase, npm(script, name)))
    .collect()
}

/// Where the package's build writes its bundle, declarations, and
/// documentation; the gates clear it first, so a module the layer no longer
/// has leaves no declaration in the package.
#[must_use]
pub(crate) fn bundle() -> PathBuf {
    Path::new(WORKSPACE).join(PROJECT).join("dist")
}

/// The steps that build the documentation, with warnings as errors, and the
/// package: its declarations and its bundle.
#[must_use]
pub(crate) fn package() -> Vec<Action> {
    vec![
        Action::run(Phase::Docs, npm("docs", "the TypeScript documentation")),
        Action::run(Phase::Package, npm("build", "the TypeScript package")),
    ]
}

/// The step that runs the package's script `script`, reported as `name`.
fn npm(script: &str, name: &str) -> Step {
    Step::new(&["run", script], name, "npm").in_directory(PROJECT)
}

/// Where the ports run, in `sequential`'s target directory: a copy of the
/// layer and the ports over TypeScript generated from the library whose
/// identifiers count up, since wasm-bindgen's glue follows the module it is
/// written for, and one that reads no clock imports less.
#[must_use]
pub(crate) fn copied_package(sequential: &Metadata) -> PathBuf {
    PathBuf::from(format!("{}/typescript", sequential.target_directory))
}

/// What is cleared before the package is copied: the copy's layer and its
/// ports, so a file the package no longer has leaves nothing behind.
#[must_use]
pub(crate) fn copied_directories(sequential: &Metadata) -> Vec<PathBuf> {
    let package = copied_package(sequential);
    vec![package.join("src"), package.join("examples")]
}

/// The step that builds the library for WebAssembly, with identifiers that
/// count up, into `sequential`'s target directory.
#[must_use]
pub(crate) fn sequential_library(sequential: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-web",
            "--target",
            TARGET,
            "--lib",
            "--features",
            "sequential-ids",
        ],
        "the WebAssembly library, with identifiers that count up",
    )
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// The steps that copy the layer, the ports, and the package's manifest and
/// compiler settings into the copy, and link the package's installed
/// dependencies there. The generator writes every file of the copied
/// `src/internal` again.
#[must_use]
pub(crate) fn copy(sequential: &Metadata) -> Vec<Step> {
    let package = copied_package(sequential).to_string_lossy().into_owned();
    let dependencies = format!("{WORKSPACE}/{PROJECT}/node_modules");
    vec![
        Step::new(&["-p", &package], "the TypeScript package's copy", "mkdir"),
        Step::new(
            &[
                "-R",
                &format!("{PROJECT}/src"),
                &format!("{PROJECT}/examples"),
                &format!("{PROJECT}/package.json"),
                &format!("{PROJECT}/tsconfig.json"),
                &package,
            ],
            "the TypeScript package into its copy",
            "cp",
        ),
        Step::new(
            &["-sfn", &dependencies, &format!("{package}/node_modules")],
            "the TypeScript dependencies of the copy",
            "ln",
        ),
    ]
}

/// Where the generator's settings for the copy go.
#[must_use]
pub(crate) fn copied_configuration(sequential: &Metadata) -> PathBuf {
    copied_package(sequential).join("ubrn.config.yaml")
}

/// The generator's settings for the copy: the workspace's crate, and the
/// copy's layer as where the generated TypeScript goes.
#[must_use]
pub(crate) fn configuration() -> String {
    format!(
        "rust:\n    directory: {WORKSPACE}\n    manifestPath: crates/eranos-web/Cargo.toml\nwasm2:\n    tsBindings: src/internal\n"
    )
}

/// The step that generates the copy's TypeScript from the library whose
/// identifiers count up, which the generator finds through cargo's target
/// directory.
#[must_use]
pub(crate) fn copied_generator(sequential: &Metadata) -> Step {
    Step::new(
        &[
            "build",
            "wasm2",
            "--config",
            "ubrn.config.yaml",
            "--no-wasm-build",
        ],
        "the copy's generated TypeScript",
        "uniffi-bindgen-react-native",
    )
    .in_directory(&copied_package(sequential).to_string_lossy())
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// The step that runs the TypeScript port of `example` in the copy, on Node
/// through tsx.
#[must_use]
pub(crate) fn port(example: &str, sequential: &Metadata) -> Step {
    Step::new(
        &["--import", "tsx", &format!("examples/{example}.ts")],
        &format!("the TypeScript port of {example}"),
        "node",
    )
    .in_directory(&copied_package(sequential).to_string_lossy())
}

/// The TypeScript package's formatter, applied: Prettier over the layer, its
/// tests, and the examples, installed first as the lock file pins it, since
/// it is one of the package's dependencies.
#[must_use]
pub(crate) fn formats() -> Vec<Action> {
    vec![
        Action::run(Phase::Format, install()),
        Action::run(
            Phase::Format,
            npm("format", "the TypeScript format, applied"),
        ),
    ]
}

#[cfg(test)]
mod tests;
