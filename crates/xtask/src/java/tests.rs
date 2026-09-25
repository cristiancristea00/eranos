//! The Java package's gates: the generated Java in the package, and the
//! package's format, tests, documentation, and JAR, in that order, stopping
//! at the first that fails, and none of them without the JDK.

use super::*;
use crate::tests::{FileOperation, METADATA, Recorder};

fn metadata() -> Metadata {
    Metadata::parse(METADATA).unwrap()
}

/// The package would build against stale generated Java, or its checks would
/// pass against another library than the one just built, if the steps ran in
/// another order or read another directory.
#[test]
fn the_package_is_formatted_tested_documented_and_packed_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder, &metadata()).unwrap();
    let mut expected = sources(&metadata());
    expected.extend(checks(&metadata()));
    assert_eq!(recorder.steps, expected);
    assert_eq!(recorder.cleared, [generated_sources()]);
    assert_eq!(
        generated_sources(),
        Path::new(WORKSPACE).join("bindings/java/src/generated")
    );
    let copied: Vec<String> = sources(&metadata())
        .iter()
        .map(Step::command_line)
        .collect();
    assert_eq!(
        copied,
        [
            "mkdir -p bindings/java/src/generated/java/eranos",
            "cp -R /eranos/target/uniffi/java/eranos/internal bindings/java/src/generated/java/eranos",
        ]
    );
    let lines: Vec<String> = checks(&metadata()).iter().map(Step::command_line).collect();
    let gradle =
        "cd bindings/java && sh gradlew --warning-mode=fail -Peranos.library=/eranos/target/debug";
    assert_eq!(
        lines,
        [
            format!("{gradle} formatCheck"),
            "cd bindings/java && ktlint --relative *.kts".to_owned(),
            format!("{gradle} test jacocoTestCoverageVerification"),
            format!("{gradle} javadoc"),
            format!("{gradle} jar"),
        ]
    );
}

/// Without the JDK the gates would fail deep in Gradle's start, or run on a
/// JDK the package does not build with; they fail first instead, naming the
/// variable.
#[test]
fn a_missing_jdk_stops_the_java_gates_before_any_step() {
    let mut recorder = Recorder::new();
    recorder.variables.retain(|name| *name != "JAVA_HOME");
    assert!(matches!(
        run(&mut recorder, &metadata()),
        Err(Failure::Variable("JAVA_HOME"))
    ));
    assert!(recorder.steps.is_empty());
    assert!(recorder.cleared.is_empty());
}

/// A failure at any point stops the gates there: generated sources that
/// could not be cleared, or a failing step.
#[test]
fn a_failure_stops_the_java_gates_where_it_happens() {
    let mut unclear = Recorder::new();
    unclear.failing_files = Some(FileOperation::Clear);
    assert!(matches!(
        run(&mut unclear, &metadata()),
        Err(Failure::Usage)
    ));
    assert!(unclear.steps.is_empty());

    for failing in [
        sources(&metadata())[0].clone(),
        checks(&metadata())[0].clone(),
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
