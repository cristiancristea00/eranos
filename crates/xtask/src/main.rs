//! The repository's automation, run as `cargo xtask <command>` through the
//! alias in `.cargo/config.toml`.
//!
//! `cargo xtask gates` runs the commit gates AGENTS.md lists, in its order,
//! and stops at the first that fails, and with `--rust` only the Rust part;
//! `cargo xtask test`, `lint`, and `format` run one language's tests, format
//! checks and linters, or formatters, and `cargo xtask examples` the ports of
//! the examples; `cargo xtask package` builds the release packages an app
//! consumes; `cargo xtask doctor` prints the version of every tool the
//! gates run and how to install one that is missing; `cargo xtask clean`
//! removes what the builds leave in the repository, and with `--caches` the
//! caches they fill outside it; `cargo xtask coverage` measures the Rust
//! tests and the binding glue, which the foreign tests run, in one report;
//! `cargo xtask generate` writes the C glue's bridge from the binding core's
//! records and calls; `cargo xtask help` lists the commands.
#![forbid(unsafe_code)]

mod capi;
mod clean;
mod commands;
mod coverage;
mod doctor;
mod gates;
mod generate;
mod java;
mod jvm;
mod kmp;
mod kotlin;
mod language;
mod plan;
mod ports;
mod process;
mod python;
mod swift;
mod web;

use std::io;
use std::process::{ExitCode, ExitStatus};

use crate::clean::{Mode, Scope};
use crate::gates::Part;
use crate::language::Language;
use crate::process::{Runner, System};

/// What `help` prints, and what a command line xtask does not understand
/// prints after its error.
const USAGE: &str = "usage: cargo xtask <command>

commands:
  clean [--caches] [--dry-run]      remove target/ and every path .gitignore ignores, and with --caches the caches the gates
                                    fill outside the repository; with --dry-run, print what would go and remove nothing
  coverage                          measure the Rust tests and the binding glue, which the foreign tests run, in one report
  doctor                            print every tool the gates run with its version, and how to install a missing one
  examples [<example>] [--language <language>]
                                    run the examples of eranos, or one, and their ports in every language, or one,
                                    comparing each port's report with its example's, after building each package
  format <language>                 apply a language's formatters
  gates [--rust]                    run every commit gate AGENTS.md lists, or the Rust part's, stopping at the first failure
  generate                          write the C glue's bridge from the binding core's records and calls
  help                              print this message
  lint <language>                   check a language's format and run its linters, after building what they read
  package <language>                build a language's release packages, which an app consumes, into target/packages
  test <language>                   run a language's tests, after building what they read

languages: c (the C and C++ packages), java, kmp, kotlin, python, rust, swift, web, or all; examples and package take every
one but rust";

/// Why a command of xtask failed.
#[derive(Debug, thiserror::Error)]
enum Failure {
    /// A command named the Swift package alone on a machine that does not
    /// build for Apple platforms.
    #[error("the Swift package builds only on a machine that builds for Apple platforms")]
    Apple,
    /// The binding core's records or calls give no C bridge.
    #[error("the C bridge cannot be written from the binding core: {0}")]
    Bridge(#[source] syn::Error),
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
    /// Tools the gates run are missing, or of another release than the gates
    /// run on.
    #[error(
        "{0} of the tools the gates run are missing or of another release; the lines above say how to install each"
    )]
    Doctor(usize),
    /// A command named an example that eranos does not have.
    #[error("eranos has no example {example}; its examples are {known}")]
    Example { example: String, known: String },
    /// A command named a language that it does not take.
    #[error("{language} is not one of the command's languages: {known}")]
    Language { known: String, language: String },
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
    /// A file or a directory a command reads, such as a source of the
    /// binding core, could not be read.
    #[error("{path} could not be read: {source}")]
    Read {
        path: String,
        #[source]
        source: io::Error,
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
    let every = ports::LANGUAGES;
    match words.as_slice() {
        ["clean"] => clean::run(runner, Scope::Repository, Mode::Remove),
        ["clean", "--caches"] => clean::run(runner, Scope::WithCaches, Mode::Remove),
        ["clean", "--caches", "--dry-run"] | ["clean", "--dry-run", "--caches"] => {
            clean::run(runner, Scope::WithCaches, Mode::DryRun)
        }
        ["clean", "--dry-run"] => clean::run(runner, Scope::Repository, Mode::DryRun),
        ["coverage"] => coverage::run(runner),
        ["doctor"] => doctor::run(runner),
        ["examples"] => commands::examples(runner, None, &every),
        ["examples", "--language", language] => {
            commands::examples(runner, None, &commands::languages(language, &every)?)
        }
        ["examples", example] if !example.starts_with('-') => {
            commands::examples(runner, Some(example), &every)
        }
        ["examples", example, "--language", language] if !example.starts_with('-') => {
            commands::examples(
                runner,
                Some(example),
                &commands::languages(language, &every)?,
            )
        }
        ["format", language] => {
            commands::format(runner, &commands::languages(language, &Language::ALL)?)
        }
        ["gates"] => gates::run(runner, Part::Every),
        ["gates", "--rust"] => gates::run(runner, Part::Rust),
        ["generate"] => generate::run(runner),
        ["help"] => {
            println!("{USAGE}");
            Ok(())
        }
        ["package", language] => commands::package(runner, &commands::languages(language, &every)?),
        ["lint", language] => {
            commands::lint(runner, &commands::languages(language, &Language::ALL)?)
        }
        ["test", language] => {
            commands::test(runner, &commands::languages(language, &Language::ALL)?)
        }
        _ => Err(Failure::Usage),
    }
}

#[cfg(test)]
mod tests;
