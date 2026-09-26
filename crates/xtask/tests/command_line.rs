//! The xtask binary as `cargo xtask` runs it: what it prints and the status
//! it exits with, for `help` and for a command it does not know.
#![forbid(unsafe_code)]
#![expect(
    clippy::unwrap_used,
    reason = "a failing test helper panics to report the failure, as clippy.toml allows inside test functions"
)]

use std::process::{Command, Output};

fn xtask(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(arguments)
        .output()
        .unwrap()
}

/// `help` prints the commands and exits successfully.
#[test]
fn help_prints_the_commands_and_succeeds() {
    let output = xtask(&["help"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("usage: cargo xtask <command>"), "{text}");
    assert!(text.contains("\n  gates "), "{text}");
}

/// A command xtask does not know exits with failure and prints the usage on
/// standard error, so a mistyped gate run never reads as a pass.
#[test]
fn an_unknown_command_fails_and_prints_the_usage() {
    let output = xtask(&["gate"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let text = String::from_utf8(output.stderr).unwrap();
    assert!(
        text.starts_with("error: unknown command\n\nusage: cargo xtask <command>"),
        "{text}"
    );
}
