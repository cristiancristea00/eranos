//! The gates: the checks in AGENTS.md's order, the examples cargo lists, and
//! stopping at the first gate that fails.

use std::fs;

use super::*;
use crate::process::WORKSPACE;
use crate::tests::{FileOperation, METADATA, Recorder};

/// AGENTS.md, which lists the gates.
const AGENTS: &str = include_str!("../../../../AGENTS.md");

/// DEVELOPMENT.md, whose § 2 times the gates.
const DEVELOPMENT: &str = include_str!("../../../../docs/DEVELOPMENT.md");

/// Every document of the repository that could cite a gate by number, with
/// its path.
const DOCUMENTS: [(&str, &str); 7] = [
    ("AGENTS.md", AGENTS),
    ("README.md", include_str!("../../../../README.md")),
    (
        "docs/ARCHITECTURE.md",
        include_str!("../../../../docs/ARCHITECTURE.md"),
    ),
    (
        "docs/BINDINGS.md",
        include_str!("../../../../docs/BINDINGS.md"),
    ),
    ("docs/DEVELOPMENT.md", DEVELOPMENT),
    (
        "docs/TESTING.md",
        include_str!("../../../../docs/TESTING.md"),
    ),
    (
        "docs/VERSIONING.md",
        include_str!("../../../../docs/VERSIONING.md"),
    ),
];

/// The command AGENTS.md lists for the gate that runs the examples.
const EXAMPLES: &str = "cargo run --quiet --package <package> --all-features --example <name>";

/// The command AGENTS.md lists for the gate that lints the mobile and wasm
/// targets, which stands for the steps of [`target_lints`].
const TARGETS: &str = "rustup target add <the mobile and wasm targets>";

/// The commands AGENTS.md lists under its gates, in its order: the items of
/// the numbered list in that section, each a command in backticks.
fn documented_gates() -> Vec<String> {
    let (_, after) = AGENTS.split_once("## Gates before committing").unwrap();
    let (section, _) = after.split_once("\n## ").unwrap();
    section
        .lines()
        .filter_map(|line| line.split_once(". `"))
        .filter(|(number, _)| {
            !number.is_empty() && number.chars().all(|character| character.is_ascii_digit())
        })
        .filter_map(|(_, rest)| rest.split_once('`'))
        .map(|(command, _)| command.to_owned())
        .collect()
}

/// The numbers `text` cites as a gate ("gate 17", "Gate 8"), in its order.
fn cited_gates(text: &str) -> Vec<usize> {
    let words: Vec<&str> = text.split_whitespace().collect();
    words
        .windows(2)
        .filter_map(|pair| match pair {
            [word, number]
                if word
                    .trim_start_matches(|character: char| !character.is_alphabetic())
                    .eq_ignore_ascii_case("gate") =>
            {
                number
                    .trim_end_matches(|character: char| !character.is_ascii_digit())
                    .parse()
                    .ok()
            }
            _ => None,
        })
        .collect()
}

/// The first and last gate of each row of the timing table in
/// `development`, DEVELOPMENT.md, read from the row's first cell, such as
/// `1–8, the Rust checks` or `11, the Swift package`.
fn timed_gates(development: &str) -> Vec<(usize, usize)> {
    development
        .lines()
        .skip_while(|line| !line.starts_with("| Gates "))
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .map(|line| {
            let (numbers, _) = line.trim_start_matches("| ").split_once(',').unwrap();
            let (first, last) = numbers.split_once('–').unwrap_or((numbers, numbers));
            (first.parse().unwrap(), last.parse().unwrap())
        })
        .collect()
}

/// The host lints, the target lints, the tests and the documentation, then
/// async-lock's features are the gates AGENTS.md lists, in its order, the
/// target lints under the command that adds their targets, and the last
/// documented gates are the examples, the host library, the generators, the
/// Swift, Kotlin, Java, TypeScript, Kotlin Multiplatform, and C and C++
/// packages, and the ports; a check the code dropped, added, or reordered
/// would pass a commit the documented gates refuse.
#[test]
fn the_checks_are_the_gates_agents_md_lists_in_its_order() {
    let documented = documented_gates();
    let mut before_examples: Vec<String> = host_lints().iter().map(Step::command_line).collect();
    before_examples.push(TARGETS.to_owned());
    before_examples.extend(tests_and_documentation().iter().map(Step::command_line));
    before_examples.push(lock_features().command_line());
    assert_eq!(documented[..before_examples.len()], before_examples);
    let targets = target_lints();
    assert!(targets[0].command_line().starts_with("rustup target add "));
    assert!(
        targets[1..]
            .iter()
            .all(|step| step.command_line().starts_with("cargo clippy "))
    );
    assert_eq!(
        checks(),
        [host_lints(), targets, tests_and_documentation()].concat()
    );
    assert_eq!(
        documented[before_examples.len()..],
        [
            EXAMPLES,
            &host_library().command_line(),
            "cargo run --quiet --package eranos-uniffi-bindgen --bin <generator> -- generate <arguments>",
            "rustup target add <every Apple target>",
            "rustup target add <every Android target>",
            "cp -R target/uniffi/java/eranos/internal bindings/java/src/generated/java/eranos",
            "rustup target add wasm32-unknown-unknown",
            "sh gradlew --warning-mode=fail :eranos:jvmTest",
            &crate::capi::library().command_line(),
            "cargo build --package eranos-uniffi --features sequential-ids",
        ]
    );
    let sequential = Metadata::parse(METADATA)
        .unwrap()
        .inner(crate::ports::DIRECTORY);
    assert!(
        crate::ports::library(&sequential)
            .command_line()
            .ends_with(documented.last().unwrap())
    );
}

/// Every gate a document cites by number is the gate it means: the
/// documents cite two gates by number, the examples', whose reports the
/// ports compare with, and the ports', the last, so each citation is the
/// number of one of them in AGENTS.md's list, and the rows of DEVELOPMENT.md
/// § 2's timing table cover every gate once, in order; a gate inserted into
/// the list would otherwise leave every later number in the documents
/// naming the gate before the one it means.
#[test]
fn every_gate_the_documents_cite_by_number_is_the_one_they_mean() {
    let documented = documented_gates();
    let examples = 1 + documented
        .iter()
        .position(|command| EXAMPLES == command)
        .unwrap();
    let ports = documented.len();
    for (path, text) in DOCUMENTS {
        for number in cited_gates(text) {
            assert!(
                [examples, ports].contains(&number),
                "{path} cites gate {number}; the examples' is {examples}, the ports' {ports}"
            );
        }
    }
    let mut next = 1;
    for (first, last) in timed_gates(DEVELOPMENT) {
        assert_eq!(next, first, "the timing table skips or repeats gate {next}");
        next = last + 1;
    }
    assert_eq!(
        ports + 1,
        next,
        "the timing table ends before the last gate"
    );
}

/// Citations read as the gates they number, whatever punctuation surrounds
/// them, and a word that only contains "gate" cites nothing; a reader that
/// missed "(gate 9)" or took "gates 3" for a citation would pass stale
/// numbers or refuse good ones.
#[test]
fn a_citation_is_the_word_gate_and_a_number() {
    assert_eq!(
        cited_gates("keeps the reports for gate 18, and (Gate 9) keeps them;\ngate\n4's steps"),
        [18, 9, 4]
    );
    assert!(cited_gates("the gates 3 and 4, a gateway 5, a gate that").is_empty());
    assert_eq!(
        timed_gates(
            "| Gates | Warm |\n| --- | ---: |\n| 1–3, the checks | 5 s |\n| 4, the lint | 2 s |\n\nafter"
        ),
        [(1, 3), (4, 4)]
    );
}

/// The commands of the job `name` of `workflow`, in its order: each `run`
/// on its line, or folded after `>-` from the lines indented beneath it. The
/// lines are read with `lines`, which drops the carriage return that a
/// checkout with Windows line endings adds.
fn job_commands(workflow: &str, name: &str) -> Vec<String> {
    let heading = format!("  {name}:");
    let mut commands: Vec<String> = Vec::new();
    let mut folding = false;
    for line in workflow
        .lines()
        .skip_while(|line| heading != *line)
        .skip(1)
        .take_while(|line| line.is_empty() || line.starts_with("    "))
    {
        if let Some(command) = line.trim_start().strip_prefix("- run: ") {
            folding = ">-" == command;
            commands.push(if folding {
                String::new()
            } else {
                command.to_owned()
            });
        } else if folding && line.starts_with("          ") {
            let command = commands.last_mut().unwrap();
            if !command.is_empty() {
                command.push(' ');
            }
            command.push_str(line.trim());
        } else {
            folding = false;
        }
    }
    commands
}

/// The target lints are the steps of CI's Mobile and wasm targets job, which
/// adds `--locked` to every cargo call as the workflow does, and the job
/// reads the same from a checkout with Windows line endings; if they drifted
/// apart, a commit that passes the gates could fail its push run, and a
/// target CI lints could break with no gate to show it before the push. On
/// Windows, where a checkout ends each line with a carriage return and a
/// line feed, a split at `"\n  mobile:\n"` found no job.
#[test]
fn the_target_lints_are_the_steps_of_ci_s_mobile_job() {
    let workflow = include_str!("../../../../.github/workflows/ci.yml");
    let commands = job_commands(workflow, "mobile");
    assert_eq!(
        job_commands(&workflow.replace('\n', "\r\n"), "mobile"),
        commands
    );
    let commands: Vec<String> = commands
        .into_iter()
        .filter(|command| !command.starts_with("rustup toolchain "))
        .map(|command| command.replace(" --locked", ""))
        .collect();
    let steps: Vec<String> = target_lints().iter().map(Step::command_line).collect();
    assert_eq!(commands, steps);
}

/// Every crate root of the workspace, where Cargo finds the targets of each
/// crate: its library, binary, and build script, and each file or directory
/// with a `main.rs` of its binaries, tests, examples, and benches.
fn crate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for package in fs::read_dir(Path::new(WORKSPACE).join("crates")).unwrap() {
        let package = package.unwrap().path();
        for file in ["src/lib.rs", "src/main.rs", "build.rs"] {
            roots.push(package.join(file));
        }
        for directory in ["src/bin", "tests", "examples", "benches"] {
            let Ok(entries) = fs::read_dir(package.join(directory)) else {
                continue;
            };
            for entry in entries {
                let path = entry.unwrap().path();
                roots.push(if path.is_dir() {
                    path.join("main.rs")
                } else {
                    path
                });
            }
        }
    }
    roots.retain(|root| {
        root.is_file() && root.extension().is_some_and(|extension| "rs" == extension)
    });
    roots.sort();
    roots
}

/// Every crate root forbids unsafe code but eranos-capi's, where the
/// workspace's table only denies it, so that the bridge's one `unsafe impl
/// Send` can carry its `#[expect]`; a root without the attribute would let
/// an `#[expect(unsafe_code)]` through unnoticed.
#[test]
fn every_crate_root_but_the_c_glue_s_forbids_unsafe_code() {
    let crates = Path::new(WORKSPACE).join("crates");
    let roots = crate_roots();
    for root in [
        crates.join("eranos/src/lib.rs"),
        crates.join("eranos/tests/api.rs"),
        crates.join("eranos-capi/src/lib.rs"),
        crates.join("xtask/src/main.rs"),
    ] {
        assert!(roots.contains(&root), "{}", root.display());
    }
    for root in roots {
        let forbids = fs::read_to_string(&root)
            .unwrap()
            .lines()
            .any(|line| "#![forbid(unsafe_code)]" == line);
        assert_eq!(
            !root.starts_with(crates.join("eranos-capi")),
            forbids,
            "{}",
            root.display()
        );
    }
}

/// A package that turned on async-lock's `std` would bring the clock into
/// the library unnoticed: the gates stop at the tree that shows it, before
/// any example runs, and name every feature once.
#[test]
fn a_feature_of_async_lock_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.tree = [
        "async-lock v3.4.2",
        "async-lock feature \"default\"",
        "async-lock feature \"std\"",
        "eranos-group v0.4.0 (/eranos/crates/eranos-group)",
        "async-lock feature \"std\" (*)",
    ]
    .join("\n");
    let failure = run(&mut recorder).unwrap_err();
    assert!(
        matches!(&failure, Failure::LockFeatures(features) if "\"default\", \"std\"" == features),
        "{failure:?}"
    );
    assert_eq!(recorder.steps.last(), Some(&lock_features()));
    assert_eq!(
        failure.to_string(),
        "async-lock is built with \"default\", \"std\", which the library keeps off"
    );
}

/// When the tree cannot be printed, the gates stop there, so a broken
/// check never passes for a clean one.
#[test]
fn a_failing_feature_tree_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(lock_features());
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), Some(&lock_features()));
}

/// A tree without a feature line passes, whatever else it shows.
#[test]
fn a_lock_without_features_passes() {
    let tree = "async-lock v3.4.2\neranos-group v0.4.0 (/eranos/crates/eranos-group)\n";
    assert!(check_lock_features(tree).is_ok());
    assert!(check_lock_features("").is_ok());
}

/// Every example target of every package runs, in the order cargo lists
/// them, with every feature (the persistence example needs `serde`); other
/// targets do not run as examples.
#[test]
fn every_example_of_every_package_runs_with_every_feature() {
    let run = |package: &str, example: &str| Example {
        name: example.to_owned(),
        package: package.to_owned(),
        step: Step::cargo(
            &[
                "run",
                "--quiet",
                "--package",
                package,
                "--all-features",
                "--example",
                example,
            ],
            &format!("example {example}"),
        ),
    };
    assert_eq!(
        examples(&Metadata::parse(METADATA).unwrap()),
        [
            run("eranos", "quickstart"),
            run("eranos", "persistence"),
            run("xtask", "demo")
        ]
    );
}

/// Every language's sources come from the host library in cargo's target
/// directory, into a directory of their own under `uniffi`, with the
/// generator of the release that builds the library and without formatting,
/// which the generated code never gets; Java's generator takes the library
/// last and reads it without `--library`.
#[test]
fn every_language_is_generated_from_the_host_library() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let library = format!("/eranos/target/debug/{DLL_PREFIX}eranos_uniffi{DLL_SUFFIX}");
    let lines: Vec<String> = generators(&metadata)
        .iter()
        .map(Step::command_line)
        .collect();
    let prefix = "cargo run --quiet --package eranos-uniffi-bindgen --bin";
    assert_eq!(
        lines,
        [
            format!(
                "{prefix} uniffi-bindgen -- generate --library {library} --language swift --out-dir /eranos/target/uniffi/swift --no-format"
            ),
            format!(
                "{prefix} uniffi-bindgen -- generate --library {library} --language kotlin --out-dir /eranos/target/uniffi/kotlin --no-format"
            ),
            format!(
                "{prefix} uniffi-bindgen-java -- generate --out-dir /eranos/target/uniffi/java --no-format {library}"
            ),
        ]
    );
    assert_eq!(generated(&metadata), Path::new("/eranos/target/uniffi"));
}

/// JSON that is not cargo's metadata is refused rather than read as a
/// workspace without examples, which would skip the examples silently.
#[test]
fn json_that_is_not_cargo_metadata_is_refused() {
    let without_target = r#"{"packages": []}"#;
    for text in [
        "",
        "{",
        "{}",
        r#"{"packages": [{"name": "eranos"}]}"#,
        without_target,
    ] {
        assert!(
            matches!(Metadata::parse(text), Err(Failure::Metadata(_))),
            "{text}"
        );
    }
}

/// A failing check stops the gates after it, so the failure is the last
/// thing a run prints.
#[test]
fn a_failing_check_stops_the_gates_after_it() {
    let checks = checks();
    let mut recorder = Recorder::new();
    recorder.failing = checks.get(1).cloned();
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps, checks[..2]);
}

/// When the examples cannot be listed, no example runs.
#[test]
fn a_failing_example_list_runs_no_example() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(metadata());
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), Some(&metadata()));

    let mut unreadable = Recorder::new();
    unreadable.metadata = "{}".to_owned();
    assert!(matches!(run(&mut unreadable), Err(Failure::Metadata(_))));
    assert_eq!(unreadable.steps.last(), Some(&metadata()));
}

/// A failing step of the Swift package stops the gates there, so a package
/// that does not build is never taken for one that passed.
#[test]
fn a_failing_swift_step_stops_the_gates() {
    let mut recorder = Recorder::new();
    recorder.failing = Some(crate::swift::build());
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), Some(&crate::swift::build()));
}

/// A failing example stops the examples after it, and nothing is generated.
#[test]
fn a_failing_example_stops_the_examples_after_it() {
    let examples: Vec<Step> = examples(&Metadata::parse(METADATA).unwrap())
        .into_iter()
        .map(|example| example.step)
        .collect();
    let mut recorder = Recorder::new();
    recorder.failing = examples.first().cloned();
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), examples.first());
    assert!(!recorder.steps.contains(&examples[1]));
    assert!(recorder.cleared.is_empty());
}

/// When the host library does not build, or the directory of the generated
/// sources cannot be cleared, no generator runs, so stale sources are never
/// taken for fresh ones; a failing generator stops the ones after it.
#[test]
fn a_failure_to_build_clear_or_generate_stops_the_generators() {
    let metadata = Metadata::parse(METADATA).unwrap();
    let generators = generators(&metadata);
    let mut unbuilt = Recorder::new();
    unbuilt.failing = Some(host_library());
    assert!(matches!(run(&mut unbuilt), Err(Failure::Usage)));
    assert_eq!(unbuilt.steps.last(), Some(&host_library()));
    assert!(unbuilt.cleared.is_empty());

    let mut blocked = Recorder::new();
    blocked.failing_files = Some(FileOperation::Clear);
    assert!(matches!(run(&mut blocked), Err(Failure::Usage)));
    assert!(!blocked.steps.contains(&generators[0]));

    let mut recorder = Recorder::new();
    recorder.failing = generators.first().cloned();
    assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
    assert_eq!(recorder.steps.last(), generators.first());
}

/// A package whose gates fail stops the gates at its failing step, so the
/// packages after it never run; if they ran, the gates would end in their
/// output, and a failure scrolled out of sight could pass for a success.
#[test]
fn a_failing_package_stops_the_gates_at_its_step() {
    let metadata = Metadata::parse(METADATA).unwrap();
    for failing in [
        crate::kotlin::libraries()[0].clone(),
        crate::java::sources(&metadata)[0].clone(),
        crate::web::node(),
        plan::steps(&crate::kmp::checks("/JAVA_HOME_21", true))[0].clone(),
        crate::capi::library(),
        crate::ports::library(&metadata.inner(crate::ports::DIRECTORY)),
    ] {
        let mut recorder = Recorder::new();
        recorder.failing = Some(failing.clone());
        assert!(matches!(run(&mut recorder), Err(Failure::Usage)));
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
