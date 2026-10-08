//! The Kotlin Multiplatform package's gates: the shared layer's tests on the
//! JVM, the libraries of every target, and the build scripts, in that order,
//! stopping at the first that fails, and none of them without the Android
//! SDK or the JDK.

use super::*;
use crate::plan;
use crate::tests::Recorder;

/// The variables removed from the recorder's, and the one the failure
/// names.
struct Removed {
    /// The variables removed.
    missing: &'static [&'static str],
    /// The variable the failure names.
    reported: &'static str,
}

/// Runs the package's gates with `runner`, as the gates run them.
fn run(runner: &mut Recorder) -> Result<(), Failure> {
    let actions = actions(runner)?;
    plan::execute(runner, actions, &|_| true)
}

/// The package would pass without its Apple libraries, or run Gradle on
/// another JDK than the Kotlin package's, if the steps asked for other
/// tasks or another JDK.
#[test]
fn the_package_is_tested_built_for_every_target_and_linted_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder).unwrap();
    assert_eq!(recorder.steps, plan::steps(&checks("/JAVA_HOME", true)));
    assert_eq!(recorder.cleared, Vec::<std::path::PathBuf>::new());
    let gradle = "cd bindings/kmp && JAVA_HOME='/JAVA_HOME' sh gradlew --warning-mode=fail";
    let apple = APPLE_TARGETS
        .map(|target| format!(":eranos:compileKotlin{target}"))
        .join(" ");
    let lines: Vec<String> = recorder.steps.iter().map(Step::command_line).collect();
    assert_eq!(
        lines,
        [
            format!("{gradle} :eranos:jvmTest"),
            format!(
                "{gradle} :generated:bundleAndroidMainAar :eranos:bundleAndroidMainAar {apple}"
            ),
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
    assert_eq!(recorder.steps, plan::steps(&checks("/JAVA_HOME", false)));
    assert_eq!(
        recorder.steps[1].command_line(),
        "cd bindings/kmp && JAVA_HOME='/JAVA_HOME' sh gradlew --warning-mode=fail :generated:bundleAndroidMainAar :eranos:bundleAndroidMainAar"
    );
}

/// Without the Android SDK or the JDK the gates would fail deep in Gradle's
/// start; they fail first instead, naming the variable, the SDK's before
/// the JDK's.
#[test]
fn a_missing_variable_stops_the_kotlin_multiplatform_gates_before_any_step() {
    for Removed { missing, reported } in [
        Removed {
            missing: &VARIABLES,
            reported: "ANDROID_HOME",
        },
        Removed {
            missing: &["JAVA_HOME"],
            reported: "JAVA_HOME",
        },
    ] {
        let mut recorder = Recorder::new();
        recorder.variables.retain(|name| !missing.contains(name));
        let failure = run(&mut recorder).unwrap_err();
        assert!(
            matches!(failure, Failure::Variable(name) if name == reported),
            "{missing:?}"
        );
        assert_eq!(recorder.steps, Vec::<Step>::new());
    }
}

/// A failing step stops the gates there, so a later step's success cannot
/// hide it.
#[test]
fn a_failing_step_stops_the_kotlin_multiplatform_gates() {
    for failing in plan::steps(&checks("/JAVA_HOME", true)) {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
