//! The repository's automation, run as `cargo xtask <command>` through the
//! alias in `.cargo/config.toml`.
//!
//! `cargo xtask gates` runs the commit gates AGENTS.md lists, in its order,
//! and stops at the first that fails; `cargo xtask coverage` measures the
//! Rust tests and the binding glue, which the foreign tests run, in one
//! report; `cargo xtask help` lists the commands.
#![forbid(unsafe_code)]

mod capi;
mod coverage;
mod gates;
mod java;
mod kmp;
mod kotlin;
mod ports;
mod process;
mod swift;
mod web;

use std::io;
use std::process::{ExitCode, ExitStatus};

use crate::process::{Runner, System};

/// What `help` prints, and what a command line xtask does not understand
/// prints after its error.
const USAGE: &str = "usage: cargo xtask <command>

commands:
  coverage  measure the Rust tests and the binding glue, which the foreign tests run, in one report
  gates     run every commit gate AGENTS.md lists, stopping at the first failure
  help      print this message";

/// Why a command of xtask failed.
#[derive(Debug, thiserror::Error)]
enum Failure {
    /// A directory a gate writes into could not be emptied first.
    #[error("{directory} could not be cleared: {source}")]
    Clear {
        directory: String,
        #[source]
        source: io::Error,
    },
    /// A package's tests leave a line or a function of its hand-written layer
    /// unrun.
    #[error(
        "the {package} tests run {lines} lines and {functions} functions of the {package} layer, which they cover whole"
    )]
    Coverage {
        functions: String,
        lines: String,
        package: &'static str,
    },
    /// llvm-cov printed a coverage summary that is not the JSON the gates
    /// read.
    #[error("llvm-cov printed a coverage summary the gates cannot read: {0}")]
    CoverageSummary(#[source] serde_json::Error),
    /// A directory a gate writes into could not be created.
    #[error("{directory} could not be created: {source}")]
    Create {
        directory: String,
        #[source]
        source: io::Error,
    },
    /// A package turns on features of async-lock, which the library keeps
    /// without any.
    #[error("async-lock is built with {0}, which the library keeps off")]
    LockFeatures(String),
    /// The output of `cargo metadata` is not the JSON the gates read.
    #[error("cargo metadata printed JSON the gates cannot read: {0}")]
    Metadata(#[source] serde_json::Error),
    /// The `node` on `PATH` is not Node 24, which the TypeScript package's
    /// gates run on.
    #[error(
        "the gates run on Node 24, the oldest the package supports, and found {0}; docs/DEVELOPMENT.md § 1 says how to put it first on PATH"
    )]
    Node(String),
    /// A port of an example printed another report than the Rust example's.
    #[error(
        "the {language} port of {example} prints {found} at line {line}, where the Rust example prints {expected}"
    )]
    Port {
        example: String,
        expected: String,
        found: String,
        language: &'static str,
        line: usize,
    },
    /// A step's program could not be started.
    #[error("{step}: the program could not be started: {source}")]
    Start {
        #[source]
        source: io::Error,
        step: String,
    },
    /// A step ran and did not succeed.
    #[error("{step} failed ({status})")]
    Step { status: ExitStatus, step: String },
    /// The command line names no command xtask knows.
    #[error("unknown command\n\n{USAGE}")]
    Usage,
    /// An environment variable a gate needs, such as the Android SDK's
    /// location, is not set.
    #[error("{0} is not set; docs/DEVELOPMENT.md § 1 says where it points")]
    Variable(&'static str),
    /// A file a gate writes, such as a build log a linter reads, could not be
    /// written.
    #[error("{path} could not be written: {source}")]
    Write {
        path: String,
        #[source]
        source: io::Error,
    },
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    match run(&arguments, &mut System) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("error: {failure}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the command `arguments` name.
///
/// # Errors
///
/// [`Failure::Usage`] when `arguments` name no command, and the failure of
/// the command otherwise.
fn run(arguments: &[String], runner: &mut impl Runner) -> Result<(), Failure> {
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["coverage"] => coverage::run(runner),
        ["gates"] => gates::run(runner),
        ["help"] => {
            println!("{USAGE}");
            Ok(())
        }
        _ => Err(Failure::Usage),
    }
}

#[cfg(test)]
mod tests;
