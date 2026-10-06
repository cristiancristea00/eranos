//! The commands for one language or every one: each builds what the
//! language's package reads, then runs the phase it names and no other, and
//! names what it cannot run before it runs anything of it.

use super::*;
use crate::process::Step;
use crate::tests::{METADATA, Recorder};

/// A command, run with the recorder.
type Command = dyn Fn(&mut Recorder) -> Result<(), Failure>;

/// A language and its formatters.
struct Formats {
    /// The formatters' actions.
    formats: Vec<Action>,
    /// The language.
    language: Language,
}

/// A language whose command stops when every variable is unset, and the
/// variable it names.
struct Unset {
    /// The language.
    language: Language,
    /// The variable the failure names.
    missing: &'static str,
}

/// A language whose packages stop when only `set` of the variables is set,
/// and the variable the failure names.
struct PartlySet {
    /// The language.
    language: Language,
    /// The variable the failure names.
    missing: &'static str,
    /// The variables set.
    set: &'static [&'static str],
}

/// A step that fails, and the command that runs it.
struct Failing<'command> {
    /// The command.
    run: &'command Command,
    /// The step.
    step: Step,
}

/// A package's release actions, and command lines among their steps.
struct ReleaseLines {
    /// The command lines.
    found: &'static [&'static str],
    /// The actions.
    release: Vec<Action>,
}

/// A package's release actions, and the name of its directory of the
/// packages.
struct Packaged {
    /// The name of the directory.
    language: &'static str,
    /// The actions.
    release: Vec<Action>,
}

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
        Formats {
            formats: capi::formats("/LLVM_HOME"),
            language: Language::Capi,
        },
        Formats {
            formats: java::formats(),
            language: Language::Java,
        },
        Formats {
            formats: kmp::formats(),
            language: Language::Kmp,
        },
        Formats {
            formats: kotlin::formats(),
            language: Language::Kotlin,
        },
        Formats {
            formats: python::formats(),
            language: Language::Python,
        },
        Formats {
            formats: gates::formats(),
            language: Language::Rust,
        },
        Formats {
            formats: swift::formats(),
            language: Language::Swift,
        },
        Formats {
            formats: web::formats(),
            language: Language::Web,
        },
    ];
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        format(&mut recorder, &Language::ALL).unwrap();
        let expected: Vec<Step> = every
            .iter()
            .filter(|row| apple || Language::Swift != row.language)
            .flat_map(|row| plan::steps(&row.formats))
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
/// there by every command, before anything runs, and a missing variable
/// stops a command before any of its package's actions, after cargo's
/// metadata alone.
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
    assert!(matches!(
        package(&mut elsewhere, &[Language::Swift]),
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
    for Unset { language, missing } in [
        Unset {
            language: Language::Capi,
            missing: capi::VARIABLE,
        },
        Unset {
            language: Language::Java,
            missing: java::VARIABLES[0],
        },
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
/// printed or read, a failing test, a failing example, a failing action
/// that builds a package, and a failing release build each stop the command
/// there.
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
    let packaged = |recorder: &mut Recorder| package(recorder, &[Language::Python]);
    let wheel = plan::steps(&python::release(&metadata))[0].clone();
    let runs = [
        Failing {
            run: &python,
            step: gates::metadata(),
        },
        Failing {
            run: &python,
            step: tests,
        },
        Failing {
            run: &example,
            step: quickstart,
        },
        Failing {
            run: &example,
            step: python::module(&metadata),
        },
        Failing {
            run: &packaged,
            step: gates::metadata(),
        },
        Failing {
            run: &packaged,
            step: wheel,
        },
    ];
    for Failing { run, step: failing } in runs {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(
            matches!(run(&mut recorder), Err(Failure::Usage)),
            "{failing:?}"
        );
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
    for command in [lint, package] {
        let mut unreadable = Recorder::new();
        unreadable.metadata = String::new();
        assert!(matches!(
            command(&mut unreadable, &[Language::Rust]),
            Err(Failure::Metadata(_))
        ));
    }
}

/// A package's command that ran without the variable its gates read would
/// fail after minutes of other steps: each package that reads one stops the
/// command after cargo's metadata alone, the ports of the examples too.
#[test]
fn every_package_s_variable_stops_its_command_before_it_builds() {
    for Unset { language, missing } in [
        Unset {
            language: Language::Capi,
            missing: capi::VARIABLE,
        },
        Unset {
            language: Language::Java,
            missing: java::VARIABLES[0],
        },
        Unset {
            language: Language::Kmp,
            missing: kmp::VARIABLES[0],
        },
        Unset {
            language: Language::Kotlin,
            missing: kotlin::VARIABLES[0],
        },
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

/// A package built before what it reads, or in another profile, would ship
/// an app a stale or an unoptimised library: `package python` builds the
/// host library and its sources, then the wheel in release into its
/// directory of the packages, cleared first; the workspace's Rust has no
/// package, and with every language each one's release runs, in the order
/// of the languages, the Swift package's only where the machine builds for
/// Apple platforms.
#[test]
fn a_language_s_packages_build_in_release_after_what_they_read() {
    let metadata = metadata();
    let runner = Recorder::new();
    let mut recorder = Recorder::new();
    package(&mut recorder, &[Language::Python]).unwrap();
    let mut uniffi = vec![gates::metadata(), gates::host_library()];
    uniffi.extend(gates::generators(&metadata));
    let mut expected = uniffi.clone();
    expected.extend(plan::steps(&python::release(&metadata)));
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [
            gates::generated(&metadata),
            gates::packaged(&metadata, "python")
        ]
    );
    let mut rust = Recorder::new();
    package(&mut rust, &[Language::Rust]).unwrap();
    assert_eq!(rust.steps, [gates::metadata()]);
    for apple in [true, false] {
        let mut every = Recorder::new();
        every.apple = apple;
        package(&mut every, &ports::LANGUAGES).unwrap();
        let mut releases = uniffi.clone();
        for release in [
            capi::release(&metadata),
            java::release(&runner, &metadata).unwrap(),
            kmp::release(&runner, &metadata).unwrap(),
            kotlin::release(&runner, &metadata).unwrap(),
            python::release(&metadata),
        ] {
            releases.extend(plan::steps(&release));
        }
        if apple {
            releases.extend(plan::steps(&swift::release(&metadata)));
        }
        releases.extend(plan::steps(&web::release(&metadata)));
        assert_eq!(every.steps, releases, "{apple}");
    }
}

/// A package that built another crate, in another profile, or into another
/// directory would leave an app nothing to take, or the gates' debug build:
/// each builds its library in release and puts what an app takes into its
/// directory of the packages.
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "a row for each package's release lines and one for each package's directory, which read best as two tables"
)]
fn every_package_is_built_in_release_where_an_app_takes_it() {
    let metadata = metadata();
    let runner = Recorder::new();
    assert_eq!(
        lines(&python::release(&metadata)),
        [
            "cd bindings/python && uv run --locked maturin build --release --out /eranos/target/packages/python"
        ]
    );
    assert_eq!(
        lines(&web::release(&metadata)),
        [
            "rustup target add wasm32-unknown-unknown",
            "cargo build --release --package eranos-web --target wasm32-unknown-unknown --lib",
            "cd bindings/typescript && npm ci",
            "cd bindings/typescript && uniffi-bindgen-react-native build wasm2 --config ubrn.config.yaml --no-wasm-build --release",
            "cd bindings/typescript && npm run build",
            "mkdir -p /eranos/target/packages/web",
            "cd bindings/typescript && npm pack --pack-destination /eranos/target/packages/web",
        ]
    );
    let expected = [
        ReleaseLines {
            found: &[
                "cargo build --release --package eranos-capi",
                "cp -R bindings/c/include bindings/c/generated /eranos/target/packages/c/c",
                "cp -R bindings/cpp/include bindings/cpp/generated /eranos/target/packages/c/cpp",
            ],
            release: capi::release(&metadata),
        },
        ReleaseLines {
            found: &[
                "cd bindings/java && sh gradlew --warning-mode=fail -Peranos.library=/eranos/target/debug jar",
                "cargo build --release --package eranos-uniffi",
            ],
            release: java::release(&runner, &metadata).unwrap(),
        },
        ReleaseLines {
            found: &[
                "cd bindings/kmp && JAVA_HOME='/JAVA_HOME_21' sh gradlew --warning-mode=fail :generated:assembleRelease :eranos:assembleRelease :generated:jvmJar :eranos:jvmJar",
                "cargo build --release --package eranos-kmp",
            ],
            release: kmp::release(&runner, &metadata).unwrap(),
        },
        ReleaseLines {
            found: &[
                "cargo ndk --platform 26 --target arm64-v8a --target armeabi-v7a --target x86 --target x86_64 --output-dir bindings/kotlin/generated-android/src/main/jniLibs build --package eranos-uniffi --lib --release",
                "cd bindings/kotlin && sh gradlew --warning-mode=fail -Peranos.library=/eranos/target/debug :generated-android:assembleRelease :eranos-android:assembleRelease :generated:jar :eranos:jar",
                "cargo build --release --package eranos-uniffi",
            ],
            release: kotlin::release(&runner, &metadata).unwrap(),
        },
        ReleaseLines {
            found: &[
                "cp -R bindings/swift/Package.swift bindings/swift/Package.resolved bindings/swift/Sources bindings/swift/Tests /eranos/target/packages/swift",
                "cp /eranos/target/uniffi/swift/EranosGenerated.swift /eranos/target/packages/swift/Sources/EranosGenerated",
            ],
            release: swift::release(&metadata),
        },
    ];
    for ReleaseLines { found, release } in expected {
        let written = lines(&release);
        for line in found {
            assert!(written.iter().any(|written| line == written), "{line}");
        }
    }
    let swift = lines(&swift::release(&metadata));
    assert!(swift[1].starts_with("cargo build --package eranos-uniffi --lib --release --target "));
    assert!(swift.iter().any(|line| line.starts_with(
        "xcodebuild -create-xcframework -library /eranos/target/aarch64-apple-ios/release/"
    )
        && line.ends_with(" -output /eranos/target/packages/swift/EranosFFI.xcframework")));
    for Packaged { language, release } in [
        Packaged {
            language: "c",
            release: capi::release(&metadata),
        },
        Packaged {
            language: "java",
            release: java::release(&runner, &metadata).unwrap(),
        },
        Packaged {
            language: "kmp",
            release: kmp::release(&runner, &metadata).unwrap(),
        },
        Packaged {
            language: "kotlin",
            release: kotlin::release(&runner, &metadata).unwrap(),
        },
        Packaged {
            language: "python",
            release: python::release(&metadata),
        },
        Packaged {
            language: "swift",
            release: swift::release(&metadata),
        },
        Packaged {
            language: "web",
            release: web::release(&metadata),
        },
    ] {
        assert!(
            matches!(&release[0].work, plan::Work::Clear(directory) if gates::packaged(&metadata, language) == *directory),
            "{language}"
        );
        assert!(
            release.iter().all(|action| Phase::Package == action.phase),
            "{language}"
        );
    }
}

/// A package of a JVM language built without its JDK or the Android SDK
/// would fail after the other packages' builds: each stops the command
/// after cargo's metadata alone, the Kotlin Multiplatform one on either of
/// its variables.
#[test]
fn every_jvm_package_s_variable_stops_the_packages_before_they_build() {
    for PartlySet {
        language,
        missing,
        set,
    } in [
        PartlySet {
            language: Language::Java,
            missing: java::VARIABLES[0],
            set: &[],
        },
        PartlySet {
            language: Language::Kotlin,
            missing: kotlin::VARIABLES[0],
            set: &[],
        },
        PartlySet {
            language: Language::Kmp,
            missing: kmp::VARIABLES[0],
            set: &[],
        },
        PartlySet {
            language: Language::Kmp,
            missing: kmp::VARIABLES[1],
            set: &[kmp::VARIABLES[0]],
        },
    ] {
        let mut recorder = Recorder::new();
        recorder.variables = set.to_vec();
        let failure = package(&mut recorder, &[language]);
        assert!(
            matches!(failure, Err(Failure::Variable(name)) if missing == name),
            "{language} {failure:?}"
        );
        assert_eq!(recorder.steps, [gates::metadata()], "{language}");
    }
}
