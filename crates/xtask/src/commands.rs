//! The commands that run part of the gates for the languages they name:
//! each language's tests, format checks and linters, or formatters, and the
//! ports of the examples in each.

use crate::Failure;
use crate::gates::{self, Metadata};
use crate::language::Language;
use crate::plan::{self, Action, Phase};
use crate::ports::{self, Reference};
use crate::process::Runner;
use crate::{capi, java, kmp, kotlin, python, swift, web};

/// The languages `name` stands for on a command line: the one of `known` it
/// names, or with `all` every one of `known`.
///
/// # Errors
///
/// [`Failure::Language`] for a name that is neither `all` nor a language of
/// `known`, naming the known ones.
pub(crate) fn languages(name: &str, known: &[Language]) -> Result<Vec<Language>, Failure> {
    if "all" == name {
        return Ok(known.to_vec());
    }
    Language::parse(name)
        .filter(|language| known.contains(language))
        .map(|language| vec![language])
        .ok_or_else(|| Failure::Language {
            known: known
                .iter()
                .map(|language| language.name())
                .chain(["or all"])
                .collect::<Vec<_>>()
                .join(", "),
            language: name.to_owned(),
        })
}

/// Runs the tests of `languages`, each after what they read is built.
///
/// # Errors
///
/// As [`prepared`] fails, and the failure of the first action that fails.
pub(crate) fn test(runner: &mut impl Runner, languages: &[Language]) -> Result<(), Failure> {
    run(runner, languages, Phase::Test)
}

/// Runs the format checks and the linters of `languages`, each after what
/// they read is built.
///
/// # Errors
///
/// As [`prepared`] fails, and the failure of the first action that fails.
pub(crate) fn lint(runner: &mut impl Runner, languages: &[Language]) -> Result<(), Failure> {
    run(runner, languages, Phase::Lint)
}

/// Applies the formatters of `languages`, which read the sources alone, so
/// nothing is built first.
///
/// # Errors
///
/// [`Failure::Apple`] for the Swift package alone on a machine that does not
/// build for Apple platforms, [`Failure::Variable`] for the LLVM of the C
/// and C++ packages' clang-format or the JDK of the Java package's Gradle
/// when it is not set, before any formatter runs, and the failure of the
/// first formatter that fails.
pub(crate) fn format(runner: &mut impl Runner, languages: &[Language]) -> Result<(), Failure> {
    let languages = runnable(runner, languages)?;
    let mut formats = Vec::new();
    for language in languages {
        formats.extend(match language {
            Language::Capi => capi::formats(&runner.required(capi::VARIABLE)?),
            Language::Java => {
                runner.require_all(&java::VARIABLES)?;
                java::formats()
            }
            Language::Kmp => kmp::formats(),
            Language::Kotlin => kotlin::formats(),
            Language::Python => python::formats(),
            Language::Rust => gates::formats(),
            Language::Swift => swift::formats(),
            Language::Web => web::formats(),
        });
    }
    plan::execute(runner, formats, &|_| true)
}

/// Builds the release packages of `languages` that an app consumes, each
/// into its directory under `packages` in cargo's target directory, after
/// the host library of eranos-uniffi and the sources generated from it when
/// a package over it is among them; the workspace's Rust has no package of
/// its own.
///
/// # Errors
///
/// As [`prepared`] fails, and the failure of the first action that fails.
pub(crate) fn package(runner: &mut impl Runner, languages: &[Language]) -> Result<(), Failure> {
    let languages = runnable(runner, languages)?;
    let metadata = Metadata::parse(&runner.output(&gates::metadata())?)?;
    let mut releases = vec![uniffi(&metadata, &languages)];
    for language in languages {
        releases.push(match language {
            Language::Capi => capi::release(&metadata),
            Language::Java => java::release(runner, &metadata)?,
            Language::Kmp => kmp::release(runner, &metadata)?,
            Language::Kotlin => kotlin::release(runner, &metadata)?,
            Language::Python => python::release(&metadata),
            Language::Rust => Vec::new(),
            Language::Swift => swift::release(&metadata),
            Language::Web => web::release(&metadata),
        });
    }
    for actions in releases {
        plan::execute(runner, actions, &|_| true)?;
    }
    Ok(())
}

/// Runs the example of eranos called `example`, or every one, and its ports
/// in each of `languages`, each language's package built first, comparing
/// every port's report with its example's byte for byte, as the gates do.
///
/// # Errors
///
/// As [`prepared`] fails, [`Failure::Example`] for an `example` eranos does
/// not have, naming the ones it does, and the failure of the first example,
/// action, or port that fails, [`Failure::Port`] for a report that differs.
pub(crate) fn examples(
    runner: &mut impl Runner,
    example: Option<&str>,
    languages: &[Language],
) -> Result<(), Failure> {
    let Prepared {
        actions: prepared,
        metadata,
    } = prepared(runner, languages)?;
    let every: Vec<gates::Example> = gates::examples(&metadata)
        .into_iter()
        .filter(|found| "eranos" == found.package)
        .collect();
    let known = every
        .iter()
        .map(|found| found.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let selected: Vec<gates::Example> = every
        .into_iter()
        .filter(|found| example.is_none_or(|name| name == found.name))
        .collect();
    if selected.is_empty() {
        return Err(Failure::Example {
            example: example.unwrap_or_default().to_owned(),
            known,
        });
    }
    let mut references = Vec::new();
    for found in selected {
        references.push(Reference {
            report: runner.output(&found.step)?,
            example: found.name,
        });
    }
    for actions in prepared {
        plan::execute(runner, actions, &|phase| Phase::Prepare == phase)?;
    }
    ports::run(runner, &metadata, &references, languages)?;
    println!("\nevery port printed its example's report");
    Ok(())
}

/// Runs, for each of `languages`, its Prepare actions and those of `phase`.
///
/// # Errors
///
/// As [`prepared`] fails, and the failure of the first action that fails.
fn run(runner: &mut impl Runner, languages: &[Language], phase: Phase) -> Result<(), Failure> {
    for actions in prepared(runner, languages)?.actions {
        plan::execute(runner, actions, &|found| {
            Phase::Prepare == found || phase == found
        })?;
    }
    Ok(())
}

/// Cargo's metadata, and the actions of each of the languages a command
/// names.
struct Prepared {
    /// The actions that build the host library of eranos-uniffi and its
    /// sources, none when no language reads them, then each language's.
    actions: Vec<Vec<Action>>,
    /// Cargo's metadata.
    metadata: Metadata,
}

/// Cargo's metadata, and the actions of each of `languages` that the
/// machine builds, as the gates describe them, after the host library of
/// eranos-uniffi and the sources generated from it when a package over it
/// is among them. Every variable a package needs is read here, so a missing
/// one stops the command before any of its actions runs.
///
/// # Errors
///
/// [`Failure::Apple`] for the Swift package alone on a machine that does not
/// build for Apple platforms, [`Failure::Metadata`] when cargo's metadata
/// cannot be read, the failure of the step that prints it, and
/// [`Failure::Variable`] for a variable a package needs that is not set.
fn prepared(runner: &mut impl Runner, languages: &[Language]) -> Result<Prepared, Failure> {
    let languages = runnable(runner, languages)?;
    let metadata = Metadata::parse(&runner.output(&gates::metadata())?)?;
    let mut prepared = vec![uniffi(&metadata, &languages)];
    for language in languages {
        prepared.push(match language {
            Language::Capi => capi::actions(runner, &metadata)?,
            Language::Java => java::actions(runner, &metadata)?,
            Language::Kmp => kmp::actions(runner)?,
            Language::Kotlin => kotlin::actions(runner, &metadata)?,
            Language::Python => python::actions(&metadata),
            Language::Rust => gates::actions(),
            Language::Swift => swift::actions(&metadata),
            Language::Web => web::actions(),
        });
    }
    Ok(Prepared {
        actions: prepared,
        metadata,
    })
}

/// The actions that build the host library of eranos-uniffi and generate
/// the sources from it into a cleared directory, when a package over it is
/// among `languages`, and none otherwise: a release library's sources are
/// its debug library's, since both have one interface.
fn uniffi(metadata: &Metadata, languages: &[Language]) -> Vec<Action> {
    if !languages.iter().any(|language| language.reads_uniffi()) {
        return Vec::new();
    }
    let mut actions = vec![
        Action::run(Phase::Prepare, gates::host_library()),
        Action::clear(Phase::Prepare, gates::generated(metadata)),
    ];
    actions.extend(Action::runs(Phase::Prepare, gates::generators(metadata)));
    actions
}

/// The languages of `languages` whose packages the machine builds: the
/// Swift one only where it builds for Apple platforms, as the gates do.
///
/// # Errors
///
/// [`Failure::Apple`] when `languages` is the Swift package alone and the
/// machine does not build for Apple platforms, which would run nothing.
fn runnable(runner: &impl Runner, languages: &[Language]) -> Result<Vec<Language>, Failure> {
    let apple = runner.builds_apple_platforms();
    if !apple && [Language::Swift] == languages {
        return Err(Failure::Apple);
    }
    Ok(ports::runnable(languages, apple))
}

#[cfg(test)]
mod tests;
