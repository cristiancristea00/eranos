//! The Swift package's gates: the library of every Apple target, joined per
//! platform into one `XCFramework`, the generated Swift in the package, and
//! the package's build, tests, linters, and documentation, in that order,
//! stopping at the first that fails.

use super::*;
use crate::tests::{FileOperation, METADATA, Recorder};

fn metadata() -> Metadata {
    Metadata::parse(METADATA).unwrap()
}

/// A platform left out of the framework, or a target built for none, would
/// ship a package that does not link on that platform.
#[test]
fn every_apple_target_builds_one_library_of_the_framework() {
    let targets: Vec<&str> = APPLE_LIBRARIES
        .iter()
        .flat_map(|library| library.targets.iter().copied())
        .collect();
    assert_eq!(targets.len(), 13);
    let steps = libraries(&metadata());
    assert_eq!(
        steps[0].command_line(),
        format!("rustup target add {}", targets.join(" "))
    );
    let build = steps[1].command_line();
    for target in &targets {
        assert!(build.contains(&format!("--target {target}")), "{target}");
    }
    let joined: Vec<String> = steps
        .iter()
        .filter(|step| step.command_line().starts_with("lipo"))
        .map(Step::command_line)
        .collect();
    assert_eq!(
        joined[0],
        "lipo -create /eranos/target/aarch64-apple-ios-sim/debug/liberanos_uniffi.a \
         /eranos/target/x86_64-apple-ios/debug/liberanos_uniffi.a \
         -output /eranos/target/apple/ios-simulator/liberanos_uniffi.a"
    );
    assert_eq!(joined.len(), 3);
    let framework = package(&metadata())[0].command_line();
    assert_eq!(framework.matches(" -library ").count(), 10);
    assert!(framework.contains("-library /eranos/target/aarch64-apple-ios/debug/liberanos_uniffi.a -headers /eranos/target/apple/headers"));
    assert!(framework.ends_with("-output bindings/swift/EranosFFI.xcframework"));
}

/// The package would build against a stale framework or stale generated
/// Swift, or the analyzer would read a log that names no file or none of
/// the examples, if the steps ran in another order; the examples' earlier
/// build is cleared as the package's is, since an incremental build names
/// only what it compiles again.
#[test]
fn the_package_is_built_tested_linted_and_documented_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder, &metadata()).unwrap();
    let mut expected = libraries(&metadata());
    expected.extend(package(&metadata()));
    expected.push(build());
    expected.push(build_examples());
    expected.extend(checks(&metadata()));
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [Path::new(WORKSPACE).join("bindings/swift/EranosFFI.xcframework")]
    );
    assert_eq!(recorder.written, [build_log(&metadata())]);
    let cleared: Vec<String> = package(&metadata())[3..]
        .iter()
        .map(Step::command_line)
        .collect();
    assert_eq!(
        cleared,
        [
            "swift package --package-path bindings/swift clean",
            "swift package --package-path bindings/swift/examples clean",
        ]
    );
    assert_eq!(
        build_examples().command_line(),
        "swift build --package-path bindings/swift/examples --verbose"
    );
    let lines: Vec<String> = checks(&metadata()).iter().map(Step::command_line).collect();
    assert_eq!(
        lines,
        [
            "swift test --package-path bindings/swift",
            "cd bindings/swift && swiftlint lint --strict",
            "cd bindings/swift && swiftlint analyze --strict --compiler-log-path /eranos/target/apple/swift-build.log",
            "swift package --package-path bindings/swift generate-documentation --target Eranos --warnings-as-errors",
        ]
    );
}

/// A failure at any point stops the gates there: a framework that could not
/// be cleared, a log that could not be written, or a failing step.
#[test]
fn a_failure_stops_the_swift_gates_where_it_happens() {
    let mut unclear = Recorder::new();
    unclear.failing_files = Some(FileOperation::Clear);
    assert!(matches!(
        run(&mut unclear, &metadata()),
        Err(Failure::Usage)
    ));
    assert_eq!(unclear.steps, libraries(&metadata()));

    let mut unwritten = Recorder::new();
    unwritten.failing_files = Some(FileOperation::Write);
    assert!(matches!(
        run(&mut unwritten, &metadata()),
        Err(Failure::Usage)
    ));
    assert_eq!(unwritten.steps.last(), Some(&build_examples()));

    for failing in [
        libraries(&metadata())[0].clone(),
        package(&metadata())[0].clone(),
        build(),
        build_examples(),
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
