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
/// print nothing; the glue whose identifiers count up is built first, with
/// eranos-kmp's, eranos-web's, and eranos-capi's libraries, its sources
/// generated from it, the Swift ports run on a copy of the package over it
/// only where the machine builds for Apple platforms, and the Kotlin ones on
/// every machine, on the JDK's `java`, JNA loading that library, then again
/// over gobley's bindings, on the JDK 21's, JNA loading eranos-kmp's, then
/// the Java ones, on the JDK's `java`, which loads that library itself, the
/// TypeScript ones, on Node, in a copy of the package over the WebAssembly
/// build of the glue, whose generator's settings the ports write, the C and
/// the C++ ones, each language's built into a directory of its own, then
/// run, and last the Python ones, in a copy of the package over that
/// library. The Kotlin and Java ports refuse native access the command
/// line does not grant, as the Java tests do; JDK 21's launcher has no such
/// option.
#[test]
fn the_ports_run_on_a_glue_whose_identifiers_count_up() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let references = [
        reference("quickstart", "report"),
        reference("persistence", "report"),
    ];
    for apple in [true, false] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        recorder.metadata = "report".to_owned();
        run(&mut recorder, &metadata, &references).unwrap();
        let mut expected = vec![
            library(&sequential()),
            multiplatform_library(&sequential()),
            web::sequential_library(&sequential()),
            capi::sequential_library(&sequential()),
        ];
        expected.extend(gates::generators(&sequential()));
        if apple {
            expected.extend(swift::copy(&sequential()));
            expected.extend(swift::examples(&sequential()));
            expected.push(swift::examples_directory(&sequential()));
            expected.push(port("Swift", "quickstart", "/programs"));
            expected.push(port("Swift", "persistence", "/programs"));
        }
        expected.push(kotlin::ports(&metadata));
        expected.push(kotlin::port("quickstart", "/JAVA_HOME", &sequential()));
        expected.push(kotlin::port("persistence", "/JAVA_HOME", &sequential()));
        expected.push(kmp::ports("/JAVA_HOME_21"));
        expected.push(kmp::port("quickstart", "/JAVA_HOME_21", &sequential()));
        expected.push(kmp::port("persistence", "/JAVA_HOME_21", &sequential()));
        expected.push(java::ports(&metadata));
        expected.push(java::port("quickstart", "/JAVA_HOME", &sequential()));
        expected.push(java::port("persistence", "/JAVA_HOME", &sequential()));
        expected.extend(web::copy(&sequential()));
        expected.push(web::copied_generator(&sequential()));
        expected.push(web::port("quickstart", &sequential()));
        expected.push(web::port("persistence", &sequential()));
        for package in [capi::C_PACKAGE, capi::CPP_PACKAGE] {
            for example in ["quickstart", "persistence"] {
                expected.push(capi::port_build(
                    package,
                    "/LLVM_HOME",
                    &sequential(),
                    example,
                ));
            }
            expected.push(capi::port(package, "quickstart", &sequential()));
            expected.push(capi::port(package, "persistence", &sequential()));
        }
        expected.extend(python::copy(&sequential()));
        expected.push(python::port("quickstart", &sequential()));
        expected.push(python::port("persistence", &sequential()));
        assert_eq!(recorder.steps, expected, "{apple}");
        assert_eq!(recorder.cleared, cleared(&sequential(), apple), "{apple}");
        assert_eq!(recorder.written, [web::copied_configuration(&sequential())]);
        assert_eq!(
            recorder.created,
            [capi::C_PACKAGE, capi::CPP_PACKAGE]
                .map(|package| capi::ports_directory(package, &sequential()))
        );
    }
    assert_eq!(
        web::port("members_and_accounts", &sequential()).command_line(),
        "cd /eranos/target/sequential/typescript && node --import tsx examples/members_and_accounts.ts"
    );
    assert_eq!(
        web::copied_generator(&sequential()).command_line(),
        "cd /eranos/target/sequential/typescript && CARGO_TARGET_DIR='/eranos/target/sequential' \
         uniffi-bindgen-react-native build wasm2 --config ubrn.config.yaml --no-wasm-build"
    );
    assert_eq!(
        library(&sequential()).command_line(),
        "CARGO_TARGET_DIR='/eranos/target/sequential' cargo build --package eranos-uniffi --features sequential-ids"
    );
    assert_eq!(
        java::port("restaurant_bill", "/JAVA_HOME", &sequential()).command_line(),
        "/JAVA_HOME/bin/java --enable-native-access=ALL-UNNAMED --illegal-native-access=deny \
         -Dstdout.encoding=UTF-8 -Djava.library.path=/eranos/target/sequential/debug \
         -cp bindings/java/build/ports/* eranos.examples.restaurantbill.Main"
    );
    assert_eq!(
        multiplatform_library(&sequential()).command_line(),
        "CARGO_TARGET_DIR='/eranos/target/sequential' cargo build --package eranos-kmp --features sequential-ids"
    );
    assert_eq!(
        kmp::port("grand_tour", "/JAVA_HOME_21", &sequential()).command_line(),
        "/JAVA_HOME_21/bin/java --enable-native-access=ALL-UNNAMED -Dstdout.encoding=UTF-8 \
         -Djna.library.path=/eranos/target/sequential/debug -cp bindings/kmp/examples/build/ports/* \
         eranos.examples.grandtour.MainKt"
    );
    assert_eq!(
        kotlin::port("split_methods", "/JAVA_HOME", &sequential()).command_line(),
        "/JAVA_HOME/bin/java --enable-native-access=ALL-UNNAMED --illegal-native-access=deny \
         -Dstdout.encoding=UTF-8 -Djna.library.path=/eranos/target/sequential/debug \
         -cp bindings/kotlin/examples/build/ports/* eranos.examples.splitmethods.MainKt"
    );
}

/// A port whose report is not its example's stops the ports at it, naming
/// its language, so no later port hides it: the Swift one where the machine
/// builds for Apple platforms, the Kotlin one elsewhere.
#[test]
fn a_report_that_differs_stops_the_ports_at_its_port() {
    let metadata = Metadata::parse(METADATA).unwrap();
    for (apple, language, last) in [
        (true, "Swift", port("Swift", "quickstart", "/programs")),
        (
            false,
            "Kotlin",
            kotlin::port("quickstart", "/JAVA_HOME", &sequential()),
        ),
    ] {
        let mut recorder = Recorder::new();
        recorder.apple = apple;
        let failure = run(
            &mut recorder,
            &metadata,
            &[reference("quickstart", "other")],
        );
        assert!(
            matches!(&failure, Err(Failure::Port { language: found, .. }) if language == *found),
            "{failure:?}"
        );
        assert_eq!(recorder.steps.last(), Some(&last));
    }
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
/// be printed, the build of the Kotlin or the Kotlin Multiplatform ports or
/// of a C one, a port that fails, a directory that could not be cleared or
/// created, or a variable the JVM's or the C ports need that is not set,
/// which stops them before any step.
#[test]
fn a_failure_stops_the_ports_where_it_happens() {
    let metadata = Metadata::parse(METADATA).unwrap();
    // What the recorder answers every port with, so each report matches.
    let references = [reference("quickstart", METADATA)];
    for failing in [
        library(&sequential()),
        multiplatform_library(&sequential()),
        gates::generators(&sequential())[0].clone(),
        swift::copy(&sequential())[0].clone(),
        swift::examples(&sequential())[2].clone(),
        swift::examples_directory(&sequential()),
        port("Swift", "quickstart", "/programs"),
        kotlin::ports(&metadata),
        kotlin::port("quickstart", "/JAVA_HOME", &sequential()),
        kmp::ports("/JAVA_HOME_21"),
        kmp::port("quickstart", "/JAVA_HOME_21", &sequential()),
        java::ports(&metadata),
        java::port("quickstart", "/JAVA_HOME", &sequential()),
        web::sequential_library(&sequential()),
        web::copy(&sequential())[1].clone(),
        web::copied_generator(&sequential()),
        web::port("quickstart", &sequential()),
        capi::sequential_library(&sequential()),
        capi::port_build(capi::C_PACKAGE, "/LLVM_HOME", &sequential(), "quickstart"),
        capi::port(capi::C_PACKAGE, "quickstart", &sequential()),
        capi::port_build(capi::CPP_PACKAGE, "/LLVM_HOME", &sequential(), "quickstart"),
        capi::port(capi::CPP_PACKAGE, "quickstart", &sequential()),
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
    assert_eq!(
        blocked.steps,
        [
            library(&sequential()),
            multiplatform_library(&sequential()),
            web::sequential_library(&sequential()),
            capi::sequential_library(&sequential()),
        ]
    );
    let mut uncreated = Recorder::new();
    uncreated.failing_files = Some(FileOperation::Create);
    assert!(matches!(
        run(&mut uncreated, &metadata, &references),
        Err(Failure::Usage)
    ));
    assert_eq!(
        uncreated.steps.last(),
        Some(&web::port("quickstart", &sequential()))
    );
    let mut unwritten = Recorder::new();
    unwritten.failing_files = Some(FileOperation::Write);
    assert!(matches!(
        run(&mut unwritten, &metadata, &references),
        Err(Failure::Usage)
    ));
    assert_eq!(unwritten.steps.last(), web::copy(&sequential()).last());
    for (variables, missing) in [
        (&[][..], "JAVA_HOME"),
        (&["JAVA_HOME"][..], "ANDROID_HOME"),
        (&["JAVA_HOME", "ANDROID_HOME"][..], "JAVA_HOME_21"),
        (
            &["JAVA_HOME", "ANDROID_HOME", "JAVA_HOME_21"][..],
            "LLVM_HOME",
        ),
    ] {
        let mut unnamed = Recorder::new();
        unnamed.variables = variables.to_vec();
        let failure = run(&mut unnamed, &metadata, &references);
        assert!(
            matches!(failure, Err(Failure::Variable(name)) if missing == name),
            "{failure:?}"
        );
        assert!(unnamed.steps.is_empty());
    }
}

/// An example without a port would print its report unchecked in that
/// language, and a port without an example would check nothing; the Swift
/// examples' package has a program for every Rust example of eranos and for
/// no other, and so do the Kotlin and the Java examples, a package each,
/// named without underscores, whose `Main.kt` or `Main.java` holds its
/// `main`, the TypeScript, the C, and the C++ examples, a module or a source
/// each beside the report's, and the Python examples, a module each beside
/// the report's, or a package with a `__main__.py`, as the tour is.
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
    let kotlin = names(
        "bindings/kotlin/examples/src/main/kotlin/eranos/examples",
        |path| {
            path.join("Main.kt")
                .is_file()
                .then(|| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .flatten()
        },
    );
    let java = names("bindings/java/src/examples/java/eranos/examples", |path| {
        path.join("Main.java")
            .is_file()
            .then(|| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .flatten()
    });
    let unseparated: Vec<String> = examples
        .iter()
        .map(|example| example.replace('_', ""))
        .collect();
    assert_eq!(swift, examples);
    assert_eq!(kotlin, unseparated);
    assert_eq!(java, unseparated);
    let typescript = names("bindings/typescript/examples", |path| {
        path.file_stem()
            .filter(|_| path.extension().is_some_and(|extension| "ts" == extension))
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|name| "report" != name)
    });
    assert_eq!(typescript, examples);
    for (directory, language) in [
        ("bindings/c/examples", "c"),
        ("bindings/cpp/examples", "cpp"),
    ] {
        let sources = fs::read_dir(Path::new(WORKSPACE).join(directory))
            .unwrap()
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                path.file_stem()
                    .filter(|_| {
                        path.extension()
                            .is_some_and(|extension| language == extension)
                    })
                    .map(|stem| stem.to_string_lossy().into_owned())
                    .filter(|name| "report" != name)
            });
        let mut native: Vec<String> = sources.collect();
        native.sort();
        assert_eq!(native, examples, "{language}");
    }
    let python = names("bindings/python/examples", |path| {
        let module = path
            .extension()
            .is_some_and(|extension| "py" == extension)
            .then(|| path.file_stem())
            .flatten();
        let package = path
            .join("__main__.py")
            .is_file()
            .then(|| path.file_name())
            .flatten();
        module
            .or(package)
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !["__init__", "report"].contains(&name.as_str()))
    });
    assert_eq!(python, examples);
    assert_eq!(8, examples.len());
}
