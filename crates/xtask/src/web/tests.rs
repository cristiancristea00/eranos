//! The TypeScript package's gates: the WebAssembly library, the generated
//! TypeScript in the package, and the package's checks, documentation, and
//! bundle, in that order, stopping at the first that fails, and none of them
//! on another Node than 24.

use super::*;
use crate::plan;
use crate::process::Runner;
use crate::tests::{FileOperation, Recorder};

/// Runs the package's gates with `runner`, as the gates run them.
fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    plan::execute(runner, actions(), &|_| true)
}

/// The package would check stale generated TypeScript, or pack a declaration
/// of a module it no longer has, if the steps ran in another order or
/// cleared another directory.
#[test]
fn the_package_is_generated_checked_documented_and_bundled_in_order() {
    let mut recorder = Recorder::new();
    run(&mut recorder).unwrap();
    let mut expected = vec![node()];
    expected.extend(library());
    expected.push(install());
    expected.push(generator());
    expected.extend(plan::steps(&checks()));
    expected.extend(plan::steps(&package()));
    assert_eq!(recorder.steps, expected);
    assert_eq!(recorder.cleared, [generated_sources(), bundle()]);
    assert_eq!(
        generated_sources(),
        Path::new(WORKSPACE).join("bindings/typescript/src/internal")
    );
    assert_eq!(
        bundle(),
        Path::new(WORKSPACE).join("bindings/typescript/dist")
    );
    let lines: Vec<String> = expected.iter().map(Step::command_line).collect();
    let project = "cd bindings/typescript &&";
    assert_eq!(
        lines,
        [
            "node --version".to_owned(),
            "rustup target add wasm32-unknown-unknown".to_owned(),
            "cargo build --package eranos-web --target wasm32-unknown-unknown --lib".to_owned(),
            format!("{project} npm ci"),
            format!(
                "{project} uniffi-bindgen-react-native build wasm2 --config ubrn.config.yaml --no-wasm-build"
            ),
            format!("{project} npm run format:check"),
            format!("{project} npm run lint"),
            format!("{project} npm run typecheck"),
            format!("{project} npm run coverage"),
            format!("{project} npm run docs"),
            format!("{project} npm run build"),
        ]
    );
}

/// On a later Node the tests would pass with a call Node 24 lacks, and on an
/// earlier one they would fail for a reason the package does not have; the
/// gates stop first instead, naming the version they found.
#[test]
fn another_node_stops_the_web_gates_before_any_other_step() {
    for version in ["v26.1.0\n", "v22.19.0\n", "v2.4.0\n"] {
        let mut recorder = Recorder::new();
        recorder.node = version.to_owned();
        let failure = run(&mut recorder).unwrap_err();
        assert!(
            matches!(&failure, Failure::Node(found) if found == version.trim()),
            "{version}"
        );
        assert_eq!(recorder.steps, [node()]);
        assert_eq!(recorder.cleared, Vec::<PathBuf>::new());
    }
}

/// A failure at any point stops the gates there: old generated sources that
/// could not be cleared, or a step that fails.
#[test]
fn a_failure_stops_the_web_gates_where_it_happens() {
    let mut unclear = Recorder::new();
    unclear.failing_files = Some(FileOperation::Clear);
    assert!(matches!(run(&mut unclear), Err(Failure::Usage)));
    assert_eq!(unclear.steps, [node()]);
    assert_eq!(unclear.cleared, [generated_sources()]);

    for failing in [
        node(),
        library()[0].clone(),
        install(),
        generator(),
        plan::steps(&checks())[0].clone(),
        plan::steps(&package())[0].clone(),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
