//! The read handle, which the bindings hand out so that a report reads the
//! group under one lock. A read through the handle that answered another way
//! than the group handle, a released handle that still read or kept the lock,
//! or a change that ran while a handle was open would show the apps a report
//! that no state of the group ever had.

use std::pin::pin;
use std::task::{Context, Poll, Waker};

use eranos::{CurrencyCode, MemberId};

use super::*;
use crate::expense_group::ExpenseGroupHandle;
use crate::settlement::SettlementStrategy;
use crate::{ExpenseGroupFailure, IdFailure};

/// The group handle and the identifiers of an entry of every kind.
struct Flat {
    ana: String,
    dinner: String,
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
    let dinner = group.add_expense(dinner).unwrap();
    let paid = eranos::Money::from_minor_units(1_000, CurrencyCode::EUR);
    let repayment = eranos::Transfer::builder(bob, ana, paid, day)
        .build()
        .unwrap();
    let repayment = group.add_transfer(repayment).unwrap();
    Flat {
        ana: ana.to_string(),
        dinner: dinner.to_string(),
        food: food.to_string(),
        handle: ExpenseGroupHandle::from(group),
        repayment: repayment.to_string(),
    }
}

/// A read through the handle would answer another way than the same read
/// through the group handle.
#[test]
fn every_read_through_the_handle_answers_as_the_group_does() {
    let flat = flat();
    let (handle, ana, dinner) = (&flat.handle, &flat.ana, &flat.dinner);
    let reader = handle.read().unwrap();
    assert_eq!(reader.id(), handle.id());
    assert_eq!(reader.categories(), handle.categories());
    assert_eq!(reader.category(&flat.food), handle.category(&flat.food));
    assert_eq!(reader.currency(), handle.currency());
    assert_eq!(reader.expenses(), handle.expenses());
    assert_eq!(reader.expense(dinner), handle.expense(dinner));
    assert_eq!(reader.expense_shares(dinner), handle.expense_shares(dinner));
    assert_eq!(reader.summary(), handle.summary());
    assert_eq!(reader.members(), handle.members());
    assert_eq!(reader.member(ana), handle.member(ana));
    assert_eq!(reader.is_archived(ana), handle.is_archived(ana));
    assert_eq!(reader.name(), handle.name());
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::Direct,
        SettlementStrategy::Greedy,
        SettlementStrategy::MinimumTransfers,
    ] {
        assert_eq!(reader.settlement(strategy), handle.settlement(strategy));
    }
    let repayment = &flat.repayment;
    assert_eq!(reader.transfers(), handle.transfers());
    assert_eq!(reader.transfer(repayment), handle.transfer(repayment));
    assert_eq!(
        reader.transfer_amount(repayment),
        handle.transfer_amount(repayment)
    );
    assert_eq!(reader.description(), handle.description());
    assert_eq!(reader.language(), handle.language());
    assert_eq!(reader.save(), handle.save());
    assert!(matches!(
        reader.member(" nobody"),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));
    drop(reader);
}

/// A released handle that still read would show a report of a group that may
/// have changed since, and one that kept the lock would stop every change.
#[test]
fn a_released_handle_reads_nothing_and_holds_no_lock() {
    let flat = flat();
    let reader = flat.handle.read().unwrap();
    assert!(matches!(
        flat.handle.try_write_lock(),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::Busy,
            ..
        })
    ));
    reader.release();
    assert!(flat.handle.try_write_lock().is_ok());
    // Releasing again does nothing.
    reader.release();

    let released = [
        reader.id().err(),
        reader.categories().err(),
        reader.category(&flat.food).err(),
        reader.currency().err(),
        reader.expenses().err(),
        reader.expense(&flat.dinner).err(),
        reader.expense_shares(&flat.dinner).err(),
        reader.summary().err(),
        reader.members().err(),
        reader.member(&flat.ana).err(),
        reader.is_archived(&flat.ana).err(),
        reader.name().err(),
        reader.settlement(SettlementStrategy::Greedy).err(),
        reader.transfers().err(),
        reader.transfer(&flat.repayment).err(),
        reader.transfer_amount(&flat.repayment).err(),
        reader.description().err(),
        reader.language().err(),
        reader.save().err(),
    ];
    drop(reader);
    for error in released {
        assert_eq!(error, Some(EranosError::released()));
    }
}

/// A change that ran while a handle was open would change the group under a
/// report in progress, and one that still waited after the handle was
/// released or dropped would never run.
#[test]
fn a_change_waits_for_every_open_handle() {
    let flat = flat();
    let handle = &flat.handle;
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);

    {
        let reader = handle.read().unwrap();
        let mut renaming = pin!(handle.set_name_async("Home".to_owned(), None));
        assert!(renaming.as_mut().poll(&mut context).is_pending());
        assert_eq!(reader.name(), Ok("Flat".to_owned()));
        // Released but not yet dropped, the handle holds the lock no more.
        reader.release();
        assert_eq!(renaming.as_mut().poll(&mut context), Poll::Ready(Ok(())));
        assert_eq!(reader.name(), Err(EranosError::released()));
    }

    let dropped = handle.read().unwrap();
    let carol = MemberId::new("member-9").unwrap().to_string();
    let mut archiving = pin!(handle.archive_member_async(carol, None));
    assert!(archiving.as_mut().poll(&mut context).is_pending());
    drop(dropped);
    assert!(matches!(
        archiving.as_mut().poll(&mut context),
        Poll::Ready(Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownMember { .. },
            ..
        }))
    ));
    assert_eq!(handle.name(), Ok("Home".to_owned()));
}
