//! The C glue's bridge the repository holds is the one the generator writes.
//! A record, a call, or the generator changed without `cargo xtask generate`
//! would leave the C and C++ APIs behind every other binding, since
//! `diplomat-tool` reads the committed files, and a file the generator no
//! longer writes would stay behind as a module that nothing keeps current.

use super::*;
use crate::tests::Recorder;

/// `generate` clears the bridge's directory, writes every file into it, and
/// lays the C glue out as gate 1 checks it; a file left from an earlier
/// bridge, or one not laid out, would fail the freshness test or gate 1.
#[test]
fn generate_writes_every_file_into_a_cleared_directory_and_lays_it_out() {
    let mut recorder = Recorder::new();
    run(&mut recorder).unwrap();
    assert_eq!(recorder.cleared, [directory()]);
    assert_eq!(recorder.created, [directory()]);
    assert_eq!(
        recorder.written,
        [directory().join("group.rs"), directory().join("records.rs")]
    );
    assert_eq!(recorder.steps, [format()]);
}

/// Every committed file of the bridge holds the code the generator writes
/// from the core as it stands, and the directory holds no other file.
#[test]
fn the_committed_bridge_is_what_the_generator_writes() {
    let files = bridge(&Path::new(WORKSPACE).join(CORE)).unwrap();
    let mut committed = fs::read_dir(directory())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    committed.sort();
    assert_eq!(committed, files.iter().map(destination).collect::<Vec<_>>());
    for file in &files {
        let text = fs::read_to_string(destination(file)).unwrap();
        let path = file.path();
        assert!(
            file.is_written_in(&text).unwrap(),
            "{path} holds other code than `cargo xtask generate` writes; run it"
        );
    }
}

/// A hand edit of a committed file's code fails the comparison, and so does
/// one of its documentation, while its doc comments wrapped otherwise do
/// not, since rustfmt and a reader may rewrap them.
#[test]
fn a_hand_edit_of_the_bridge_fails_and_rewrapped_comments_do_not() {
    let files = bridge(&Path::new(WORKSPACE).join(CORE)).unwrap();
    let file = files.first().unwrap();
    let text = fs::read_to_string(destination(file)).unwrap();
    let edited = text.replacen("self.handle.clone()", "self.handle.to_owned()", 1);
    assert_ne!(edited, text);
    assert!(!file.is_written_in(&edited).unwrap());
    let reworded = text.replacen("/// A group:", "/// The group:", 1);
    assert_ne!(reworded, text);
    assert!(!file.is_written_in(&reworded).unwrap());
    let rewrapped = joined(&text);
    assert_ne!(rewrapped, text);
    assert!(file.is_written_in(&rewrapped).unwrap());
    assert!(file.is_written_in("fn broken(").is_err());
}

/// `text` with the lines of each paragraph of its doc comments joined into
/// one line, as a reader who rewraps them would leave it.
fn joined(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let words = line.trim_start().strip_prefix("/// ");
        if let (Some(words), Some(last)) = (words, lines.last_mut())
            && last.trim_start().starts_with("/// ")
        {
            last.push(' ');
            last.push_str(words);
        } else {
            lines.push(line.to_owned());
        }
    }
    lines.join("\n")
}

/// A directory that cannot be read names itself and writes nothing, and
/// sources without a trait of calls say that the core gives no bridge, so
/// the fix is on screen.
#[test]
fn unreadable_or_incomplete_sources_name_what_failed() {
    let missing = Path::new(WORKSPACE).join("crates/missing");
    let mut recorder = Recorder::new();
    let unwritten = write_bridge(&mut recorder, &missing).unwrap_err();
    assert!(matches!(unwritten, Failure::Read { .. }), "{unwritten}");
    assert!(recorder.written.is_empty());
    let unreadable = bridge(&missing).unwrap_err();
    assert!(matches!(unreadable, Failure::Read { .. }), "{unreadable}");
    assert!(
        unreadable
            .to_string()
            .starts_with(&format!("{} could not be read: ", missing.display())),
        "{unreadable}"
    );
    let incomplete = bridge(&Path::new(WORKSPACE).join("crates/xtask/src")).unwrap_err();
    assert!(
        incomplete
            .to_string()
            .starts_with("the C bridge cannot be written from the binding core: "),
        "{incomplete}"
    );
}
