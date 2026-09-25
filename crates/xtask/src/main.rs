//! The repository's automation, run as `cargo xtask <command>` through the
//! alias in `.cargo/config.toml`.
//!
//! `cargo xtask gates` runs the commit gates AGENTS.md lists, in its order,
//! and stops at the first that fails; `cargo xtask help` lists the commands.

mod gates;
mod kotlin;
mod process;
mod swift;

use std::io;
use std::process::{ExitCode, ExitStatus};

use crate::process::{Runner, System};

/// What `help` prints, and what a command line xtask does not understand
/// prints after its error.
const USAGE: &str = "usage: cargo xtask <command>

commands:
  gates  run every commit gate AGENTS.md lists, stopping at the first failure
  help   print this message";

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
    /// A package turns on features of async-lock, which the library keeps
    /// without any.
    #[error("async-lock is built with {0}, which the library keeps off")]
    LockFeatures(String),
    /// The output of `cargo metadata` is not the JSON the gates read.
    #[error("cargo metadata printed JSON the gates cannot read: {0}")]
    Metadata(#[source] serde_json::Error),
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
