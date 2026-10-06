//! The gates of the Python package: the generated module in the package,
//! beside a link to the host library it loads, the package's environment as
//! its lock file pins it, and the package's format, lint, type checks, tests
//! and their coverage, and documentation; and a copy of the package over
//! another build of the library, whose tests the coverage runs and whose
//! ports the ports gate runs.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::{Path, PathBuf};

use crate::gates::Metadata;
use crate::plan::{Action, Phase};
use crate::process::{Step, WORKSPACE};

/// The Python project, from the workspace root.
pub(crate) const PROJECT: &str = "bindings/python";

/// Where the package holds the generated module and the link to the
/// library, from the project.
const INTERNAL: &str = "src/eranos/_internal";

/// The Python package's gates, in order: the generated module and the link
/// to the library, the environment, a cleared documentation directory, then
/// the package's checks.
#[must_use]
pub(crate) fn actions(metadata: &Metadata) -> Vec<Action> {
    let mut actions = Action::runs(
        Phase::Prepare,
        vec![module(metadata), library(metadata), environment()],
    );
    actions.push(Action::clear(Phase::Prepare, documentation()));
    actions.extend(checks());
    actions
}

/// The step that copies the module generated from the host library into the
/// package, whose layer imports it.
#[must_use]
pub(crate) fn module(metadata: &Metadata) -> Step {
    module_into(metadata, &format!("{WORKSPACE}/{PROJECT}"))
}

/// The step that links the host library beside the generated module, which
/// loads it from its own directory: a link, so a rebuild of the library
/// never leaves a stale copy beside a newer module.
#[must_use]
pub(crate) fn library(metadata: &Metadata) -> Step {
    library_into(metadata, &format!("{WORKSPACE}/{PROJECT}"))
}

/// The step that copies the module generated from `metadata`'s library into
/// the package at `package`.
fn module_into(metadata: &Metadata, package: &str) -> Step {
    let module = format!("{}/uniffi/python/eranos.py", metadata.target_directory);
    let destination = format!("{package}/{INTERNAL}");
    Step::new(&[&module, &destination], "the generated Python", "cp")
}

/// The step that links `metadata`'s library beside the generated module of
/// the package at `package`.
fn library_into(metadata: &Metadata, package: &str) -> Step {
    let name = format!("{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}");
    let library = format!("{}/debug/{name}", metadata.target_directory);
    let link = format!("{package}/{INTERNAL}/{name}");
    Step::new(
        &["-sf", &library, &link],
        "the link to the host library",
        "ln",
    )
}

/// The step that installs the package's tools exactly as its lock file
/// lists them, and fails when the lock file and `pyproject.toml` disagree.
#[must_use]
pub(crate) fn environment() -> Step {
    Step::new(&["sync", "--locked"], "the Python environment", "uv").in_directory(PROJECT)
}

/// Where Sphinx writes the documentation; the gates clear it first, so a
/// page of a name the package no longer has is not left behind.
#[must_use]
pub(crate) fn documentation() -> PathBuf {
    Path::new(WORKSPACE).join(PROJECT).join("build")
}

/// A check the gates run through uv, in the package's environment.
struct Check {
    /// The tool and its arguments.
    command: &'static [&'static str],
    /// The step's name.
    name: &'static str,
    /// The phase it belongs to.
    phase: Phase,
}

/// The steps that check the package's format, lint it, type-check it, run
/// its tests with their coverage of every line and branch, and build its
/// documentation, each with warnings as errors.
#[must_use]
pub(crate) fn checks() -> Vec<Action> {
    [
        Check {
            command: &["ruff", "format", "--check"],
            name: "the Python format",
            phase: Phase::Lint,
        },
        Check {
            command: &["ruff", "check", "--no-fix"],
            name: "the Python lint",
            phase: Phase::Lint,
        },
        Check {
            command: &["ty", "check"],
            name: "the Python type checks",
            phase: Phase::Lint,
        },
        Check {
            command: &["coverage", "run", "-m", "pytest"],
            name: "the Python tests",
            phase: Phase::Test,
        },
        Check {
            command: &["coverage", "report"],
            name: "the Python tests' coverage",
            phase: Phase::Test,
        },
        Check {
            command: &[
                "sphinx-build",
                "-W",
                "--keep-going",
                "-n",
                "-q",
                "-b",
                "html",
                "docs",
                "build/docs",
            ],
            name: "the Python documentation",
            phase: Phase::Docs,
        },
    ]
    .into_iter()
    .map(|check| Action::run(check.phase, tool(check.command, check.name)))
    .collect()
}

/// Where the Python tests and ports run on another build of the library, in
/// `metadata`'s target directory: a copy of the package, since the
/// generated module loads the library from its own directory.
#[must_use]
pub(crate) fn copied_package(metadata: &Metadata) -> PathBuf {
    PathBuf::from(format!("{}/python", metadata.target_directory))
}

/// What is cleared before the package is copied: the copy's layer, its
/// tests, and its ports, so a file the package no longer has leaves nothing
/// behind; the copy's environment stays, so a later run installs only what
/// changed.
#[must_use]
pub(crate) fn copied_directories(metadata: &Metadata) -> Vec<PathBuf> {
    let package = copied_package(metadata);
    vec![
        package.join("src"),
        package.join("tests"),
        package.join("examples"),
    ]
}

/// The steps that copy the layer, its tests, its ports, and the project's
/// settings and lock file into the copy, then the module generated from
/// `metadata`'s library and a link to that library beside it, and install
/// the copy's environment as the lock file pins it.
#[must_use]
pub(crate) fn copy(metadata: &Metadata) -> Vec<Step> {
    let package = copied_package(metadata).to_string_lossy().into_owned();
    vec![
        Step::new(&["-p", &package], "the Python package's copy", "mkdir"),
        Step::new(
            &[
                "-R",
                &format!("{PROJECT}/.python-version"),
                &format!("{PROJECT}/examples"),
                &format!("{PROJECT}/pyproject.toml"),
                &format!("{PROJECT}/src"),
                &format!("{PROJECT}/tests"),
                &format!("{PROJECT}/uv.lock"),
                &package,
            ],
            "the Python package into its copy",
            "cp",
        ),
        module_into(metadata, &package),
        library_into(metadata, &package),
        Step::new(&["sync", "--locked"], "the copy's Python environment", "uv")
            .in_directory(&package),
    ]
}

/// The step that runs the copy's tests on its library.
#[must_use]
pub(crate) fn copy_tests(metadata: &Metadata) -> Step {
    Step::new(&["run", "--locked", "pytest"], "the Python tests", "uv")
        .in_directory(&copied_package(metadata).to_string_lossy())
}

/// The step that runs the port of `example` in the copy in `sequential`'s
/// target directory, over the library whose identifiers count up: the
/// module of the port run with the layer on the path, and its output in
/// UTF-8 whatever the locale, which the tables' borders need.
#[must_use]
pub(crate) fn port(example: &str, sequential: &Metadata) -> Step {
    Step::new(
        &[
            "run",
            "--locked",
            "python",
            "-m",
            &format!("examples.{example}"),
        ],
        &format!("the Python port of {example}"),
        "uv",
    )
    .with_variable("PYTHONPATH", "src")
    .with_variable("PYTHONUTF8", "1")
    .in_directory(&copied_package(sequential).to_string_lossy())
}

/// The actions that build the Python package's wheel for the host into its
/// directory of the packages, cleared first: maturin builds eranos-uniffi's
/// library in release, runs the workspace's `uniffi-bindgen` on it, and packs
/// the layer beside both, as `pyproject.toml` sets it.
#[must_use]
pub(crate) fn release(metadata: &Metadata) -> Vec<Action> {
    let packaged = crate::gates::packaged(metadata, "python");
    let wheel = packaged.to_string_lossy().into_owned();
    vec![
        Action::clear(Phase::Package, packaged),
        Action::run(
            Phase::Package,
            tool(
                &["maturin", "build", "--release", "--out", &wheel],
                "the Python wheel",
            ),
        ),
    ]
}

/// The Python package's formatter, applied: Ruff's, over the layer, its
/// tests, the examples, and the documentation's settings.
#[must_use]
pub(crate) fn formats() -> Vec<Action> {
    vec![Action::run(
        Phase::Format,
        tool(&["ruff", "format"], "the Python format, applied"),
    )]
}

/// The step that runs `command` with the tools of the locked environment,
/// reported as `name`.
fn tool(command: &[&str], name: &str) -> Step {
    let mut arguments = vec!["run", "--locked"];
    arguments.extend(command);
    Step::new(&arguments, name, "uv").in_directory(PROJECT)
}

#[cfg(test)]
mod tests;
