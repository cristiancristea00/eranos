//! Cancelling an asynchronous call: a flag the caller sets and the call
//! checks before each poll of its work.

use std::collections::BTreeMap;
use std::future::{IntoFuture, poll_fn};
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Poll, Waker};

use crate::EranosError;

/// Cancels the asynchronous calls it is given to.
///
/// A call checks its cancellation before every poll of its work: once
/// [`cancel`](Self::cancel) is called, the call stops at its next yield, or
/// at once while it waits for the group's lock, drops its work, and fails
/// with [`EranosError::Cancelled`]. The library's calls change nothing until
/// they complete, so a cancelled call leaves the group as it was. One
/// cancellation can serve several calls, and cancelling one that no call
/// holds any more does nothing.
#[derive(Debug, Default)]
pub struct Cancellation {
    cancelled: AtomicBool,
    /// The key of the next call given the cancellation.
    next_call: AtomicU64,
    /// The task of every call that waits on the cancellation, under the
    /// call's key, which `cancel` wakes so that the call sees it. A call
    /// keeps one entry, however often it is polled, and removes it when it
    /// ends, so a cancellation that serves many calls holds only the ones in
    /// progress.
    waiting: Mutex<BTreeMap<u64, Waker>>,
}

impl Cancellation {
    /// A cancellation not yet cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Cancels every call that holds the cancellation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        // Woken after the lock is released: a waker that polls its task at
        // once would otherwise find the lock taken by its own thread.
        let waiting = std::mem::take(&mut *self.waiting());
        for waker in waiting.into_values() {
            waker.wake();
        }
    }

    /// Whether [`cancel`](Self::cancel) was called.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// The calls that wait; a panic while they were changed leaves a map
    /// that is still whole, so a poisoned lock is taken as it is.
    fn waiting(&self) -> MutexGuard<'_, BTreeMap<u64, Waker>> {
        self.waiting.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A call's entry among the calls that wait on a cancellation, removed when
/// the call ends: completed, cancelled, or dropped.
struct Waiting<'cancellation> {
    /// The call's key.
    call: u64,
    cancellation: &'cancellation Cancellation,
}

impl<'cancellation> Waiting<'cancellation> {
    /// The entry of a new call.
    fn new(cancellation: &'cancellation Cancellation) -> Self {
        Self {
            call: cancellation.next_call.fetch_add(1, Ordering::Relaxed),
            cancellation,
        }
    }

    /// Keeps `waker` to wake when the cancellation is cancelled, in place of
    /// the one the call was polled with before.
    fn register(&self, waker: &Waker) {
        let mut waiting = self.cancellation.waiting();
        if !waiting
            .get(&self.call)
            .is_some_and(|known| known.will_wake(waker))
        {
            waiting.insert(self.call, waker.clone());
        }
    }
}

impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        self.cancellation.waiting().remove(&self.call);
    }
}

/// Runs `work` to its end, unless `cancellation` is cancelled first.
///
/// # Errors
///
/// [`EranosError::Cancelled`] when the cancellation is cancelled before the
/// work completes; the work is dropped at that poll.
pub(crate) async fn cancellable<Work: IntoFuture>(
    work: Work,
    cancellation: Option<Arc<Cancellation>>,
) -> Result<Work::Output, EranosError> {
    let waiting = cancellation.as_deref().map(Waiting::new);
    let mut work = pin!(work.into_future());
    poll_fn(|context| {
        if let Some(waiting) = &waiting {
            // Registered before the check, so that a cancel between the two
            // still wakes this task.
            waiting.register(context.waker());
            if waiting.cancellation.is_cancelled() {
                return Poll::Ready(Err(EranosError::cancelled()));
            }
        }
        work.as_mut().poll(context).map(Ok)
    })
    .await
}

#[cfg(test)]
mod tests;
