//! The gates of the Java package: the generated Java in the package, and the
//! package's format, tests and their coverage, documentation, and JAR; and
//! the steps that build and run the ports of the examples.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::jvm::{self, Jvm};
use crate::plan::{Action, Phase};
use crate::process::{self, Profile, Runner, Step, WORKSPACE};

/// The Java project, from the workspace root.
const PROJECT: &str = "bindings/java";

/// The environment variable the Java gates need: the JDK that Gradle runs
/// on.
pub(crate) const VARIABLES: [&str; 1] = ["JAVA_HOME"];

/// The Java ports: gathered by the project's `ports` task, each a `Main`
/// whose library the JDK loads from its library path.
pub(crate) const JVM: Jvm = Jvm {
    class: "Main",
    language: "Java",
    library_path: "java.library.path",
    ports: "bindings/java/build/ports",
};

/// The Java package's gates, in order: the generated Java in a cleared
/// directory, then the package's checks.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn actions(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    if let Some(name) = VARIABLES
        .into_iter()
        .find(|name| runner.variable(name).is_none())
    {
        return Err(Failure::Variable(name));
    }
    let mut actions = vec![Action::clear(Phase::Prepare, generated_sources())];
    actions.extend(Action::runs(Phase::Prepare, sources(metadata)));
    actions.extend(checks(metadata));
    Ok(actions)
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
    process::generated_copy(
        "Java",
        &generated,
        &directory,
        "the generated source set's directory",
        &["-R"],
    )
}

/// The step that runs Gradle's `tasks` in the project, reported as `name`,
/// with the tests loading the host library from `metadata`'s target
/// directory.
fn gradle(metadata: &Metadata, tasks: &[&str], name: &str) -> Step {
    jvm::gradle_on_library(PROJECT, metadata, tasks, name)
}

/// The actions that check the format of the package and of its build
/// scripts, test it with its coverage, build its documentation, and build
/// its JAR, each with warnings as errors; every compilation of the layer
/// runs the lint and Error Prone.
#[must_use]
pub(crate) fn checks(metadata: &Metadata) -> Vec<Action> {
    let gradle = |tasks: &[&str], name: &str| gradle(metadata, tasks, name);
    vec![
        Action::run(Phase::Lint, gradle(&["formatCheck"], "google-java-format")),
        Action::run(
            Phase::Lint,
            Step::new(
                &["--relative", "*.kts"],
                "ktlint on the build scripts",
                "ktlint",
            )
            .in_directory(PROJECT),
        ),
        Action::run(
            Phase::Test,
            gradle(
                &["test", "jacocoTestCoverageVerification"],
                "the Java tests and their coverage",
            ),
        ),
        Action::run(Phase::Docs, gradle(&["javadoc"], "the Java documentation")),
        Action::run(Phase::Package, gradle(&["jar"], "the Java package")),
    ]
}

/// The step that runs the package's tests again, whatever Gradle holds of
/// an earlier run, against the host library in `metadata`'s target
/// directory: the instrumented one, whose coverage the run writes.
#[must_use]
pub(crate) fn tests(metadata: &Metadata) -> Step {
    gradle(metadata, &["test", "--rerun"], "the Java tests")
}

/// The step that compiles the ports of the examples, with every lint as an
/// error and Error Prone, and gathers them with the module they run on into
/// one directory.
#[must_use]
pub(crate) fn ports(metadata: &Metadata) -> Step {
    gradle(metadata, &["ports"], "the Java ports")
}

/// The step that runs the Java port of `example` on the `java` of the JDK at
/// `jdk`, the JDK loading the host library from `sequential`'s target
/// directory.
#[must_use]
pub(crate) fn port(example: &str, jdk: &str, sequential: &Metadata) -> Step {
    jvm::port(&JVM, example, jdk, sequential)
}

/// The Java package's formatters, applied: google-java-format over the
/// layer, its tests, and the examples, and ktlint over the build scripts.
#[must_use]
pub(crate) fn formats() -> Vec<Action> {
    vec![
        Action::run(
            Phase::Format,
            jvm::gradle(PROJECT, &["format"], "the Java format, applied"),
        ),
        Action::run(
            Phase::Format,
            Step::new(
                &["--format", "--relative", "*.kts"],
                "the format of the build scripts, applied",
                "ktlint",
            )
            .in_directory(PROJECT),
        ),
    ]
}

/// The actions that build the Java package an app consumes into its
/// directory of the packages, cleared first: the generated Java, the JAR,
/// and beside it the host library in release, which the JVM loads from the
/// directory `java.library.path` names.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn release(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    if let Some(name) = VARIABLES
        .into_iter()
        .find(|name| runner.variable(name).is_none())
    {
        return Err(Failure::Variable(name));
    }
    let packaged = crate::gates::packaged(metadata, "java");
    let package = packaged.to_string_lossy().into_owned();
    let mut actions = vec![
        Action::clear(Phase::Package, packaged),
        Action::clear(Phase::Package, generated_sources()),
    ];
    actions.extend(Action::runs(Phase::Package, sources(metadata)));
    actions.extend(Action::runs(
        Phase::Package,
        vec![
            gradle(metadata, &["jar"], "the Java package"),
            crate::gates::release_host_library(),
            Step::new(&["-p", &package], "the Java package's directory", "mkdir"),
            Step::new(
                &[
                    &format!("{PROJECT}/build/libs/eranos.jar"),
                    &crate::gates::host_library_file(metadata, Profile::Release),
                    &package,
                ],
                "the Java package",
                "cp",
            ),
        ],
    ));
    Ok(actions)
}

#[cfg(test)]
mod tests;
