//! The doctor: every tool the gates run is checked, each missing one is
//! reported with the entry of Brewfile.develop that installs it, and the
//! doctor fails when one is missing, after checking every other.

use super::*;
use crate::gates::{self, Part};
use crate::process::WORKSPACE;
use crate::tests::Recorder;

/// Brewfile.develop, whose entries install the tools.
const BREWFILE: &str = include_str!("../../../../Brewfile.develop");

/// What the recorder answers `cargo install --list`, and every other probe,
/// with: both tools that print no version of their own.
const INSTALLED_TOOLS: &str = "diplomat-tool v0.16.1 (https://github.com/rust-diplomat/diplomat?tag=v0.16.1#125df445):
    diplomat-tool
uniffi-bindgen-react-native v0.31.0-5 (https://github.com/jhugman/uniffi-bindgen-react-native?tag=0.31.0-5#0f7fc67e):
    uniffi-bindgen-react-native
";

/// A recorder whose probes find every tool, and whose `node` is Node 24.
fn machine() -> Recorder {
    let mut recorder = Recorder::new();
    recorder.metadata = INSTALLED_TOOLS.to_owned();
    recorder
}

/// The steps the doctor runs on a machine that builds for Apple platforms
/// when `apple`, in order.
fn checks(apple: bool) -> Vec<Step> {
    let tools = if apple {
        [TOOLS.as_slice(), APPLE_TOOLS.as_slice()].concat()
    } else {
        TOOLS.to_vec()
    };
    let mut steps: Vec<Step> = tools
        .iter()
        .map(|tool| step(tool.name, tool.program, tool.arguments))
        .collect();
    steps.push(installed());
    steps.push(web::node());
    steps.extend(DIRECTORIES.iter().map(|directory| {
        step(
            directory.variable,
            &format!("/{}/{}", directory.variable, directory.program),
            &["--version"],
        )
    }));
    steps.push(python());
    steps
}

/// A doctor that skipped a tool would pass a machine the gates then fail on:
/// it checks every tool, the Swift ones only where the machine builds for
/// Apple platforms, the tools cargo installed through one list, Node, the
/// program under each directory a variable names, and the package's Python,
/// and passes when it finds them all.
#[test]
fn a_machine_with_every_tool_passes() {
    for apple in [true, false] {
        let mut recorder = machine();
        recorder.apple = apple;
        run(&mut recorder).unwrap();
        assert_eq!(recorder.steps, checks(apple), "{apple}");
    }
}

/// A doctor that stopped at the first missing tool would leave the others
/// for the next run, and one that passed would let the gates fail mid-run:
/// each missing tool, a Node of another release, an unset variable, and a
/// variable whose directory holds no program count as missing, the tools
/// cargo installed each when its list cannot be read, and the doctor fails
/// with their number once it has checked every tool.
#[test]
fn every_missing_tool_is_counted_after_every_tool_is_checked() {
    for failing in checks(true) {
        let mut recorder = machine();
        recorder.failing = Some(failing.clone());
        let failure = run(&mut recorder);
        // Without cargo's list, every tool that only it names is missing.
        let count = if installed() == failing {
            INSTALLED.len()
        } else {
            1
        };
        assert!(
            matches!(failure, Err(Failure::Doctor(missing)) if count == missing),
            "{failing:?} {failure:?}"
        );
        assert_eq!(recorder.steps, checks(true), "{failing:?}");
    }
    let mut another_node = machine();
    another_node.node = "v26.1.0\n".to_owned();
    assert!(matches!(run(&mut another_node), Err(Failure::Doctor(1))));
    let mut unset = machine();
    unset.variables.clear();
    assert!(matches!(
        run(&mut unset),
        Err(Failure::Doctor(missing)) if DIRECTORIES.len() == missing
    ));
    let mut uninstalled = machine();
    uninstalled.metadata = "{}".to_owned();
    assert!(matches!(
        run(&mut uninstalled),
        Err(Failure::Doctor(missing)) if INSTALLED.len() == missing
    ));
    assert_eq!(
        Failure::Doctor(2).to_string(),
        "2 of the tools the gates run are missing or of another release; the lines above say how to install each"
    );
}

/// Advice that named an entry Brewfile.develop does not hold would send the
/// reader looking for a line that is not there: every entry the doctor names
/// starts a line of it, and the advice says how to install from it.
#[test]
fn every_entry_the_doctor_names_is_one_of_brewfile_develop() {
    let mut entries: Vec<&str> = [TOOLS.as_slice(), APPLE_TOOLS.as_slice()]
        .concat()
        .iter()
        .filter_map(|tool| match tool.source {
            Source::Brewfile(entry) => Some(entry),
            Source::Elsewhere(_) => None,
        })
        .collect();
    entries.extend(INSTALLED.iter().map(|(_, entry)| *entry));
    entries.extend(DIRECTORIES.iter().map(|directory| directory.entry));
    entries.extend(["brew \"node@24\"", "brew \"python@3.14\""]);
    for entry in entries {
        assert!(
            BREWFILE.lines().any(|line| line.starts_with(entry)),
            "{entry}"
        );
    }
    assert_eq!(
        missing_from(Source::Brewfile("brew \"uv\"")),
        "missing: `brew bundle --file Brewfile.develop` installs it, from its `brew \"uv\"`"
    );
    assert_eq!(
        missing_from(Source::Elsewhere("rustup.rs")),
        "missing: rustup.rs"
    );
}

/// A tool the gates run that the doctor did not check would pass a machine
/// that lacks it: every program a run of the gates starts is one the doctor
/// checks, a program under a directory it checks, one the system or a
/// checked tool brings, or one the gates built.
#[test]
fn every_program_the_gates_run_is_one_the_doctor_checks() {
    let mut recorder = Recorder::new();
    gates::run(&mut recorder, Part::Every).unwrap();
    let checked: Vec<&str> = [TOOLS.as_slice(), APPLE_TOOLS.as_slice()]
        .concat()
        .iter()
        .map(|tool| tool.program)
        .chain(INSTALLED.iter().map(|(name, _)| *name))
        .chain(["node"])
        .collect();
    // The system's, Xcode's beside xcodebuild, and Node's own npm.
    let brought = ["cp", "lipo", "ln", "mkdir", "npm", "sh"];
    let workspace = format!("{WORKSPACE}/");
    for step in &recorder.steps {
        let program = step.program();
        let under_directory = DIRECTORIES
            .iter()
            .any(|directory| program.starts_with(&format!("/{}/", directory.variable)));
        let built = program.starts_with(&workspace)
            || program.starts_with("/eranos/target/")
            || program.starts_with("/programs/");
        assert!(
            checked.contains(&program) || brought.contains(&program) || under_directory || built,
            "{program}"
        );
    }
}
