//! The bindings of eranos for C and C++, through Diplomat: one bridge, from
//! which `diplomat-tool` generates the C API and the C++ classes beneath the
//! hand-written C++ layer.
//!
//! The bridge keeps to the shapes Diplomat can carry, and most of it is
//! written from the binding core by `cargo xtask generate` into the module
//! `generated`: the records, lists, and enums, and the group. A value that
//! holds text or a list is an opaque object: a record has a constructor of
//! its required fields, a getter of every field, and, for an entry, a setter
//! of every field; a list has `len` and `get`; an enum with data has `kind`,
//! which returns a C-like enum, a constructor per variant, and a getter of
//! every field its variants hold. A date and a time of day are plain
//! structs, and an enum without data is a C-like enum. Text crosses as
//! bytes, checked to be UTF-8 here, and text a call returns is written into
//! a `DiplomatWrite`.
//!
//! Every group call has a synchronous form, which blocks while another call
//! holds the lock, and an asynchronous one, which runs on the group's worker
//! thread and reports its outcome to a callback there. The worker is one
//! thread per group, started by the first asynchronous call, so the calls run
//! and report in the order they were made.
//!
//! An internal crate of eranos: depend on `eranos`, or on the C and C++
//! packages this crate builds.

use std::sync::{Arc, OnceLock, mpsc};
use std::thread;

use diplomat_runtime::{DiplomatStr, DiplomatWrite};

use crate::generated::expense_group::ffi::{EranosCancellation, EranosExpenseGroup};

pub mod error;
pub mod generated;
pub mod values;

/// A failure of a call: the library's, or text that is not UTF-8, which only
/// C and C++ can pass.
#[derive(Debug, Clone)]
pub(crate) enum Failure {
    /// Text that is not UTF-8.
    InvalidText,
    /// The library's failure.
    Library(eranos_bindings::EranosError),
}

/// The text the bytes `bytes` hold.
///
/// # Errors
///
/// [`Failure::InvalidText`] for bytes that are not UTF-8.
pub(crate) fn decoded(bytes: &DiplomatStr) -> Result<String, Failure> {
    String::from_utf8(bytes.to_vec()).map_err(|_| Failure::InvalidText)
}

/// The text the bytes `bytes` hold, for a synchronous call.
///
/// # Errors
///
/// The error of the kind `InvalidText` for bytes that are not UTF-8.
pub(crate) fn text(bytes: &DiplomatStr) -> Result<String, Box<error::ffi::EranosError>> {
    decoded(bytes).map_err(|failure| Box::new(failure.into()))
}

/// The error of the library's `error`, for a synchronous call.
pub(crate) fn failed(error: eranos_bindings::EranosError) -> Box<error::ffi::EranosError> {
    Box::new(Failure::Library(error).into())
}

/// The value of `result`, which the group handle's `read` and `write`
/// return: the binding core's synchronous call waits for the lock, and is
/// refused, as busy, only on wasm, where a call cannot wait. This crate is never built
/// for wasm, since its worker thread drives the asynchronous calls with
/// `block_on`, which is not compiled for wasm.
#[expect(
    clippy::expect_used,
    reason = "a call that cannot fail is refused only as busy, on wasm, for which this crate is never built"
)]
pub(crate) fn waited<Value>(result: Result<Value, eranos_bindings::EranosError>) -> Value {
    result.expect("a synchronous call waits for the lock outside wasm")
}

/// Writes `text` into `write`. A write that cannot grow records that it
/// could not, which the caller reads, so its failure needs no other report.
pub(crate) fn write_text(write: &mut DiplomatWrite, text: &str) {
    std::fmt::Write::write_str(write, text).unwrap_or_default();
}

/// Writes `text` into `write`, if present, and returns whether it is, as the
/// optional text that Diplomat carries.
pub(crate) fn write_optional(write: &mut DiplomatWrite, text: Option<&str>) -> Option<()> {
    text.map(|text| write_text(write, text))
}

/// A job for a group's worker thread.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// A completion callback, moved to the thread that calls it.
struct Completion<Callback>(Callback);

#[expect(
    unsafe_code,
    reason = "the C and C++ APIs document that a completion callback runs on another thread"
)]
#[expect(
    clippy::non_send_fields_in_send_ty,
    reason = "the callback is the caller's, documented to be callable from another thread"
)]
// SAFETY: the C and C++ APIs document that a completion callback runs on the
// group's worker thread, or for a load on a thread of its own, so the caller
// passes one that is safe to call from another thread.
unsafe impl<Callback> Send for Completion<Callback> {}

/// Starts a worker thread, which runs every job sent to it, in order, until
/// its sender is dropped, and returns the sender.
fn worker() -> mpsc::Sender<Job> {
    let (sender, receiver) = mpsc::channel::<Job>();
    thread::spawn(move || {
        for job in receiver {
            guarded(job);
        }
    });
    sender
}

/// Runs `job`, and ends the process if it panics: the library does not
/// panic, so a panic is a defect of the bindings, and a thread that ended
/// on one would leave every later job unrun and its callbacks waiting.
fn guarded(job: impl FnOnce()) {
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)).is_err() {
        std::process::abort();
    }
}

/// Runs `future` on the worker thread of `jobs`, starting it on the first
/// call, and then `callback` with the outcome that `outcome` makes of its
/// result.
pub(crate) fn spawn<Value: 'static, Outcome: 'static>(
    jobs: &OnceLock<mpsc::Sender<Job>>,
    future: impl Future<Output = Result<Value, Failure>> + Send + 'static,
    callback: impl Fn(Box<Outcome>) + 'static,
    outcome: fn(Result<Value, Failure>) -> Outcome,
) {
    let completion = Completion(callback);
    let job: Job = Box::new(move || {
        // The whole completion moves, not its field, which is not `Send`.
        let completion = completion;
        (completion.0)(Box::new(outcome(eranos::cooperative::block_on(future))));
    });
    // The worker ends only when its group drops the sender, so it receives
    // every job sent while the group lives.
    jobs.get_or_init(worker).send(job).unwrap_or_default();
}

/// Runs `future` on a thread of its own, and then `callback` with the outcome
/// that `outcome` makes of its result.
pub(crate) fn run<Value: 'static, Outcome: 'static>(
    future: impl Future<Output = Result<Value, Failure>> + Send + 'static,
    callback: impl Fn(Box<Outcome>) + 'static,
    outcome: fn(Result<Value, Failure>) -> Outcome,
) {
    let completion = Completion(callback);
    thread::spawn(move || {
        guarded(move || {
            // The whole completion moves, not its field, which is not `Send`.
            let completion = completion;
            (completion.0)(Box::new(outcome(eranos::cooperative::block_on(future))));
        });
    });
}

impl From<eranos_bindings::ExpenseGroupHandle> for EranosExpenseGroup {
    fn from(handle: eranos_bindings::ExpenseGroupHandle) -> Self {
        Self {
            handle,
            jobs: OnceLock::new(),
        }
    }
}

impl EranosCancellation {
    /// The cancellation, for a call of the binding core.
    pub(crate) fn shared(&self) -> Arc<eranos_bindings::Cancellation> {
        Arc::clone(&self.0)
    }
}
