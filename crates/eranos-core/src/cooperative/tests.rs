//! Cooperative futures let the library's long computations pause without an
//! executor. These tests keep a yield announcing itself before it pends,
//! keep a pacer yielding exactly once per budget, keep the no-op driver
//! polling until the future is ready, and keep the parking driver asleep
//! while its future waits and awake once it is woken.

use std::cell::Cell;
use std::future::poll_fn;
#[cfg(not(target_family = "wasm"))]
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(not(target_family = "wasm"))]
use std::time::Duration;

use super::*;

/// A waker that counts how often it was woken.
#[derive(Default)]
struct Counter(AtomicUsize);

impl Wake for Counter {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Polls `future` until it is ready, and returns its output and how many
/// polls pended; fails when a poll pends without waking the task first.
fn run<F: Future>(future: F) -> (F::Output, usize) {
    let counter = Arc::new(Counter::default());
    let waker = Waker::from(Arc::clone(&counter));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    let mut pending = 0;
    loop {
        let woken_before = counter.0.load(Ordering::SeqCst);
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return (output, pending),
            Poll::Pending => {
                assert!(
                    counter.0.load(Ordering::SeqCst) > woken_before,
                    "a poll pended without waking its task"
                );
                pending += 1;
            }
        }
    }
}

/// A yield that pended without waking its task would stall every executor
/// but one that spins.
#[test]
fn a_yield_wakes_its_task_and_pends_once() {
    assert_eq!(run(yield_now()), ((), 1));
}

/// A computation would yield more often than its budget says, which slows it
/// down, or less often, which keeps the other tasks waiting.
#[test]
fn a_pacer_yields_once_per_budget() {
    let mut pacer = Pacer::new(10);
    // Within the budget, no yield.
    assert_eq!(run(pacer.advance(3)).1, 0);
    assert_eq!(run(pacer.advance(6)).1, 0);
    // The unit that spends it yields, and the budget starts again in full.
    assert_eq!(run(pacer.advance(1)).1, 1);
    assert_eq!(run(pacer.advance(9)).1, 0);
    // More than the budget at once yields once.
    assert_eq!(run(pacer.advance(25)).1, 1);
    // A budget of nothing yields at every advance.
    let mut eager = Pacer::new(0);
    assert_eq!(run(eager.advance(0)).1, 1);
    assert_eq!(run(eager.advance(0)).1, 1);
}

/// A computation run without an executor would stop at its first yield
/// instead of running to its result.
#[test]
fn complete_polls_through_every_yield() {
    let polls = Cell::new(0);
    let mut future = Box::pin(async {
        for _ in 0..3 {
            yield_now().await;
        }
        42
    });
    let counted = poll_fn(|context| {
        polls.set(polls.get() + 1);
        future.as_mut().poll(context)
    });
    assert_eq!(Cooperative::new(counted).complete(), 42);
    // Three yields, then the result.
    assert_eq!(polls.get(), 4);
}

/// A cooperative call awaited in an executor would yield differently from
/// the same call completed on its own.
#[test]
fn a_cooperative_future_awaits_as_the_future_it_marks() {
    let outer = async {
        Cooperative::new(async {
            yield_now().await;
            42
        })
        .await
    };
    assert_eq!(run(outer), (42, 1));
}

/// The thread of a synchronous call would spin while the call waits for
/// another thread, or never wake when the other thread is done.
#[cfg(not(target_family = "wasm"))]
#[test]
fn block_on_sleeps_until_its_future_is_woken() {
    // Whether the other thread is done, and the waker to call when it is.
    let shared = Arc::new(Mutex::new((false, None::<Waker>)));
    let signal = Arc::clone(&shared);
    let polls = Cell::new(0);
    let waiting = poll_fn(|context| {
        polls.set(polls.get() + 1);
        let mut state = shared.lock().unwrap();
        if state.0 {
            Poll::Ready(42)
        } else {
            state.1 = Some(context.waker().clone());
            Poll::Pending
        }
    });
    let other = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        let mut state = signal.lock().unwrap();
        state.0 = true;
        if let Some(waker) = state.1.take() {
            waker.wake();
        }
    });
    assert_eq!(block_on(waiting), 42);
    other.join().unwrap();
    // One poll to wait and one to finish, with room for a spurious wake-up; a
    // driver that spun would have polled thousands of times in 50 ms.
    assert!(polls.get() <= 3, "{} polls", polls.get());
    // A yield wakes the thread before it sleeps, so it polls again at once.
    assert_eq!(
        block_on(async {
            yield_now().await;
            42
        }),
        42
    );
}
