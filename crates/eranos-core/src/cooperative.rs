//! Cooperative futures: long computations that pause at points of their own
//! choosing, and the two ways to drive one.
//!
//! A cooperative future pends only by yielding: it wakes its own task,
//! returns `Pending`, and is ready to go on at its next poll. [`Cooperative`]
//! marks such a future, so it can run to completion without an executor,
//! through [`Cooperative::complete`], or be awaited in any executor, where
//! each yield lets other tasks run. A [`Pacer`] decides where a computation
//! yields: after a budget of work, never after an amount of time, so a
//! computation yields at the same points on every device.

use std::future::{Future, IntoFuture, poll_fn};
use std::pin::pin;
#[cfg(not(target_family = "wasm"))]
use std::sync::Arc;
#[cfg(not(target_family = "wasm"))]
use std::task::Wake;
use std::task::{Context, Poll, Waker};
#[cfg(not(target_family = "wasm"))]
use std::thread::{self, Thread};

/// Lets the executor run other tasks once: the returned future wakes its
/// task and pends, then completes on its next poll.
pub fn yield_now() -> impl Future<Output = ()> + Send {
    let mut yielded = false;
    poll_fn(move |context| {
        if yielded {
            return Poll::Ready(());
        }
        yielded = true;
        context.waker().wake_by_ref();
        Poll::Pending
    })
}

/// Counts the units of work a computation does, and yields each time it has
/// spent a budget of them.
///
/// It counts work, never time, so a computation paced by it yields at the
/// same points on every device, however fast.
#[derive(Debug, Clone, Copy)]
pub struct Pacer {
    remaining: u64,
    units_per_yield: u64,
}

impl Pacer {
    /// Returns a pacer that yields once per `units_per_yield` units of work;
    /// with 0, it yields at every advance.
    #[must_use]
    pub const fn new(units_per_yield: u64) -> Self {
        Self {
            remaining: units_per_yield,
            units_per_yield,
        }
    }

    /// Records `units` more units of work, yielding when they spend what is
    /// left of the budget, which then starts again in full.
    pub async fn advance(&mut self, units: u64) {
        if units < self.remaining {
            self.remaining -= units;
        } else {
            self.remaining = self.units_per_yield;
            yield_now().await;
        }
    }
}

/// A future that pends only by yielding, through [`yield_now`] or a
/// [`Pacer`]: it never waits for another task, a lock, or a timer.
///
/// [`complete`](Self::complete) runs it to completion without an executor,
/// and it can be awaited like any future, since it converts into one.
///
/// # Examples
///
/// ```
/// use eranos::cooperative::{Cooperative, block_on};
///
/// // Run to completion on this thread, or awaited like any future.
/// assert_eq!(Cooperative::new(async { 6 * 7 }).complete(), 42);
/// assert_eq!(block_on(Cooperative::new(async { 6 * 7 })), 42);
/// ```
#[derive(Debug)]
#[must_use = "a cooperative future does nothing unless it is completed or awaited"]
pub struct Cooperative<F> {
    future: F,
}

impl<F: Future> Cooperative<F> {
    /// Marks `future` as cooperative.
    ///
    /// The future must pend only by yielding: [`complete`](Self::complete)
    /// polls it again at once after every `Pending`, so a future that waits
    /// for something else keeps the thread busy until that is ready.
    pub const fn new(future: F) -> Self {
        Self { future }
    }

    /// Runs the future to completion on this thread, polling it again at once
    /// whenever it yields.
    pub fn complete(self) -> F::Output {
        let mut future = pin!(self.future);
        let mut context = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
                return output;
            }
        }
    }
}

impl<F: Future> IntoFuture for Cooperative<F> {
    type Output = F::Output;
    type IntoFuture = F;

    fn into_future(self) -> F {
        self.future
    }
}

/// Runs `future` to completion on this thread, which sleeps while the future
/// waits.
///
/// The thread wakes when the future's waker is called, which makes this the
/// driver for a future that waits for another thread, such as one for a
/// shared group's lock. It is not compiled for wasm, where the one thread would sleep with no
/// thread left to wake it.
///
/// # Examples
///
/// ```
/// use eranos::cooperative::block_on;
///
/// assert_eq!(block_on(async { 6 * 7 }), 42);
/// ```
#[cfg(not(target_family = "wasm"))]
pub fn block_on<F: IntoFuture>(future: F) -> F::Output {
    let mut future = pin!(future.into_future());
    let waker = Waker::from(Arc::new(Unparker(thread::current())));
    let mut context = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
            return output;
        }
        // A wake before the park makes it return at once, and a spurious
        // return only polls again.
        thread::park();
    }
}

/// The waker of [`block_on`]: it wakes the thread that drives the future.
#[cfg(not(target_family = "wasm"))]
struct Unparker(Thread);

#[cfg(not(target_family = "wasm"))]
impl Wake for Unparker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

#[cfg(test)]
mod tests;
