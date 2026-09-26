//! The ports of the examples: built on a glue whose identifiers count up,
//! each run where its language builds, and each report compared byte for
//! byte with its Rust example's, the first line that differs named.

use std::fs;
use std::path::Path;

use super::*;
use crate::process::WORKSPACE;
use crate::tests::{FileOperation, METADATA, Recorder};

/// The metadata of the target directory of the glue whose identifiers count
/// up, in [`METADATA`]'s.
fn sequential() -> Metadata {
    Metadata::parse(METADATA).unwrap().inner(DIRECTORY)
}

/// A reference for `example` whose report is `report`.
fn reference(example: &str, report: &str) -> Reference {
    Reference {
        example: example.to_owned(),
        report: report.to_owned(),
    }
}

/// A port that ran on the gates' glue would print identifiers the Rust
/// example does not, and one that ran before its library was built would
/// print nothing; the glue whose identifiers count up is built first, its
/// sources generated from it, and the Swift ports run on a copy of the
/// package over it, only where the machine builds for Apple platforms.
#[test]
fn the_ports_run_on_a_glue_whose_identifiers_count_up() {
    let references = [
        reference("quickstart", "report"),
        reference("persistence", "saved"),
    ];
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        let answered = Metadata::parse(METADATA).unwrap();
        recorder.metadata = "report".to_owned();
        let failure = run(&mut recorder, &answered, &references);
        let mut expected = vec![library(&sequential())];
        expected.extend(gates::generators(&sequential()));
        if apple {
            expected.extend(swift::copy(&sequential()));
            expected.extend(swift::examples(&sequential()));
            expected.push(swift::examples_directory(&sequential()));
            expected.push(port("Swift", "quickstart", "/programs"));
            expected.push(port("Swift", "persistence", "/programs"));
            assert!(
                matches!(&failure, Err(Failure::Port { example, .. }) if "persistence" == example),
                "{failure:?}"
            );
        } else {
            assert!(failure.is_ok());
        }
        assert_eq!(recorder.steps, expected, "{apple}");
        assert_eq!(recorder.cleared, cleared(&sequential(), apple), "{apple}");
    }
    assert_eq!(
        library(&sequential()).command_line(),
        "CARGO_TARGET_DIR='/eranos/target/sequential' cargo build --package eranos-uniffi --features sequential-ids"
    );
}

/// A report that differs anywhere, its end included, would pass for the
/// Rust example's otherwise; the failure names the first line that
/// differs and both versions of it.
#[test]
fn a_report_passes_only_byte_for_byte() {
    let example = reference("quickstart", "Weekend\n══\ntotal\n");
    assert!(compare("Swift", &example, "Weekend\n══\ntotal\n").is_ok());
    for (report, line, expected, found) in [
        ("Weekend\n══\nTotal\n", 3, "\"total\"", "\"Total\""),
        ("Weekend\n══\ntotal", 4, "\"\"", "the end of the report"),
        ("Weekend\n══\ntotal\n\n", 5, "the end of the report", "\"\""),
    ] {
        let failure = compare("Swift", &example, report).unwrap_err();
        assert_eq!(
            failure.to_string(),
            format!(
                "the Swift port of quickstart prints {found} at line {line}, where the Rust example prints {expected}"
            ),
            "{report:?}"
        );
    }
}

/// A failure stops the ports there, so no port runs on a library that was
/// not built or on sources that were not generated, and no report is
/// compared with another example's: the library, a generator, the copy or
/// the build of the Swift examples, the programs' directory that could not
/// be printed, a port that fails, or a directory that could not be cleared.
#[test]
fn a_failure_stops_the_ports_where_it_happens() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let references = [reference("quickstart", "report")];
    for failing in [
        library(&sequential()),
        gates::generators(&sequential())[0].clone(),
        swift::copy(&sequential())[0].clone(),
        swift::examples(&sequential())[2].clone(),
        swift::examples_directory(&sequential()),
        port("Swift", "quickstart", "/programs"),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(
            run(&mut recorder, &metadata, &references),
            Err(Failure::Usage)
        ));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
    let mut blocked = Recorder::new();
    blocked.failing_files = Some(FileOperation::Clear);
    assert!(matches!(
        run(&mut blocked, &metadata, &[]),
        Err(Failure::Usage)
    ));
    assert_eq!(blocked.steps, [library(&sequential())]);
}

/// An example without a port would print its report unchecked in that
/// language, and a port without an example would check nothing; the Swift
/// examples' package has a program for every Rust example of eranos and for
/// no other.
#[test]
fn every_example_has_a_port_in_every_language() {
    let names = |directory: &str, keep: fn(&Path) -> Option<String>| {
        let mut names: Vec<String> = fs::read_dir(Path::new(WORKSPACE).join(directory))
            .unwrap()
            .filter_map(|entry| keep(&entry.unwrap().path()))
            .collect();
        names.sort();
        names
    };
    let examples = names("crates/eranos/examples", |path| {
        path.file_stem()
            .filter(|_| path.extension().is_some_and(|extension| "rs" == extension))
            .map(|stem| stem.to_string_lossy().into_owned())
    });
    let swift = names("bindings/swift/examples/Sources", |path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| "Report" != name)
    });
    assert_eq!(swift, examples);
    assert_eq!(8, examples.len());
}
