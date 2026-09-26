//! The JVM packages' shared steps. A Gradle run without deprecations as
//! errors would pass a build that the next Gradle refuses, and one without
//! the host library would test a stale one.

use super::*;
use crate::tests::METADATA;

/// Every Gradle run fails on a deprecation, runs in its project, and, for
/// the tests, loads the host library from cargo's target directory.
#[test]
fn every_gradle_run_fails_on_a_deprecation() {
    assert_eq!(
        gradle("bindings/kmp", &[":eranos:jvmTest"], "the tests").command_line(),
        "cd bindings/kmp && sh gradlew --warning-mode=fail :eranos:jvmTest"
    );
    assert_eq!(
        gradle_on_library(
            "bindings/java",
            &Metadata::parse(METADATA).unwrap(),
            &["test", "javadoc"],
            "the tests"
        )
        .command_line(),
        "cd bindings/java && sh gradlew --warning-mode=fail -Peranos.library=/eranos/target/debug test javadoc"
    );
}
