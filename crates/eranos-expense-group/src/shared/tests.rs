//! Tests for the shared group. Apps call it from several tasks and threads
//! at once, so these pin how its guards wait for each other (reads
//! together, changes alone, a waiting change ahead of new reads), that a
//! guard keeps the lock across a yield and releases it when it is dropped,
//! that a wait dropped before the lock is granted takes nothing, that the
//! tries and the blocking calls take the lock as they say, and that a wait
//! for the lock can move to another thread and stays small.

use std::cell::Cell;
use std::future::poll_fn;
use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

use eranos_calendar::CalendarDate;
use eranos_core::MemberId;
use eranos_core::cooperative::block_on;
use eranos_expense::{Expense, Split};
use eranos_member::Member;
use eranos_money::{CurrencyCode, Money};
use eranos_settlement::SettlementStrategy;

use super::*;

fn day() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn person(name: &str) -> Member {
    Member::builder(name).build().unwrap()
}

fn shared_by(amount: Money, payer: MemberId, participants: &[MemberId]) -> Expense {
    let split = Split::equal(participants.iter().copied());
    Expense::builder("Shared", amount, payer, split, day())
        .build()
        .unwrap()
}

/// A group in which sixteen members each paid a different amount for a
/// seventeenth: seventeen open balances, so the exact solver yields once,
/// holding the shared lock through its guard.
fn owed_by_a_hub() -> ExpenseGroup {
    let mut group = ExpenseGroup::builder("Club", CurrencyCode::EUR)
        .build()
        .unwrap();
    let hub = group.add_member(person("Hub")).unwrap();
    for minor_units in 1..=16 {
        let payer = group.add_member(person("Member")).unwrap();
        group
            .add_expense(shared_by(eur(minor_units), payer, &[hub]))
            .unwrap();
    }
    group
}

/// A waker that counts how often it was woken.
#[derive(Default)]
struct Counter(AtomicUsize);

impl Counter {
    fn woken(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

impl Wake for Counter {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// The size of a wait's future, which the bound keeps `Send + 'static`.
fn size_of_wait<F: Future + Send + 'static>(wait: &F) -> usize {
    size_of_val(wait)
}

/// Reads that waited for each other would slow a screen of totals down,
/// and a change that did not wait for them could show a total half
/// changed: a read guard runs alongside a settlement at its yield, a change
/// waits for both, and a new read waits behind the change so that it
/// cannot starve.
#[test]
fn reads_share_the_lock_and_a_change_waits_its_turn() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    let counter = Arc::new(Counter::default());
    let waker = Waker::from(Arc::clone(&counter));
    let mut context = Context::from_waker(&waker);

    // The settlement yields once, holding the shared lock through its guard.
    let settling = shared.clone();
    let mut settlement = pin!(async move {
        let group = settling.read().await;
        group
            .settlement_async(SettlementStrategy::MinimumTransfers)
            .await
            .cloned()
    });
    assert!(settlement.as_mut().poll(&mut context).is_pending());
    // Another read runs alongside it.
    let Poll::Ready(reading) = pin!(shared.read()).poll(&mut context) else {
        panic!("a read waited for a settlement at its yield");
    };
    assert_eq!(reading.name(), "Club");
    // A change waits for the reads, and a new read waits behind it.
    let mut change = pin!(shared.write());
    assert!(change.as_mut().poll(&mut context).is_pending());
    let mut later = pin!(shared.read());
    assert!(later.as_mut().poll(&mut context).is_pending());

    // The reads complete and wake the change, which runs alone, then wakes
    // the later read.
    drop(reading);
    let woken_before_settling = counter.woken();
    let Poll::Ready(Ok(plan)) = settlement.as_mut().poll(&mut context) else {
        panic!("the settlement did not complete after its one yield");
    };
    assert_eq!(plan.len(), 16);
    assert!(counter.woken() > woken_before_settling);
    let Poll::Ready(mut writing) = change.as_mut().poll(&mut context) else {
        panic!("the change did not get the lock once the reads were done");
    };
    writing.set_name("Society").unwrap();
    assert!(later.as_mut().poll(&mut context).is_pending());
    let woken_before_releasing = counter.woken();
    drop(writing);
    assert!(counter.woken() > woken_before_releasing);
    let Poll::Ready(later) = later.as_mut().poll(&mut context) else {
        panic!("the later read did not get the lock once the change was done");
    };
    assert_eq!(later.name(), "Society");
}

/// A wait dropped before the lock is granted would leave the lock marked
/// for a change that never comes, or held by nobody: it takes nothing, and
/// the next call takes the lock at once.
#[test]
fn a_dropped_wait_takes_nothing() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    let mut context = Context::from_waker(Waker::noop());
    {
        // A report holds the shared lock; a change waits for it, and a read
        // waits behind the change. Both waits are dropped before the guard.
        let report = shared.read_blocking();
        let mut change = pin!(shared.write());
        assert!(change.as_mut().poll(&mut context).is_pending());
        let mut later = pin!(shared.read());
        assert!(later.as_mut().poll(&mut context).is_pending());
        assert_eq!(report.name(), "Club");
    }
    assert!(shared.try_write().is_ok());
    assert_eq!(shared.read_blocking().name(), "Club");
}

/// `Debug` that locked would wait for a long change, or deadlock a caller
/// that holds the lock: it prints no detail of the group, even while a
/// change holds the lock.
#[test]
fn debug_never_locks() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    let writing = shared.write_blocking();
    assert_eq!(format!("{shared:?}"), "SharedExpenseGroup { .. }");
    drop(writing);
}

/// An executor spawns a wait for the lock onto any of its threads, so a
/// wait that was not `Send + 'static` could not be spawned, and one larger
/// than a few kilobytes would cost a large copy every time it is spawned.
#[test]
fn a_wait_for_the_lock_is_send_static_and_small() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    let largest = size_of_wait(&shared.read()).max(size_of_wait(&shared.write()));
    assert!(4096 >= largest, "a wait takes {largest} bytes");
}

/// A guard that released the lock early would let another call in between
/// two reads or changes, and one that kept it would block every call after
/// it: the group's own methods run under the guard, both tries fail as busy
/// while a change holds the lock, a read succeeds beside a read while a
/// change fails, and the lock is free once the guard is dropped.
#[test]
fn a_guard_holds_the_lock_until_it_is_dropped() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    {
        let mut guard = shared.write_blocking();
        assert_eq!(shared.try_read().map(|_| ()), Err(ExpenseGroupError::Busy));
        assert_eq!(shared.try_write().map(|_| ()), Err(ExpenseGroupError::Busy));
        // The guard reads the group as well as changing it.
        assert_eq!(guard.name(), "Club");
        guard.set_name("Society").unwrap();
    }
    let guard = shared.read_blocking();
    assert_eq!(guard.name(), "Society");
    // Reads share the lock, which a change cannot take meanwhile.
    assert_eq!(
        shared.try_read().map(|other| other.name().to_owned()),
        Ok("Society".to_owned())
    );
    assert_eq!(shared.try_write().map(|_| ()), Err(ExpenseGroupError::Busy));
    drop(guard);
    assert!(shared.try_write().is_ok());
}

/// A synchronous caller would spin while another thread holds the lock, or
/// never wake when it lets go: the wait sleeps, then reads what the holder
/// wrote as soon as it releases the lock.
#[test]
fn a_blocked_wait_sleeps_until_the_lock_is_released() {
    let shared = SharedExpenseGroup::new(owed_by_a_hub());
    let (locked, holding) = mpsc::channel();
    let holder = {
        let shared = shared.clone();
        thread::spawn(move || {
            let mut guard = shared.write_blocking();
            locked.send(()).unwrap();
            thread::sleep(Duration::from_millis(50));
            guard.set_name("Society").unwrap();
        })
    };
    holding.recv().unwrap();
    let polls = Cell::new(0);
    let mut read = pin!(shared.read());
    let group = block_on(poll_fn(|context| {
        polls.set(polls.get() + 1);
        read.as_mut().poll(context)
    }));
    holder.join().unwrap();
    assert_eq!(group.name(), "Society");
    // One poll to wait and one to read, with room for a spurious wake-up;
    // a driver that spun would have polled thousands of times in 50 ms.
    assert!(3 >= polls.get(), "{} polls", polls.get());
}

/// A group taken back while another handle or a guard still shared it
/// would change under that handle's calls: it comes back only from the
/// last handle.
#[test]
fn the_group_comes_back_only_from_its_last_handle() {
    let group = owed_by_a_hub();
    let shared = SharedExpenseGroup::new(group.clone());
    let other = shared.clone();
    let shared = shared
        .into_inner()
        .expect_err("a clone still shares the group");
    drop(other);
    let reading = shared.read_blocking();
    let shared = shared
        .into_inner()
        .expect_err("a guard still shares the group");
    drop(reading);
    assert_eq!(shared.into_inner().ok(), Some(group));
}
