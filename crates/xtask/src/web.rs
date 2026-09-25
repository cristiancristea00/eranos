//! The gates of the TypeScript package: the library for the WebAssembly
//! target of browsers and Node, the generated TypeScript in the package, and
//! the package's format, lint, type checks, tests and their coverage,
//! documentation, and bundle.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::process::{Runner, Step, WORKSPACE};

/// The TypeScript project, from the workspace root.
const PROJECT: &str = "bindings/typescript";

/// The WebAssembly target of browsers and Node, which has no operating
/// system.
const TARGET: &str = "wasm32-unknown-unknown";

/// How `node --version` starts for the Node the gates run on: the oldest
/// release line the package supports, so a call that only a later Node has
/// fails its tests.
const NODE: &str = "v24.";

/// Runs the TypeScript package's gates, stopping at the first that fails.
///
/// # Errors
///
/// [`Failure::Node`] when the `node` on `PATH` is not Node 24, before any
/// other step runs; the failure of clearing the old generated sources or the
/// old bundle, and of the first step that fails.
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    let version = runner.output(&node())?;
    if !version.starts_with(NODE) {
        return Err(Failure::Node(version.trim().to_owned()));
    }
    for directory in [generated_sources(), bundle()] {
        runner.clear(&directory)?;
    }
    for step in library() {
        runner.run(&step)?;
    }
    runner.run(&install())?;
    runner.run(&generator())?;
    for step in checks() {
        runner.run(&step)?;
    }
    for step in package() {
        runner.run(&step)?;
    }
    Ok(())
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
pub(crate) fn checks() -> Vec<Step> {
    [
        ("format:check", "Prettier"),
        ("lint", "ESLint"),
        ("typecheck", "the TypeScript type checks"),
        ("coverage", "the TypeScript tests and their coverage"),
    ]
    .into_iter()
    .map(|(script, name)| npm(script, name))
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
pub(crate) fn package() -> Vec<Step> {
    vec![
        npm("docs", "the TypeScript documentation"),
        npm("build", "the TypeScript package"),
    ]
}

/// The step that runs the package's script `script`, reported as `name`.
fn npm(script: &str, name: &str) -> Step {
    Step::new(&["run", script], name, "npm").in_directory(PROJECT)
}

#[cfg(test)]
mod tests;
