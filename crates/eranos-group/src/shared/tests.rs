//! Tests for the shared group. Apps call it from several tasks and threads
//! at once, so these pin what each call returns (what the group's own
//! method returns), how calls wait for each other (reads together, changes
//! alone, a waiting change ahead of new reads), that a dropped call changes
//! nothing and releases the lock, that the guards, the tries, and the
//! blocking calls take the lock as they say, and that every call's future
//! can move to another thread and stays small.

use std::cell::Cell;
use std::future::poll_fn;
use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

use eranos_calendar::CalendarDate;
use eranos_core::cooperative::block_on;
use eranos_expense::Split;
use eranos_money::ExchangeRate;

use super::*;

fn day() -> CalendarDate {
    CalendarDate::new(23, 9, 2026).unwrap()
}

fn eur(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

fn person(name: &str) -> Member {
    Member::builder(name).build().unwrap()
}

fn shared_by(amount: Money, payer: MemberId, participants: &[MemberId]) -> Expense {
    let split = Split::equal(participants.iter().copied());
    Expense::builder(amount, day(), "Shared", payer, split)
        .build()
        .unwrap()
}

/// A group in which sixteen members each paid a different amount for a
/// seventeenth: seventeen open balances, so the exact solver yields once,
/// holding the shared lock.
fn owed_by_a_hub() -> Group {
    let mut group = Group::builder(Currency::EUR, "Club").build().unwrap();
    let hub = group.add_member(person("Hub")).unwrap();
    for minor_units in 1..=16 {
        let payer = group.add_member(person("Member")).unwrap();
        group
            .add_expense(shared_by(eur(minor_units), payer, &[hub]))
            .unwrap();
    }
    group
}

/// A group of 2 100 expenses, so a currency change yields twice, holding
/// the exclusive lock.
fn busy() -> Group {
    let mut group = Group::builder(Currency::EUR, "Office").build().unwrap();
    let ana = group.add_member(person("Ana")).unwrap();
    let bob = group.add_member(person("Bob")).unwrap();
    for _ in 0..2_100 {
        group
            .add_expense(shared_by(eur(100), ana, &[ana, bob]))
            .unwrap();
    }
    group
}

/// The rates that move [`busy`] to lei.
fn lei() -> [CurrencyRate; 1] {
    [CurrencyRate::new(
        Currency::EUR,
        ExchangeRate::parse("5").unwrap(),
    )]
}

/// Everything a caller can read of a group, owned.
#[derive(Debug, PartialEq)]
struct Contents {
    categories: Vec<Category>,
    currency: Currency,
    expenses: Vec<Expense>,
    members: Vec<Member>,
    name: String,
    summary: Summary,
    transfers: Vec<Transfer>,
    description: Option<String>,
    language: Option<Language>,
}

fn contents_of(group: &Group) -> Contents {
    Contents {
        categories: group.categories().cloned().collect(),
        currency: group.currency(),
        expenses: group.expenses().cloned().collect(),
        members: group.members().cloned().collect(),
        name: group.name().to_owned(),
        summary: group.summary().clone(),
        transfers: group.transfers().cloned().collect(),
        description: group.description().map(str::to_owned),
        language: group.language().cloned(),
    }
}

fn shared_contents(shared: &SharedGroup) -> Contents {
    block_on(async {
        Contents {
            categories: shared.categories().await,
            currency: shared.currency().await,
            expenses: shared.expenses().await,
            members: shared.members().await,
            name: shared.name().await,
            summary: shared.summary().await,
            transfers: shared.transfers().await,
            description: shared.description().await,
            language: shared.language().await,
        }
    })
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

/// The size of a call's future, which the bound keeps `Send + 'static`.
fn size_of_call<F: Future + Send + 'static>(call: &F) -> usize {
    size_of_val(call)
}

/// A group with an entry of every kind, and the identifiers the tests read
/// them by.
struct Trip {
    ana: MemberId,
    bob: MemberId,
    dinner: ExpenseId,
    food: CategoryId,
    group: Group,
    repayment: TransferId,
}

fn trip() -> Trip {
    let mut group = Group::builder(Currency::EUR, "Trip")
        .description("Autumn")
        .build()
        .unwrap();
    let ana = group.add_member(person("Ana")).unwrap();
    let bob = group.add_member(person("Bob")).unwrap();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let dinner = shared_by(eur(3_000), ana, &[ana, bob])
        .to_builder()
        .category(food)
        .build()
        .unwrap();
    let dinner = group.add_expense(dinner).unwrap();
    let repayment = Transfer::builder(eur(500), day(), bob, ana)
        .build()
        .unwrap();
    let repayment = group.add_transfer(repayment).unwrap();
    Trip {
        ana,
        bob,
        dinner,
        food,
        group,
        repayment,
    }
}

/// A read that returned something other than the group's own method would
/// make the shared group a second implementation of every rule.
#[test]
fn every_read_returns_what_the_group_returns() {
    let Trip {
        ana,
        dinner,
        food,
        group,
        repayment,
        ..
    } = trip();
    let shared = SharedGroup::new(group.clone());
    assert_eq!(block_on(shared.id()), group.id());
    assert_eq!(
        block_on(shared.category(food)),
        group.category(food).cloned()
    );
    assert_eq!(
        block_on(shared.expense(dinner)),
        group.expense(dinner).cloned()
    );
    assert_eq!(
        block_on(shared.expense_shares(dinner)),
        group.expense_shares(dinner).cloned()
    );
    assert_eq!(block_on(shared.member(ana)), group.member(ana).cloned());
    assert_eq!(block_on(shared.is_archived(ana)), group.is_archived(ana));
    for strategy in SettlementStrategy::all() {
        assert_eq!(
            block_on(shared.settlement(strategy)),
            group.settlement(strategy).cloned()
        );
    }
    assert_eq!(
        block_on(shared.transfer(repayment)),
        group.transfer(repayment).cloned()
    );
    assert_eq!(
        block_on(shared.transfer_amount(repayment)),
        group.transfer_amount(repayment)
    );
    assert_eq!(shared_contents(&shared), contents_of(&group));
}

/// A change that did something other than the group's own method would
/// make the shared group a second implementation of every rule: each
/// change, refused ones included, returns what the group's method returns
/// and leaves the group as that method leaves it.
#[test]
fn every_change_does_what_the_group_does() {
    let Trip {
        ana,
        bob,
        mut group,
        ..
    } = trip();
    let shared = SharedGroup::new(group.clone());
    let carol = person("Carol");
    let caroline = carol.to_builder().first_name("Caroline").build().unwrap();
    assert_eq!(
        block_on(shared.add_member(carol.clone())),
        group.add_member(carol.clone())
    );
    assert_eq!(
        block_on(shared.replace_member(caroline.clone())),
        group.replace_member(caroline)
    );
    assert_eq!(
        block_on(shared.archive_member(bob)),
        group.archive_member(bob)
    );
    assert_eq!(
        block_on(shared.unarchive_member(bob)),
        group.unarchive_member(bob)
    );
    assert_eq!(
        block_on(shared.remove_member(carol.id())),
        group.remove_member(carol.id())
    );
    // Ana paid for dinner, so she cannot be removed.
    assert_eq!(
        block_on(shared.remove_member(ana)),
        group.remove_member(ana)
    );

    let transport = Category::new("Transport").unwrap();
    let travel = transport.renamed("Travel").unwrap();
    assert_eq!(
        block_on(shared.add_category(transport.clone())),
        group.add_category(transport)
    );
    assert_eq!(
        block_on(shared.replace_category(travel.clone())),
        group.replace_category(travel.clone())
    );
    assert_eq!(
        block_on(shared.remove_category(travel.id())),
        group.remove_category(travel.id())
    );

    let lunch = shared_by(eur(1_200), bob, &[ana, bob]);
    let dearer = lunch.to_builder().cost(eur(1_500)).build().unwrap();
    assert_eq!(
        block_on(shared.add_expense(lunch.clone())),
        group.add_expense(lunch.clone())
    );
    assert_eq!(
        block_on(shared.replace_expense(dearer.clone())),
        group.replace_expense(dearer)
    );
    assert_eq!(
        block_on(shared.remove_expense(lunch.id())),
        group.remove_expense(lunch.id())
    );

    let cash = Transfer::builder(eur(300), day(), ana, bob)
        .build()
        .unwrap();
    let more = cash.to_builder().cost(eur(400)).build().unwrap();
    assert_eq!(
        block_on(shared.add_transfer(cash.clone())),
        group.add_transfer(cash.clone())
    );
    assert_eq!(
        block_on(shared.replace_transfer(more.clone())),
        group.replace_transfer(more)
    );
    assert_eq!(
        block_on(shared.remove_transfer(cash.id())),
        group.remove_transfer(cash.id())
    );

    // A blank name is refused.
    assert_eq!(block_on(shared.set_name("  ")), group.set_name("  "));
    assert_eq!(
        block_on(shared.set_name("Lisbon")),
        group.set_name("Lisbon")
    );
    block_on(shared.set_description("Spring"));
    group.set_description("Spring");
    let portuguese = Language::parse("pt-PT").unwrap();
    block_on(shared.set_language(portuguese.clone()));
    group.set_language(portuguese);
    assert_eq!(
        block_on(shared.change_currency(Currency::RON, lei())),
        group.change_currency(Currency::RON, &lei())
    );

    assert_eq!(shared_contents(&shared), contents_of(&group));
}

/// Reads that waited for each other would slow a screen of totals down,
/// and a change that did not wait for them could show a total half
/// changed: reads run alongside a settlement at its yield, a change waits
/// for it, and new reads wait behind the change so that it cannot starve.
#[test]
fn reads_share_the_lock_and_a_change_waits_its_turn() {
    let shared = SharedGroup::new(owed_by_a_hub());
    let counter = Arc::new(Counter::default());
    let waker = Waker::from(Arc::clone(&counter));
    let mut context = Context::from_waker(&waker);

    // The settlement yields once, holding the shared lock.
    let mut settlement = pin!(shared.settlement(SettlementStrategy::MinimumTransfers));
    assert!(settlement.as_mut().poll(&mut context).is_pending());
    // Another read runs alongside it.
    assert_eq!(
        pin!(shared.name()).poll(&mut context),
        Poll::Ready("Club".to_owned())
    );
    // A change waits for the settlement, and a new read waits behind it.
    let mut rename = pin!(shared.set_name("Society"));
    assert!(rename.as_mut().poll(&mut context).is_pending());
    let mut name = pin!(shared.name());
    assert!(name.as_mut().poll(&mut context).is_pending());

    // The settlement completes and wakes the change, which runs alone,
    // then wakes the read.
    let woken_before_settling = counter.woken();
    let Poll::Ready(Ok(plan)) = settlement.as_mut().poll(&mut context) else {
        panic!("the settlement did not complete after its one yield");
    };
    assert_eq!(plan.len(), 16);
    assert!(counter.woken() > woken_before_settling);
    let woken_before_renaming = counter.woken();
    assert_eq!(rename.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert!(counter.woken() > woken_before_renaming);
    assert_eq!(
        name.as_mut().poll(&mut context),
        Poll::Ready("Society".to_owned())
    );
}

/// A call dropped at a yield, or while it waits for the lock, would leave
/// the lock held or the group half changed: it changes nothing, and the
/// next call takes the lock at once.
#[test]
fn a_dropped_call_changes_nothing_and_releases_the_lock() {
    let shared = SharedGroup::new(busy());
    let before = shared_contents(&shared);
    let mut context = Context::from_waker(Waker::noop());
    {
        // The change holds the exclusive lock at its first yield, and a
        // read waits for it.
        let mut change = pin!(shared.change_currency(Currency::RON, lei()));
        assert!(change.as_mut().poll(&mut context).is_pending());
        let mut read = pin!(shared.currency());
        assert!(read.as_mut().poll(&mut context).is_pending());
    }
    assert_eq!(
        pin!(shared.currency()).poll(&mut context),
        Poll::Ready(Currency::EUR)
    );
    assert_eq!(shared_contents(&shared), before);
}

/// `Debug` that locked would wait for a long change, or deadlock a caller
/// that holds the lock: it prints no detail of the group, even while a
/// change holds the lock.
#[test]
fn debug_never_locks() {
    let shared = SharedGroup::new(busy());
    let mut context = Context::from_waker(Waker::noop());
    let mut change = pin!(shared.change_currency(Currency::RON, lei()));
    assert!(change.as_mut().poll(&mut context).is_pending());
    assert_eq!(format!("{shared:?}"), "SharedGroup { .. }");
}

/// An executor spawns a call's future onto any of its threads, so a future
/// that was not `Send + 'static` could not be spawned, and one larger than
/// a few kilobytes would cost a large copy every time it is spawned.
#[test]
fn every_call_is_send_static_and_small() {
    let shared = SharedGroup::new(owed_by_a_hub());
    let member = person("Ana");
    let other = person("Bob");
    let category = Category::new("Food").unwrap();
    let expense = shared_by(eur(100), member.id(), &[member.id()]);
    let transfer = Transfer::builder(eur(100), day(), member.id(), other.id())
        .build()
        .unwrap();
    let sizes = [
        size_of_call(&shared.id()),
        size_of_call(&shared.categories()),
        size_of_call(&shared.category(category.id())),
        size_of_call(&shared.add_category(category.clone())),
        size_of_call(&shared.replace_category(category.clone())),
        size_of_call(&shared.remove_category(category.id())),
        size_of_call(&shared.currency()),
        size_of_call(&shared.change_currency(Currency::RON, lei())),
        size_of_call(&shared.expenses()),
        size_of_call(&shared.expense(expense.id())),
        size_of_call(&shared.expense_shares(expense.id())),
        size_of_call(&shared.add_expense(expense.clone())),
        size_of_call(&shared.replace_expense(expense.clone())),
        size_of_call(&shared.remove_expense(expense.id())),
        size_of_call(&shared.summary()),
        size_of_call(&shared.members()),
        size_of_call(&shared.member(member.id())),
        size_of_call(&shared.is_archived(member.id())),
        size_of_call(&shared.add_member(member.clone())),
        size_of_call(&shared.replace_member(member.clone())),
        size_of_call(&shared.archive_member(member.id())),
        size_of_call(&shared.unarchive_member(member.id())),
        size_of_call(&shared.remove_member(member.id())),
        size_of_call(&shared.name()),
        size_of_call(&shared.set_name("Club")),
        size_of_call(&shared.settlement(SettlementStrategy::MinimumTransfers)),
        size_of_call(&shared.transfers()),
        size_of_call(&shared.transfer(transfer.id())),
        size_of_call(&shared.transfer_amount(transfer.id())),
        size_of_call(&shared.add_transfer(transfer.clone())),
        size_of_call(&shared.replace_transfer(transfer.clone())),
        size_of_call(&shared.remove_transfer(transfer.id())),
        size_of_call(&shared.description()),
        size_of_call(&shared.set_description("Weekly")),
        size_of_call(&shared.language()),
        size_of_call(&shared.set_language(None)),
        size_of_call(&shared.read()),
        size_of_call(&shared.write()),
    ];
    let largest = sizes.iter().max().copied().unwrap();
    assert!(largest <= 4096, "a call's future takes {largest} bytes");
}

/// A guard that released the lock early would let another call in between
/// two reads or changes, and one that kept it would block every call after
/// it: the group's own methods run under the guard, and the lock is free
/// once it is dropped.
#[test]
fn a_guard_holds_the_lock_until_it_is_dropped() {
    let shared = SharedGroup::new(owed_by_a_hub());
    {
        let mut guard = shared.write_blocking();
        assert!(shared.is_locked());
        assert_eq!(shared.try_read().map(|_| ()), Err(GroupError::Busy));
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
    assert_eq!(shared.try_write().map(|_| ()), Err(GroupError::Busy));
    drop(guard);
    assert!(!shared.is_locked());
    assert!(shared.try_write().is_ok());
}

/// On wasm a synchronous call cannot wait, so it tries the lock instead: a
/// try that succeeded under a change in progress would see the group half
/// changed. Both tries fail as busy while a change holds the lock at a
/// yield, and a read succeeds alongside a settlement at its yield.
#[test]
fn trying_the_lock_fails_as_busy_while_another_call_holds_it() {
    let mut context = Context::from_waker(Waker::noop());
    let office = SharedGroup::new(busy());
    {
        let mut change = pin!(office.change_currency(Currency::RON, lei()));
        assert!(change.as_mut().poll(&mut context).is_pending());
        assert!(office.is_locked());
        assert_eq!(office.try_read().map(|_| ()), Err(GroupError::Busy));
        assert_eq!(office.try_write().map(|_| ()), Err(GroupError::Busy));
    }
    assert!(!office.is_locked());

    let club = SharedGroup::new(owed_by_a_hub());
    let mut settlement = pin!(club.settlement(SettlementStrategy::MinimumTransfers));
    assert!(settlement.as_mut().poll(&mut context).is_pending());
    assert!(club.is_locked());
    assert!(club.try_read().is_ok());
    assert_eq!(club.try_write().map(|_| ()), Err(GroupError::Busy));
}

/// A synchronous caller would spin while another thread holds the lock, or
/// never wake when it lets go: the call sleeps, then reads what the holder
/// wrote as soon as it releases the lock.
#[test]
fn a_blocked_call_sleeps_until_the_lock_is_released() {
    let shared = SharedGroup::new(owed_by_a_hub());
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
    let mut read = pin!(shared.name());
    let name = block_on(poll_fn(|context| {
        polls.set(polls.get() + 1);
        read.as_mut().poll(context)
    }));
    holder.join().unwrap();
    assert_eq!(name, "Society");
    // One poll to wait and one to read, with room for a spurious wake-up;
    // a driver that spun would have polled thousands of times in 50 ms.
    assert!(polls.get() <= 3, "{} polls", polls.get());
}

/// A group taken back while another handle still shared it would change
/// under that handle's calls: it comes back only from the last handle.
#[test]
fn the_group_comes_back_only_from_its_last_handle() {
    let group = owed_by_a_hub();
    let shared = SharedGroup::new(group.clone());
    let other = shared.clone();
    let shared = shared
        .into_group()
        .expect_err("a clone still shares the group");
    drop(other);
    assert_eq!(shared.into_group().ok(), Some(group));
}
