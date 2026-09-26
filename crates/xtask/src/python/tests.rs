//! The Python package's gates: the generated module and the link to the host
//! library in the package, the locked environment, and the package's checks
//! and documentation, in that order; and the copy in which the coverage runs
//! the tests.

use super::*;
use crate::plan;
use crate::tests::{METADATA, Recorder};

/// The package would check a stale module, load another library than the
/// one the module was generated from, run tools the lock file does not pin,
/// or keep pages of names it no longer has, if its steps ran in another
/// order, linked another file, or cleared another directory; the commands
/// pin every argument.
#[test]
fn the_package_is_prepared_checked_tested_and_documented_in_order() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let mut recorder = Recorder::new();
    plan::execute(&mut recorder, actions(&metadata), &|_| true).unwrap();
    let mut expected = vec![module(&metadata), library(&metadata), environment()];
    expected.extend(plan::steps(&checks()));
    assert_eq!(recorder.steps, expected);
    assert_eq!(recorder.cleared, [documentation()]);
    assert_eq!(
        documentation(),
        Path::new(WORKSPACE).join("bindings/python/build")
    );
    let lines: Vec<String> = expected.iter().map(Step::command_line).collect();
    let project = "cd bindings/python && uv run --locked";
    let library = format!("{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}");
    assert_eq!(
        lines,
        [
            format!(
                "cp /eranos/target/uniffi/python/eranos.py {WORKSPACE}/bindings/python/src/eranos/_internal"
            ),
            format!(
                "ln -sf /eranos/target/debug/{library} {WORKSPACE}/bindings/python/src/eranos/_internal/{library}"
            ),
            "cd bindings/python && uv sync --locked".to_owned(),
            format!("{project} ruff format --check"),
            format!("{project} ruff check --no-fix"),
            format!("{project} ty check"),
            format!("{project} coverage run -m pytest"),
            format!("{project} coverage report"),
            format!("{project} sphinx-build -W --keep-going -n -q -b html docs build/docs"),
        ]
    );
}

/// Each step belongs to the phase a command for one kind of check runs: the
/// module, the link, and the environment prepare, and the rest lint, test,
/// or document, so a command that ran only the tests would run neither the
/// linters nor Sphinx.
#[test]
fn every_step_is_in_the_phase_it_checks() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let phases: Vec<Phase> = actions(&metadata)
        .iter()
        .map(|action| action.phase)
        .collect();
    assert_eq!(
        phases,
        [
            Phase::Prepare,
            Phase::Prepare,
            Phase::Prepare,
            Phase::Prepare,
            Phase::Lint,
            Phase::Lint,
            Phase::Lint,
            Phase::Test,
            Phase::Test,
            Phase::Docs,
        ]
    );
}

/// The coverage would measure the gates' library, or run a module of another
/// build, if the copy took another module, linked another library, or kept
/// the files of an earlier copy; its steps pin every argument, and it clears
/// the copy's layer and tests but not its environment.
#[test]
fn the_copy_runs_the_tests_on_the_library_beside_its_module() {
    let metadata = Metadata::parse(METADATA).unwrap().inner("measured");
    let package = "/eranos/target/measured/python";
    assert_eq!(copied_package(&metadata), Path::new(package));
    assert_eq!(
        copied_directories(&metadata),
        [
            Path::new(package).join("src"),
            Path::new(package).join("tests")
        ]
    );
    let library = format!("{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}");
    let lines: Vec<String> = copy(&metadata)
        .iter()
        .chain([&copy_tests(&metadata)])
        .map(Step::command_line)
        .collect();
    assert_eq!(
        lines,
        [
            format!("mkdir -p {package}"),
            format!(
                "cp -R bindings/python/.python-version bindings/python/pyproject.toml bindings/python/src bindings/python/tests bindings/python/uv.lock {package}"
            ),
            format!(
                "cp /eranos/target/measured/uniffi/python/eranos.py {package}/src/eranos/_internal"
            ),
            format!(
                "ln -sf /eranos/target/measured/debug/{library} {package}/src/eranos/_internal/{library}"
            ),
            format!("cd {package} && uv sync --locked"),
            format!("cd {package} && uv run --locked pytest"),
        ]
    );
}
