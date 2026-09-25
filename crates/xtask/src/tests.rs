//! The command line: each command runs what it names, and anything else is a
//! usage error that lists the commands.

use std::path::{Path, PathBuf};

use super::*;
use crate::gates::Metadata;
use crate::process::{Step, WORKSPACE};

/// Two packages as `cargo metadata --no-deps --format-version 1` prints
/// them: one with a library, a test, and two examples, one with a binary.
pub(crate) const METADATA: &str = r#"{
    "packages": [
        {
            "name": "eranos",
            "targets": [
                {"kind": ["lib"], "name": "eranos"},
                {"kind": ["example"], "name": "quickstart"},
                {"kind": ["test"], "name": "scenarios"},
                {"kind": ["example"], "name": "persistence"}
            ]
        },
        {"name": "xtask", "targets": [{"kind": ["bin"], "name": "xtask"}]}
    ],
    "target_directory": "/eranos/target",
    "version": 1
}"#;

/// A runner that records every step instead of running it, answers the
/// step that prints async-lock's features with `tree` and every other
/// output with `metadata`, records every directory it clears and every file
/// it writes, builds for Apple platforms when `apple`, and fails at the step
/// `failing` or, when `clearing_fails` or `writing_fails`, at clearing or
/// writing.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub(crate) apple: bool,
    pub(crate) cleared: Vec<PathBuf>,
    pub(crate) metadata: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) tree: String,
    pub(crate) written: Vec<PathBuf>,
    pub(crate) clearing_fails: bool,
    pub(crate) failing: Option<Step>,
    pub(crate) writing_fails: bool,
}

impl Recorder {
    /// A recorder that answers with [`METADATA`] and a tree without
    /// features, builds for Apple platforms, and fails at no step.
    pub(crate) fn new() -> Self {
        Self {
            apple: true,
            metadata: METADATA.to_owned(),
            ..Self::default()
        }
    }

    /// Records `step`, and fails if it is the failing one.
    fn record(&mut self, step: &Step) -> Result<(), Failure> {
        self.steps.push(step.clone());
        if self.failing.as_ref() == Some(step) {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }
}

impl Runner for Recorder {
    fn run(&mut self, step: &Step) -> Result<(), Failure> {
        self.record(step)
    }

    fn output(&mut self, step: &Step) -> Result<String, Failure> {
        self.record(step)?;
        if *step == gates::lock_features() {
            Ok(self.tree.clone())
        } else {
            Ok(self.metadata.clone())
        }
    }

    fn clear(&mut self, directory: &Path) -> Result<(), Failure> {
        self.cleared.push(directory.to_owned());
        if self.clearing_fails {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }

    fn write(&mut self, path: &Path, _contents: &str) -> Result<(), Failure> {
        self.written.push(path.to_owned());
        if self.writing_fails {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }

    fn builds_apple_platforms(&self) -> bool {
        self.apple
    }
}

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(|&word| word.to_owned()).collect()
}

/// `gates` runs every check, reads async-lock's features, lists the
/// examples, runs each one, builds the host library, generates every
/// language's sources into a cleared directory, and, where the machine
/// builds for Apple platforms, the Swift package's gates; if it ran less,
/// `cargo xtask gates` would pass commits the documented gates refuse.
#[test]
fn gates_runs_every_check_every_example_and_every_generator() {
    let mut recorder = Recorder::new();
    run(&words(&["gates"]), &mut recorder).unwrap();

    let metadata = Metadata::parse(METADATA).unwrap();
    let mut expected = gates::checks();
    expected.push(gates::lock_features());
    expected.push(gates::metadata());
    expected.extend(gates::examples(&metadata));
    expected.push(gates::host_library());
    expected.extend(gates::generators(&metadata));
    let cleared = gates::generated(&metadata);
    let mut elsewhere = Recorder::new();
    elsewhere.apple = false;
    run(&words(&["gates"]), &mut elsewhere).unwrap();
    assert_eq!(elsewhere.steps, expected);
    assert_eq!(elsewhere.cleared, std::slice::from_ref(&cleared));

    expected.extend(swift::libraries(&metadata));
    expected.extend(swift::package(&metadata));
    expected.push(swift::build());
    expected.extend(swift::checks(&metadata));
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [
            cleared,
            Path::new(WORKSPACE).join("bindings/swift/EranosFFI.xcframework")
        ]
    );
}

/// `help` succeeds and runs nothing.
#[test]
fn help_runs_nothing_and_succeeds() {
    let mut recorder = Recorder::new();
    run(&words(&["help"]), &mut recorder).unwrap();
    assert!(recorder.steps.is_empty());
}

/// No command, an unknown one, or a known one with more words is a usage
/// error that runs nothing, so a mistyped gate run never passes for one.
#[test]
fn anything_but_one_known_command_is_a_usage_error() {
    for line in [&[][..], &["gate"], &["gates", "--rust"], &["help", "gates"]] {
        let mut recorder = Recorder::new();
        let failure = run(&words(line), &mut recorder).unwrap_err();
        assert!(matches!(failure, Failure::Usage), "{line:?}");
        assert!(recorder.steps.is_empty(), "{line:?}");
    }
}

/// The usage error lists every command, so the fix is on screen.
#[test]
fn a_usage_error_lists_every_command() {
    let message = Failure::Usage.to_string();
    assert!(message.starts_with("unknown command\n\nusage: cargo xtask <command>"));
    for command in ["gates", "help"] {
        assert!(message.contains(&format!("\n  {command} ")), "{command}");
    }
}

/// A metadata failure says where the JSON came from and why it was refused.
#[test]
fn a_metadata_failure_names_cargo_metadata_and_the_reason() {
    let reason = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let message = Failure::Metadata(reason).to_string();
    assert!(
        message.starts_with("cargo metadata printed JSON the gates cannot read: EOF"),
        "{message}"
    );
}
