//! The Kotlin package's gates: the library of every Android ABI, the
//! generated Kotlin in the package, and the package's tests, linters, API
//! check, documentation, and Android libraries, in that order, stopping at
//! the first that fails, and none of them without the Android SDK and the
//! JDK.

use super::*;
use crate::plan;
use crate::tests::{FileOperation, METADATA, Recorder};

/// Runs the package's gates with `runner`, as the gates run them.
fn run(runner: &mut Recorder, metadata: &Metadata) -> Result<(), Failure> {
    let actions = actions(runner, metadata)?;
    plan::execute(runner, actions, &|_| true)
}

fn metadata() -> Metadata {
    Metadata::parse(METADATA).unwrap()
}

/// An ABI left out of the Android library, or built for a newer API level
/// than the library's `minSdk`, would ship a package that does not load on
/// those devices.
#[test]
fn every_android_abi_builds_its_library_for_the_oldest_supported_api() {
    let steps = libraries();
    assert_eq!(
        steps[0].command_line(),
        "rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android"
    );
    assert_eq!(
        steps[1].command_line(),
        "cargo ndk --platform 26 --target arm64-v8a --target armeabi-v7a --target x86 --target x86_64 \
         --output-dir bindings/kotlin/generated-android/src/main/jniLibs build --package eranos-uniffi --lib"
    );
}

/// The package would build against stale generated Kotlin, or its checks
/// would pass against another library than the one just built, if the steps
/// ran in another order or read another directory.
#[test]
fn the_package_is_tested_linted_and_documented_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder, &metadata()).unwrap();
    let mut expected = libraries();
    expected.extend(sources(&metadata()));
    expected.extend(plan::steps(&checks(&metadata())));
    assert_eq!(recorder.steps, expected);
    assert_eq!(recorder.cleared, [generated_sources()]);
    assert_eq!(
        generated_sources(),
        Path::new(WORKSPACE).join("bindings/kotlin/generated/src")
    );
    let copied: Vec<String> = sources(&metadata())
        .iter()
        .map(Step::command_line)
        .collect();
    assert_eq!(
        copied,
        [
            "mkdir -p bindings/kotlin/generated/src/main/kotlin/eranos/internal",
            "cp /eranos/target/uniffi/kotlin/eranos/internal/eranos.kt bindings/kotlin/generated/src/main/kotlin/eranos/internal",
        ]
    );
    let lines: Vec<String> = plan::steps(&checks(&metadata()))
        .iter()
        .map(Step::command_line)
        .collect();
    let gradle = "cd bindings/kotlin && sh gradlew --warning-mode=fail -Peranos.library=/eranos/target/debug";
    assert_eq!(
        lines,
        [
            format!("{gradle} :eranos:test :eranos:koverVerify"),
            "cd bindings/kotlin && ktlint --relative **/*.kt **/*.kts !generated/** !**/build/**"
                .to_owned(),
            format!("{gradle} :eranos:detekt :examples:detekt"),
            format!("{gradle} :eranos:apiCheck"),
            format!("{gradle} :eranos:dokkaGenerate"),
            format!("{gradle} :generated-android:assembleDebug :eranos-android:assembleDebug"),
        ]
    );
}

/// Without the Android SDK or the JDK the gates would fail deep in a build
/// with a message about another variable, or not at all on a machine that
/// finds a stray tool; they fail first instead, naming the variable.
#[test]
fn a_missing_variable_stops_the_kotlin_gates_before_any_step() {
    for missing in VARIABLES {
        let mut recorder = Recorder::new();
        recorder.variables.retain(|name| *name != missing);
        assert!(matches!(
            run(&mut recorder, &metadata()),
            Err(Failure::Variable(name)) if name == missing
        ));
        assert!(recorder.steps.is_empty());
    }
}

/// A failure at any point stops the gates there: generated sources that
/// could not be cleared, or a failing step.
#[test]
fn a_failure_stops_the_kotlin_gates_where_it_happens() {
    let mut unclear = Recorder::new();
    unclear.failing_files = Some(FileOperation::Clear);
    assert!(matches!(
        run(&mut unclear, &metadata()),
        Err(Failure::Usage)
    ));
    assert_eq!(unclear.steps, libraries());

    for failing in [
        libraries()[0].clone(),
        sources(&metadata())[0].clone(),
        plan::steps(&checks(&metadata()))[0].clone(),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(
            run(&mut recorder, &metadata()),
            Err(Failure::Usage)
        ));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
