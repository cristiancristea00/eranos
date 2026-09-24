//! Running steps as processes: a step that succeeds, one that fails, one
//! whose program is missing, what a step runs, and the line it prints.

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use super::*;

fn version() -> Step {
    Step::cargo(&["--version"], "cargo's version")
}

/// A step whose program exits with failure and prints nothing, so a passing
/// test run shows no error line.
#[cfg(unix)]
fn failing() -> Step {
    Step::new(&[], "a failing program", "false")
}

/// A step whose program exits with failure and prints nothing, so a passing
/// test run shows no error line.
#[cfg(windows)]
fn failing() -> Step {
    Step::new(&["/c", "exit 1"], "a failing program", "cmd")
}

fn missing_program() -> Step {
    Step::new(&[], "a missing program", "eranos-no-such-program")
}

/// A step that succeeds passes; without it no gate could pass.
#[test]
fn a_succeeding_step_passes() {
    System.run(&version()).unwrap();
}

/// A step that fails is reported under its name with the status it exited
/// with, so the run names the gate that stopped it.
#[test]
fn a_failing_step_fails_under_its_name() {
    let failure = System.run(&failing()).unwrap_err();
    assert!(
        matches!(&failure, Failure::Step { step, .. } if step == "a failing program"),
        "{failure:?}"
    );
    assert!(
        failure
            .to_string()
            .starts_with("a failing program failed (exit")
    );
}

/// A program that cannot be started fails as such, under the step's name,
/// whether its output is shown or captured.
#[test]
fn a_program_that_cannot_start_fails_under_the_step_name() {
    for failure in [
        System.run(&missing_program()).unwrap_err(),
        System.output(&missing_program()).unwrap_err(),
    ] {
        assert!(
            matches!(&failure, Failure::Start { step, .. } if step == "a missing program"),
            "{failure:?}"
        );
        assert!(
            failure
                .to_string()
                .starts_with("a missing program: the program could not be started: "),
            "{failure}"
        );
    }
}

/// A captured step returns its standard output whole.
#[test]
fn output_returns_standard_output() {
    let output = System.output(&version()).unwrap();
    assert!(output.starts_with("cargo "), "{output}");
    assert!(output.ends_with('\n'), "{output}");
}

/// A captured step that fails fails like a shown one.
#[test]
fn output_fails_when_its_step_fails() {
    let failure = System.output(&failing()).unwrap_err();
    assert!(matches!(failure, Failure::Step { .. }), "{failure:?}");
}

/// A step runs its program with its arguments and variables from the
/// workspace root, so `cargo xtask gates` checks the whole workspace from
/// any directory it is started in.
#[test]
fn a_step_runs_its_program_from_the_workspace_root() {
    let step = Step::cargo(&["doc", "--workspace"], "documentation")
        .with_variable("RUSTDOCFLAGS", "-D warnings");
    let command = step.command();
    assert_eq!(command.get_program(), "cargo");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [OsStr::new("doc"), OsStr::new("--workspace")]
    );
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        [(OsStr::new("RUSTDOCFLAGS"), Some(OsStr::new("-D warnings")))]
    );
    let directory = command.get_current_dir().unwrap();
    assert!(
        fs::read_to_string(directory.join("Cargo.toml"))
            .unwrap()
            .contains("[workspace]")
    );
    assert_eq!(directory, Path::new(WORKSPACE));
}

/// The printed line puts the variables first, as a shell takes them, so it
/// can be copied into a terminal to rerun the one gate that failed.
#[test]
fn the_command_line_puts_the_variables_first() {
    let step = Step::cargo(&["doc", "--workspace"], "documentation")
        .with_variable("RUSTDOCFLAGS", "-D warnings");
    assert_eq!(
        step.command_line(),
        "RUSTDOCFLAGS='-D warnings' cargo doc --workspace"
    );
    assert_eq!(version().command_line(), "cargo --version");
}
