//! Removing what the builds leave: cargo's target directory, every path the
//! repository's `.gitignore` files ignore but two that a developer's tools
//! write for their machine, and, when asked, the caches outside the
//! repository that the gates fill.
//!
//! Like the gates, it runs on macOS and Linux: Windows refuses to delete the
//! running `xtask.exe`, which `cargo clean` removes.

use crate::Failure;
use crate::process::{Runner, Step};
use crate::{python, swift, web};

/// What a clean does with the steps that remove.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Prints each step instead of running it, after cargo's own dry run.
    DryRun,
    /// Runs each step.
    Remove,
}

/// How far a clean reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    /// The repository: cargo's target directory and every ignored path.
    Repository,
    /// The repository, and the caches outside it that the gates fill.
    WithCaches,
}

/// Where the caches outside the repository that the gates fill live.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Caches {
    /// Cargo's home, which holds its registry and the git checkouts of the
    /// tools it installed.
    cargo: String,
    /// Gradle's home, which holds its daemons' registry, its caches, and the
    /// wrappers' distributions.
    gradle: String,
    /// Kotlin/Native's data directory, which holds only downloads.
    konan: String,
}

/// The names clean keeps wherever `.gitignore` ignores them: Finder's view
/// settings, and the SDK path Android Studio writes for its machine.
const KEPT: [&str; 2] = [".DS_Store", "local.properties"];

/// Cargo's target directory as the listing names it, which `cargo clean`
/// removes.
const TARGET: &str = "target/";

/// Removes what the builds leave in the repository, and with
/// [`Scope::WithCaches`] the caches the gates fill outside it; with
/// [`Mode::DryRun`], prints what it would remove instead.
///
/// # Errors
///
/// [`Failure::Variable`] naming `HOME` when a cache's root is named by
/// neither its tool's variable nor `HOME`, before any step runs, and the
/// failure of the first step that fails.
pub(crate) fn run(runner: &mut impl Runner, scope: Scope, mode: Mode) -> Result<(), Failure> {
    let caches = match scope {
        Scope::Repository => None,
        Scope::WithCaches => Some(caches(runner)?),
    };
    runner.run(&cargo_clean(mode))?;
    let listing = runner.output(&listing())?;
    for step in steps(
        &entries(&listing),
        caches.as_ref(),
        runner.builds_apple_platforms(),
    ) {
        match mode {
            Mode::DryRun => println!("would run: {}", step.command_line()),
            Mode::Remove => runner.run(&step)?,
        }
    }
    Ok(())
}

/// The step that removes cargo's target directory, wherever the workspace
/// or `CARGO_TARGET_DIR` puts it, or with [`Mode::DryRun`] the one that
/// prints what it would remove.
#[must_use]
pub(crate) fn cargo_clean(mode: Mode) -> Step {
    match mode {
        Mode::DryRun => Step::cargo(&["clean", "--dry-run"], "what cargo clean would remove"),
        Mode::Remove => Step::cargo(&["clean"], "cargo's target directory"),
    }
}

/// The step that lists every path the repository's `.gitignore` files
/// ignore, from the workspace root, each followed by NUL and unquoted, a
/// wholly ignored directory once, with `/` after it, but the [`KEPT`] ones.
/// Git names no directory that holds a tracked or an unignored file. Only
/// the repository's `.gitignore` files count: `--exclude-standard` would
/// add git's per-checkout and per-user ignore files, which name a
/// developer's own files, such as an editor's local settings. The command
/// line's patterns outrank `.gitignore`'s, so the negated ones keep what
/// they name.
#[must_use]
pub(crate) fn listing() -> Step {
    let kept: Vec<String> = KEPT
        .iter()
        .map(|name| format!("--exclude=!{name}"))
        .collect();
    let mut arguments = vec![
        "ls-files",
        "-z",
        "--others",
        "--ignored",
        "--directory",
        "--exclude-per-directory=.gitignore",
    ];
    arguments.extend(kept.iter().map(String::as_str));
    Step::new(&arguments, "the paths .gitignore ignores", "git")
}

/// The paths of `listing` to remove, in its order: every one but those
/// under [`TARGET`], which `cargo clean` removes, and those under a
/// directory it names before them, which go with that directory. Each path
/// ends in NUL, the last too, so splitting at every NUL would end in an
/// empty path, which is the workspace root; no path is ever empty.
fn entries(listing: &str) -> Vec<&str> {
    let mut entries: Vec<&str> = Vec::new();
    for entry in listing.split_terminator('\0') {
        let removed = entry.is_empty()
            || entry.starts_with(TARGET)
            || entries
                .iter()
                .any(|directory| directory.ends_with('/') && entry.starts_with(directory));
        if !removed {
            entries.push(entry);
        }
    }
    entries
}

/// Where each cache lives: the directory its tool's variable names, else
/// its directory under `HOME`.
///
/// # Errors
///
/// [`Failure::Variable`] naming `HOME` for a cache whose variable is unset
/// while `HOME` is too.
fn caches(runner: &impl Runner) -> Result<Caches, Failure> {
    let root = |variable: &str, under_home: &str| {
        runner
            .variable(variable)
            .or_else(|| {
                runner
                    .variable("HOME")
                    .map(|home| format!("{home}/{under_home}"))
            })
            .ok_or(Failure::Variable("HOME"))
    };
    Ok(Caches {
        cargo: root("CARGO_HOME", ".cargo")?,
        gradle: root("GRADLE_USER_HOME", ".gradle")?,
        konan: root("KONAN_DATA_DIR", ".konan")?,
    })
}

/// The steps that remove `entries`, each a path the listing names from the
/// workspace root, and with `caches` the caches the gates fill: each tool's
/// own cleaner first, so a missing tool stops the clean before any cache's
/// directory goes, the Swift package manager's only where the machine builds
/// for Apple platforms (`apple`), then the directories. Gradle's daemons' registry
/// goes before its caches: a client finds a daemon only through it, so no
/// build reaches a daemon whose caches are gone, and each idle one expires
/// once the registry is, where `gradlew --stop` would stop only its own
/// Gradle version's daemons and download a missing distribution first.
/// Cargo's `bin`, where it installed the tools Brewfile.develop names, and
/// Gradle's settings, such as `gradle.properties`, stay. `cargo clean gc` is
/// not used: on stable it removes only what has gone unused for months.
fn steps(entries: &[&str], caches: Option<&Caches>, apple: bool) -> Vec<Step> {
    let mut steps: Vec<Step> = entries.iter().map(|entry| removal(entry)).collect();
    if let Some(caches) = caches {
        steps
            .push(Step::new(&["cache", "clean"], "uv's cache", "uv").in_directory(python::PROJECT));
        steps.push(
            Step::new(&["cache", "clean", "--force"], "npm's cache", "npm")
                .in_directory(web::PROJECT),
        );
        if apple {
            steps.push(
                Step::new(&["package", "purge-cache"], "SwiftPM's cache", "swift")
                    .in_directory(swift::PACKAGE),
            );
        }
        for directory in [
            format!("{}/daemon", caches.gradle),
            format!("{}/caches", caches.gradle),
            format!("{}/wrapper/dists", caches.gradle),
            caches.konan.clone(),
            format!("{}/registry", caches.cargo),
            format!("{}/git", caches.cargo),
        ] {
            steps.push(removal(&directory));
        }
    }
    steps
}

/// The step that removes `path`: a file, a symbolic link but not what it
/// points to, or a directory with everything in it, which may be missing.
fn removal(path: &str) -> Step {
    Step::new(&["-rf", "--", path], &format!("{path}, removed"), "rm")
}

#[cfg(test)]
mod tests;
