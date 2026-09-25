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
        matches!(&failure, Failure::Step { step, .. } if "a failing program" == step),
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
            matches!(&failure, Failure::Start { step, .. } if "a missing program" == step),
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
    let measured = Step::new(&[], "the C tests", "build/tests").with_environment(&[
        Variable::new("LLVM_PROFILE_FILE", "eranos-%p.profraw"),
        Variable::new("RUSTC_WRAPPER", "cargo-llvm-cov"),
    ]);
    assert_eq!(
        measured.command_line(),
        "LLVM_PROFILE_FILE='eranos-%p.profraw' RUSTC_WRAPPER='cargo-llvm-cov' build/tests"
    );
}

/// Clearing removes a directory and what it holds, and a missing directory
/// is already clear; without either, a gate would generate over stale files
/// or fail on its first run.
#[test]
fn clearing_removes_a_directory_and_accepts_a_missing_one() {
    let directory = std::env::temp_dir().join(format!("xtask-clear-{}", std::process::id()));
    fs::create_dir_all(directory.join("nested")).unwrap();
    fs::write(directory.join("nested/stale.txt"), "stale").unwrap();
    System.clear(&directory).unwrap();
    assert!(!directory.exists());
    System.clear(&directory).unwrap();
}

/// A path that cannot be removed as a directory, such as a file, fails as a
/// clearing of that path, so the gates stop instead of generating over it.
#[test]
fn a_path_that_is_not_a_directory_cannot_be_cleared() {
    let file = std::env::temp_dir().join(format!("xtask-clear-file-{}", std::process::id()));
    fs::write(&file, "not a directory").unwrap();
    let failure = System.clear(&file).unwrap_err();
    fs::remove_file(&file).unwrap();
    assert!(
        matches!(&failure, Failure::Clear { directory, .. } if *directory == file.display().to_string()),
        "{failure:?}"
    );
    assert!(
        failure
            .to_string()
            .starts_with(&format!("{} could not be cleared: ", file.display())),
        "{failure}"
    );
}

/// A step that names a directory runs its program there, under the workspace
/// root, and prints the change of directory first, so a linter that reads
/// its configuration from the directory finds it, and the printed line
/// reruns the step as it ran.
#[test]
fn a_step_runs_in_the_directory_it_names() {
    let step = Step::new(&["lint"], "SwiftLint", "swiftlint").in_directory("bindings/swift");
    assert_eq!(
        step.command().get_current_dir(),
        Some(Path::new(WORKSPACE).join("bindings/swift").as_path())
    );
    assert_eq!(step.command_line(), "cd bindings/swift && swiftlint lint");
}

/// A file the gates write, such as the build log the analyzer reads, holds
/// what was written, and a path that cannot be written fails as a writing
/// of that path instead of leaving the old file to be read.
#[test]
fn writing_replaces_a_file_or_fails_under_its_path() {
    let file = std::env::temp_dir().join(format!("xtask-write-{}", std::process::id()));
    System.write(&file, "first").unwrap();
    System.write(&file, "second").unwrap();
    assert_eq!(fs::read_to_string(&file).unwrap(), "second");
    fs::remove_file(&file).unwrap();

    let missing = std::env::temp_dir()
        .join(format!("xtask-missing-{}", std::process::id()))
        .join("log");
    let failure = System.write(&missing, "log").unwrap_err();
    assert!(
        matches!(&failure, Failure::Write { path, .. } if *path == missing.display().to_string()),
        "{failure:?}"
    );
    assert!(
        failure
            .to_string()
            .starts_with(&format!("{} could not be written: ", missing.display())),
        "{failure}"
    );
}

/// Creating makes a directory and every one above it, and leaves one that
/// is there as it is; a path under a file cannot be created and fails as a
/// creation of that path, so the C and C++ gates stop before a compiler
/// writes into a directory that is not there.
#[test]
fn creating_makes_every_missing_directory_or_fails_under_its_path() {
    let root = std::env::temp_dir().join(format!("xtask-create-{}", std::process::id()));
    let nested = root.join("nested").join("deeper");
    System.create(&nested).unwrap();
    fs::write(nested.join("kept.txt"), "kept").unwrap();
    System.create(&nested).unwrap();
    assert_eq!(fs::read_to_string(nested.join("kept.txt")).unwrap(), "kept");

    let blocked = nested.join("kept.txt").join("inside");
    let failure = System.create(&blocked).unwrap_err();
    fs::remove_dir_all(&root).unwrap();
    assert!(
        matches!(&failure, Failure::Create { directory, .. } if *directory == blocked.display().to_string()),
        "{failure:?}"
    );
    assert!(
        failure
            .to_string()
            .starts_with(&format!("{} could not be created: ", blocked.display())),
        "{failure}"
    );
}

/// The Swift package's gates run exactly where Xcode's tools do: on macOS.
#[test]
fn only_macos_builds_for_apple_platforms() {
    assert_eq!(System.builds_apple_platforms(), cfg!(target_os = "macos"));
}

/// A variable reads as set only when it holds a value, so a gate that needs
/// one fails on an empty one as it does on a missing one.
#[test]
fn a_variable_is_set_only_when_it_holds_a_value() {
    assert_eq!(
        System.variable("CARGO_PKG_NAME"),
        std::env::var("CARGO_PKG_NAME").ok()
    );
    assert!(System.variable("CARGO_PKG_NAME").is_some());
    assert_eq!(System.variable("ERANOS_XTASK_UNSET_VARIABLE"), None);
}
