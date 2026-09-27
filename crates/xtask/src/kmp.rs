//! The gates of the Kotlin Multiplatform package: the shared layer's tests on
//! the JVM against gobley's bindings, the libraries of every target, and the
//! package's build scripts; and the steps that build and run the Kotlin
//! package's ports of the examples on the package's JVM target. The Kotlin
//! package's gates lint, document, and measure the layer's sources and the
//! ports', which this package compiles unchanged.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};

use crate::Failure;
use crate::gates::Metadata;
use crate::jvm::{self, Jvm};
use crate::plan::{Action, Phase};
use crate::process::{Runner, Step};

/// The Kotlin Multiplatform project, from the workspace root.
const PROJECT: &str = "bindings/kmp";

/// The Kotlin package's ports, compiled here over gobley's bindings:
/// gathered by the examples' `ports` task, each a `MainKt` whose library JNA
/// loads.
pub(crate) const JVM: Jvm = Jvm {
    class: "MainKt",
    language: "Kotlin Multiplatform",
    library_path: "jna.library.path",
    ports: "bindings/kmp/examples/build/ports",
    strict_native_access: false,
};

/// The environment variable that names the JDK 21 that Gradle 9.0 runs on:
/// the last Gradle on which the project's Android plugin builds without a
/// deprecation warning, and which runs on JDK 24 at the latest.
pub(crate) const JDK: &str = "JAVA_HOME_21";

/// The environment variables the gates need: the Android SDK, which holds
/// the NDK and the platform, and the JDK that [`JDK`] names.
pub(crate) const VARIABLES: [&str; 2] = ["ANDROID_HOME", JDK];

/// The Apple targets of the package, as Kotlin names them: those that
/// Kotlin/Native, gobley, and rustup's standard libraries all have, but for
/// macOS on `x86_64`, which Kotlin deprecates.
pub(crate) const APPLE_TARGETS: [&str; 8] = [
    "IosArm64",
    "IosSimulatorArm64",
    "IosX64",
    "MacosArm64",
    "TvosArm64",
    "TvosSimulatorArm64",
    "WatchosDeviceArm64",
    "WatchosSimulatorArm64",
];

/// The Kotlin Multiplatform package's gates, in order; the Apple targets
/// compile only where the machine builds for Apple platforms.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn actions(runner: &impl Runner) -> Result<Vec<Action>, Failure> {
    let [sdk, jdk] = VARIABLES.map(|name| runner.variable(name).ok_or(Failure::Variable(name)));
    // Gradle finds the Android SDK through the environment the gates run in.
    sdk?;
    Ok(checks(&jdk?, runner.builds_apple_platforms()))
}

/// The actions that run the layer's tests on the JVM, build the Android
/// libraries and, when `apple`, compile the layer for every Apple target,
/// each Gradle run on the JDK at `jdk` and with deprecations as errors, and
/// check the format of the build scripts. gobley's plugins build the library
/// for each target, adding its Rust target first, and generate the bindings.
#[must_use]
pub(crate) fn checks(jdk: &str, apple: bool) -> Vec<Action> {
    let mut compiled = Vec::new();
    if apple {
        compiled.extend(
            APPLE_TARGETS
                .iter()
                .map(|target| format!(":eranos:compileKotlin{target}")),
        );
    }
    let mut libraries = vec![":generated:assembleDebug", ":eranos:assembleDebug"];
    libraries.extend(compiled.iter().map(String::as_str));
    vec![
        Action::run(
            Phase::Test,
            gradle(
                jdk,
                &[":eranos:jvmTest"],
                "the Kotlin Multiplatform tests on the JVM",
            ),
        ),
        Action::run(
            Phase::Package,
            gradle(jdk, &libraries, "the Kotlin Multiplatform libraries"),
        ),
        Action::run(
            Phase::Lint,
            Step::new(
                &["--relative", "**/*.kts", "!**/build/**"],
                "ktlint on the Kotlin Multiplatform build scripts",
                "ktlint",
            )
            .in_directory(PROJECT),
        ),
    ]
}

/// The step that runs Gradle's `tasks` in the project, reported as `name`,
/// on the JDK at `jdk`.
fn gradle(jdk: &str, tasks: &[&str], name: &str) -> Step {
    jvm::gradle(PROJECT, tasks, name).with_variable("JAVA_HOME", jdk)
}

/// The step that compiles the Kotlin package's ports of the examples against
/// gobley's bindings and gathers them, with every library they run with,
/// into one directory, Gradle running on the JDK at `jdk`.
#[must_use]
pub(crate) fn ports(jdk: &str) -> Step {
    gradle(jdk, &[":examples:ports"], "the Kotlin Multiplatform ports")
}

/// The step that runs the Kotlin Multiplatform port of `example` on the
/// `java` of the JDK at `jdk`, JNA loading eranos-kmp's library from
/// `sequential`'s target directory before the one gobley embeds.
#[must_use]
pub(crate) fn port(example: &str, jdk: &str, sequential: &Metadata) -> Step {
    jvm::port(&JVM, example, jdk, sequential)
}

/// The Kotlin Multiplatform package's formatter, applied: ktlint over the
/// build scripts, the package's only sources of its own.
#[must_use]
pub(crate) fn formats() -> Vec<Action> {
    vec![Action::run(
        Phase::Format,
        Step::new(
            &["--format", "--relative", "**/*.kts", "!**/build/**"],
            "the format of the Kotlin Multiplatform build scripts, applied",
            "ktlint",
        )
        .in_directory(PROJECT),
    )]
}

/// The actions that build the Kotlin Multiplatform packages an app consumes
/// into their directory of the packages, cleared first: gobley's release
/// libraries of the Android ABIs in both Android libraries in release, and
/// the JVM's JARs beside eranos-kmp's host library in release, which JNA
/// loads from the directory `jna.library.path` names; gobley's own JAR of
/// the host library is its debug one. An Apple target's layer reaches an app
/// through a publication, which the packages leave to the app's build.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn release(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    let [sdk, jdk] = VARIABLES.map(|name| runner.variable(name).ok_or(Failure::Variable(name)));
    // Gradle finds the Android SDK through the environment the packages build in.
    sdk?;
    let jdk = jdk?;
    let packaged = crate::gates::packaged(metadata, "kmp");
    let package = packaged.to_string_lossy().into_owned();
    Ok(vec![
        Action::clear(Phase::Package, packaged),
        Action::run(
            Phase::Package,
            gradle(
                &jdk,
                &[
                    ":generated:assembleRelease",
                    ":eranos:assembleRelease",
                    ":generated:jvmJar",
                    ":eranos:jvmJar",
                ],
                "the Kotlin Multiplatform packages",
            ),
        ),
        Action::run(
            Phase::Package,
            Step::cargo(
                &["build", "--release", "--package", "eranos-kmp"],
                "the host library of eranos-kmp, in release",
            ),
        ),
        Action::run(
            Phase::Package,
            Step::new(
                &["-p", &package],
                "the Kotlin Multiplatform packages' directory",
                "mkdir",
            ),
        ),
        Action::run(
            Phase::Package,
            Step::new(
                &[
                    &format!("{PROJECT}/generated/build/outputs/aar/generated-release.aar"),
                    &format!("{PROJECT}/eranos/build/outputs/aar/eranos-release.aar"),
                    &format!("{PROJECT}/generated/build/libs/generated-jvm.jar"),
                    &format!("{PROJECT}/eranos/build/libs/eranos-jvm.jar"),
                    &format!(
                        "{}/release/{DLL_PREFIX}eranos_kmp{DLL_SUFFIX}",
                        metadata.target_directory
                    ),
                    &package,
                ],
                "the Kotlin Multiplatform packages",
                "cp",
            ),
        ),
    ])
}

#[cfg(test)]
mod tests;
