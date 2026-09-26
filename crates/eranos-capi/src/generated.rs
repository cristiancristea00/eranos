//! The bridge that `cargo xtask generate` writes from the binding core.
//!
//! The generator writes these files from the core's records and calls, and
//! `diplomat-tool` reads them with the hand-written modules. Edit the core
//! or the generator in eranos-glue, never these files, and run the command
//! again: a test of xtask fails while a file holds other code than the
//! generator writes.

/// The group, its read handle, the cancellation of asynchronous calls, and
/// the outcomes that their callbacks receive.
pub mod group;
/// The records, lists, and enums the group's calls take and answer.
///
/// Each record is an opaque object with a getter of every field, and a
/// record a call takes also has a constructor of its required fields and a
/// setter of every field.
pub mod records;
