//! The C and C++ packages' gates: the static library, then for each package
//! the headers Diplomat generates, the tests under the sanitizers with their
//! coverage, the format, the lint, and the documentation, in that order,
//! stopping at the first that fails, and none of them without an LLVM.

use std::fs;

use super::*;
use crate::plan;
use crate::tests::{FileOperation, METADATA, Recorder};

/// Runs the packages' gates with `runner`, as the gates run them.
fn run(runner: &mut Recorder, metadata: &Metadata) -> Result<(), Failure> {
    let actions = actions(runner, metadata)?;
    plan::execute(runner, actions, &|_| true)
}

/// The steps the gates run for `package`, from the library on, with the LLVM
/// the recorder names.
fn package_steps(package: Package, metadata: &Metadata) -> Vec<Step> {
    let mut steps = vec![headers(package)];
    steps.extend(tests(package, "/LLVM_HOME", metadata));
    steps.push(coverage(package, "/LLVM_HOME"));
    steps.extend(plan::steps(&checks(package, "/LLVM_HOME")));
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
            "cd bindings/cpp && /LLVM_HOME/bin/clang++ -std=c++23 -fno-exceptions {flags} tests/tests.cpp /eranos/target/debug/liberanos_capi.a {} \
             -o build/tests",
            SYSTEM_LIBRARIES.join(" ")
        )
    );
    assert_eq!(
        lines[1],
        format!(
            "cd bindings/cpp && LLVM_PROFILE_FILE='build/tests.profraw' {WORKSPACE}/bindings/cpp/build/tests"
        )
    );
    assert_eq!(
        lines[2],
        "cd bindings/cpp && /LLVM_HOME/bin/llvm-profdata merge -sparse build/tests.profraw -o build/tests.profdata"
    );
    assert!(
        tests(C_PACKAGE, "/LLVM_HOME", &metadata)[0]
            .command_line()
            .contains("/LLVM_HOME/bin/clang -std=c23 ")
    );
}

/// A port built without the tests' flags would pass code the layer's rules
/// refuse, or read freed memory unnoticed, and one linked with the gates'
/// library would print identifiers its Rust example does not; each port
/// builds under the sanitizers, its program with the report module, on the
/// library whose identifiers count up, into a directory of its own, and runs
/// with `LeakSanitizer`, which macOS leaves off.
#[test]
fn the_ports_build_as_the_tests_do_on_the_library_whose_identifiers_count_up() {
    let sequential = Metadata::parse(METADATA)
        .unwrap()
        .inner(crate::ports::DIRECTORY);
    let flags = "-Wall -Wextra -Wpedantic -Werror -g -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer \
                 -pthread -Iinclude -isystem generated";
    assert_eq!(
        port_build(C_PACKAGE, "/LLVM_HOME", &sequential, "grand_tour").command_line(),
        format!(
            "cd bindings/c && /LLVM_HOME/bin/clang -std=c23 {flags} examples/report.c examples/grand_tour.c \
             /eranos/target/sequential/debug/liberanos_capi.a {} -o /eranos/target/sequential/c/grand_tour",
            SYSTEM_LIBRARIES.join(" ")
        )
    );
    assert_eq!(
        port(C_PACKAGE, "quickstart", &sequential).command_line(),
        "ASAN_OPTIONS='detect_leaks=1' /eranos/target/sequential/c/quickstart"
    );
    let cpp = port_build(CPP_PACKAGE, "/LLVM_HOME", &sequential, "quickstart").command_line();
    assert!(
        cpp.starts_with(&format!(
            "cd bindings/cpp && /LLVM_HOME/bin/clang++ -std=c++23 -fno-exceptions {flags} examples/report.cpp examples/quickstart.cpp "
        )),
        "{cpp}"
    );
    assert!(
        cpp.ends_with(" -o /eranos/target/sequential/cpp/quickstart"),
        "{cpp}"
    );
    assert_eq!(
        sequential_library(&sequential).command_line(),
        "CARGO_TARGET_DIR='/eranos/target/sequential' cargo build --package eranos-capi --features sequential-ids"
    );
}

/// A lint that read the C++ package with exceptions on would hold its code
/// to what only a build with them needs, such as that no exception escapes a
/// `noexcept` function, and one on another standard would read another
/// language; the linter reads each package in its builds' dialect.
#[test]
fn the_lint_reads_each_package_in_its_builds_dialect() {
    for (package, dialect) in [
        (C_PACKAGE, " -- -std=c23 -Iinclude "),
        (CPP_PACKAGE, " -- -std=c++23 -fno-exceptions -Iinclude "),
    ] {
        let lint = plan::steps(&checks(package, "/LLVM_HOME"))[1].command_line();
        assert!(lint.contains(dialect), "{lint}");
    }
}

/// A port the formatter skipped could drift from the package's style, one
/// the linter skipped could hide what the tests' rules refuse, and one the
/// list named but the directory lost would fail for a file no one wrote;
/// the list is the files of each package's `examples` but the linter's
/// settings, which the formatter reads whole and the linter by its sources.
#[test]
fn the_format_and_the_lint_check_every_port() {
    for package in [C_PACKAGE, CPP_PACKAGE] {
        let mut found: Vec<String> = fs::read_dir(
            Path::new(WORKSPACE)
                .join(package.directory)
                .join("examples"),
        )
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| !name.starts_with('.'))
                .map(|name| format!("examples/{name}"))
                .collect()
        })
        .unwrap_or_default();
        found.sort();
        assert_eq!(found, package.examples, "{}", package.name);
        let words = |step: &Step| -> Vec<String> {
            step.command_line().split(' ').map(str::to_owned).collect()
        };
        let steps = plan::steps(&checks(package, "/LLVM_HOME"));
        let format = words(&steps[0]);
        let lint = words(&steps[1]);
        for example in package.examples {
            assert!(format.iter().any(|word| word == example), "{example}");
            assert_eq!(
                lint.iter().any(|word| word == example),
                example.ends_with(&format!(".{}", package.language)),
                "{example}"
            );
        }
    }
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

/// A failure at any point stops the gates there, so no check reads what a
/// failed step did not make: the library, the headers, the tests' build or
/// run, the coverage summary, or a check that fails, or a directory that
/// could not be cleared.
#[test]
fn a_failure_stops_the_c_and_cpp_gates_where_it_happens() {
    let metadata = Metadata::parse(METADATA).unwrap();
    for failing in [
        library(),
        headers(C_PACKAGE),
        tests(C_PACKAGE, "/LLVM_HOME", &metadata)[0].clone(),
        coverage(C_PACKAGE, "/LLVM_HOME"),
        plan::steps(&checks(C_PACKAGE, "/LLVM_HOME"))[0].clone(),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder, &metadata), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
    let mut blocked = Recorder::new();
    blocked.failing_files = Some(FileOperation::Clear);
    assert!(matches!(run(&mut blocked, &metadata), Err(Failure::Usage)));
    assert_eq!(blocked.steps, [library()]);
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
