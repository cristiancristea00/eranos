//! Cleaning: cargo's target directory, then every path the repository's
//! `.gitignore` files ignore but the kept ones, and with `--caches` the
//! caches the gates fill, each tool's cleaner first; a dry run runs only
//! cargo's own dry run and the listing.

use super::*;
use crate::tests::Recorder;

/// The repository's `.gitignore`, whose entries name what a clean removes.
const GITIGNORE: &str = include_str!("../../../../.gitignore");

/// A listing as git prints it, each path followed by NUL: a directory, a
/// file, a link, cargo's target directory, and a directory inside the first.
const LISTING: &str = "bindings/c/build/\0bindings/python/.coverage\0bindings/python/src/eranos/_internal/liberanos_uniffi.dylib\0target/\0bindings/c/build/tests/\0";

/// The paths of [`LISTING`] that a clean removes, in its order.
const REMOVED: [&str; 3] = [
    "bindings/c/build/",
    "bindings/python/.coverage",
    "bindings/python/src/eranos/_internal/liberanos_uniffi.dylib",
];

/// A recorder whose git lists [`LISTING`] and whose machine sets `HOME`.
fn machine() -> Recorder {
    let mut recorder = Recorder::new();
    recorder.ignored = LISTING.to_owned();
    recorder.variables.push("HOME");
    recorder
}

/// The command lines of `steps`.
fn lines(steps: &[Step]) -> Vec<String> {
    steps.iter().map(Step::command_line).collect()
}

/// A clean that skipped cargo's target directory or a path the listing names
/// would leave the space it was run to free: it runs `cargo clean`, then the
/// listing, then removes each path the listing names, in its order.
#[test]
fn a_clean_removes_cargo_s_target_then_every_ignored_path() {
    let mut recorder = machine();
    run(&mut recorder, Scope::Repository, Mode::Remove).unwrap();
    assert_eq!(
        lines(&recorder.steps),
        [
            "cargo clean",
            "git ls-files -z --others --ignored --directory --exclude-per-directory=.gitignore --exclude=!.DS_Store --exclude=!local.properties",
            "rm -rf -- bindings/c/build/",
            "rm -rf -- bindings/python/.coverage",
            "rm -rf -- bindings/python/src/eranos/_internal/liberanos_uniffi.dylib",
        ]
    );
}

/// A listing whose last NUL were taken for a separator would end in an
/// empty path, the workspace root, which `rm -rf` would remove with the
/// repository: no path is empty, none is inside cargo's target directory,
/// which `cargo clean` removes, and none is inside a directory named before
/// it, which goes with that directory; a path that only starts with a file's
/// name, or with a directory's without its `/`, is not inside it. Leaving
/// out what a directory named before takes with it keeps a dry run to one
/// line per tree; a directory named after one inside it still gets its own
/// step, which `rm -rf` takes whether or not the first went.
#[test]
fn no_path_is_the_workspace_root_cargo_s_target_or_inside_one_removed_before() {
    assert_eq!(entries(LISTING), REMOVED);
    assert_eq!(entries("a/\0\0b\0"), ["a/", "b"]);
    assert_eq!(entries("\0"), Vec::<&str>::new());
    assert_eq!(entries(""), Vec::<&str>::new());
    assert_eq!(entries("target/\0target/debug/\0"), Vec::<&str>::new());
    assert_eq!(
        entries("x/src/\0x/src/main/\0x/src/main/jniLibs/\0x/srcs\0x\0xy\0"),
        ["x/src/", "x/srcs", "x", "xy"]
    );
    assert_eq!(entries("x/src/main/\0x/src/\0"), ["x/src/main/", "x/src/"]);
}

/// A listing that read git's per-user or per-checkout ignore files would
/// remove a developer's own files, and one that named the kept ones would
/// remove Android Studio's SDK path: the listing reads the repository's
/// `.gitignore` files alone and keeps `.DS_Store` and `local.properties`,
/// each a name `.gitignore` ignores, so the list cannot outlive its entries.
#[test]
fn the_listing_reads_only_the_repository_s_ignore_files_and_keeps_two_names() {
    let arguments = listing().command_line();
    assert!(!arguments.contains("--exclude-standard"), "{arguments}");
    for name in KEPT {
        assert!(arguments.contains(&format!(" --exclude=!{name}")), "{name}");
        assert!(GITIGNORE.lines().any(|line| line.ends_with(name)), "{name}");
    }
}

/// A dry run that removed anything would not be one: it runs only cargo's
/// own dry run and the listing, whatever it reaches, and prints the steps a
/// clean would run, built as a clean builds them.
#[test]
fn a_dry_run_runs_only_cargo_s_dry_run_and_the_listing() {
    for scope in [Scope::Repository, Scope::WithCaches] {
        let mut recorder = machine();
        run(&mut recorder, scope, Mode::DryRun).unwrap();
        assert_eq!(
            lines(&recorder.steps),
            ["cargo clean --dry-run".to_owned(), listing().command_line()],
            "{scope:?}"
        );
    }
}

/// The caches outside the repository are other projects' too, so a clean
/// that emptied them unasked would cost each project a download, and one
/// that removed a cache's directory before a missing tool stopped it would
/// leave the caches half emptied: with `--caches`, each tool's cleaner runs
/// first, the Swift package manager's only where the machine builds for
/// Apple platforms, then Gradle's daemons' registry, its caches, and the
/// wrappers' distributions, Kotlin/Native's data, and cargo's registry and
/// git checkouts go.
#[test]
fn the_caches_go_only_with_caches_each_tool_s_cleaner_first() {
    for apple in [true, false] {
        let mut recorder = machine();
        recorder.apple = apple;
        run(&mut recorder, Scope::WithCaches, Mode::Remove).unwrap();
        let mut expected = lines(&recorder.steps[..5]);
        expected.extend(
            [
                "cd bindings/python && uv cache clean",
                "cd bindings/typescript && npm cache clean --force",
            ]
            .map(str::to_owned),
        );
        if apple {
            expected.push("cd bindings/swift && swift package purge-cache".to_owned());
        }
        expected.extend(
            [
                "rm -rf -- /HOME/.gradle/daemon",
                "rm -rf -- /HOME/.gradle/caches",
                "rm -rf -- /HOME/.gradle/wrapper/dists",
                "rm -rf -- /HOME/.konan",
                "rm -rf -- /HOME/.cargo/registry",
                "rm -rf -- /HOME/.cargo/git",
            ]
            .map(str::to_owned),
        );
        assert_eq!(lines(&recorder.steps), expected, "{apple}");
        let mut repository = machine();
        repository.apple = apple;
        run(&mut repository, Scope::Repository, Mode::Remove).unwrap();
        assert_eq!(repository.steps, recorder.steps[..5], "{apple}");
    }
}

/// A clean that ignored a tool's own variable would empty another
/// directory than the one the tool uses: each cache's root is its
/// variable's value, and `HOME` names only the root of a cache whose
/// variable is unset.
#[test]
fn each_cache_s_variable_names_its_root() {
    let mut named = Recorder::new();
    named
        .variables
        .extend(["CARGO_HOME", "GRADLE_USER_HOME", "KONAN_DATA_DIR"]);
    assert_eq!(
        caches(&named).unwrap(),
        Caches {
            cargo: "/CARGO_HOME".to_owned(),
            gradle: "/GRADLE_USER_HOME".to_owned(),
            konan: "/KONAN_DATA_DIR".to_owned(),
        }
    );
    assert_eq!(
        caches(&machine()).unwrap(),
        Caches {
            cargo: "/HOME/.cargo".to_owned(),
            gradle: "/HOME/.gradle".to_owned(),
            konan: "/HOME/.konan".to_owned(),
        }
    );
}

/// A clean that found no root for a cache after removing the repository's
/// paths would stop half done: whichever cache neither its variable nor
/// `HOME` names, cargo's, Gradle's, or Kotlin/Native's, stops `--caches`
/// before any step runs, even when the other caches' variables are set.
#[test]
fn a_cache_without_a_root_stops_the_clean_before_any_step() {
    for variables in [
        &[][..],
        &["CARGO_HOME"],
        &["CARGO_HOME", "GRADLE_USER_HOME"],
    ] {
        let mut recorder = Recorder::new();
        recorder.variables = variables.to_vec();
        let failure = run(&mut recorder, Scope::WithCaches, Mode::Remove).unwrap_err();
        assert!(
            matches!(failure, Failure::Variable("HOME")),
            "{variables:?} {failure:?}"
        );
        assert!(recorder.steps.is_empty(), "{variables:?}");
    }
}

/// A clean that went on after a step failed would report a clean it did not
/// make: `cargo clean`, the listing, and a removal each stop it with their
/// failure, and nothing after them runs.
#[test]
fn a_failing_step_stops_the_clean() {
    for failing in [cargo_clean(Mode::Remove), listing(), removal(REMOVED[0])] {
        let mut recorder = machine();
        recorder.failing = Some(failing.clone());
        let failure = run(&mut recorder, Scope::Repository, Mode::Remove).unwrap_err();
        assert!(matches!(failure, Failure::Usage), "{failing:?} {failure:?}");
        assert_eq!(recorder.steps.last(), Some(&failing));
    }
}
