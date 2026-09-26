//! The actions a package's gates are made of, run in order: a read's
//! continuation that ran late, a phase run that a command did not name, or
//! an action that ran after one failed would show a command a state no
//! gates run ever had.

use std::path::{Path, PathBuf};

use super::*;
use crate::tests::{FileOperation, Recorder};

/// A step named `name`, which the recorder answers with its metadata.
fn step(name: &str) -> Step {
    Step::new(&[], name, name)
}

/// Every kind of action runs on the runner in order, and a read's
/// continuation, given the step's output, runs its actions before the ones
/// after the read.
#[test]
fn actions_run_in_order_and_a_read_s_continuation_runs_next() {
    let mut recorder = Recorder::new();
    recorder.metadata = "the output".to_owned();
    let actions = vec![
        Action::clear(Phase::Prepare, PathBuf::from("/cleared")),
        Action::create(Phase::Prepare, PathBuf::from("/created")),
        Action::read(Phase::Test, step("first"), |output| {
            assert_eq!(output, "the output");
            Ok(vec![
                Action::run(Phase::Test, step("second")),
                Action::write(Phase::Docs, PathBuf::from("/written"), output),
            ])
        }),
        Action::run(Phase::Lint, step("third")),
    ];
    execute(&mut recorder, actions, |_| true).unwrap();
    assert_eq!(
        recorder.steps,
        [step("first"), step("second"), step("third")]
    );
    assert_eq!(recorder.cleared, [Path::new("/cleared")]);
    assert_eq!(recorder.created, [Path::new("/created")]);
    assert_eq!(recorder.written, [Path::new("/written")]);
}

/// A command runs the phases it names alone, a read's continuation
/// included; a read whose phase it does not name runs no continuation.
#[test]
fn only_the_selected_phases_run() {
    let mut recorder = Recorder::new();
    let actions = vec![
        Action::run(Phase::Prepare, step("prepare")),
        Action::read(Phase::Test, step("read"), |_| {
            Ok(vec![
                Action::run(Phase::Package, step("package")),
                Action::run(Phase::Test, step("test")),
            ])
        }),
        Action::read(Phase::Docs, step("docs"), unselected),
    ];
    execute(&mut recorder, actions, |phase| {
        matches!(phase, Phase::Prepare | Phase::Test)
    })
    .unwrap();
    assert_eq!(
        recorder.steps,
        [step("prepare"), step("read"), step("test")]
    );
    assert!(unselected(String::new()).is_err());
}

/// The continuation of a read in a phase that `only_the_selected_phases_run`
/// does not select: it refuses what it reads, so a run that called it would
/// fail.
fn unselected(_output: String) -> Result<Vec<Action>, Failure> {
    Err(Failure::Usage)
}

/// A failing action, or a continuation that refuses what it read, stops
/// the actions after it, so a failure is the last thing a run prints.
#[test]
fn a_failure_stops_the_actions_after_it() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(step("failing"));
    let actions = vec![
        Action::run(Phase::Prepare, step("failing")),
        Action::run(Phase::Prepare, step("after")),
    ];
    assert!(matches!(
        execute(&mut recorder, actions, |_| true),
        Err(Failure::Usage)
    ));
    assert_eq!(recorder.steps, [step("failing")]);

    let mut refused = Recorder::new();
    let refusing = vec![
        Action::read(Phase::Test, step("read"), |_| Err(Failure::Usage)),
        Action::run(Phase::Test, step("after")),
    ];
    assert!(matches!(
        execute(&mut refused, refusing, |_| true),
        Err(Failure::Usage)
    ));
    assert_eq!(refused.steps, [step("read")]);

    for operation in [
        FileOperation::Clear,
        FileOperation::Create,
        FileOperation::Write,
    ] {
        let mut blocked = Recorder::new();
        blocked.failing_files = Some(operation);
        let operations = vec![
            Action::clear(Phase::Prepare, PathBuf::from("/cleared")),
            Action::create(Phase::Prepare, PathBuf::from("/created")),
            Action::write(Phase::Prepare, PathBuf::from("/written"), String::new()),
            Action::run(Phase::Prepare, step("after")),
        ];
        assert!(
            execute(&mut blocked, operations, |_| true).is_err(),
            "{operation:?}"
        );
        assert!(blocked.steps.is_empty(), "{operation:?}");
    }
}

/// A test that compares a package's description with its steps reads the
/// steps it runs and reads, and none of its file operations.
#[test]
fn a_description_s_steps_are_its_runs_and_reads() {
    let actions = vec![
        Action::clear(Phase::Prepare, PathBuf::from("/cleared")),
        Action::run(Phase::Prepare, step("run")),
        Action::read(Phase::Test, step("read"), |_| Ok(Vec::new())),
        Action::create(Phase::Prepare, PathBuf::from("/created")),
        Action::write(Phase::Prepare, PathBuf::from("/written"), String::new()),
    ];
    assert_eq!(steps(&actions), [step("run"), step("read")]);
}
