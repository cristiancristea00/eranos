//! The ports of the examples: every example of eranos prints a report, run
//! with every feature, which its port in each language prints byte for byte
//! on a build of the glue whose identifiers count up from one as the Rust
//! example's do. The ports run where their language builds: the Swift ones
//! where the machine builds for Apple platforms, and the Kotlin, Java,
//! TypeScript, C, C++, and Python ones on every machine, the Kotlin ones
//! over the Kotlin package's bindings and again over the Kotlin
//! Multiplatform package's, the TypeScript ones on Node, over a WebAssembly
//! build of the glue, the C and C++ ones under the sanitizers, and the
//! Python ones in a copy of the Python package.

use std::path::PathBuf;

use crate::Failure;
use crate::gates::{self, Metadata};
use crate::language::Language;
use crate::plan::{self, Action, Phase};
use crate::process::{Runner, Step};
use crate::{capi, java, kmp, kotlin, python, swift, web};

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

/// Every language with ports of the examples.
pub(crate) const LANGUAGES: [Language; 7] = [
    Language::Capi,
    Language::Java,
    Language::Kmp,
    Language::Kotlin,
    Language::Python,
    Language::Swift,
    Language::Web,
];

/// Builds the glue whose identifiers count up for the ports of `languages`,
/// runs each of their ports of the examples of `references` on it, and
/// compares each report with its example's, stopping at the first that
/// fails.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable the ports of `languages` need that
/// is not set, before any step runs; the failure of the first step that
/// fails, of clearing an earlier copy or generated source, of writing the
/// generator's settings for the TypeScript copy, and of creating the
/// directory of the C or the C++ ports, and [`Failure::Port`] for a port
/// whose report is not its example's.
pub(crate) fn run(
    runner: &mut impl Runner,
    metadata: &Metadata,
    references: &[Reference],
    languages: &[Language],
) -> Result<(), Failure> {
    let actions = actions(runner, metadata, references, languages)?;
    plan::execute(runner, actions, &|_| true)
}

/// The languages of `languages` whose ports run on this machine: the Swift
/// ones only where it builds for Apple platforms, as `apple` says.
#[must_use]
pub(crate) fn runnable(languages: &[Language], apple: bool) -> Vec<Language> {
    languages
        .iter()
        .copied()
        .filter(|&language| apple || Language::Swift != language)
        .collect()
}

/// The actions of the ports of `languages`, in the order the gates run
/// them: the libraries with identifiers that count up that they run on,
/// every directory they write cleared, the sources generated from the first
/// library, then each language's ports.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable the ports of `languages` need that
/// is not set.
fn actions(
    runner: &impl Runner,
    metadata: &Metadata,
    references: &[Reference],
    languages: &[Language],
) -> Result<Vec<Action>, Failure> {
    let languages = runnable(languages, runner.builds_apple_platforms());
    let sequential = metadata.inner(DIRECTORY);
    // Each language's ports read their variables before any action is made,
    // so a missing one stops the ports before any step runs.
    let ports = languages_ports(runner, metadata, &sequential, references, &languages)?;
    let mut actions = libraries(&sequential, &languages);
    actions.extend(
        cleared(&sequential, &languages)
            .into_iter()
            .map(|directory| Action::clear(Phase::Prepare, directory)),
    );
    if languages.iter().any(|language| language.reads_uniffi()) {
        actions.extend(Action::runs(Phase::Prepare, gates::generators(&sequential)));
    }
    actions.extend(ports);
    Ok(actions)
}

/// The actions that build the libraries with identifiers that count up, in
/// `sequential`'s target directory, that the ports of `languages` run on:
/// eranos-uniffi's for the packages over it, and eranos-kmp's, eranos-web's,
/// and eranos-capi's for theirs.
fn libraries(sequential: &Metadata, languages: &[Language]) -> Vec<Action> {
    let ported = |language| languages.contains(&language);
    let mut steps = Vec::new();
    if languages.iter().any(|language| language.reads_uniffi()) {
        steps.push(library(sequential));
    }
    if ported(Language::Kmp) {
        steps.push(multiplatform_library(sequential));
    }
    if ported(Language::Web) {
        steps.push(web::sequential_library(sequential));
    }
    if ported(Language::Capi) {
        steps.push(capi::sequential_library(sequential));
    }
    Action::runs(Phase::Prepare, steps)
}

/// The actions of each of `languages`' ports, over the libraries in
/// `sequential`'s target directory, in the order the gates run them: the
/// Swift ports, the Kotlin ones over the Kotlin package's bindings and again
/// over the Kotlin Multiplatform package's, the Java ones, the TypeScript
/// ones, the C and the C++ ones, and the Python ones, each report compared
/// with its example's in `references`.
///
/// # Errors
///
/// [`Failure::Variable`] for a variable the ports of `languages` need that
/// is not set.
fn languages_ports(
    runner: &impl Runner,
    metadata: &Metadata,
    sequential: &Metadata,
    references: &[Reference],
    languages: &[Language],
) -> Result<Vec<Action>, Failure> {
    let ported = |language| languages.contains(&language);
    let variable = |name: &'static str| runner.variable(name).ok_or(Failure::Variable(name));
    let mut ports = Vec::new();
    if ported(Language::Swift) {
        ports.extend(swift_ports(sequential, references));
    }
    if ported(Language::Kotlin) {
        let jdk = variable(kotlin::JDK)?;
        ports.push(Action::run(Phase::Prepare, kotlin::ports(metadata)));
        ports.extend(checks(kotlin::JVM.language, references, &|example| {
            kotlin::port(example, &jdk, sequential)
        }));
    }
    if ported(Language::Kmp) {
        let [sdk, jdk] = kmp::VARIABLES.map(variable);
        // Gradle finds the Android SDK through the environment the ports run in.
        sdk?;
        let jdk = jdk?;
        ports.push(Action::run(Phase::Prepare, kmp::ports(&jdk)));
        ports.extend(checks(kmp::JVM.language, references, &|example| {
            kmp::port(example, &jdk, sequential)
        }));
    }
    if ported(Language::Java) {
        let jdk = variable(kotlin::JDK)?;
        ports.push(Action::run(Phase::Prepare, java::ports(metadata)));
        ports.extend(checks(java::JVM.language, references, &|example| {
            java::port(example, &jdk, sequential)
        }));
    }
    if ported(Language::Web) {
        ports.extend(Action::runs(Phase::Prepare, web::copy(sequential)));
        ports.push(Action::write(
            Phase::Prepare,
            web::copied_configuration(sequential),
            web::configuration(),
        ));
        ports.push(Action::run(
            Phase::Prepare,
            web::copied_generator(sequential),
        ));
        ports.extend(checks("TypeScript", references, &|example| {
            web::port(example, sequential)
        }));
    }
    if ported(Language::Capi) {
        let llvm = variable(capi::VARIABLE)?;
        for package in [capi::C_PACKAGE, capi::CPP_PACKAGE] {
            ports.push(Action::create(
                Phase::Prepare,
                capi::ports_directory(package, sequential),
            ));
            ports.extend(references.iter().map(|reference| {
                Action::run(
                    Phase::Prepare,
                    capi::port_build(package, &llvm, sequential, &reference.example),
                )
            }));
            ports.extend(checks(package.name, references, &|example| {
                capi::port(package, example, sequential)
            }));
        }
    }
    if ported(Language::Python) {
        ports.extend(Action::runs(Phase::Prepare, python::copy(sequential)));
        ports.extend(checks("Python", references, &|example| {
            python::port(example, sequential)
        }));
    }
    Ok(ports)
}

/// The actions that copy the Swift package into `sequential`'s target
/// directory over the library there, build the examples in the copy, and
/// run the port of every example of `references` from the directory of the
/// examples' programs, which the build prints.
fn swift_ports(sequential: &Metadata, references: &[Reference]) -> Vec<Action> {
    let mut actions = Action::runs(Phase::Prepare, swift::copy(sequential));
    actions.extend(Action::runs(Phase::Prepare, swift::examples(sequential)));
    let references = references.to_vec();
    actions.push(Action::read(
        Phase::Prepare,
        swift::examples_directory(sequential),
        move |programs| {
            let programs = programs.trim().to_owned();
            Ok(checks("Swift", &references, &|example| {
                port("Swift", example, &programs)
            }))
        },
    ));
    actions
}

/// The actions that run the port in `language` of every example of
/// `references`, the step `port` makes for its name, and compare its report
/// with the example's.
fn checks(
    language: &'static str,
    references: &[Reference],
    port: &dyn Fn(&str) -> Step,
) -> Vec<Action> {
    references
        .iter()
        .map(|reference| {
            let reference = reference.clone();
            Action::read(Phase::Test, port(&reference.example), move |report| {
                compare(language, &reference, &report).map(|()| Vec::new())
            })
        })
        .collect()
}

/// What the ports of `languages` clear before they build on the library, so
/// nothing an earlier run left behind is read: the sources generated from
/// `sequential`'s library for the packages over eranos-uniffi, the Swift
/// package's copy but its build, the TypeScript package's copy but its
/// dependencies' link, the C and C++ ports' programs, and the Python
/// package's copy but its environment.
#[must_use]
pub(crate) fn cleared(sequential: &Metadata, languages: &[Language]) -> Vec<PathBuf> {
    let ported = |language| languages.contains(&language);
    let mut directories = Vec::new();
    if languages.iter().any(|language| language.reads_uniffi()) {
        directories.push(gates::generated(sequential));
    }
    if ported(Language::Swift) {
        directories.extend(swift::copied_directories(sequential));
    }
    if ported(Language::Web) {
        directories.extend(web::copied_directories(sequential));
    }
    if ported(Language::Capi) {
        directories.extend(
            [capi::C_PACKAGE, capi::CPP_PACKAGE]
                .map(|package| capi::ports_directory(package, sequential)),
        );
    }
    if ported(Language::Python) {
        directories.extend(python::copied_directories(sequential));
    }
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
