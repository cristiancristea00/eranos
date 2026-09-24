//! The gates: the checks in AGENTS.md's order, the examples cargo lists, and
//! stopping at the first gate that fails.

use super::*;
use crate::tests::{METADATA, Recorder};

/// The commands AGENTS.md lists under its gates, in its order: the items of
/// the numbered list in that section, each a command in backticks.
fn documented_gates() -> Vec<String> {
    let agents = include_str!("../../../../AGENTS.md");
    let (_, after) = agents.split_once("## Gates before committing").unwrap();
    let (section, _) = after.split_once("\n## ").unwrap();
    section
        .lines()
        .filter_map(|line| line.split_once(". `"))
        .filter(|(number, _)| {
            !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
        })
        .filter_map(|(_, rest)| rest.split_once('`'))
        .map(|(command, _)| command.to_owned())
        .collect()
}

/// The checks are the gates AGENTS.md lists, in its order, and the last
/// documented gate is the examples; a check the code dropped, added, or
/// reordered would pass a commit the documented gates refuse.
#[test]
fn the_checks_are_the_gates_agents_md_lists_in_its_order() {
    let documented = documented_gates();
    let checks: Vec<String> = checks().iter().map(Step::command_line).collect();
    assert_eq!(documented[..checks.len()], checks);
    assert_eq!(
        documented[checks.len()..],
        ["cargo run --quiet --package <package> --all-features --example <name>"]
    );
}

/// Every example target of every package runs, in the order cargo lists
/// them, with every feature (the persistence example needs `serde`); other
/// targets do not run as examples.
#[test]
fn every_example_of_every_package_runs_with_every_feature() {
    let run = |example: &str| {
        Step::cargo(
            &[
                "run",
                "--quiet",
                "--package",
                "eranos",
                "--all-features",
                "--example",
                example,
            ],
            &format!("example {example}"),
        )
    };
    assert_eq!(
        examples(METADATA).unwrap(),
        [run("quickstart"), run("persistence")]
    );
}

/// JSON that is not cargo's metadata is refused rather than read as a
/// workspace without examples, which would skip the examples silently.
#[test]
fn json_that_is_not_cargo_metadata_is_refused() {
    for text in ["", "{", "{}", r#"{"packages": [{"name": "eranos"}]}"#] {
        assert!(
            matches!(examples(text), Err(Failure::Metadata(_))),
            "{text}"
        );
    }
}

/// A failing check stops the gates after it, so the failure is the last
/// thing a run prints.
#[test]
fn a_failing_check_stops_the_gates_after_it() {
    let checks = checks();
    let mut recorder = Recorder::new();
    recorder.failing = checks.get(1).cloned();
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps, checks[..2]);
}

/// When the examples cannot be listed, no example runs.
#[test]
fn a_failing_example_list_runs_no_example() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(metadata());
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), Some(&metadata()));

    let mut unreadable = Recorder::new();
    unreadable.metadata = "{}".to_owned();
    assert!(matches!(run(&mut unreadable), Err(Failure::Metadata(_))));
    assert_eq!(unreadable.steps.last(), Some(&metadata()));
}

/// A failing example stops the examples after it.
#[test]
fn a_failing_example_stops_the_examples_after_it() {
    let examples = examples(METADATA).unwrap();
    let mut recorder = Recorder::new();
    recorder.failing = examples.first().cloned();
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), examples.first());
    assert!(!recorder.steps.contains(&examples[1]));
}
