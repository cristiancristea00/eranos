//! The commands for one language or every one: each builds what the
//! language's package reads, then runs the phase it names and no other, and
//! names what it cannot run before it runs anything of it.

use super::*;
use crate::process::Step;
use crate::tests::{METADATA, Recorder};

/// A command, run with the recorder.
type Command = dyn Fn(&mut Recorder) -> Result<(), Failure>;

/// The metadata the recorder answers with.
fn metadata() -> Metadata {
    Metadata::parse(METADATA).unwrap()
}

/// The steps of the actions of `actions` in `phase`, in order.
fn steps_in(actions: Vec<Action>, phase: Phase) -> Vec<Step> {
    let selected: Vec<Action> = actions
        .into_iter()
        .filter(|action| phase == action.phase)
        .collect();
    plan::steps(&selected)
}

/// The command lines of the steps of `actions`.
fn lines(actions: &[Action]) -> Vec<String> {
    plan::steps(actions)
        .iter()
        .map(Step::command_line)
        .collect()
}

/// A test run that skipped the host library would test the layer against a
/// stale module, and one that ran the linters or built the documentation
/// too would take minutes more: `test python` reads cargo's metadata,
/// builds the host library and generates its sources into a cleared
/// directory, prepares the package, and runs its tests with their coverage,
/// and nothing else.
#[test]
fn a_language_s_tests_run_after_what_they_read_and_nothing_else() {
    let metadata = metadata();
    let mut recorder = Recorder::new();
    test(&mut recorder, &[Language::Python]).unwrap();
    let mut expected = vec![gates::metadata(), gates::host_library()];
    expected.extend(gates::generators(&metadata));
    expected.extend([
        python::module(&metadata),
        python::library(&metadata),
        python::environment(),
    ]);
    expected.extend(steps_in(python::checks(), Phase::Test));
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [gates::generated(&metadata), python::documentation()]
    );
    let mut rust = Recorder::new();
    test(&mut rust, &[Language::Rust]).unwrap();
    let mut tests = vec![gates::metadata()];
    tests.extend(gates::tests());
    assert_eq!(rust.steps, tests);
}

/// A lint that skipped the headers a package's linters read would lint
/// against missing ones, and one that ran the tests would take minutes
/// more: `lint c` builds eranos-capi's library, then for each of the C and
/// C++ packages generates its headers and runs its format check and its
/// linter, and `lint rust` runs the workspace's lints, the target lints, and
/// the check of async-lock's features.
#[test]
fn a_language_s_linters_run_after_what_they_read_and_nothing_else() {
    let mut recorder = Recorder::new();
    lint(&mut recorder, &[Language::Capi]).unwrap();
    let mut expected = vec![gates::metadata(), capi::library()];
    for package in [capi::C_PACKAGE, capi::CPP_PACKAGE] {
        expected.push(capi::headers(package));
        expected.extend(steps_in(capi::checks(package, "/LLVM_HOME"), Phase::Lint));
    }
    assert_eq!(recorder.steps, expected);
    let mut rust = Recorder::new();
    lint(&mut rust, &[Language::Rust]).unwrap();
    let mut lints = vec![gates::metadata()];
    lints.extend(gates::host_lints());
    lints.extend(gates::target_lints());
    lints.push(gates::lock_features());
    assert_eq!(rust.steps, lints);
}

/// A run of every language would test a package against another's stale
/// bindings, or build the host library once per package, if it did
/// otherwise: `all` builds the host library and its sources once, first,
/// then runs each language's part in turn, the Swift package's only where
/// the machine builds for Apple platforms.
#[test]
fn every_language_runs_once_after_one_host_library() {
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        test(&mut recorder, &Language::ALL).unwrap();
        let host = gates::host_library();
        assert_eq!(recorder.steps[1], host, "{apple}");
        assert_eq!(
            1,
            recorder.steps.iter().filter(|step| host == **step).count(),
            "{apple}"
        );
        let swift = swift::build();
        assert_eq!(apple, recorder.steps.contains(&swift), "{apple}");
        assert!(recorder.steps.contains(&web::node()), "{apple}");
    }
}

/// A formatter run that built the packages first would take minutes to
/// rewrite a file, and one that formatted another language would rewrite
/// files the command did not name: `format` runs the language's formatters
/// alone, and with every language each one's, the Swift package's only
/// where the machine builds for Apple platforms.
#[test]
fn the_formatters_run_alone_and_every_language_s_with_all() {
    let mut alone = Recorder::new();
    format(&mut alone, &[Language::Python]).unwrap();
    assert_eq!(alone.steps, plan::steps(&python::formats()));
    let every = [
        (Language::Capi, capi::formats("/LLVM_HOME")),
        (Language::Java, java::formats()),
        (Language::Kmp, kmp::formats()),
        (Language::Kotlin, kotlin::formats()),
        (Language::Python, python::formats()),
        (Language::Rust, gates::formats()),
        (Language::Swift, swift::formats()),
        (Language::Web, web::formats()),
    ];
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        format(&mut recorder, &Language::ALL).unwrap();
        let expected: Vec<Step> = every
            .iter()
            .filter(|(language, _)| apple || Language::Swift != *language)
            .flat_map(|(_, formats)| plan::steps(formats))
            .collect();
        assert_eq!(recorder.steps, expected, "{apple}");
    }
}

/// A formatter that rewrote other files than its check reads would leave
/// the check failing, or change files nobody checks: each applies the
/// formatter the gates check with, to the same files, and every one is in
/// the phase no gate runs.
#[test]
fn every_formatter_applies_what_its_check_reads() {
    assert_eq!(lines(&gates::formats()), ["cargo fmt --all"]);
    assert_eq!(
        lines(&swift::formats()),
        ["cd bindings/swift && swiftlint lint --fix"]
    );
    assert_eq!(
        lines(&kotlin::formats()),
        [
            "cd bindings/kotlin && ktlint --format --relative **/*.kt **/*.kts !generated/** !**/build/**"
        ]
    );
    assert_eq!(
        lines(&java::formats()),
        [
            "cd bindings/java && sh gradlew --warning-mode=fail format",
            "cd bindings/java && ktlint --format --relative *.kts",
        ]
    );
    assert_eq!(
        lines(&kmp::formats()),
        ["cd bindings/kmp && ktlint --format --relative **/*.kts !**/build/**"]
    );
    assert_eq!(
        lines(&web::formats()),
        [
            web::install().command_line(),
            "cd bindings/typescript && npm run format".to_owned()
        ]
    );
    assert_eq!(
        lines(&python::formats()),
        ["cd bindings/python && uv run --locked ruff format"]
    );
    let checked: Vec<String> = [capi::C_PACKAGE, capi::CPP_PACKAGE]
        .map(|package| {
            plan::steps(&capi::checks(package, "/LLVM_HOME"))[0]
                .command_line()
                .replace("--dry-run --Werror", "-i")
        })
        .into();
    assert_eq!(lines(&capi::formats("/LLVM_HOME")), checked);
    for formats in [
        capi::formats("/LLVM_HOME"),
        gates::formats(),
        java::formats(),
        kmp::formats(),
        kotlin::formats(),
        python::formats(),
        swift::formats(),
        web::formats(),
    ] {
        assert!(formats.iter().all(|action| Phase::Format == action.phase));
    }
}

/// A port run on another build than the example's, or before the package it
/// runs over was prepared, would print identifiers or fail where the gates
/// pass: `examples quickstart --language python` runs the Rust example,
/// prepares the Python package as `test` does, builds the library whose
/// identifiers count up and generates its sources, copies the package over
/// it, and runs the port, which needs no JDK and no LLVM.
#[test]
fn an_example_s_port_runs_after_its_report_and_its_package() {
    let metadata = metadata();
    let sequential = metadata.inner(ports::DIRECTORY);
    let mut recorder = Recorder::new();
    recorder.variables.clear();
    examples(&mut recorder, Some("quickstart"), &[Language::Python]).unwrap();
    let quickstart = gates::examples(&metadata)
        .into_iter()
        .find(|example| "quickstart" == example.name)
        .unwrap();
    let mut expected = vec![gates::metadata(), quickstart.step, gates::host_library()];
    expected.extend(gates::generators(&metadata));
    expected.extend([
        python::module(&metadata),
        python::library(&metadata),
        python::environment(),
        ports::library(&sequential),
    ]);
    expected.extend(gates::generators(&sequential));
    expected.extend(python::copy(&sequential));
    expected.push(python::port("quickstart", &sequential));
    assert_eq!(recorder.steps, expected);
    let mut cleared = vec![
        gates::generated(&metadata),
        python::documentation(),
        gates::generated(&sequential),
    ];
    cleared.extend(python::copied_directories(&sequential));
    assert_eq!(recorder.cleared, cleared);
    let mut every = Recorder::new();
    examples(&mut every, None, &[Language::Python]).unwrap();
    for example in ["quickstart", "persistence"] {
        assert!(every.steps.contains(&python::port(example, &sequential)));
    }
}

/// A port that failed and let the command pass would hide a port that no
/// longer runs: the first port that fails stops the command there.
#[test]
fn a_port_that_fails_stops_the_examples_at_it() {
    let sequential = metadata().inner(ports::DIRECTORY);
    let port = python::port("quickstart", &sequential);
    let mut recorder = Recorder::new();
    recorder.failing = Some(port.clone());
    let failure = examples(&mut recorder, None, &[Language::Python]);
    assert!(matches!(failure, Err(Failure::Usage)), "{failure:?}");
    assert_eq!(recorder.steps.last(), Some(&port));
}

/// A mistyped example or language that ran something, or failed without
/// naming the right ones, would leave the fix off screen: each is refused
/// with the names the command takes, an example after cargo's metadata
/// alone, and `all` stands for every language the command takes.
#[test]
fn an_unknown_example_or_language_names_the_known_ones() {
    let mut recorder = Recorder::new();
    let failure = examples(&mut recorder, Some("tour"), &[Language::Python]).unwrap_err();
    assert_eq!(
        failure.to_string(),
        "eranos has no example tour; its examples are quickstart, persistence"
    );
    assert_eq!(recorder.steps, [gates::metadata()]);
    assert_eq!(
        languages("typescript", &Language::ALL)
            .unwrap_err()
            .to_string(),
        "typescript is not one of the command's languages: c, java, kmp, kotlin, python, rust, swift, web, or all"
    );
    assert_eq!(
        languages("rust", &ports::LANGUAGES)
            .unwrap_err()
            .to_string(),
        "rust is not one of the command's languages: c, java, kmp, kotlin, python, swift, web, or all"
    );
    assert_eq!(
        languages("all", &ports::LANGUAGES).unwrap(),
        ports::LANGUAGES
    );
    assert_eq!(languages("kmp", &Language::ALL).unwrap(), [Language::Kmp]);
}

/// A command for the Swift package on a machine that cannot build it would
/// pass having run nothing, and one that read a package's variable late
/// would fail after minutes of steps: the Swift package alone is refused
/// there, before anything runs, and a missing variable stops a command
/// before any of its package's actions, after cargo's metadata alone.
#[test]
fn a_command_that_cannot_run_stops_before_its_package_s_actions() {
    let mut elsewhere = Recorder::new();
    elsewhere.apple = false;
    assert!(matches!(
        test(&mut elsewhere, &[Language::Swift]),
        Err(Failure::Apple)
    ));
    assert!(matches!(
        format(&mut elsewhere, &[Language::Swift]),
        Err(Failure::Apple)
    ));
    assert!(elsewhere.steps.is_empty());
    let mut unset = Recorder::new();
    unset.variables.retain(|&name| "ANDROID_HOME" != name);
    assert!(matches!(
        test(&mut unset, &[Language::Kotlin]),
        Err(Failure::Variable("ANDROID_HOME"))
    ));
    assert_eq!(unset.steps, [gates::metadata()]);
    for (language, missing) in [
        (Language::Capi, capi::VARIABLE),
        (Language::Java, java::VARIABLES[0]),
    ] {
        let mut unformatted = Recorder::new();
        unformatted.variables.clear();
        let failure = format(&mut unformatted, &[language]);
        assert!(
            matches!(failure, Err(Failure::Variable(name)) if missing == name),
            "{failure:?}"
        );
        assert!(unformatted.steps.is_empty());
    }
}

/// A command that went on after a step failed would report a pass, or run a
/// package's checks on what was not built: the metadata that could not be
/// printed or read, a failing test, a failing example, and a failing action
/// that builds a package each stop the command there.
#[test]
fn a_failure_stops_a_command_where_it_happens() {
    let metadata = metadata();
    let tests = steps_in(python::checks(), Phase::Test)[0].clone();
    let quickstart = gates::examples(&metadata)
        .into_iter()
        .find(|example| "quickstart" == example.name)
        .unwrap()
        .step;
    let python = |recorder: &mut Recorder| test(recorder, &[Language::Python]);
    let example =
        |recorder: &mut Recorder| examples(recorder, Some("quickstart"), &[Language::Python]);
    let runs: [(Step, &Command); 4] = [
        (gates::metadata(), &python),
        (tests, &python),
        (quickstart, &example),
        (python::module(&metadata), &example),
    ];
    for (failing, run) in runs {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(
            matches!(run(&mut recorder), Err(Failure::Usage)),
            "{failing:?}"
        );
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
    let mut unreadable = Recorder::new();
    unreadable.metadata = String::new();
    assert!(matches!(
        lint(&mut unreadable, &[Language::Rust]),
        Err(Failure::Metadata(_))
    ));
}

/// A package's command that ran without the variable its gates read would
/// fail after minutes of other steps: each package that reads one stops the
/// command after cargo's metadata alone, the ports of the examples too.
#[test]
fn every_package_s_variable_stops_its_command_before_it_builds() {
    for (language, missing) in [
        (Language::Capi, capi::VARIABLE),
        (Language::Java, java::VARIABLES[0]),
        (Language::Kmp, kmp::VARIABLES[0]),
        (Language::Kotlin, kotlin::VARIABLES[0]),
    ] {
        for command in [test, lint] {
            let mut recorder = Recorder::new();
            recorder.variables.clear();
            let failure = command(&mut recorder, &[language]);
            assert!(
                matches!(failure, Err(Failure::Variable(name)) if missing == name),
                "{language} {failure:?}"
            );
            assert_eq!(recorder.steps, [gates::metadata()], "{language}");
        }
        let mut recorder = Recorder::new();
        recorder.variables.clear();
        let failure = examples(&mut recorder, None, &[language]);
        assert!(
            matches!(failure, Err(Failure::Variable(name)) if missing == name),
            "{language} {failure:?}"
        );
    }
}
