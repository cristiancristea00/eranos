//! The gates of the Swift package, on macOS: the `XCFramework` of the library
//! for every Apple platform, the generated Swift in the package, and the
//! package's build, tests, linters, and documentation.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::gates::Metadata;
use crate::process::{Runner, Step, WORKSPACE};

/// The Swift package, from the workspace root.
const PACKAGE: &str = "bindings/swift";

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
/// `XCFramework` or writing the build log the analyzer reads.
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
    let log = runner.output(&build())?;
    runner.write(&build_log(metadata), &log)?;
    for step in checks(metadata) {
        runner.run(&step)?;
    }
    Ok(())
}

/// Where the gates keep the Apple libraries, their headers, and the build
/// log, under cargo's target directory.
fn apple(metadata: &Metadata) -> PathBuf {
    Path::new(&metadata.target_directory).join("apple")
}

/// The static library cargo builds for `target`.
fn target_library(metadata: &Metadata, target: &str) -> String {
    let path = Path::new(&metadata.target_directory)
        .join(target)
        .join("debug")
        .join(LIBRARY);
    path.to_string_lossy().into_owned()
}

/// The library of the `XCFramework` for `library`: the one target's own, or
/// the one `lipo` joins from two.
fn joined_library(metadata: &Metadata, library: &AppleLibrary) -> String {
    match library.targets {
        [target] => target_library(metadata, target),
        _ => apple(metadata)
            .join(library.name)
            .join(LIBRARY)
            .to_string_lossy()
            .into_owned(),
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
        .filter(|library| library.targets.len() > 1)
    {
        let directory = apple(metadata).join(library.name);
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
            &["-p", &directory.to_string_lossy()],
            &format!("the {} directory", library.name),
            "mkdir",
        ));
        steps.push(Step::new(
            &lipo,
            &format!("the {} library", library.name),
            "lipo",
        ));
    }
    let generated = Path::new(&metadata.target_directory)
        .join("uniffi")
        .join("swift");
    let headers = apple(metadata).join("headers");
    let copy = |from: &str, to: &str, name: &str| {
        Step::new(
            &[
                &generated.join(from).to_string_lossy(),
                &headers.join(to).to_string_lossy(),
            ],
            name,
            "cp",
        )
    };
    steps.push(Step::new(
        &["-p", &headers.to_string_lossy()],
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
/// Swift into the package, and clear the package's earlier build, so that
/// the analyzer's log names every file.
#[must_use]
pub(crate) fn package(metadata: &Metadata) -> Vec<Step> {
    let headers = apple(metadata)
        .join("headers")
        .to_string_lossy()
        .into_owned();
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
    let generated = Path::new(&metadata.target_directory)
        .join("uniffi")
        .join("swift")
        .join("EranosGenerated.swift");
    vec![
        Step::new(&framework, "the XCFramework", "xcodebuild"),
        Step::new(
            &["-p", &sources],
            "the generated module's directory",
            "mkdir",
        ),
        Step::new(
            &[&generated.to_string_lossy(), &sources],
            "the generated Swift",
            "cp",
        ),
        Step::new(
            &["package", "--package-path", PACKAGE, "clean"],
            "the Swift package's earlier build",
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

/// Where the build's compiler calls go, for the analyzer.
#[must_use]
pub(crate) fn build_log(metadata: &Metadata) -> PathBuf {
    apple(metadata).join("swift-build.log")
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

#[cfg(test)]
mod tests;
