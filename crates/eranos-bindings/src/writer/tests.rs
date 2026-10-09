//! The write handle, which the bindings hand out so that an app reads and
//! changes the group under one lock. A call through the handle that answered
//! another way than the group handle, a released handle that still ran or
//! kept the lock, or another call that ran while a handle was open would
//! show the apps a group that no sequence of calls made.

use std::pin::pin;
use std::task::{Context, Poll, Waker};

use eranos::CurrencyCode;

use super::*;
use crate::expense_group::ExpenseGroupHandle;
use crate::settlement::SettlementStrategy;
use crate::{ExpenseGroupFailure, IdFailure};

/// The group handle and the identifiers of an entry of every kind.
struct Flat {
    ana: String,
    bob: String,
    food: String,
    handle: ExpenseGroupHandle,
    repayment: String,
}

fn flat() -> Flat {
    let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR)
        .build()
        .unwrap();
    let ana = group
        .add_member(eranos::Member::builder("Ana").build().unwrap())
        .unwrap();
    let bob = group
        .add_member(eranos::Member::builder("Bob").build().unwrap())
        .unwrap();
    let food = group
        .add_category(eranos::Category::new("Food").unwrap())
        .unwrap();
    let day = eranos::CalendarDate::new(2026, 9, 23).unwrap();
    let cost = eranos::Money::from_minor_units(3_000, CurrencyCode::EUR);
    let split = eranos::Split::equal([ana, bob]);
    let dinner = eranos::Expense::builder("Dinner", cost, ana, split, day)
        .category(food)
        .build()
        .unwrap();
    group.add_expense(dinner).unwrap();
    let paid = eranos::Money::from_minor_units(1_000, CurrencyCode::EUR);
    let repayment = eranos::Transfer::builder(bob, ana, paid, day)
        .build()
        .unwrap();
    let repayment = group.add_transfer(repayment).unwrap();
    Flat {
        ana: ana.to_string(),
        bob: bob.to_string(),
        food: food.to_string(),
        handle: ExpenseGroupHandle::from(group),
        repayment: repayment.to_string(),
    }
}

/// A call through the handle would answer another way than the same call
/// through the group handle, or a change through it would not reach the
/// group. The group handle's synchronous forms wait for the handle, so the
/// test reads through the group handle only before the handle is taken and
/// after it is released.
#[test]
fn every_call_through_the_handle_does_what_the_group_handle_does() {
    let flat = flat();
    let handle = &flat.handle;
    let (name, members, summary, expenses, transfers, saved) = (
        handle.name(),
        handle.members(),
        handle.summary(),
        handle.expenses(),
        handle.transfers(),
        handle.save(),
    );
    let writer = handle.write().unwrap();
    assert_eq!(writer.name(), Ok(name));
    assert_eq!(writer.members(), Ok(members));
    assert_eq!(writer.summary(), Ok(summary));
    assert_eq!(writer.expenses(), Ok(expenses));
    assert_eq!(writer.transfers(), Ok(transfers));
    assert_eq!(writer.save(), Ok(saved));
    assert_eq!(
        writer
            .category(&flat.food)
            .map(|category| category.is_some()),
        Ok(true)
    );
    assert_eq!(writer.is_archived(&flat.ana), Ok(Some(false)));
    assert_eq!(
        writer
            .transfer_amount(&flat.repayment)
            .map(|amount| amount.map(|amount| amount.minor_units)),
        Ok(Some(1_000))
    );
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::Direct,
        SettlementStrategy::Greedy,
        SettlementStrategy::MinimumTransfers,
    ] {
        assert!(writer.settlement(strategy).is_ok());
    }
    assert!(matches!(
        writer.member(" nobody"),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));

    writer.set_name("Home").unwrap();
    assert_eq!(writer.name(), Ok("Home".to_owned()));
    writer.archive_member(&flat.bob).unwrap();
    assert_eq!(writer.is_archived(&flat.bob), Ok(Some(true)));
    writer.set_note(Some("Bills")).unwrap();
    assert_eq!(writer.note(), Ok(Some("Bills".to_owned())));
    writer.set_language(Some("ro")).unwrap();
    assert_eq!(writer.language(), Ok(Some("ro".to_owned())));
    assert!(matches!(
        writer.remove_member(&flat.ana),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::MemberInUse { .. },
            ..
        })
    ));
    writer.release();
    drop(writer);

    assert_eq!(handle.name(), "Home");
    assert_eq!(handle.is_archived(&flat.bob), Ok(Some(true)));
    assert_eq!(handle.note(), Some("Bills".to_owned()));
    assert_eq!(handle.language(), Some("ro".to_owned()));
}

/// A released handle that still ran would read or change a group that other
/// calls may have changed since, and one that kept the lock would stop every
/// other call.
#[test]
fn a_released_handle_runs_nothing_and_holds_no_lock() {
    let flat = flat();
    let writer = flat.handle.write().unwrap();
    assert!(matches!(
        flat.handle.try_read_lock(),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            ..
        })
    ));
    writer.release();
    assert!(flat.handle.try_read_lock().is_ok());
    // Releasing again does nothing.
    writer.release();

    let released = [
        writer.name().err(),
        writer.set_name("Home").err(),
        writer.members().err(),
        writer.archive_member(&flat.bob).err(),
        writer.save().err(),
        writer.settlement(SettlementStrategy::Greedy).err(),
    ];
    drop(writer);
    for error in released {
        assert_eq!(error, Some(EranosError::released()));
    }
    assert_eq!(flat.handle.name(), "Flat");
    assert_eq!(flat.handle.is_archived(&flat.bob), Ok(Some(false)));
}

/// Another call that ran while a handle was open would read or change the
/// group between the handle's calls, and one that still waited after the
/// handle was released or dropped would never run.
#[test]
fn every_other_call_waits_for_an_open_handle() {
    let flat = flat();
    let handle = &flat.handle;
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);

    {
        let writer = handle.write().unwrap();
        let mut reading = pin!(handle.name_async(None));
        assert!(reading.as_mut().poll(&mut context).is_pending());
        let mut renaming = pin!(handle.set_name_async("Home".to_owned(), None));
        assert!(renaming.as_mut().poll(&mut context).is_pending());
        writer.set_name("House").unwrap();
        // Released but not yet dropped, the handle holds the lock no more.
        writer.release();
        assert_eq!(
            reading.as_mut().poll(&mut context),
            Poll::Ready(Ok("House".to_owned()))
        );
        assert_eq!(renaming.as_mut().poll(&mut context), Poll::Ready(Ok(())));
        assert_eq!(writer.name(), Err(EranosError::released()));
    }
    assert_eq!(handle.name(), "Home");

    let dropped = handle.write().unwrap();
    let mut archiving = pin!(handle.archive_member_async(flat.bob.clone(), None));
    assert!(archiving.as_mut().poll(&mut context).is_pending());
    drop(dropped);
    assert_eq!(archiving.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    assert_eq!(handle.is_archived(&flat.bob), Ok(Some(true)));
}
