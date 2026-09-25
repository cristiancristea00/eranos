//! The Kotlin Multiplatform package's gates: the shared layer's tests on the
//! JVM, the libraries of every target, and the build scripts, in that order,
//! stopping at the first that fails, and none of them without the Android
//! SDK or a JDK 21.

use super::*;
use crate::tests::Recorder;

/// The package would pass without its Apple libraries, or run Gradle on a
/// JDK that Gradle 9.0 cannot run on, if the steps asked for other tasks or
/// another JDK.
#[test]
fn the_package_is_tested_built_for_every_target_and_linted_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder).unwrap();
    assert_eq!(recorder.steps, checks("/JAVA_HOME_21", true));
    assert!(recorder.cleared.is_empty());
    let gradle = "cd bindings/kmp && JAVA_HOME='/JAVA_HOME_21' sh gradlew --warning-mode=fail";
    let apple = APPLE_TARGETS
        .map(|target| format!(":eranos:compileKotlin{target}"))
        .join(" ");
    let lines: Vec<String> = recorder.steps.iter().map(Step::command_line).collect();
    assert_eq!(
        lines,
        [
            format!("{gradle} :eranos:jvmTest"),
            format!("{gradle} :generated:assembleDebug :eranos:assembleDebug {apple}"),
            "cd bindings/kmp && ktlint --relative **/*.kts !**/build/**".to_owned(),
        ]
    );
}

/// Kotlin/Native builds for Apple platforms only on macOS, so elsewhere the
/// libraries step builds the Android libraries alone; asking for the Apple
/// libraries there would build nothing and pass for a check.
#[test]
fn the_apple_libraries_build_only_where_the_machine_builds_for_apple_platforms() {
    let mut recorder = Recorder::new();
    recorder.apple = false;
    run(&mut recorder).unwrap();
    assert_eq!(recorder.steps, checks("/JAVA_HOME_21", false));
    assert_eq!(
        recorder.steps[1].command_line(),
        "cd bindings/kmp && JAVA_HOME='/JAVA_HOME_21' sh gradlew --warning-mode=fail :generated:assembleDebug :eranos:assembleDebug"
    );
}

/// Without the Android SDK or a JDK 21 the gates would fail deep in Gradle's
/// start, or run it on a JDK it cannot run on; they fail first instead,
/// naming the variable, the SDK's before the JDK's.
#[test]
fn a_missing_variable_stops_the_kotlin_multiplatform_gates_before_any_step() {
    for (missing, reported) in [
        (&VARIABLES[..], "ANDROID_HOME"),
        (&["JAVA_HOME_21"][..], "JAVA_HOME_21"),
    ] {
        let mut recorder = Recorder::new();
        recorder.variables.retain(|name| !missing.contains(name));
        let failure = run(&mut recorder).unwrap_err();
        assert!(
            matches!(failure, Failure::Variable(name) if name == reported),
            "{missing:?}"
        );
        assert!(recorder.steps.is_empty());
    }
}

/// A failing step stops the gates there, so a later step's success cannot
/// hide it.
#[test]
fn a_failing_step_stops_the_kotlin_multiplatform_gates() {
    for failing in checks("/JAVA_HOME_21", true) {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
