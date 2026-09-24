//! The command line: each command runs what it names, and anything else is a
//! usage error that lists the commands.

use super::*;
use crate::process::Step;

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
    "version": 1
}"#;

/// A runner that records every step instead of running it, answers every
/// output with `metadata`, and fails at the step `failing`.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub(crate) metadata: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) failing: Option<Step>,
}

impl Recorder {
    /// A recorder that answers with [`METADATA`] and fails at no step.
    pub(crate) fn new() -> Self {
        Self {
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
        Ok(self.metadata.clone())
    }
}

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(|&word| word.to_owned()).collect()
}

/// `gates` runs every check, lists the examples, and runs each one; if it
/// ran less, `cargo xtask gates` would pass commits the documented gates
/// refuse.
#[test]
fn gates_runs_every_check_then_every_example() {
    let mut recorder = Recorder::new();
    run(&words(&["gates"]), &mut recorder).unwrap();

    let mut expected = gates::checks();
    expected.push(gates::metadata());
    expected.extend(gates::examples(METADATA).unwrap());
    assert_eq!(recorder.steps, expected);
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
