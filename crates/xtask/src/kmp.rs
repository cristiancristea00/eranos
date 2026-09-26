//! The gates of the Kotlin Multiplatform package: the shared layer's tests on
//! the JVM against gobley's bindings, the libraries of every target, and the
//! package's build scripts; and the steps that build and run the Kotlin
//! package's ports of the examples on the package's JVM target. The Kotlin
//! package's gates lint, document, and measure the layer's sources and the
//! ports', which this package compiles unchanged.

use crate::Failure;
use crate::gates::Metadata;
use crate::ports::{self, Jvm};
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

/// Runs the Kotlin Multiplatform package's gates, stopping at the first that
/// fails; the Apple targets compile only where the machine builds for Apple
/// platforms.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set,
/// before any step runs, and the failure of the first step that fails.
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    let [sdk, jdk] = VARIABLES.map(|name| runner.variable(name).ok_or(Failure::Variable(name)));
    // Gradle finds the Android SDK through the environment the gates run in.
    sdk?;
    let apple = runner.builds_apple_platforms();
    for step in checks(&jdk?, apple) {
        runner.run(&step)?;
    }
    Ok(())
}

/// The steps that run the layer's tests on the JVM, build the Android
/// libraries and, when `apple`, compile the layer for every Apple target,
/// each Gradle run on the JDK at `jdk` and with deprecations as errors, and
/// check the format of the build scripts. gobley's plugins build the library
/// for each target, adding its Rust target first, and generate the bindings.
#[must_use]
pub(crate) fn checks(jdk: &str, apple: bool) -> Vec<Step> {
    let gradle = |tasks: &[String], name: &str| gradle(jdk, tasks, name);
    let mut libraries = vec![
        ":generated:assembleDebug".to_owned(),
        ":eranos:assembleDebug".to_owned(),
    ];
    if apple {
        libraries.extend(
            APPLE_TARGETS
                .iter()
                .map(|target| format!(":eranos:compileKotlin{target}")),
        );
    }
    vec![
        gradle(
            &[":eranos:jvmTest".to_owned()],
            "the Kotlin Multiplatform tests on the JVM",
        ),
        gradle(&libraries, "the Kotlin Multiplatform libraries"),
        Step::new(
            &["--relative", "**/*.kts", "!**/build/**"],
            "ktlint on the Kotlin Multiplatform build scripts",
            "ktlint",
        )
        .in_directory(PROJECT),
    ]
}

/// The step that runs Gradle's `tasks` in the project, reported as `name`,
/// on the JDK at `jdk` and with deprecations as errors.
fn gradle(jdk: &str, tasks: &[String], name: &str) -> Step {
    let mut arguments = vec!["gradlew", "--warning-mode=fail"];
    arguments.extend(tasks.iter().map(String::as_str));
    Step::new(&arguments, name, "sh")
        .in_directory(PROJECT)
        .with_variable("JAVA_HOME", jdk)
}

/// The step that compiles the Kotlin package's ports of the examples against
/// gobley's bindings and gathers them, with every library they run with,
/// into one directory, Gradle running on the JDK at `jdk`.
#[must_use]
pub(crate) fn ports(jdk: &str) -> Step {
    gradle(
        jdk,
        &[":examples:ports".to_owned()],
        "the Kotlin Multiplatform ports",
    )
}

/// The step that runs the Kotlin Multiplatform port of `example` on the
/// `java` of the JDK at `jdk`, JNA loading eranos-kmp's library from
/// `sequential`'s target directory before the one gobley embeds.
#[must_use]
pub(crate) fn port(example: &str, jdk: &str, sequential: &Metadata) -> Step {
    ports::jvm_port(&JVM, example, jdk, sequential)
}

#[cfg(test)]
mod tests;
