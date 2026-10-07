//! The group handle, which every binding's group object forwards to. The
//! two forms of a call answering differently, a synchronous call that failed
//! instead of waiting, a trying form that waited, a cancelled call that
//! changed the group or kept its lock, or a load that took a saved group the
//! library refuses would reach every binding at once.

use std::pin::pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread;
use std::time::Duration;

use eranos::cooperative::block_on;
use eranos::{CurrencyCode, MemberId};

use super::*;
use crate::calendar::CalendarDate;
use crate::expense::{Expense, ExpenseSplit, Split};
use crate::member::{Category, Member};
use crate::money::{Cost, Money};
use crate::transfer::Transfer;
use crate::{ExpenseGroupFailure, LanguageFailure, MoneyFailure, SettlementFailure};

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

/// A group of two with a category, an expense, and a transfer, and the
/// identifiers the group gave them.
struct Flat {
    ana: String,
    bob: String,
    dinner: String,
    food: String,
    handle: ExpenseGroupHandle,
    repayment: String,
}

fn euros(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "EUR".to_owned(),
    }
}

fn rate(currency: &str, rate: &str) -> CurrencyRate {
    CurrencyRate {
        currency: currency.to_owned(),
        rate: rate.to_owned(),
    }
}

fn new_group() -> NewExpenseGroup {
    NewExpenseGroup {
        name: "Flat".to_owned(),
        currency: "EUR".to_owned(),
        description: Some("Rent".to_owned()),
        language: Some("pt-pt".to_owned()),
    }
}

fn new_member(first_name: &str) -> Member {
    Member {
        id: None,
        first_name: first_name.to_owned(),
        last_name: None,
        email: None,
        phone: None,
        accounts: Vec::new(),
    }
}

fn new_category(name: &str) -> Category {
    Category {
        id: None,
        name: name.to_owned(),
    }
}

/// An expense of 30.00 EUR that `payer` paid and `members` share equally.
fn new_expense(name: &str, payer: &str, members: &[&str]) -> Expense {
    Expense {
        id: None,
        name: name.to_owned(),
        cost: Cost::Local {
            amount: euros(3_000),
        },
        payers: Split::Equal {
            members: vec![payer.to_owned()],
        },
        split: ExpenseSplit::Whole {
            split: Split::Equal {
                members: members.iter().map(|&member| member.to_owned()).collect(),
            },
        },
        date: CalendarDate {
            year: 2026,
            month: 9,
            day: 23,
        },
        tip: None,
        time: None,
        category: None,
        description: None,
    }
}

fn new_transfer(from: &str, to: &str, minor_units: i64) -> Transfer {
    Transfer {
        id: None,
        from: from.to_owned(),
        to: to.to_owned(),
        cost: Cost::Local {
            amount: euros(minor_units),
        },
        date: CalendarDate {
            year: 2026,
            month: 9,
            day: 24,
        },
        time: None,
        note: None,
    }
}

fn flat() -> Flat {
    let handle = ExpenseGroupHandle::new(&new_group()).unwrap();
    let ana = handle.add_member(&new_member("Ana")).unwrap();
    let bob = handle.add_member(&new_member("Bob")).unwrap();
    let food = handle.add_category(&new_category("Food")).unwrap();
    let dinner = Expense {
        category: Some(food.clone()),
        ..new_expense("Dinner", &ana, &[&ana, &bob])
    };
    let dinner = handle.add_expense(&dinner).unwrap();
    let repayment = handle
        .add_transfer(&new_transfer(&bob, &ana, 1_000))
        .unwrap();
    Flat {
        ana,
        bob,
        dinner,
        food,
        handle,
        repayment,
    }
}

fn library_day() -> eranos::CalendarDate {
    eranos::CalendarDate::new(2026, 9, 23).unwrap()
}

/// A group in which `payers` members each paid a different amount for one
/// more: with 16, seventeen open balances, so the exact settlement yields
/// once, holding the shared lock; with 63, a table the solver cannot hold.
fn owed_by_a_hub(payers: i64) -> ExpenseGroupHandle {
    let mut group = ExpenseGroup::builder("Club", CurrencyCode::EUR)
        .build()
        .unwrap();
    let hub = group
        .add_member(eranos::Member::builder("Hub").build().unwrap())
        .unwrap();
    for minor_units in 1..=payers {
        let payer = group
            .add_member(eranos::Member::builder("Payer").build().unwrap())
            .unwrap();
        let amount = eranos::Money::from_minor_units(minor_units, CurrencyCode::EUR);
        let split = eranos::Split::equal([hub]);
        let expense = eranos::Expense::builder("Share", amount, payer, split, library_day())
            .build()
            .unwrap();
        group.add_expense(expense).unwrap();
    }
    ExpenseGroupHandle::from(group)
}

/// A group of 1 100 expenses, so a currency change yields once, holding the
/// exclusive lock.
fn busy() -> ExpenseGroupHandle {
    let mut group = ExpenseGroup::builder("Office", CurrencyCode::EUR)
        .build()
        .unwrap();
    let ana = group
        .add_member(eranos::Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(eranos::Member::builder("Bob").build().unwrap())
        .unwrap();
    for _ in 0..1_100 {
        let amount = eranos::Money::from_minor_units(100, CurrencyCode::EUR);
        let split = eranos::Split::equal([ana, bob]);
        let expense = eranos::Expense::builder("Coffee", amount, ana, split, library_day())
            .build()
            .unwrap();
        group.add_expense(expense).unwrap();
    }
    ExpenseGroupHandle::from(group)
}

/// Whether `future` may move to another thread and outlive its caller, as
/// every binding's executor needs.
const fn assert_send_and_static<Call: Future + Send + 'static>(_call: &Call) {}

/// A new group would drop what it starts with, or start from a currency, a
/// name, or a language the library refuses.
#[test]
fn a_new_group_takes_what_it_starts_with() {
    let handle = ExpenseGroupHandle::new(&new_group()).unwrap();
    assert_eq!(handle.currency(), Ok("EUR".to_owned()));
    assert_eq!(handle.name(), Ok("Flat".to_owned()));
    assert_eq!(handle.description(), Ok(Some("Rent".to_owned())));
    assert_eq!(handle.language(), Ok(Some("pt-PT".to_owned())));
    let plain = ExpenseGroupHandle::new(&NewExpenseGroup {
        description: None,
        language: None,
        ..new_group()
    })
    .unwrap();
    assert_eq!(plain.description(), Ok(None));
    assert_eq!(plain.language(), Ok(None));

    let refusals = [
        NewExpenseGroup {
            currency: "XYZ".to_owned(),
            ..new_group()
        },
        NewExpenseGroup {
            name: " ".to_owned(),
            ..new_group()
        },
        NewExpenseGroup {
            language: Some("not a tag".to_owned()),
            ..new_group()
        },
    ]
    .map(|group| ExpenseGroupHandle::new(&group).unwrap_err());
    assert!(
        matches!(
            refusals,
            [
                EranosError::Money {
                    failure: MoneyFailure::UnknownCurrency,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::EmptyName,
                    ..
                },
                EranosError::Language {
                    failure: LanguageFailure::InvalidLanguage,
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );
}

/// A read's two forms would answer differently, so an app would show one
/// thing from a background task and another from its main thread.
#[test]
fn every_read_answers_alike_in_both_forms() {
    let flat = flat();
    let handle = &flat.handle;
    let (ana, dinner, food) = (&flat.ana, &flat.dinner, &flat.food);
    assert_eq!(block_on(handle.id_async(None)), handle.id());
    assert_eq!(block_on(handle.categories_async(None)), handle.categories());
    assert_eq!(
        block_on(handle.category_async(food.clone(), None)),
        handle.category(food)
    );
    assert_eq!(block_on(handle.currency_async(None)), handle.currency());
    assert_eq!(block_on(handle.expenses_async(None)), handle.expenses());
    assert_eq!(
        block_on(handle.expense_async(dinner.clone(), None)),
        handle.expense(dinner)
    );
    assert_eq!(
        block_on(handle.expense_shares_async(dinner.clone(), None)),
        handle.expense_shares(dinner)
    );
    assert_eq!(block_on(handle.summary_async(None)), handle.summary());
    assert_eq!(block_on(handle.members_async(None)), handle.members());
    assert_eq!(
        block_on(handle.member_async(ana.clone(), None)),
        handle.member(ana)
    );
    assert_eq!(
        block_on(handle.is_archived_async(ana.clone(), None)),
        handle.is_archived(ana)
    );
    assert_eq!(block_on(handle.name_async(None)), handle.name());
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::Direct,
        SettlementStrategy::Greedy,
        SettlementStrategy::MinimumTransfers,
    ] {
        assert_eq!(
            block_on(handle.settlement_async(strategy, None)),
            handle.settlement(strategy)
        );
    }
    let repayment = &flat.repayment;
    assert_eq!(block_on(handle.transfers_async(None)), handle.transfers());
    assert_eq!(
        block_on(handle.transfer_async(repayment.clone(), None)),
        handle.transfer(repayment)
    );
    assert_eq!(
        block_on(handle.transfer_amount_async(repayment.clone(), None)),
        handle.transfer_amount(repayment)
    );
    assert_eq!(
        block_on(handle.description_async(None)),
        handle.description()
    );
    assert_eq!(block_on(handle.language_async(None)), handle.language());
    assert_eq!(block_on(handle.save_async(None)), handle.save());
    let reader = handle.read().unwrap();
    let async_reader = block_on(handle.read_async(None)).unwrap();
    assert_eq!(async_reader.save(), reader.save());
    assert_eq!(reader.save(), handle.save());
}

/// A category change's two forms would change the group differently.
#[test]
fn categories_change_alike_in_both_forms() {
    let handle = flat().handle;
    let travel = handle.add_category(&new_category("Travel")).unwrap();
    let rent = block_on(handle.add_category_async(new_category("Rent"), None)).unwrap();
    let trips = Category {
        id: Some(travel.clone()),
        name: "Trips".to_owned(),
    };
    let home = Category {
        id: Some(rent.clone()),
        name: "Home".to_owned(),
    };
    assert_eq!(
        handle.replace_category(&trips).map(|old| old.name),
        Ok("Travel".to_owned())
    );
    assert_eq!(
        block_on(handle.replace_category_async(home.clone(), None)).map(|old| old.name),
        Ok("Rent".to_owned())
    );
    assert_eq!(handle.remove_category(&travel), Ok(trips));
    assert_eq!(block_on(handle.remove_category_async(rent, None)), Ok(home));
    assert_eq!(
        handle.categories().map(|categories| categories.len()),
        Ok(2)
    );
}

/// A member change's two forms would change the group differently.
#[test]
fn members_change_alike_in_both_forms() {
    let handle = flat().handle;
    let carol = handle.add_member(&new_member("Carol")).unwrap();
    let dan = block_on(handle.add_member_async(new_member("Dan"), None)).unwrap();
    let carol_popa = Member {
        id: Some(carol.clone()),
        last_name: Some("Popa".to_owned()),
        ..new_member("Carol")
    };
    let dan_popa = Member {
        id: Some(dan.clone()),
        last_name: Some("Popa".to_owned()),
        ..new_member("Dan")
    };
    handle.replace_member(&carol_popa).unwrap();
    block_on(handle.replace_member_async(dan_popa.clone(), None)).unwrap();

    handle.archive_member(&carol).unwrap();
    block_on(handle.archive_member_async(dan.clone(), None)).unwrap();
    assert_eq!(
        [handle.is_archived(&carol), handle.is_archived(&dan)],
        [Ok(true), Ok(true)]
    );
    handle.unarchive_member(&carol).unwrap();
    block_on(handle.unarchive_member_async(dan.clone(), None)).unwrap();
    assert_eq!(
        [handle.is_archived(&carol), handle.is_archived(&dan)],
        [Ok(false), Ok(false)]
    );

    assert_eq!(handle.remove_member(&carol), Ok(carol_popa));
    assert_eq!(
        block_on(handle.remove_member_async(dan, None)),
        Ok(dan_popa)
    );
}

/// An expense change's two forms would change the group differently.
#[test]
fn expenses_change_alike_in_both_forms() {
    let flat = flat();
    let (ana, bob, handle) = (flat.ana.as_str(), flat.bob.as_str(), &flat.handle);
    let lunch = handle
        .add_expense(&new_expense("Lunch", ana, &[ana, bob]))
        .unwrap();
    let supper =
        block_on(handle.add_expense_async(new_expense("Supper", bob, &[ana, bob]), None)).unwrap();
    let late_lunch = Expense {
        id: Some(lunch.clone()),
        ..new_expense("Late lunch", ana, &[ana])
    };
    let late_supper = Expense {
        id: Some(supper.clone()),
        ..new_expense("Late supper", bob, &[bob])
    };
    assert_eq!(
        handle.replace_expense(&late_lunch).map(|old| old.name),
        Ok("Lunch".to_owned())
    );
    assert_eq!(
        block_on(handle.replace_expense_async(late_supper.clone(), None)).map(|old| old.name),
        Ok("Supper".to_owned())
    );
    assert_eq!(handle.remove_expense(&lunch), Ok(late_lunch));
    assert_eq!(
        block_on(handle.remove_expense_async(supper, None)),
        Ok(late_supper)
    );
    assert_eq!(handle.expenses().map(|expenses| expenses.len()), Ok(1));
}

/// A transfer change's two forms would change the group differently.
#[test]
fn transfers_change_alike_in_both_forms() {
    let flat = flat();
    let (ana, bob, handle) = (flat.ana.as_str(), flat.bob.as_str(), &flat.handle);
    let first = handle.add_transfer(&new_transfer(ana, bob, 100)).unwrap();
    let second = block_on(handle.add_transfer_async(new_transfer(bob, ana, 200), None)).unwrap();
    let larger_first = Transfer {
        id: Some(first.clone()),
        ..new_transfer(ana, bob, 300)
    };
    let larger_second = Transfer {
        id: Some(second.clone()),
        ..new_transfer(bob, ana, 400)
    };
    assert_eq!(
        handle.replace_transfer(&larger_first),
        Ok(Transfer {
            id: Some(first.clone()),
            ..new_transfer(ana, bob, 100)
        })
    );
    assert_eq!(
        block_on(handle.replace_transfer_async(larger_second.clone(), None)),
        Ok(Transfer {
            id: Some(second.clone()),
            ..new_transfer(bob, ana, 200)
        })
    );
    assert_eq!(handle.remove_transfer(&first), Ok(larger_first));
    assert_eq!(
        block_on(handle.remove_transfer_async(second, None)),
        Ok(larger_second)
    );
    assert_eq!(handle.transfers().map(|transfers| transfers.len()), Ok(1));
}

/// A change of the name, the description, or the language would take
/// effect in one form only.
#[test]
fn the_name_description_and_language_change_alike_in_both_forms() {
    let handle = flat().handle;
    handle.set_name("Home").unwrap();
    assert_eq!(handle.name(), Ok("Home".to_owned()));
    block_on(handle.set_name_async("House".to_owned(), None)).unwrap();
    assert_eq!(handle.name(), Ok("House".to_owned()));

    handle.set_description("Bills").unwrap();
    assert_eq!(handle.description(), Ok(Some("Bills".to_owned())));
    block_on(handle.set_description_async(String::new(), None)).unwrap();
    assert_eq!(handle.description(), Ok(None));

    handle.set_language(Some("ro")).unwrap();
    assert_eq!(handle.language(), Ok(Some("ro".to_owned())));
    block_on(handle.set_language_async(None, None)).unwrap();
    assert_eq!(handle.language(), Ok(None));
}

/// The currency change's two forms would change the group differently, or
/// the asynchronous one would let a refusal through as a success.
#[test]
fn the_currency_changes_alike_in_both_forms() {
    let handle = flat().handle;
    handle.change_currency("RON", &[rate("EUR", "5")]).unwrap();
    assert_eq!(handle.currency(), Ok("RON".to_owned()));
    block_on(handle.change_currency_async("EUR".to_owned(), Vec::new(), None)).unwrap();
    assert_eq!(handle.currency(), Ok("EUR".to_owned()));
    assert_eq!(
        handle.summary().map(|summary| summary.total_spent),
        Ok(euros(3_000))
    );

    let refusals = [
        block_on(handle.change_currency_async("XYZ".to_owned(), Vec::new(), None)).unwrap_err(),
        block_on(handle.change_currency_async("USD".to_owned(), Vec::new(), None)).unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::Money {
                    failure: MoneyFailure::UnknownCurrency,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::MissingExchangeRate { .. },
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );
    assert_eq!(handle.currency(), Ok("EUR".to_owned()));
}

/// A settlement the solver cannot hold would be reported another way in
/// the asynchronous form, or not at all.
#[test]
fn a_settlement_too_large_to_compute_is_refused_in_both_forms() {
    let hub = owed_by_a_hub(63);
    let strategy = SettlementStrategy::MinimumTransfers;
    for refusal in [
        hub.settlement(strategy),
        block_on(hub.settlement_async(strategy, None)),
    ] {
        assert!(
            matches!(
                refusal,
                Err(EranosError::Settlement {
                    failure: SettlementFailure::OutOfMemory { .. },
                    ..
                })
            ),
            "{refusal:?}"
        );
    }
}

/// A saved group would load another way in the two forms, a text that is
/// not the saved form or a saved group the library refuses would load, or
/// the asynchronous load could not move to another thread.
#[test]
fn a_saved_group_loads_alike_in_both_forms() {
    let flat = flat();
    let json = flat.handle.save().unwrap();
    assert_eq!(
        ExpenseGroupHandle::load(&json).and_then(|group| group.save()),
        Ok(json.clone())
    );
    let loading = ExpenseGroupHandle::load_async(json.clone(), None);
    assert_send_and_static(&loading);
    assert_eq!(
        block_on(loading).and_then(|group| group.save()),
        Ok(json.clone())
    );

    // One of Ana's references now names a member the group does not hold.
    let tampered = json.replacen(
        &flat.ana,
        &MemberId::new("member-9").unwrap().to_string(),
        1,
    );
    let cancelled = Arc::new(Cancellation::new());
    cancelled.cancel();
    let refusals = [
        ExpenseGroupHandle::load("{").unwrap_err(),
        block_on(ExpenseGroupHandle::load_async("{".to_owned(), None)).unwrap_err(),
        ExpenseGroupHandle::load(&tampered).unwrap_err(),
        block_on(ExpenseGroupHandle::load_async(tampered, None)).unwrap_err(),
        block_on(ExpenseGroupHandle::load_async(json, Some(cancelled))).unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::InvalidJson { .. },
                EranosError::InvalidJson { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::Cancelled { .. },
            ]
        ),
        "{refusals:?}"
    );
}

/// A clone of a handle that held a group of its own would let two parts of
/// an app see different groups.
#[test]
fn every_clone_of_a_handle_shares_one_group() {
    let handle = flat().handle;
    let other = handle.clone();
    other.set_name("Home").unwrap();
    assert_eq!(handle.name(), Ok("Home".to_owned()));
}

/// On wasm a synchronous call tries the lock instead of waiting for it. A
/// try that waited would freeze the page's one thread, and one that took the
/// lock from a call at its yield would let a change into a computation in
/// progress.
#[test]
fn trying_the_lock_is_busy_while_an_async_call_holds_it() {
    let waker = Waker::from(Arc::new(Task::default()));
    let mut context = Context::from_waker(&waker);

    let hub = owed_by_a_hub(16);
    let mut settling = pin!(hub.settlement_async(SettlementStrategy::MinimumTransfers, None));
    assert!(settling.as_mut().poll(&mut context).is_pending());
    // The settlement holds the shared lock at its yield: reads join it, and
    // a change may not.
    assert!(hub.try_read_lock().is_ok());
    assert!(matches!(
        hub.try_write_lock(),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            ..
        })
    ));
    let Poll::Ready(settlement) = settling.as_mut().poll(&mut context) else {
        panic!("the settlement did not complete after its one yield");
    };
    assert_eq!(
        settlement.map(|settlement| settlement.payments.len()),
        Ok(16)
    );
    assert!(hub.try_write_lock().is_ok());

    let busy = busy();
    let mut changing =
        pin!(busy.change_currency_async("RON".to_owned(), vec![rate("EUR", "5")], None));
    assert!(changing.as_mut().poll(&mut context).is_pending());
    // The change holds the exclusive lock at its yield.
    assert!(matches!(
        busy.try_read_lock(),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            ..
        })
    ));
    assert!(matches!(
        busy.try_write_lock(),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            ..
        })
    ));
    assert_eq!(changing.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert_eq!(busy.currency(), Ok("RON".to_owned()));
}

/// A synchronous call where a thread can sleep would fail, or read the
/// group halfway through a change, instead of waiting for the change to end.
#[test]
fn a_synchronous_call_waits_for_a_change_in_progress() {
    let waker = Waker::from(Arc::new(Task::default()));
    let mut context = Context::from_waker(&waker);
    let busy = busy();
    let mut changing =
        pin!(busy.change_currency_async("RON".to_owned(), vec![rate("EUR", "5")], None));
    assert!(changing.as_mut().poll(&mut context).is_pending());

    let (sender, receiver) = mpsc::channel();
    let reading = thread::spawn(move || sender.send(busy.currency()));
    assert!(receiver.recv_timeout(Duration::from_millis(50)).is_err());
    assert_eq!(changing.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert_eq!(receiver.recv(), Ok(Ok("RON".to_owned())));
    reading.join().unwrap().unwrap();
}

/// A change cancelled while it waits for the lock would run once the lock
/// is free, after its caller gave it up, or keep its place in the queue, so
/// that every later change waited for it.
#[test]
fn a_change_cancelled_while_it_waits_changes_nothing() {
    let task = Arc::new(Task::default());
    let waker = Waker::from(Arc::clone(&task));
    let mut context = Context::from_waker(&waker);
    let handle = flat().handle;
    let reader = handle.read().unwrap();
    let cancellation = Arc::new(Cancellation::new());
    let mut adding =
        pin!(handle.add_member_async(new_member("Carol"), Some(Arc::clone(&cancellation))));
    let mut changing = pin!(handle.change_currency_async(
        "RON".to_owned(),
        vec![rate("EUR", "5")],
        Some(Arc::clone(&cancellation))
    ));
    assert!(adding.as_mut().poll(&mut context).is_pending());
    assert!(changing.as_mut().poll(&mut context).is_pending());

    cancellation.cancel();
    assert_eq!(task.wakes(), 2);
    assert_eq!(
        adding.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert_eq!(
        changing.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    drop(reader);
    assert!(handle.try_write_lock().is_ok());
    assert_eq!(handle.members().map(|members| members.len()), Ok(2));
    assert_eq!(handle.currency(), Ok("EUR".to_owned()));
}

/// A read cancelled while a change holds the lock would still run once the
/// change ends, after its caller gave it up.
#[test]
fn a_read_cancelled_while_it_waits_ends_at_once() {
    let task = Arc::new(Task::default());
    let waker = Waker::from(Arc::clone(&task));
    let mut context = Context::from_waker(&waker);
    let handle = flat().handle;
    let change = handle.try_write_lock().unwrap();
    let cancellation = Arc::new(Cancellation::new());
    let mut naming = pin!(handle.name_async(Some(Arc::clone(&cancellation))));
    let mut settling =
        pin!(handle.settlement_async(SettlementStrategy::Greedy, Some(Arc::clone(&cancellation))));
    let mut reading = pin!(handle.read_async(Some(Arc::clone(&cancellation))));
    assert!(naming.as_mut().poll(&mut context).is_pending());
    assert!(settling.as_mut().poll(&mut context).is_pending());
    assert!(reading.as_mut().poll(&mut context).is_pending());

    cancellation.cancel();
    assert_eq!(task.wakes(), 3);
    assert_eq!(
        naming.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert_eq!(
        settling.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert!(matches!(
        reading.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::Cancelled { .. }))
    ));
    drop(change);
    assert!(handle.try_write_lock().is_ok());
}

/// A currency change cancelled at its yield would leave the group half in
/// the new currency, or keep the exclusive lock.
#[test]
fn a_currency_change_cancelled_at_its_yield_changes_nothing() {
    let waker = Waker::from(Arc::new(Task::default()));
    let mut context = Context::from_waker(&waker);
    let busy = busy();
    let before = busy.save();
    let cancellation = Arc::new(Cancellation::new());
    let mut changing = pin!(busy.change_currency_async(
        "RON".to_owned(),
        vec![rate("EUR", "5")],
        Some(Arc::clone(&cancellation))
    ));
    assert!(changing.as_mut().poll(&mut context).is_pending());

    cancellation.cancel();
    assert_eq!(
        changing.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert!(busy.try_write_lock().is_ok());
    assert_eq!(busy.save(), before);
}

/// A settlement cancelled at its yield would keep the shared lock, so that
/// every later change waited for it.
#[test]
fn a_settlement_cancelled_at_its_yield_releases_the_lock() {
    let waker = Waker::from(Arc::new(Task::default()));
    let mut context = Context::from_waker(&waker);
    let hub = owed_by_a_hub(16);
    let cancellation = Arc::new(Cancellation::new());
    let mut settling = pin!(hub.settlement_async(
        SettlementStrategy::MinimumTransfers,
        Some(Arc::clone(&cancellation))
    ));
    assert!(settling.as_mut().poll(&mut context).is_pending());

    cancellation.cancel();
    assert_eq!(
        settling.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::cancelled()))
    );
    assert!(hub.try_write_lock().is_ok());
}
