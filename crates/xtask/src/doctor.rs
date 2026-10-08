//! What the gates need of the machine: every tool they run, with the version
//! the machine has, and for one it lacks, or one of another release than the
//! gates run on, the line of Brewfile.develop that installs the right one.

use crate::Failure;
use crate::process::{Runner, Step};
use crate::{capi, kotlin, web};

/// Where a machine gets a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    /// The entry of Brewfile.develop that installs it, such as
    /// `brew "ktlint"`, the start of its line there.
    Brewfile(&'static str),
    /// Elsewhere, as the text says.
    Elsewhere(&'static str),
}

/// A tool the gates run, found by the step that prints its version.
#[derive(Debug, Clone, Copy)]
struct Tool {
    /// The tool, as the report names it.
    name: &'static str,
    /// The program that prints its version.
    program: &'static str,
    /// The arguments that make it print its version.
    arguments: &'static [&'static str],
    /// Where a machine gets it.
    source: Source,
}

/// The tools on `PATH` that every machine's gates run.
const TOOLS: [Tool; 7] = [
    Tool {
        name: "cargo",
        program: "cargo",
        arguments: &["--version"],
        source: Source::Elsewhere("rustup, from rustup.rs, installs it from rust-toolchain.toml"),
    },
    Tool {
        name: "rustup",
        program: "rustup",
        arguments: &["--version"],
        source: Source::Elsewhere("rustup.rs"),
    },
    Tool {
        name: "cargo-llvm-cov",
        program: "cargo",
        arguments: &["llvm-cov", "--version"],
        source: Source::Brewfile("brew \"cargo-llvm-cov\""),
    },
    Tool {
        name: "cargo-ndk",
        program: "cargo",
        arguments: &["ndk", "--version"],
        source: Source::Brewfile("cargo \"cargo-ndk\""),
    },
    Tool {
        name: "ktlint",
        program: "ktlint",
        arguments: &["--version"],
        source: Source::Brewfile("brew \"ktlint\""),
    },
    Tool {
        name: "doxygen",
        program: "doxygen",
        arguments: &["--version"],
        source: Source::Brewfile("brew \"doxygen\""),
    },
    Tool {
        name: "uv",
        program: "uv",
        arguments: &["--version"],
        source: Source::Brewfile("brew \"uv\""),
    },
];

/// The tools that only a machine that builds for Apple platforms runs.
const APPLE_TOOLS: [Tool; 3] = [
    Tool {
        name: "swift",
        program: "swift",
        arguments: &["--version"],
        source: Source::Elsewhere("Xcode, from the App Store"),
    },
    Tool {
        name: "xcodebuild",
        program: "xcodebuild",
        arguments: &["-version"],
        source: Source::Elsewhere("Xcode, from the App Store"),
    },
    Tool {
        name: "swiftlint",
        program: "swiftlint",
        arguments: &["version"],
        source: Source::Brewfile("brew \"swiftlint\""),
    },
];

/// A tool that cargo installed from the release tag Brewfile.develop names.
struct Installed {
    /// Its entry of Brewfile.develop.
    entry: &'static str,
    /// Its crate's name.
    name: &'static str,
}

/// The tools that cargo installed from the release tags Brewfile.develop
/// names, and that print no version of their own: `cargo install --list`
/// names each with its version.
const INSTALLED: [Installed; 3] = [
    Installed {
        entry: "cargo \"diplomat-tool\"",
        name: "diplomat-tool",
    },
    Installed {
        entry: "cargo \"uniffi-bindgen-react-native\"",
        name: "uniffi-bindgen-react-native",
    },
    Installed {
        entry: "cargo \"wasm-bindgen-cli\"",
        name: "wasm-bindgen-cli",
    },
];

/// A variable the gates read, the program under it that prints its
/// version, and the entry of Brewfile.develop that installs what it names.
struct Directory {
    /// The variable.
    variable: &'static str,
    /// The program under the directory it names, which prints the version.
    program: &'static str,
    /// The entry of Brewfile.develop that installs what it names.
    entry: &'static str,
}

/// The directories the gates read from variables.
const DIRECTORIES: [Directory; 3] = [
    Directory {
        variable: kotlin::JDK,
        program: "bin/java",
        entry: "brew \"openjdk@25\"",
    },
    Directory {
        variable: "ANDROID_HOME",
        program: "cmdline-tools/latest/bin/sdkmanager",
        entry: "cask \"android-commandlinetools\"",
    },
    Directory {
        variable: capi::VARIABLE,
        program: "bin/clang",
        entry: "brew \"llvm\"",
    },
];

/// Prints every tool the gates run with the version found, and how to
/// install each that is missing or of another release than the gates run
/// on.
///
/// # Errors
///
/// [`Failure::Doctor`] with the number of tools missing or of another
/// release, once every one is printed.
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    let mut missing = 0;
    let mut report = |name: &str, found: Result<String, String>| match found {
        Ok(version) => println!("{name:<28} {version}"),
        Err(fix) => {
            missing += 1;
            println!("{name:<28} {fix}");
        }
    };
    let tools = if runner.builds_apple_platforms() {
        [TOOLS.as_slice(), APPLE_TOOLS.as_slice()].concat()
    } else {
        TOOLS.to_vec()
    };
    for tool in tools {
        let found = version(runner, &step(tool.name, tool.program, tool.arguments))
            .ok_or_else(|| missing_from(tool.source));
        report(tool.name, found);
    }
    let installed = runner.probe(&installed()).unwrap_or_default();
    for Installed { entry, name } in INSTALLED {
        let found = installed
            .lines()
            .find(|line| line.starts_with(&format!("{name} v")))
            .map(|line| line.trim_end_matches(':').to_owned())
            .ok_or_else(|| missing_from(Source::Brewfile(entry)));
        report(name, found);
    }
    let node_entry = Source::Brewfile("brew \"node@24\"");
    let node = version(runner, &web::node()).ok_or_else(|| missing_from(node_entry));
    report(
        "node",
        node.and_then(|found| {
            if found.starts_with(web::NODE) {
                Ok(found)
            } else {
                Err(format!(
                    "{found} is first on PATH, and the gates run on Node 24: {}, then docs/DEVELOPMENT.md § 1",
                    install(node_entry)
                ))
            }
        }),
    );
    for directory in DIRECTORIES {
        let found = found_directory(runner, &directory);
        report(directory.variable, found);
    }
    let python = version(runner, &python())
        .ok_or_else(|| missing_from(Source::Brewfile("brew \"python@3.14\"")));
    report("Python", python);
    if 0 == missing {
        Ok(())
    } else {
        Err(Failure::Doctor(missing))
    }
}

/// What the doctor finds of `directory`: the variable's value and the
/// version of the program under the directory it names, or how to set it,
/// or to install what it names.
fn found_directory(runner: &mut impl Runner, directory: &Directory) -> Result<String, String> {
    let install = install(Source::Brewfile(directory.entry));
    let value = runner
        .variable(directory.variable)
        .ok_or_else(|| format!("not set: {install}, then docs/DEVELOPMENT.md § 1"))?;
    let program = format!("{value}/{}", directory.program);
    version(runner, &step(directory.variable, &program, &["--version"]))
        .map(|found| format!("{value}: {found}"))
        .ok_or_else(|| format!("{value} holds no {}: {install}", directory.program))
}

/// The step that runs `program` with `arguments`, reported as `name`.
fn step(name: &str, program: &str, arguments: &[&str]) -> Step {
    Step::new(arguments, &format!("the version of {name}"), program)
}

/// The step that lists the tools cargo installed, each with its version.
fn installed() -> Step {
    Step::cargo(&["install", "--list"], "the tools cargo installed")
}

/// The step that prints the version of the Python the Python package's
/// environment runs on, as uv picks it for the package.
fn python() -> Step {
    Step::new(
        &["run", "--locked", "python", "--version"],
        "the version of Python",
        "uv",
    )
    .in_directory(crate::python::PROJECT)
}

/// The first line `step` prints, or none when it cannot run or fails.
fn version(runner: &mut impl Runner, step: &Step) -> Option<String> {
    runner.probe(step).map(|output| {
        output
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or_default()
            .trim()
            .to_owned()
    })
}

/// What the report says of a missing tool that `source` installs.
fn missing_from(source: Source) -> String {
    format!("missing: {}", install(source))
}

/// How `source` installs a tool: `brew bundle` with Brewfile.develop, which
/// holds its entry, or where else it comes from.
fn install(source: Source) -> String {
    match source {
        Source::Brewfile(entry) => {
            format!("`brew bundle --file Brewfile.develop` installs it, from its `{entry}`")
        }
        Source::Elsewhere(text) => text.to_owned(),
    }
}

#[cfg(test)]
mod tests;
