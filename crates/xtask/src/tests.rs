//! The command line: each command runs what it names, and anything else is a
//! usage error that lists the commands.

use std::path::{Path, PathBuf};

use super::*;
use crate::gates::Metadata;
use crate::process::{Step, WORKSPACE};

/// Two packages as `cargo metadata --no-deps --format-version 1` prints
/// them: one with a library, a test, and two examples, one with a binary.
pub(crate) const METADATA: &str = r#"{
    "packages": [
        {
            "name": "eranos",
            "targets": [
                {"kind": ["lib"], "name": "eranos"},
                {"kind": ["example"], "name": "quickstart"},
                {"kind": ["test"], "name": "scenarios"},
                {"kind": ["example"], "name": "persistence"}
            ]
        },
        {"name": "xtask", "targets": [{"kind": ["bin"], "name": "xtask"}]}
    ],
    "target_directory": "/eranos/target",
    "version": 1
}"#;

/// A coverage summary as `llvm-cov export -summary-only` prints it, of a
/// program whose tests ran every line and every function.
pub(crate) const COVERED: &str = r#"{
    "data": [{"files": [], "totals": {"functions": {"count": 2, "covered": 2}, "lines": {"count": 3, "covered": 3}}}],
    "type": "llvm.coverage.json.export",
    "version": "3.1.0"
}"#;

/// What `cargo llvm-cov show-env --sh` prints, cut to two variables: where
/// a program writes its coverage, and the wrapper that instruments a build.
pub(crate) const EXPORTS: &str =
    "export LLVM_PROFILE_FILE='/eranos/target/llvm-cov-target/eranos-%p-%14m.profraw'
export RUSTC_WRAPPER=/cargo-llvm-cov
";

/// A runner that records every step instead of running it, answers the
/// step that prints async-lock's features with `tree`, the one that prints
/// Node's version with `node`, those that print the C and C++ tests'
/// coverage with `coverage`, the one that prints cargo-llvm-cov's
/// environment with `exports`, and every other output with `metadata`,
/// records every directory it clears or creates and every file it writes,
/// builds for Apple platforms when `apple`, has set only the environment
/// variables `variables`, and fails at the step `failing` or at every file
/// operation of the kind `failing_files`.
#[derive(Debug, Default)]
pub(crate) struct Recorder {
    pub(crate) apple: bool,
    pub(crate) cleared: Vec<PathBuf>,
    pub(crate) coverage: String,
    pub(crate) created: Vec<PathBuf>,
    pub(crate) exports: String,
    pub(crate) metadata: String,
    pub(crate) node: String,
    pub(crate) steps: Vec<Step>,
    pub(crate) tree: String,
    pub(crate) variables: Vec<&'static str>,
    pub(crate) written: Vec<PathBuf>,
    pub(crate) failing: Option<Step>,
    pub(crate) failing_files: Option<FileOperation>,
}

/// A kind of file operation a runner does besides running steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileOperation {
    /// Clearing a directory.
    Clear,
    /// Creating a directory.
    Create,
    /// Writing a file.
    Write,
}

impl Recorder {
    /// A recorder that answers with [`METADATA`], a tree without features,
    /// Node 24, and [`COVERED`], builds for Apple platforms, has set every
    /// variable the Kotlin, Kotlin Multiplatform, and C and C++ gates need,
    /// and fails at no step.
    pub(crate) fn new() -> Self {
        Self {
            apple: true,
            coverage: COVERED.to_owned(),
            exports: EXPORTS.to_owned(),
            metadata: METADATA.to_owned(),
            node: "v24.21.0\n".to_owned(),
            variables: [
                kotlin::VARIABLES.as_slice(),
                kmp::VARIABLES.as_slice(),
                &[capi::VARIABLE],
            ]
            .concat(),
            ..Self::default()
        }
    }

    /// Records `step`, and fails if it is the failing one.
    fn record(&mut self, step: &Step) -> Result<(), Failure> {
        self.steps.push(step.clone());
        if self.failing.as_ref() == Some(step) {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }
}

impl Runner for Recorder {
    fn run(&mut self, step: &Step) -> Result<(), Failure> {
        self.record(step)
    }

    fn output(&mut self, step: &Step) -> Result<String, Failure> {
        self.record(step)?;
        if *step == gates::lock_features() {
            Ok(self.tree.clone())
        } else if *step == web::node() {
            Ok(self.node.clone())
        } else if [capi::C_PACKAGE, capi::CPP_PACKAGE]
            .iter()
            .any(|&package| *step == capi::coverage(package, "/LLVM_HOME"))
        {
            Ok(self.coverage.clone())
        } else if step
            .command_line()
            .ends_with("cargo llvm-cov show-env --sh")
        {
            Ok(self.exports.clone())
        } else {
            Ok(self.metadata.clone())
        }
    }

    fn clear(&mut self, directory: &Path) -> Result<(), Failure> {
        self.cleared.push(directory.to_owned());
        if Some(FileOperation::Clear) == self.failing_files {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }

    fn create(&mut self, directory: &Path) -> Result<(), Failure> {
        self.created.push(directory.to_owned());
        if Some(FileOperation::Create) == self.failing_files {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }

    fn write(&mut self, path: &Path, _contents: &str) -> Result<(), Failure> {
        self.written.push(path.to_owned());
        if Some(FileOperation::Write) == self.failing_files {
            Err(Failure::Usage)
        } else {
            Ok(())
        }
    }

    fn builds_apple_platforms(&self) -> bool {
        self.apple
    }

    fn variable(&self, name: &str) -> Option<String> {
        self.variables.contains(&name).then(|| format!("/{name}"))
    }
}

fn words(line: &[&str]) -> Vec<String> {
    line.iter().map(|&word| word.to_owned()).collect()
}

/// `gates` runs every check, reads async-lock's features, lists the
/// examples, runs each one, builds the host library, generates every
/// language's sources into a cleared directory, runs the Swift package's
/// gates where the machine builds for Apple platforms, and the Kotlin,
/// Java, TypeScript, Kotlin Multiplatform, C, and C++ packages', the Kotlin
/// Multiplatform package with its Apple libraries only where the machine
/// builds them; if it ran less, `cargo xtask gates` would pass commits the
/// documented gates refuse.
#[test]
fn gates_runs_every_check_every_example_and_every_generator() {
    let mut recorder = Recorder::new();
    run(&words(&["gates"]), &mut recorder).unwrap();

    let metadata = Metadata::parse(METADATA).unwrap();
    let mut expected = gates::checks();
    expected.push(gates::lock_features());
    expected.push(gates::metadata());
    expected.extend(gates::examples(&metadata));
    expected.push(gates::host_library());
    expected.extend(gates::generators(&metadata));
    let mut packages = kotlin::libraries();
    packages.extend(kotlin::sources(&metadata));
    packages.extend(kotlin::checks(&metadata));
    packages.extend(java::sources(&metadata));
    packages.extend(java::checks(&metadata));
    packages.push(web::node());
    packages.extend(web::library());
    packages.push(web::install());
    packages.push(web::generator());
    packages.extend(web::checks());
    packages.extend(web::package());
    let mut elsewhere_packages = packages.clone();
    packages.extend(kmp::checks("/JAVA_HOME_21", true));
    elsewhere_packages.extend(kmp::checks("/JAVA_HOME_21", false));
    let mut native = vec![capi::library()];
    for package in [capi::C_PACKAGE, capi::CPP_PACKAGE] {
        native.push(capi::headers(package));
        native.extend(capi::tests(package, "/LLVM_HOME", &metadata));
        native.push(capi::coverage(package, "/LLVM_HOME"));
        native.extend(capi::checks(package, "/LLVM_HOME"));
    }
    packages.extend(native.clone());
    elsewhere_packages.extend(native);
    let native_cleared = [capi::C_PACKAGE, capi::CPP_PACKAGE]
        .map(|package| {
            [
                capi::directory(package, "generated"),
                capi::directory(package, "build"),
            ]
        })
        .concat();
    let native_created =
        [capi::C_PACKAGE, capi::CPP_PACKAGE].map(|package| capi::directory(package, "build"));
    let cleared = gates::generated(&metadata);
    let mut elsewhere = Recorder::new();
    elsewhere.apple = false;
    run(&words(&["gates"]), &mut elsewhere).unwrap();
    assert_eq!(
        elsewhere.steps,
        [expected.clone(), elsewhere_packages].concat()
    );
    assert_eq!(
        elsewhere.cleared,
        [
            cleared.clone(),
            kotlin::generated_sources(),
            java::generated_sources(),
            web::generated_sources(),
            web::bundle(),
        ]
        .into_iter()
        .chain(native_cleared.clone())
        .collect::<Vec<_>>()
    );
    assert_eq!(elsewhere.created, native_created);

    expected.extend(swift::libraries(&metadata));
    expected.extend(swift::package(&metadata));
    expected.push(swift::build());
    expected.extend(swift::checks(&metadata));
    expected.extend(packages);
    assert_eq!(recorder.steps, expected);
    assert_eq!(
        recorder.cleared,
        [
            cleared,
            Path::new(WORKSPACE).join("bindings/swift/EranosFFI.xcframework"),
            kotlin::generated_sources(),
            java::generated_sources(),
            web::generated_sources(),
            web::bundle(),
        ]
        .into_iter()
        .chain(native_cleared)
        .collect::<Vec<_>>()
    );
    assert_eq!(recorder.created, native_created);
}

/// `coverage` measures the Rust tests and the glue, and ends with the
/// report; if it ran less, the report would read the glue as unrun.
#[test]
fn coverage_measures_everything_and_ends_with_the_report() {
    let mut recorder = Recorder::new();
    run(&words(&["coverage"]), &mut recorder).unwrap();
    assert_eq!(recorder.steps.first(), Some(&gates::metadata()));
    assert_eq!(recorder.steps.last(), Some(&coverage::report()));
}

/// `help` succeeds and runs nothing.
#[test]
fn help_runs_nothing_and_succeeds() {
    let mut recorder = Recorder::new();
    run(&words(&["help"]), &mut recorder).unwrap();
    assert!(recorder.steps.is_empty());
}

/// No command, an unknown one, or a known one with more words is a usage
/// error that runs nothing, so a mistyped gate run never passes for one.
#[test]
fn anything_but_one_known_command_is_a_usage_error() {
    for line in [&[][..], &["gate"], &["gates", "--rust"], &["help", "gates"]] {
        let mut recorder = Recorder::new();
        let failure = run(&words(line), &mut recorder).unwrap_err();
        assert!(matches!(failure, Failure::Usage), "{line:?}");
        assert!(recorder.steps.is_empty(), "{line:?}");
    }
}

/// The usage error lists every command, so the fix is on screen.
#[test]
fn a_usage_error_lists_every_command() {
    let message = Failure::Usage.to_string();
    assert!(message.starts_with("unknown command\n\nusage: cargo xtask <command>"));
    for command in ["coverage", "gates", "help"] {
        assert!(message.contains(&format!("\n  {command} ")), "{command}");
    }
}

/// A missing variable names itself and where the setup is described, so
/// the fix is on screen.
#[test]
fn a_missing_variable_names_itself_and_the_setup() {
    assert_eq!(
        Failure::Variable("ANDROID_HOME").to_string(),
        "ANDROID_HOME is not set; docs/DEVELOPMENT.md § 1 says where it points"
    );
}

/// Another Node names the version found, the one the gates need, and where
/// the setup is described, so the fix is on screen.
#[test]
fn another_node_names_itself_and_the_setup() {
    assert_eq!(
        Failure::Node("v26.1.0".to_owned()).to_string(),
        "the gates run on Node 24, the oldest the package supports, and found v26.1.0; docs/DEVELOPMENT.md § 1 says how to put it first on PATH"
    );
}

/// A metadata failure says where the JSON came from and why it was refused.
#[test]
fn a_metadata_failure_names_cargo_metadata_and_the_reason() {
    let reason = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
    let message = Failure::Metadata(reason).to_string();
    assert!(
        message.starts_with("cargo metadata printed JSON the gates cannot read: EOF"),
        "{message}"
    );
}
