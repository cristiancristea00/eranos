//! The C glue's bridge, which `cargo xtask generate` writes from the binding
//! core's records and calls. `diplomat-tool` reads the C glue's source
//! files, not what a macro would expand them to, so the bridge is code the
//! repository holds; a test here fails while a committed file holds other
//! code than the generator writes, however rustfmt laid it out.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use eranos_glue::Generated;

use crate::Failure;
use crate::plan::{self, Action, Phase};
use crate::process::{Runner, Step, WORKSPACE};

/// The binding core's sources, from the workspace root: the files of its
/// records and of its trait of calls, whose submodules hold none.
const CORE: &str = "crates/eranos-bindings/src";

/// The C glue's sources, from the workspace root, under which each file of
/// the bridge names its path.
const GLUE: &str = "crates/eranos-capi/src";

/// The directory of the bridge's files, under [`GLUE`], which holds no other
/// file.
const GENERATED: &str = "generated";

/// Writes the bridge's files from the core's sources as they stand, and lays
/// them out with rustfmt.
///
/// # Errors
///
/// As [`write_bridge`].
pub(crate) fn run(runner: &mut impl Runner) -> Result<(), Failure> {
    write_bridge(runner, &Path::new(WORKSPACE).join(CORE))
}

/// Writes the bridge's files from the core's sources in `directory`, and
/// lays them out with rustfmt.
///
/// # Errors
///
/// The failure of reading the sources, of writing the bridge from them, and
/// of the first action that fails.
fn write_bridge(runner: &mut impl Runner, directory: &Path) -> Result<(), Failure> {
    plan::execute(runner, actions(&bridge(directory)?), |_| true)
}

/// The bridge's files, from the core's sources in `directory`, read in the
/// order of their paths.
///
/// # Errors
///
/// [`Failure::Read`] for a directory or a source that cannot be read, and
/// [`Failure::Bridge`] for sources the generator cannot write a bridge from.
pub(crate) fn bridge(directory: &Path) -> Result<Vec<Generated>, Failure> {
    let sources = fs::read_dir(directory)
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<io::Result<Vec<_>>>()
        })
        .and_then(|mut paths| {
            paths.retain(|path| Some("rs".as_ref()) == path.extension());
            paths.sort();
            paths
                .iter()
                .map(fs::read_to_string)
                .collect::<io::Result<Vec<_>>>()
        })
        .map_err(|source| Failure::Read {
            path: directory.display().to_string(),
            source,
        })?;
    eranos_glue::capi_bridge(&sources).map_err(Failure::Bridge)
}

/// The directory of the bridge's files.
pub(crate) fn directory() -> PathBuf {
    Path::new(WORKSPACE).join(GLUE).join(GENERATED)
}

/// Where the bridge's file `file` goes.
pub(crate) fn destination(file: &Generated) -> PathBuf {
    Path::new(WORKSPACE).join(GLUE).join(file.path())
}

/// The actions that write `files`: the directory cleared, so a file the
/// generator no longer writes is not left behind as a module, and made
/// again; each file written; and the C glue laid out by `cargo fmt`, as
/// gate 1 checks it.
pub(crate) fn actions(files: &[Generated]) -> Vec<Action> {
    let mut actions = vec![
        Action::clear(Phase::Prepare, directory()),
        Action::create(Phase::Prepare, directory()),
    ];
    actions.extend(
        files
            .iter()
            .map(|file| Action::write(Phase::Prepare, destination(file), file.text())),
    );
    actions.push(Action::run(Phase::Prepare, format()));
    actions
}

/// The step that lays out the C glue's sources, the bridge's among them.
pub(crate) fn format() -> Step {
    Step::cargo(
        &["fmt", "--package", "eranos-capi"],
        "rustfmt on the C glue and its bridge",
    )
}

#[cfg(test)]
mod tests;
