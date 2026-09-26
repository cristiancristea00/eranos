//! The gates of the Swift package, on macOS: the `XCFramework` of the library
//! for every Apple platform, the generated Swift in the package, and the
//! package's build, tests, linters, and documentation.

use std::env::consts::{DLL_PREFIX, DLL_SUFFIX};
use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::process::{Runner, Step, WORKSPACE};

/// The Swift package, from the workspace root.
const PACKAGE: &str = "bindings/swift";

/// The package of the Swift ports of the examples, from the workspace root,
/// which depends on the Swift package as `..`.
const EXAMPLES: &str = "bindings/swift/examples";

/// The static library every Apple target builds.
const LIBRARY: &str = "liberanos_uniffi.a";

/// A library of the `XCFramework`, for one platform and kind of device, and
/// the Rust targets it is built from: two are joined by `lipo`.
#[derive(Debug)]
pub(crate) struct AppleLibrary {
    pub(crate) name: &'static str,
    pub(crate) targets: &'static [&'static str],
}

/// Every library of the `XCFramework`: iOS, iPadOS, macOS, Mac Catalyst, tvOS,
/// watchOS, and visionOS, on devices and in simulators, every one on stable
/// Rust.
pub(crate) const APPLE_LIBRARIES: [AppleLibrary; 10] = [
    AppleLibrary {
        name: "ios",
        targets: &["aarch64-apple-ios"],
    },
    AppleLibrary {
        name: "ios-simulator",
        targets: &["aarch64-apple-ios-sim", "x86_64-apple-ios"],
    },
    AppleLibrary {
        name: "mac-catalyst",
        targets: &["aarch64-apple-ios-macabi", "x86_64-apple-ios-macabi"],
    },
    AppleLibrary {
        name: "macos",
        targets: &["aarch64-apple-darwin", "x86_64-apple-darwin"],
    },
    AppleLibrary {
        name: "tvos",
        targets: &["aarch64-apple-tvos"],
    },
    AppleLibrary {
        name: "tvos-simulator",
        targets: &["aarch64-apple-tvos-sim"],
    },
    AppleLibrary {
        name: "visionos",
        targets: &["aarch64-apple-visionos"],
    },
    AppleLibrary {
        name: "visionos-simulator",
        targets: &["aarch64-apple-visionos-sim"],
    },
    AppleLibrary {
        name: "watchos",
        targets: &["aarch64-apple-watchos"],
    },
    AppleLibrary {
        name: "watchos-simulator",
        targets: &["aarch64-apple-watchos-sim"],
    },
];

/// Runs the Swift package's gates, stopping at the first that fails.
///
/// # Errors
///
/// The failure of the first step that fails, and of clearing the old
/// `XCFramework` or writing the builds' log the analyzer reads.
pub(crate) fn run(runner: &mut impl Runner, metadata: &Metadata) -> Result<(), Failure> {
    for step in libraries(metadata) {
        runner.run(&step)?;
    }
    runner.clear(
        &Path::new(WORKSPACE)
            .join(PACKAGE)
            .join("EranosFFI.xcframework"),
    )?;
    for step in package(metadata) {
        runner.run(&step)?;
    }
    let package = runner.output(&build())?;
    let examples = runner.output(&build_examples())?;
    runner.write(&build_log(metadata), &format!("{package}{examples}"))?;
    for step in checks(metadata) {
        runner.run(&step)?;
    }
    Ok(())
}

/// Where the gates keep the Apple libraries, their headers, and the build
/// log, under cargo's target directory. The steps' paths are written with
/// `/`, which the programs of every host accept, so a step reads alike on
/// every host.
fn apple(metadata: &Metadata) -> String {
    format!("{}/apple", metadata.target_directory)
}

/// The static library cargo builds for `target`.
fn target_library(metadata: &Metadata, target: &str) -> String {
    format!("{}/{target}/debug/{LIBRARY}", metadata.target_directory)
}

/// The library of the `XCFramework` for `library`: the one target's own, or
/// the one `lipo` joins from two.
fn joined_library(metadata: &Metadata, library: &AppleLibrary) -> String {
    match library.targets {
        [target] => target_library(metadata, target),
        _ => format!("{}/{}/{LIBRARY}", apple(metadata), library.name),
    }
}

/// The steps that build the static library for every Apple target, join
/// the pairs that share a platform, and gather the C header and module map
/// the generated Swift imports.
#[must_use]
pub(crate) fn libraries(metadata: &Metadata) -> Vec<Step> {
    let targets: Vec<&str> = APPLE_LIBRARIES
        .iter()
        .flat_map(|library| library.targets.iter().copied())
        .collect();
    let mut add = vec!["target", "add"];
    add.extend(&targets);
    let mut build = vec!["build", "--package", "eranos-uniffi", "--lib"];
    for target in &targets {
        build.extend(["--target", target]);
    }
    let mut steps = vec![
        Step::new(&add, "the Apple targets", "rustup"),
        Step::cargo(&build, "the static library of every Apple target"),
    ];
    for library in APPLE_LIBRARIES
        .iter()
        .filter(|library| 1 < library.targets.len())
    {
        let directory = format!("{}/{}", apple(metadata), library.name);
        let mut lipo = vec!["-create".to_owned()];
        lipo.extend(
            library
                .targets
                .iter()
                .map(|target| target_library(metadata, target)),
        );
        lipo.extend(["-output".to_owned(), joined_library(metadata, library)]);
        let lipo: Vec<&str> = lipo.iter().map(String::as_str).collect();
        steps.push(Step::new(
            &["-p", &directory],
            &format!("the {} directory", library.name),
            "mkdir",
        ));
        steps.push(Step::new(
            &lipo,
            &format!("the {} library", library.name),
            "lipo",
        ));
    }
    let generated = format!("{}/uniffi/swift", metadata.target_directory);
    let headers = format!("{}/headers", apple(metadata));
    let copy = |from: &str, to: &str, name: &str| {
        Step::new(
            &[&format!("{generated}/{from}"), &format!("{headers}/{to}")],
            name,
            "cp",
        )
    };
    steps.push(Step::new(
        &["-p", &headers],
        "the headers directory",
        "mkdir",
    ));
    steps.push(copy(
        "EranosGeneratedFFI.h",
        "EranosGeneratedFFI.h",
        "the C header",
    ));
    steps.push(copy(
        "EranosGeneratedFFI.modulemap",
        "module.modulemap",
        "the module map",
    ));
    steps
}

/// The steps that make the `XCFramework` of every library, copy the generated
/// Swift into the package, and clear the earlier builds of the package and
/// of its examples, so that the analyzer's log names every file.
#[must_use]
pub(crate) fn package(metadata: &Metadata) -> Vec<Step> {
    let headers = format!("{}/headers", apple(metadata));
    let libraries: Vec<String> = APPLE_LIBRARIES
        .iter()
        .map(|library| joined_library(metadata, library))
        .collect();
    let mut framework = vec!["-create-xcframework"];
    for library in &libraries {
        framework.extend(["-library", library, "-headers", &headers]);
    }
    let output = format!("{PACKAGE}/EranosFFI.xcframework");
    framework.extend(["-output", &output]);
    let sources = format!("{PACKAGE}/Sources/EranosGenerated");
    let generated = format!(
        "{}/uniffi/swift/EranosGenerated.swift",
        metadata.target_directory
    );
    vec![
        Step::new(&framework, "the XCFramework", "xcodebuild"),
        Step::new(
            &["-p", &sources],
            "the generated module's directory",
            "mkdir",
        ),
        Step::new(&[&generated, &sources], "the generated Swift", "cp"),
        Step::new(
            &["package", "--package-path", PACKAGE, "clean"],
            "the Swift package's earlier build",
            "swift",
        ),
        Step::new(
            &["package", "--package-path", EXAMPLES, "clean"],
            "the Swift examples' earlier build",
            "swift",
        ),
    ]
}

/// The step that builds the package and its tests, printing every compiler
/// call for the analyzer.
#[must_use]
pub(crate) fn build() -> Step {
    Step::new(
        &[
            "build",
            "--package-path",
            PACKAGE,
            "--build-tests",
            "--verbose",
        ],
        "the Swift package's build",
        "swift",
    )
}

/// The step that builds the examples' package over the package, printing
/// every compiler call for the analyzer; the ports run from a copy, on a
/// library whose identifiers count up.
#[must_use]
pub(crate) fn build_examples() -> Step {
    Step::new(
        &["build", "--package-path", EXAMPLES, "--verbose"],
        "the Swift examples' build",
        "swift",
    )
}

/// Where the builds' compiler calls go, for the analyzer: the package's,
/// then its examples'.
#[must_use]
pub(crate) fn build_log(metadata: &Metadata) -> PathBuf {
    PathBuf::from(format!("{}/swift-build.log", apple(metadata)))
}

/// The steps that test the package, lint it, analyze it, and build its
/// documentation, each with warnings as errors.
#[must_use]
pub(crate) fn checks(metadata: &Metadata) -> Vec<Step> {
    let log = build_log(metadata).to_string_lossy().into_owned();
    vec![
        Step::new(
            &["test", "--package-path", PACKAGE],
            "the Swift tests",
            "swift",
        ),
        Step::new(&["lint", "--strict"], "SwiftLint", "swiftlint").in_directory(PACKAGE),
        Step::new(
            &["analyze", "--strict", "--compiler-log-path", &log],
            "SwiftLint's analyzer",
            "swiftlint",
        )
        .in_directory(PACKAGE),
        Step::new(
            &[
                "package",
                "--package-path",
                PACKAGE,
                "generate-documentation",
                "--target",
                "Eranos",
                "--warnings-as-errors",
            ],
            "the Swift documentation",
            "swift",
        ),
    ]
}

/// Where the package is copied in `metadata`'s target directory, beside a
/// library built there: the instrumented one the coverage measures, or the
/// one whose identifiers count up that the examples print. The package keeps
/// the `XCFramework` the gates built.
#[must_use]
pub(crate) fn copied_package(metadata: &Metadata) -> PathBuf {
    Path::new(&metadata.target_directory).join("swift")
}

/// What is cleared before the package is copied: the copy's sources, tests,
/// and examples, so a file the package no longer has leaves nothing behind,
/// its `XCFramework`, and the headers it gathers; the copy's builds stay, so
/// a later run builds only what changed.
#[must_use]
pub(crate) fn copied_directories(metadata: &Metadata) -> Vec<PathBuf> {
    let package = copied_package(metadata);
    vec![
        package.join("Sources"),
        package.join("Tests"),
        package.join("examples").join("Sources"),
        package.join("EranosFFI.xcframework"),
        PathBuf::from(format!("{}/headers", apple(metadata))),
    ]
}

/// The steps that copy the package, with the generated Swift, into
/// `metadata`'s target directory, and make its `XCFramework` of the shared
/// library there, which a program loads when it starts: the library's
/// instrumentation, when it has any, then writes a profile as the program
/// ends.
#[must_use]
pub(crate) fn copy(metadata: &Metadata) -> Vec<Step> {
    let package = copied_package(metadata).to_string_lossy().into_owned();
    let headers = format!("{}/headers", apple(metadata));
    let generated = format!("{}/uniffi/swift", metadata.target_directory);
    let sources = format!("{package}/Sources/EranosGenerated");
    let library = format!(
        "{}/debug/{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}",
        metadata.target_directory
    );
    let framework = format!("{package}/EranosFFI.xcframework");
    vec![
        Step::new(
            &["-p", &package, &headers],
            "the package's copy and its headers",
            "mkdir",
        ),
        Step::new(
            &[
                "-R",
                &format!("{PACKAGE}/Package.swift"),
                &format!("{PACKAGE}/Package.resolved"),
                &format!("{PACKAGE}/Sources"),
                &format!("{PACKAGE}/Tests"),
                &package,
            ],
            "the Swift package",
            "cp",
        ),
        Step::new(
            &["-p", &sources],
            "the generated module's directory",
            "mkdir",
        ),
        Step::new(
            &[&format!("{generated}/EranosGenerated.swift"), &sources],
            "the generated Swift",
            "cp",
        ),
        Step::new(
            &[
                &format!("{generated}/EranosGeneratedFFI.h"),
                &format!("{headers}/EranosGeneratedFFI.h"),
            ],
            "the C header",
            "cp",
        ),
        Step::new(
            &[
                &format!("{generated}/EranosGeneratedFFI.modulemap"),
                &format!("{headers}/module.modulemap"),
            ],
            "the module map",
            "cp",
        ),
        Step::new(
            &[
                "-create-xcframework",
                "-library",
                &library,
                "-headers",
                &headers,
                "-output",
                &framework,
            ],
            "the XCFramework of the copy's library",
            "xcodebuild",
        ),
    ]
}

/// The step that runs the copy's tests on its library.
#[must_use]
pub(crate) fn copy_tests(metadata: &Metadata) -> Step {
    Step::new(
        &[
            "test",
            "--package-path",
            &copied_package(metadata).to_string_lossy(),
        ],
        "the Swift tests",
        "swift",
    )
}

/// Where the examples' package sits in the copy, whose `..` it depends on.
fn copied_examples(metadata: &Metadata) -> String {
    copied_package(metadata)
        .join("examples")
        .to_string_lossy()
        .into_owned()
}

/// The steps that copy the examples' package into the copy of the package
/// and build every example there, on the copy's library.
#[must_use]
pub(crate) fn examples(metadata: &Metadata) -> Vec<Step> {
    let examples = copied_examples(metadata);
    vec![
        Step::new(&["-p", &examples], "the examples' directory", "mkdir"),
        Step::new(
            &[
                "-R",
                &format!("{EXAMPLES}/Package.swift"),
                &format!("{EXAMPLES}/Sources"),
                &examples,
            ],
            "the Swift examples",
            "cp",
        ),
        Step::new(
            &["build", "--package-path", &examples],
            "the Swift examples' build in the copy",
            "swift",
        ),
    ]
}

/// The step that prints the directory of the examples' programs.
#[must_use]
pub(crate) fn examples_directory(metadata: &Metadata) -> Step {
    Step::new(
        &[
            "build",
            "--package-path",
            &copied_examples(metadata),
            "--show-bin-path",
        ],
        "the Swift examples' directory",
        "swift",
    )
}

#[cfg(test)]
mod tests;
