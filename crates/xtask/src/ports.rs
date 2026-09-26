//! The ports of the examples: every example of eranos prints a report, run
//! with every feature, which its port in each language prints byte for byte
//! on a build of the glue whose identifiers count up from one as the Rust
//! example's do. The ports run where their language builds: the Swift ones
//! where the machine builds for Apple platforms, and the Kotlin, Java,
//! TypeScript, and C ones on every machine, the Kotlin ones over the Kotlin
//! package's bindings and again over the Kotlin Multiplatform package's,
//! the TypeScript ones on Node, over a WebAssembly build of the glue, and
//! the C ones under the sanitizers.

use std::path::PathBuf;

use crate::Failure;
use crate::gates::{self, Metadata};
use crate::process::{Runner, Step};
use crate::{capi, java, kmp, kotlin, swift, web};

/// The target directory, inside cargo's, of the glue whose identifiers
/// count up: a directory of its own, so the gates' builds stay as they are.
pub(crate) const DIRECTORY: &str = "sequential";

/// What a Rust example printed, which each of its ports prints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reference {
    /// The example's name.
    pub(crate) example: String,
    /// Its standard output.
    pub(crate) report: String,
}

/// Builds the glue whose identifiers count up, runs every port of every
/// example on it, and compares each report with its example's `references`,
/// stopping at the first that fails.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable the Kotlin, the Kotlin
/// Multiplatform, or the C ports need that is not set, before any step runs;
/// the failure of the first step that fails, of clearing an earlier copy or
/// generated source, of writing the generator's settings for the TypeScript
/// copy, and of creating the directory of the C ports, and
/// [`Failure::Port`] for a port whose report is not its example's.
pub(crate) fn run(
    runner: &mut impl Runner,
    metadata: &Metadata,
    references: &[Reference],
) -> Result<(), Failure> {
    let jdk = runner
        .variable(kotlin::JDK)
        .ok_or(Failure::Variable(kotlin::JDK))?;
    let [sdk, multiplatform_jdk] =
        kmp::VARIABLES.map(|name| runner.variable(name).ok_or(Failure::Variable(name)));
    // Gradle finds the Android SDK through the environment the gates run in.
    sdk?;
    let multiplatform_jdk = multiplatform_jdk?;
    let llvm = runner
        .variable(capi::VARIABLE)
        .ok_or(Failure::Variable(capi::VARIABLE))?;
    let sequential = metadata.inner(DIRECTORY);
    let apple = runner.builds_apple_platforms();
    runner.run(&library(&sequential))?;
    runner.run(&multiplatform_library(&sequential))?;
    runner.run(&web::sequential_library(&sequential))?;
    runner.run(&capi::sequential_library(&sequential))?;
    for directory in cleared(&sequential, apple) {
        runner.clear(&directory)?;
    }
    for step in gates::generators(&sequential) {
        runner.run(&step)?;
    }
    if apple {
        for step in swift::copy(&sequential)
            .into_iter()
            .chain(swift::examples(&sequential))
        {
            runner.run(&step)?;
        }
        let programs = runner.output(&swift::examples_directory(&sequential))?;
        let swift = |example: &str| port("Swift", example, programs.trim());
        check(runner, "Swift", references, &swift)?;
    }
    runner.run(&kotlin::ports(metadata))?;
    let kotlin = |example: &str| kotlin::port(example, &jdk, &sequential);
    check(runner, kotlin::JVM.language, references, &kotlin)?;
    runner.run(&kmp::ports(&multiplatform_jdk))?;
    let multiplatform = |example: &str| kmp::port(example, &multiplatform_jdk, &sequential);
    check(runner, kmp::JVM.language, references, &multiplatform)?;
    runner.run(&java::ports(metadata))?;
    let java = |example: &str| java::port(example, &jdk, &sequential);
    check(runner, java::JVM.language, references, &java)?;
    for step in web::copy(&sequential) {
        runner.run(&step)?;
    }
    runner.write(
        &web::copied_configuration(&sequential),
        &web::configuration(),
    )?;
    runner.run(&web::copied_generator(&sequential))?;
    let typescript = |example: &str| web::port(example, &sequential);
    check(runner, "TypeScript", references, &typescript)?;
    runner.create(&capi::ports_directory(capi::C_PACKAGE, &sequential))?;
    for reference in references {
        runner.run(&capi::port_build(
            capi::C_PACKAGE,
            &llvm,
            &sequential,
            &reference.example,
        ))?;
    }
    let native = |example: &str| capi::port(capi::C_PACKAGE, example, &sequential);
    check(runner, capi::C_PACKAGE.name, references, &native)
}

/// Runs the port in `language` of every example of `references`, the step
/// `port` makes for its name, and compares its report with the example's,
/// stopping at the first port that fails or differs.
fn check(
    runner: &mut impl Runner,
    language: &'static str,
    references: &[Reference],
    port: &dyn Fn(&str) -> Step,
) -> Result<(), Failure> {
    for reference in references {
        compare(
            language,
            reference,
            &runner.output(&port(&reference.example))?,
        )?;
    }
    Ok(())
}

/// What the ports clear before they build on the library, so nothing an
/// earlier run left behind is read: the sources generated from
/// `sequential`'s library, where `apple` the Swift package's copy but its
/// build, the TypeScript package's copy but its dependencies' link, and the
/// C ports' programs.
#[must_use]
pub(crate) fn cleared(sequential: &Metadata, apple: bool) -> Vec<PathBuf> {
    let mut directories = vec![gates::generated(sequential)];
    if apple {
        directories.extend(swift::copied_directories(sequential));
    }
    directories.extend(web::copied_directories(sequential));
    directories.push(capi::ports_directory(capi::C_PACKAGE, sequential));
    directories
}

/// The step that builds eranos-uniffi's library for the host, with
/// identifiers that count up, into `sequential`'s target directory.
#[must_use]
pub(crate) fn library(sequential: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-uniffi",
            "--features",
            "sequential-ids",
        ],
        "the host library of eranos-uniffi, with identifiers that count up",
    )
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// The step that builds eranos-kmp's library for the host, with
/// identifiers that count up, into `sequential`'s target directory, where
/// the Kotlin Multiplatform ports find it.
#[must_use]
pub(crate) fn multiplatform_library(sequential: &Metadata) -> Step {
    Step::cargo(
        &[
            "build",
            "--package",
            "eranos-kmp",
            "--features",
            "sequential-ids",
        ],
        "the host library of eranos-kmp, with identifiers that count up",
    )
    .with_variable("CARGO_TARGET_DIR", &sequential.target_directory)
}

/// Where a JVM language's ports are gathered, and how the JVM starts one.
pub(crate) struct Jvm {
    /// The class that holds a port's `main`, in the package
    /// `eranos.examples.<example>`, named without underscores.
    pub(crate) class: &'static str,
    /// The language, as a report that differs names it.
    pub(crate) language: &'static str,
    /// The system property that names the directory the library loads from.
    pub(crate) library_path: &'static str,
    /// The directory, from the workspace root, that holds the ports and
    /// every library they run with.
    pub(crate) ports: &'static str,
}

/// The step that runs the port of `example` in `jvm`'s language on the
/// `java` of the JDK at `jdk`, the library loading from `sequential`'s
/// target directory. The port writes its report in UTF-8, as the Rust
/// example does, whatever the machine's locale, from which the JVM would
/// otherwise take the encoding of its output.
#[must_use]
pub(crate) fn jvm_port(jvm: &Jvm, example: &str, jdk: &str, sequential: &Metadata) -> Step {
    let library = format!(
        "-D{}={}/debug",
        jvm.library_path, sequential.target_directory
    );
    let classes = format!("{}/*", jvm.ports);
    let class = format!("eranos.examples.{}.{}", example.replace('_', ""), jvm.class);
    Step::new(
        &[
            "--enable-native-access=ALL-UNNAMED",
            "-Dstdout.encoding=UTF-8",
            &library,
            "-cp",
            &classes,
            &class,
        ],
        &format!("the {} port of {example}", jvm.language),
        &format!("{jdk}/bin/java"),
    )
}

/// The step that runs the port of `example` in `language`, a program in
/// the directory `programs`.
#[must_use]
pub(crate) fn port(language: &str, example: &str, programs: &str) -> Step {
    Step::new(
        &[],
        &format!("the {language} port of {example}"),
        &format!("{programs}/{example}"),
    )
}

/// Succeeds when `report`, what a port in `language` printed, is the
/// `reference`'s report byte for byte.
///
/// # Errors
///
/// [`Failure::Port`] with the first line that differs and both versions of
/// it, the end of a report standing for a line it lacks.
pub(crate) fn compare(
    language: &'static str,
    reference: &Reference,
    report: &str,
) -> Result<(), Failure> {
    if reference.report == report {
        return Ok(());
    }
    let expected: Vec<&str> = reference.report.split('\n').collect();
    let found: Vec<&str> = report.split('\n').collect();
    let index = expected
        .iter()
        .zip(&found)
        .position(|(expected, found)| expected != found)
        .unwrap_or_else(|| expected.len().min(found.len()));
    let line = |lines: &[&str]| {
        lines.get(index).map_or_else(
            || "the end of the report".to_owned(),
            |line| format!("{line:?}"),
        )
    };
    Err(Failure::Port {
        example: reference.example.clone(),
        expected: line(&expected),
        found: line(&found),
        language,
        line: index + 1,
    })
}

#[cfg(test)]
mod tests;
