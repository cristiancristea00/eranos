//! The coverage: the Rust runs, the instrumented glue, every foreign suite on
//! it, and the report, in that order, stopping at the first that fails, and
//! none of them without the variables the suites need.

use super::*;
use crate::tests::{EXPORTS, FileOperation, METADATA, Recorder};

/// The metadata of cargo-llvm-cov's target directory in [`METADATA`]'s.
fn measured() -> Metadata {
    Metadata::parse(METADATA).unwrap().inner(DIRECTORY)
}

/// Coverage that ran a suite before the glue it loads was instrumented would
/// measure the gates' library instead; one that skipped a suite would report
/// its glue unrun, and one that read an earlier run's files would report
/// what that run ran. It reads the metadata and cargo-llvm-cov's
/// environment, clears what earlier runs left, and then runs the Rust tests,
/// the instrumented build, the generators, every suite in the environment,
/// the Swift one only where the machine builds for Apple platforms, and the
/// report.
#[test]
fn the_rust_tests_run_then_every_foreign_suite_on_the_instrumented_glue_then_the_report() {
    let environment = environment(EXPORTS);
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        run(&mut recorder).unwrap();
        let mut expected = vec![gates::metadata(), show_environment(&measured())];
        expected.extend(steps(&measured(), "/LLVM_HOME", &environment, apple));
        assert_eq!(recorder.steps, expected, "{apple}");
        assert_eq!(recorder.cleared, cleared(&measured(), apple), "{apple}");
        assert_eq!(recorder.created, created(&measured()), "{apple}");
    }
    let every = steps(&measured(), "/LLVM_HOME", &environment, true);
    for step in swift::copy(&measured()).into_iter().chain([
        swift::copy_tests(&measured()),
        kotlin::tests(&measured()),
        java::tests(&measured()),
        python::copy_tests(&measured()),
    ]) {
        assert!(every.contains(&step.with_environment(&environment)));
    }
    assert_eq!(every.first(), rust().first());
    assert!(
        every
            .iter()
            .position(|step| *step == libraries(&measured()).with_environment(&environment))
            < every
                .iter()
                .position(|step| *step == gates::generators(&measured())[0])
    );
}

/// The Rust runs would measure the glue, which has no Rust tests, and report
/// it unrun, or the glue's proc macros, whose runs depend on which crates a
/// build compiles again, and a run that kept an earlier run's coverage would
/// count what that run ran; they drop the earlier coverage and leave every
/// glue crate out, and the report leaves out the macros' files, which the
/// runs build instrumented all the same.
#[test]
fn the_rust_runs_start_clean_and_leave_the_glue_out() {
    let lines: Vec<String> = rust().iter().map(Step::command_line).collect();
    let excluded = "--exclude eranos-capi --exclude eranos-glue-macros --exclude eranos-uniffi \
                    --exclude eranos-uniffi-bindgen --exclude eranos-web";
    assert_eq!(
        lines,
        [
            "cargo llvm-cov clean --workspace".to_owned(),
            format!("cargo llvm-cov --no-report --workspace --all-features {excluded}"),
            format!("cargo llvm-cov --no-report --workspace --no-default-features {excluded}"),
        ]
    );
    assert_eq!(
        show_environment(&measured()).command_line(),
        "CARGO_TARGET_DIR='/eranos/target/llvm-cov-target' cargo llvm-cov show-env --sh"
    );
    assert_eq!(
        report().command_line(),
        "cargo llvm-cov report --ignore-filename-regex eranos-glue-macros/"
    );
}

/// Exports read wrongly would build the glue without instrumentation, or have
/// a suite write its coverage where the report never looks; each `export
/// NAME=value` line becomes a variable, whether its value is plain or
/// quoted, a quote inside a quoted value included, and a line of another
/// shape none.
#[test]
fn every_export_of_cargo_llvm_cov_becomes_a_variable() {
    let exports = "export LLVM_PROFILE_FILE='/target/eranos-%p.profraw'
export RUSTC_WRAPPER=/bin/cargo-llvm-cov
export QUOTED='it'\\''s'
info: cargo-llvm-cov currently setting cfg(coverage)
export NAMELESS
export OPEN='value
";
    assert_eq!(
        environment(exports),
        [
            Variable::new("LLVM_PROFILE_FILE", "/target/eranos-%p.profraw"),
            Variable::new("RUSTC_WRAPPER", "/bin/cargo-llvm-cov"),
            Variable::new("QUOTED", "it's"),
        ]
    );
}

/// Coverage without the Android SDK, a JDK, or an LLVM would fail in its
/// first suite, after minutes of Rust runs; it fails first, naming the
/// variable.
#[test]
fn a_missing_variable_stops_the_coverage_before_any_step() {
    for name in ["ANDROID_HOME", "JAVA_HOME", "LLVM_HOME"] {
        let mut recorder = Recorder::new();
        recorder.variables.retain(|&variable| name != variable);
        assert!(
            matches!(run(&mut recorder), Err(Failure::Variable(missing)) if missing == name),
            "{name}"
        );
        assert!(recorder.steps.is_empty(), "{name}");
    }
}

/// A failure at any point stops the coverage there, so no suite runs on a
/// library that was not built and no report reads a run that failed: the
/// metadata or cargo-llvm-cov's environment that could not be read, a
/// directory that could not be cleared or created, or a failing step.
#[test]
fn a_failure_stops_the_coverage_where_it_happens() {
    for failing in [
        gates::metadata(),
        show_environment(&measured()),
        rust()[1].clone(),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
    let mut unreadable = Recorder::new();
    unreadable.metadata = String::new();
    assert!(matches!(run(&mut unreadable), Err(Failure::Metadata(_))));
    for operation in [FileOperation::Clear, FileOperation::Create] {
        let mut recorder = Recorder::new();
        recorder.failing_files = Some(operation);
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.len(), 2);
    }
}
