//! The gates of the Kotlin package: the library of every Android ABI, the
//! generated Kotlin in the package, and the package's tests and their
//! coverage, linters, API dump, documentation, and Android libraries; and
//! the steps that build and run the ports of the examples on the JVM.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::jvm::{self, Jvm};
use crate::plan::{Action, Phase};
use crate::process::{self, Profile, Runner, Step, WORKSPACE};

/// The Kotlin project, from the workspace root.
const PROJECT: &str = "bindings/kotlin";

/// The files ktlint checks and formats: the layer, its tests, the examples,
/// and the build scripts, without the generated sources and Gradle's output.
const KOTLIN_FILES: [&str; 4] = ["**/*.kt", "**/*.kts", "!generated/**", "!**/build/**"];

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

/// The Kotlin package's gates, in order: the libraries of the Android ABIs
/// and the generated Kotlin in a cleared directory, then the package's
/// checks.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn actions(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    runner.require_all(&VARIABLES)?;
    let mut actions = Action::runs(Phase::Prepare, libraries());
    actions.push(Action::clear(Phase::Prepare, generated_sources()));
    actions.extend(Action::runs(Phase::Prepare, sources(metadata)));
    actions.extend(checks(metadata));
    Ok(actions)
}

/// The steps that add the Android targets and build the library of every
/// ABI into the generated Android library's `jniLibs`, where the package
/// loads it from.
#[must_use]
pub(crate) fn libraries() -> Vec<Step> {
    libraries_in(Profile::Debug)
}

/// The steps of [`libraries`] in `profile`.
fn libraries_in(profile: Profile) -> Vec<Step> {
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
    build.extend(profile.arguments());
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
    process::generated_copy(
        "Kotlin",
        &generated,
        &directory,
        "the generated module's directory",
        &[],
    )
}

/// The step that runs Gradle's `tasks` in the project, reported as `name`,
/// with the tests loading the host library from `metadata`'s target
/// directory.
fn gradle(metadata: &Metadata, tasks: &[&str], name: &str) -> Step {
    jvm::gradle_on_library(PROJECT, metadata, tasks, name)
}

/// The actions that test the package with its coverage, lint it, check its
/// API against the committed dump, build its documentation, and build the
/// Android libraries, each with warnings as errors.
#[must_use]
pub(crate) fn checks(metadata: &Metadata) -> Vec<Action> {
    let gradle = |tasks: &[&str], name: &str| gradle(metadata, tasks, name);
    vec![
        Action::run(
            Phase::Test,
            gradle(
                &[":eranos:test", ":eranos:koverVerify"],
                "the Kotlin tests and their coverage",
            ),
        ),
        Action::run(Phase::Lint, jvm::ktlint(PROJECT, &KOTLIN_FILES, "ktlint")),
        Action::run(
            Phase::Lint,
            gradle(&[":eranos:detekt", ":examples:detekt"], "detekt"),
        ),
        Action::run(
            Phase::Lint,
            gradle(&[":eranos:apiCheck"], "the Kotlin API against its dump"),
        ),
        Action::run(
            Phase::Docs,
            gradle(&[":eranos:dokkaGenerate"], "the Kotlin documentation"),
        ),
        Action::run(
            Phase::Package,
            gradle(
                &[
                    ":generated-android:assembleDebug",
                    ":eranos-android:assembleDebug",
                ],
                "the Android libraries",
            ),
        ),
    ]
}

/// The step that compiles the ports of the examples and gathers them, with
/// every library they run with, into one directory.
#[must_use]
pub(crate) fn ports(metadata: &Metadata) -> Step {
    gradle(metadata, &[":examples:ports"], "the Kotlin ports")
}

/// The step that runs the package's tests again, whatever Gradle holds of
/// an earlier run, against the host library in `metadata`'s target
/// directory: the instrumented one, whose coverage the run writes.
#[must_use]
pub(crate) fn tests(metadata: &Metadata) -> Step {
    gradle(metadata, &[":eranos:test", "--rerun"], "the Kotlin tests")
}

/// The Kotlin package's formatter, applied: ktlint over the layer, its
/// tests, the examples, and the build scripts, as its check reads them.
#[must_use]
pub(crate) fn formats() -> Vec<Action> {
    vec![Action::run(
        Phase::Format,
        jvm::ktlint_format(PROJECT, &KOTLIN_FILES, "the Kotlin format, applied"),
    )]
}

/// The actions that build the Kotlin packages an app consumes into their
/// directory of the packages, cleared first: the library of every Android
/// ABI in release, in the generated Android library's `jniLibs`, the
/// generated Kotlin, both Android libraries in release, and the JVM's JARs
/// beside the host library in release, which JNA loads from the directory
/// `jna.library.path` names.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable of [`VARIABLES`] that is not set.
pub(crate) fn release(runner: &impl Runner, metadata: &Metadata) -> Result<Vec<Action>, Failure> {
    runner.require_all(&VARIABLES)?;
    let packaged = crate::gates::packaged(metadata, "kotlin");
    let package = packaged.to_string_lossy().into_owned();
    let mut actions = vec![Action::clear(Phase::Package, packaged)];
    actions.extend(Action::runs(Phase::Package, libraries_in(Profile::Release)));
    actions.push(Action::clear(Phase::Package, generated_sources()));
    actions.extend(Action::runs(Phase::Package, sources(metadata)));
    actions.extend(Action::runs(
        Phase::Package,
        vec![
            gradle(
                metadata,
                &[
                    ":generated-android:assembleRelease",
                    ":eranos-android:assembleRelease",
                    ":generated:jar",
                    ":eranos:jar",
                ],
                "the Kotlin packages",
            ),
            crate::gates::release_host_library(),
            Step::new(&["-p", &package], "the Kotlin packages' directory", "mkdir"),
            Step::new(
                &[
                    &format!("{PROJECT}/generated-android/build/outputs/aar/generated-android-release.aar"),
                    &format!("{PROJECT}/eranos-android/build/outputs/aar/eranos-android-release.aar"),
                    &format!("{PROJECT}/generated/build/libs/generated.jar"),
                    &format!("{PROJECT}/eranos/build/libs/eranos.jar"),
                    &crate::gates::host_library_file(metadata, Profile::Release),
                    &package,
                ],
                "the Kotlin packages",
                "cp",
            ),
        ],
    ));
    Ok(actions)
}

#[cfg(test)]
mod tests;
