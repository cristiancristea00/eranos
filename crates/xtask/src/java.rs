//! The gates of the Java package: the generated Java in the package, and the
//! package's format, tests and their coverage, documentation, and JAR.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::process::{Runner, Step, WORKSPACE};

/// The Java project, from the workspace root.
const PROJECT: &str = "bindings/java";

/// The environment variable the Java gates need: the JDK that Gradle runs
/// on.
pub(crate) const VARIABLES: [&str; 1] = ["JAVA_HOME"];

/// Runs the Java package's gates, stopping at the first that fails.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set,
/// before any step runs; the failure of the first step that fails, and of
/// clearing the old generated sources.
pub(crate) fn run(runner: &mut impl Runner, metadata: &Metadata) -> Result<(), Failure> {
    if let Some(name) = VARIABLES
        .into_iter()
        .find(|name| runner.variable(name).is_none())
    {
        return Err(Failure::Variable(name));
    }
    runner.clear(&generated_sources())?;
    for step in sources(metadata) {
        runner.run(&step)?;
    }
    for step in checks(metadata) {
        runner.run(&step)?;
    }
    Ok(())
}

/// Where the package keeps the generated Java, the source set it compiles
/// apart from the layer; the gates clear it first, so a type the library no
/// longer has leaves no file behind.
#[must_use]
pub(crate) fn generated_sources() -> PathBuf {
    Path::new(WORKSPACE)
        .join(PROJECT)
        .join("src")
        .join("generated")
}

/// The steps that copy the generated package, `eranos.internal`, into the
/// package's generated source set. The paths of the steps are written with
/// `/`, which the programs of every host accept, so a step reads alike on
/// every host.
#[must_use]
pub(crate) fn sources(metadata: &Metadata) -> Vec<Step> {
    let directory = format!("{PROJECT}/src/generated/java/eranos");
    let generated = format!("{}/uniffi/java/eranos/internal", metadata.target_directory);
    vec![
        Step::new(
            &["-p", &directory],
            "the generated source set's directory",
            "mkdir",
        ),
        Step::new(&["-R", &generated, &directory], "the generated Java", "cp"),
    ]
}

/// The steps that check the format of the package and of its build scripts,
/// test it with its coverage, build its documentation, and build its JAR,
/// each with warnings as errors; every compilation of the layer runs the
/// lint and Error Prone.
#[must_use]
pub(crate) fn checks(metadata: &Metadata) -> Vec<Step> {
    let library = format!("-Peranos.library={}/debug", metadata.target_directory);
    let gradle = |tasks: &[&str], name: &str| {
        let mut arguments = vec!["gradlew", "--warning-mode=fail", &library];
        arguments.extend(tasks);
        Step::new(&arguments, name, "sh").in_directory(PROJECT)
    };
    vec![
        gradle(&["formatCheck"], "google-java-format"),
        Step::new(
            &["--relative", "*.kts"],
            "ktlint on the build scripts",
            "ktlint",
        )
        .in_directory(PROJECT),
        gradle(
            &["test", "jacocoTestCoverageVerification"],
            "the Java tests and their coverage",
        ),
        gradle(&["javadoc"], "the Java documentation"),
        gradle(&["jar"], "the Java package"),
    ]
}

#[cfg(test)]
mod tests;
