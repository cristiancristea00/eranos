//! The C and C++ packages' gates: the static library, then for each package
//! the headers Diplomat generates, the tests under the sanitizers with their
//! coverage, the format, the lint, and the documentation, in that order,
//! stopping at the first that fails, and none of them without an LLVM.

use std::fs;

use super::*;
use crate::tests::{FileOperation, METADATA, Recorder};

/// The steps the gates run for `package`, from the library on, with the LLVM
/// the recorder names.
fn package_steps(package: Package, metadata: &Metadata) -> Vec<Step> {
    let mut steps = vec![headers(package)];
    steps.extend(tests(package, "/LLVM_HOME", metadata));
    steps.push(coverage(package, "/LLVM_HOME"));
    steps.extend(checks(package, "/LLVM_HOME"));
    steps
}

/// A package whose steps ran out of order, or on another LLVM, would lint
/// headers Diplomat had not yet written, or measure a program built without
/// coverage; the gates build the library once, then run the C package's
/// steps and the C++ package's, each in a cleared `generated` and a new
/// `build`.
#[test]
fn each_package_is_generated_tested_measured_and_checked_in_order() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let mut recorder = Recorder::new();
    run(&mut recorder, &metadata).unwrap();
    let mut expected = vec![library()];
    expected.extend(package_steps(C_PACKAGE, &metadata));
    expected.extend(package_steps(CPP_PACKAGE, &metadata));
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [
            directory(C_PACKAGE, "generated"),
            directory(C_PACKAGE, "build"),
            directory(CPP_PACKAGE, "generated"),
            directory(CPP_PACKAGE, "build"),
        ]
    );
    assert_eq!(
        recorder.created,
        [
            directory(C_PACKAGE, "build"),
            directory(CPP_PACKAGE, "build")
        ]
    );
}

/// The tests' build would pass code the layer's rules refuse, or tests that
/// read freed memory, without warnings as errors and the sanitizers, and a
/// finding the sanitizers let the program recover from would end in a
/// success; the commands pin every flag.
#[test]
fn the_tests_build_with_warnings_as_errors_the_sanitizers_and_coverage() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let lines: Vec<String> = tests(CPP_PACKAGE, "/LLVM_HOME", &metadata)
        .iter()
        .map(Step::command_line)
        .collect();
    let flags = "-Wall -Wextra -Wpedantic -Werror -g -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer \
                 -fprofile-instr-generate -fcoverage-mapping -pthread -Iinclude -isystem generated";
    assert_eq!(
        lines[0],
        format!(
            "cd bindings/cpp && /LLVM_HOME/bin/clang++ -std=c++17 {flags} tests/tests.cpp /eranos/target/debug/liberanos_capi.a {} -o build/tests",
            SYSTEM_LIBRARIES.join(" ")
        )
    );
    assert!(lines[1].starts_with("cd bindings/cpp && LLVM_PROFILE_FILE='build/tests.profraw' /"));
    assert!(lines[1].ends_with("/bindings/cpp/build/tests"));
    assert_eq!(
        lines[2],
        "cd bindings/cpp && /LLVM_HOME/bin/llvm-profdata merge -sparse build/tests.profraw -o build/tests.profdata"
    );
    assert!(
        tests(C_PACKAGE, "/LLVM_HOME", &metadata)[0]
            .command_line()
            .contains("/LLVM_HOME/bin/clang -std=c11 ")
    );
}

/// A header the formatter skipped could drift from the package's style, and
/// one the list named but the directory lost would fail the formatter for a
/// file no one wrote; the lists are the files of each package's `include`.
#[test]
fn the_formatted_headers_are_every_hand_written_header() {
    for package in [C_PACKAGE, CPP_PACKAGE] {
        let root = Path::new(WORKSPACE).join(package.directory);
        let mut found = Vec::new();
        let mut pending = vec![root.join("include")];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(&directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    found.push(
                        path.strip_prefix(&root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('\\', "/"),
                    );
                }
            }
        }
        found.sort();
        let mut listed: Vec<String> = package
            .headers
            .iter()
            .map(|&header| header.to_owned())
            .collect();
        listed.sort();
        assert_eq!(found, listed, "{}", package.name);
    }
}

/// Tests that ran less than the whole layer would let a function no test
/// calls pass for a tested one; a summary with a line or a function unrun
/// fails and says what ran of each.
#[test]
fn a_line_or_a_function_left_unrun_stops_the_gates() {
    let partial = COVERED_PARTIAL;
    let failure = check_coverage(C_PACKAGE, partial).unwrap_err();
    assert_eq!(
        failure.to_string(),
        "the C tests run 2 of 3 lines and 2 of 2 functions of the C layer, which they cover whole"
    );
    let mut recorder = Recorder::new();
    recorder.coverage = partial.to_owned();
    let metadata = Metadata::parse(METADATA).unwrap();
    assert!(matches!(
        run(&mut recorder, &metadata),
        Err(Failure::Coverage { package: "C", .. })
    ));
    assert_eq!(
        recorder.steps.last(),
        Some(&coverage(C_PACKAGE, "/LLVM_HOME"))
    );
}

/// A summary with a line of three left unrun.
const COVERED_PARTIAL: &str = r#"{"data": [{"totals": {"functions": {"count": 2, "covered": 2}, "lines": {"count": 3, "covered": 2}}}]}"#;

/// A summary the gates cannot read would otherwise pass for full coverage;
/// it fails as what it is.
#[test]
fn a_summary_that_is_not_llvm_covs_is_refused() {
    for summary in ["", "{}", r#"{"data": []}"#, r#"{"data": [{"totals": {}}]}"#] {
        assert!(
            matches!(
                check_coverage(CPP_PACKAGE, summary),
                Err(Failure::CoverageSummary(_))
            ),
            "{summary}"
        );
    }
    let functions = r#"{"data": [{"totals": {"functions": {"count": 5, "covered": 4}, "lines": {"count": 3, "covered": 3}}}]}"#;
    assert!(matches!(
        check_coverage(CPP_PACKAGE, functions),
        Err(Failure::Coverage { package: "C++", .. })
    ));
}

/// Without an LLVM the gates would fail deep in the first build, or run
/// Xcode's clang, which has no linters; they fail first instead, naming the
/// variable.
#[test]
fn a_missing_llvm_stops_the_gates_before_any_step() {
    let mut recorder = Recorder::new();
    recorder.variables.retain(|&name| VARIABLE != name);
    let metadata = Metadata::parse(METADATA).unwrap();
    assert!(matches!(
        run(&mut recorder, &metadata),
        Err(Failure::Variable("LLVM_HOME"))
    ));
    assert!(recorder.steps.is_empty());
}

/// A build directory that could not be made would leave the compiler
/// without a place for the tests; the gates stop there, before the headers
/// are generated.
#[test]
fn a_build_directory_that_cannot_be_created_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.failing_files = Some(FileOperation::Create);
    let metadata = Metadata::parse(METADATA).unwrap();
    assert!(matches!(run(&mut recorder, &metadata), Err(Failure::Usage)));
    assert_eq!(recorder.steps, [library()]);
}
