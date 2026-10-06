//! Cancelling a call. A cancel that did not wake a waiting call would leave
//! it waiting for a lock it no longer needs, a cancelled call that still ran
//! its work would change the group after its caller gave it up, and a
//! cancellation that kept the tasks of the calls that ended would grow with
//! every call it served.

use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::task::{Context, Wake};

use super::*;

/// A task that counts how often it is woken.
#[derive(Default)]
struct Task {
    wakes: AtomicUsize,
}

impl Task {
    fn wakes(&self) -> usize {
        self.wakes.load(Ordering::SeqCst)
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wakes.fetch_add(1, Ordering::SeqCst);
    }
}

/// Work that never completes, and records that it was dropped.
#[derive(Default)]
struct Endless {
    dropped: Arc<AtomicBool>,
}

impl Future for Endless {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}

impl Drop for Endless {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

fn waker(task: &Arc<Task>) -> Waker {
    Waker::from(Arc::clone(task))
}

fn waiting(cancellation: &Cancellation) -> usize {
    cancellation.waiting().len()
}

/// A call given no cancellation would fail, or would not run its work.
#[test]
fn a_call_without_a_cancellation_runs_its_work() {
    let waker = waker(&Arc::new(Task::default()));
    let call = pin!(cancellable(async { 7 }, None));
    assert_eq!(
        call.poll(&mut Context::from_waker(&waker)),
        Poll::Ready(Ok(7))
    );
}

/// A call given a cancelled cancellation would run its work, which changes
/// the group, after its caller gave it up.
#[test]
fn a_call_cancelled_before_it_starts_never_runs_its_work() {
    let cancellation = Arc::new(Cancellation::new());
    cancellation.cancel();
    let ran = Arc::new(AtomicBool::new(false));
    let work = {
        let ran = Arc::clone(&ran);
        async move { ran.store(true, Ordering::SeqCst) }
    };
    let waker = waker(&Arc::new(Task::default()));
    let call = pin!(cancellable(work, Some(Arc::clone(&cancellation))));
    assert_eq!(
        call.poll(&mut Context::from_waker(&waker)),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert!(!ran.load(Ordering::SeqCst));
    assert_eq!(waiting(&cancellation), 0);
}

/// A cancel that did not wake a waiting call would leave it waiting for a
/// lock or a yield it no longer needs, and one that did not drop the work
/// would keep what the work holds, the group's lock among it.
#[test]
fn cancelling_wakes_a_waiting_call_which_drops_its_work() {
    let cancellation = Arc::new(Cancellation::new());
    let task = Arc::new(Task::default());
    let waker = waker(&task);
    let mut context = Context::from_waker(&waker);
    let work = Endless::default();
    let dropped = Arc::clone(&work.dropped);
    let mut call = pin!(cancellable(work, Some(Arc::clone(&cancellation))));
    assert_eq!(call.as_mut().poll(&mut context), Poll::Pending);
    assert_eq!(call.as_mut().poll(&mut context), Poll::Pending);
    // Polled twice by one task, the call waits under one entry.
    assert_eq!(waiting(&cancellation), 1);
    assert!(!cancellation.is_cancelled());

    cancellation.cancel();
    assert!(cancellation.is_cancelled());
    assert_eq!(task.wakes(), 1);
    assert!(!dropped.load(Ordering::SeqCst));
    assert_eq!(
        call.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(waiting(&cancellation), 0);
}

/// A call moved to another task would be woken on the task it left, which
/// no longer polls it, and would never see the cancel.
#[test]
fn a_call_polled_by_another_task_is_woken_there() {
    let cancellation = Arc::new(Cancellation::new());
    let (first, second) = (Arc::new(Task::default()), Arc::new(Task::default()));
    let mut call = pin!(cancellable(
        Endless::default(),
        Some(Arc::clone(&cancellation))
    ));
    for task in [&first, &second] {
        let waker = waker(task);
        assert_eq!(
            call.as_mut().poll(&mut Context::from_waker(&waker)),
            Poll::Pending
        );
    }
    assert_eq!(waiting(&cancellation), 1);
    cancellation.cancel();
    assert_eq!(first.wakes(), 0);
    assert_eq!(second.wakes(), 1);
}

/// A cancellation serving many calls would keep the task of every call that
/// ended, a call ending on a task would take the entry of another call of
/// that task with it, as a join polls them, and one cancel would miss calls
/// in progress.
#[test]
fn a_cancellation_keeps_only_the_calls_in_progress_and_stops_them_all() {
    let cancellation = Arc::new(Cancellation::new());
    let (joined, alone) = (Arc::new(Task::default()), Arc::new(Task::default()));
    let (joined_waker, alone_waker) = (waker(&joined), waker(&alone));
    let mut joined_context = Context::from_waker(&joined_waker);
    let mut alone_context = Context::from_waker(&alone_waker);

    // A call dropped while it waits leaves nothing behind.
    {
        let work = Endless::default();
        let dropped = Arc::clone(&work.dropped);
        let call = pin!(cancellable(work, Some(Arc::clone(&cancellation))));
        assert_eq!(call.poll(&mut alone_context), Poll::Pending);
        assert_eq!(waiting(&cancellation), 1);
        assert!(!dropped.load(Ordering::SeqCst));
    }
    assert_eq!(waiting(&cancellation), 0);

    // Of two calls of one task, the one that completes leaves the other's
    // entry in place.
    let mut waits = pin!(cancellable(
        Endless::default(),
        Some(Arc::clone(&cancellation))
    ));
    assert_eq!(waits.as_mut().poll(&mut joined_context), Poll::Pending);
    let completes = pin!(cancellable(async { 1 }, Some(Arc::clone(&cancellation))));
    assert_eq!(completes.poll(&mut joined_context), Poll::Ready(Ok(1)));
    assert_eq!(waiting(&cancellation), 1);

    // One cancel stops the calls of every task.
    let mut other = pin!(cancellable(
        Endless::default(),
        Some(Arc::clone(&cancellation))
    ));
    assert_eq!(other.as_mut().poll(&mut alone_context), Poll::Pending);
    assert_eq!(waiting(&cancellation), 2);
    cancellation.cancel();
    assert_eq!(joined.wakes(), 1);
    assert_eq!(alone.wakes(), 1);
    let cancelled = Poll::Ready(Err(EranosError::cancelled()));
    assert_eq!(waits.poll(&mut joined_context), cancelled);
    assert_eq!(other.poll(&mut alone_context), cancelled);

    // Cancelling again wakes no one.
    cancellation.cancel();
    assert_eq!(joined.wakes(), 1);
    assert_eq!(alone.wakes(), 1);
    assert_eq!(waiting(&cancellation), 0);
}
