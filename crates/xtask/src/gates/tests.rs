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

/// The checks, then async-lock's features, are the gates AGENTS.md lists, in
/// its order, and the last documented gate is the examples; a check the
/// code dropped, added, or reordered would pass a commit the documented
/// gates refuse.
#[test]
fn the_checks_are_the_gates_agents_md_lists_in_its_order() {
    let documented = documented_gates();
    let mut checks: Vec<String> = checks().iter().map(Step::command_line).collect();
    checks.push(lock_features().command_line());
    assert_eq!(documented[..checks.len()], checks);
    assert_eq!(
        documented[checks.len()..],
        ["cargo run --quiet --package <package> --all-features --example <name>"]
    );
}

/// A package that turned on async-lock's `std` would bring the clock into
/// the library unnoticed: the gates stop at the tree that shows it, before
/// any example runs, and name every feature once.
#[test]
fn a_feature_of_async_lock_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.tree = [
        "async-lock v3.4.2",
        "async-lock feature \"default\"",
        "async-lock feature \"std\"",
        "eranos-group v0.4.0 (/eranos/crates/eranos-group)",
        "async-lock feature \"std\" (*)",
    ]
    .join("\n");
    let failure = run(&mut recorder).unwrap_err();
    assert!(
        matches!(&failure, Failure::LockFeatures(features) if features == "\"default\", \"std\""),
        "{failure:?}"
    );
    assert_eq!(recorder.steps.last(), Some(&lock_features()));
    assert_eq!(
        failure.to_string(),
        "async-lock is built with \"default\", \"std\", which the library keeps off"
    );
}

/// When the tree cannot be printed, the gates stop there, so a broken
/// check never passes for a clean one.
#[test]
fn a_failing_feature_tree_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(lock_features());
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), Some(&lock_features()));
}

/// A tree without a feature line passes, whatever else it shows.
#[test]
fn a_lock_without_features_passes() {
    let tree = "async-lock v3.4.2\neranos-group v0.4.0 (/eranos/crates/eranos-group)\n";
    assert!(check_lock_features(tree).is_ok());
    assert!(check_lock_features("").is_ok());
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
