//! The gates of the Kotlin package: the library of every Android ABI, the
//! generated Kotlin in the package, and the package's tests and their
//! coverage, linters, API dump, documentation, and Android libraries; and
//! the steps that build and run the ports of the examples on the JVM.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::ports::{self, Jvm};
use crate::process::{Runner, Step, WORKSPACE};

/// The Kotlin project, from the workspace root.
const PROJECT: &str = "bindings/kotlin";

/// The Kotlin ports: gathered by the examples' `ports` task, each a `MainKt`
/// whose library JNA loads.
pub(crate) const JVM: Jvm = Jvm {
    class: "MainKt",
    language: "Kotlin",
    library_path: "jna.library.path",
    ports: "bindings/kotlin/examples/build/ports",
};

/// Every Android ABI the package ships, as the NDK names them.
pub(crate) const ANDROID_ABIS: [&str; 4] = ["arm64-v8a", "armeabi-v7a", "x86", "x86_64"];

/// The Rust targets of the Android ABIs, in the order of [`ANDROID_ABIS`].
pub(crate) const ANDROID_TARGETS: [&str; 4] = [
    "aarch64-linux-android",
    "armv7-linux-androideabi",
    "i686-linux-android",
    "x86_64-linux-android",
];

/// The oldest Android API level the package runs on: the libraries'
/// `minSdk`, which the NDK's linker is named for.
const API_LEVEL: &str = "26";

/// The environment variable that names the JDK Gradle and the ports run on.
pub(crate) const JDK: &str = "JAVA_HOME";

/// The environment variables the Kotlin gates need: the Android SDK, which
/// holds the NDK and the platform, and the JDK that Gradle runs on.
pub(crate) const VARIABLES: [&str; 2] = ["ANDROID_HOME", JDK];

/// Runs the Kotlin package's gates, stopping at the first that fails.
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
    for step in libraries() {
        runner.run(&step)?;
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

/// The steps that add the Android targets and build the library of every
/// ABI into the generated Android library's `jniLibs`, where the package
/// loads it from.
#[must_use]
pub(crate) fn libraries() -> Vec<Step> {
    let mut add = vec!["target", "add"];
    add.extend(ANDROID_TARGETS);
    let jni_libraries = format!("{PROJECT}/generated-android/src/main/jniLibs");
    let mut build = vec!["ndk", "--platform", API_LEVEL];
    for abi in ANDROID_ABIS {
        build.extend(["--target", abi]);
    }
    build.extend([
        "--output-dir",
        &jni_libraries,
        "build",
        "--package",
        "eranos-uniffi",
        "--lib",
    ]);
    vec![
        Step::new(&add, "the Android targets", "rustup"),
        Step::cargo(&build, "the library of every Android ABI"),
    ]
}

/// Where the package keeps the generated Kotlin; the gates clear it first,
/// so a type the library no longer has leaves no file behind.
#[must_use]
pub(crate) fn generated_sources() -> PathBuf {
    Path::new(WORKSPACE)
        .join(PROJECT)
        .join("generated")
        .join("src")
}

/// The steps that copy the generated Kotlin into the package's generated
/// module, which the JVM and the Android library both compile. The paths of
/// the steps are written with `/`, which the programs of every host accept,
/// so a step reads alike on every host.
#[must_use]
pub(crate) fn sources(metadata: &Metadata) -> Vec<Step> {
    let directory = format!("{PROJECT}/generated/src/main/kotlin/eranos/internal");
    let generated = format!(
        "{}/uniffi/kotlin/eranos/internal/eranos.kt",
        metadata.target_directory
    );
    vec![
        Step::new(
            &["-p", &directory],
            "the generated module's directory",
            "mkdir",
        ),
        Step::new(&[&generated, &directory], "the generated Kotlin", "cp"),
    ]
}

/// The step that runs Gradle's `tasks` in the project, reported as `name`,
/// with the tests loading the host library from `metadata`'s target
/// directory.
fn gradle(metadata: &Metadata, tasks: &[&str], name: &str) -> Step {
    let library = format!("-Peranos.library={}/debug", metadata.target_directory);
    let mut arguments = vec!["gradlew", "--warning-mode=fail", &library];
    arguments.extend(tasks);
    Step::new(&arguments, name, "sh").in_directory(PROJECT)
}

/// The steps that test the package with its coverage, lint it, check its
/// API against the committed dump, build its documentation, and build the
/// Android libraries, each with warnings as errors.
#[must_use]
pub(crate) fn checks(metadata: &Metadata) -> Vec<Step> {
    let gradle = |tasks: &[&str], name: &str| gradle(metadata, tasks, name);
    vec![
        gradle(
            &[":eranos:test", ":eranos:koverVerify"],
            "the Kotlin tests and their coverage",
        ),
        Step::new(
            &[
                "--relative",
                "**/*.kt",
                "**/*.kts",
                "!generated/**",
                "!**/build/**",
            ],
            "ktlint",
            "ktlint",
        )
        .in_directory(PROJECT),
        gradle(&[":eranos:detekt", ":examples:detekt"], "detekt"),
        gradle(&[":eranos:apiCheck"], "the Kotlin API against its dump"),
        gradle(&[":eranos:dokkaGenerate"], "the Kotlin documentation"),
        gradle(
            &[
                ":generated-android:assembleDebug",
                ":eranos-android:assembleDebug",
            ],
            "the Android libraries",
        ),
    ]
}

/// The step that compiles the ports of the examples and gathers them, with
/// every library they run with, into one directory.
#[must_use]
pub(crate) fn ports(metadata: &Metadata) -> Step {
    gradle(metadata, &[":examples:ports"], "the Kotlin ports")
}

/// The step that runs the Kotlin port of `example` on the `java` of the JDK
/// at `jdk`, JNA loading the host library from `sequential`'s target
/// directory.
#[must_use]
pub(crate) fn port(example: &str, jdk: &str, sequential: &Metadata) -> Step {
    ports::jvm_port(&JVM, example, jdk, sequential)
}

/// The step that runs the package's tests again, whatever Gradle holds of
/// an earlier run, against the host library in `metadata`'s target
/// directory: the instrumented one, whose coverage the run writes.
#[must_use]
pub(crate) fn tests(metadata: &Metadata) -> Step {
    gradle(metadata, &[":eranos:test", "--rerun"], "the Kotlin tests")
}

#[cfg(test)]
mod tests;
