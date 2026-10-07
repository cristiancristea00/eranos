//! Tests for the group aggregate. The group is the only way the apps change
//! data, so these pin its consistency rules: references are checked, a
//! failed change leaves the group exactly as it was, the incremental totals
//! always equal a full rebuild, archived members follow their rules, cached
//! settlements are dropped exactly when balances change, and a currency
//! change re-converts every entry. The random-operations property drives
//! all of it at once.

use eranos_calendar::CalendarDate;
use eranos_expense::{ExpenseBuilder, MemberParts, Split};
use eranos_money::{Cost, Parts};
use eranos_transfer::TransferBuilder;
use std::future::IntoFuture;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use proptest::prelude::*;

use super::*;

fn eur(text: &str) -> Money {
    Money::parse(text, Currency::EUR).unwrap()
}

fn day() -> CalendarDate {
    CalendarDate::new(2026, 9, 23).unwrap()
}

fn person(name: &str) -> Member {
    Member::builder(name).build().unwrap()
}

/// A group with three members, and the members in creation order.
struct Trip {
    group: Group,
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
}

/// A group with three members, in creation order.
fn trip() -> Trip {
    let mut group = Group::builder("Trip", Currency::EUR).build().unwrap();
    let ana = group.add_member(person("Ana")).unwrap();
    let bob = group.add_member(person("Bob")).unwrap();
    let carol = group.add_member(person("Carol")).unwrap();
    Trip {
        group,
        ana,
        bob,
        carol,
    }
}

fn shared(amount: &str, payer: MemberId, participants: &[MemberId]) -> Expense {
    Expense::builder(
        "Shared",
        eur(amount),
        payer,
        Split::equal(participants.iter().copied()),
        day(),
    )
    .build()
    .unwrap()
}

fn transfer(from: MemberId, to: MemberId, amount: &str) -> Transfer {
    Transfer::builder(from, to, eur(amount), day())
        .build()
        .unwrap()
}

/// A new version of the group's expense `id`, made as an app makes one: the
/// stored expense's builder, changed by `edit`.
fn edited_expense(
    group: &Group,
    id: ExpenseId,
    edit: impl FnOnce(ExpenseBuilder) -> ExpenseBuilder,
) -> Expense {
    edit(group.expense(id).unwrap().to_builder())
        .build()
        .unwrap()
}

/// A new version of the group's transfer `id`, made as an app makes one:
/// the stored transfer's builder, changed by `edit`.
fn edited_transfer(
    group: &Group,
    id: TransferId,
    edit: impl FnOnce(TransferBuilder) -> TransferBuilder,
) -> Transfer {
    edit(group.transfer(id).unwrap().to_builder())
        .build()
        .unwrap()
}

/// The ledger recomputed from scratch from the group's entries, checking
/// every stored resolution, and that every entry is kept under its own
/// identifier, along the way.
fn rebuilt(group: &Group) -> Ledger {
    let mut delta = Delta::default();
    for (&id, record) in &group.expenses {
        assert_eq!(id, record.expense.id());
        let resolved = resolve_expense(&record.expense, group.currency).unwrap();
        assert_eq!(resolved, record.resolved);
        delta.add_expense(&resolved, record.expense.category(), 1);
    }
    for (&id, record) in &group.transfers {
        assert_eq!(id, record.transfer.id());
        assert_eq!(
            resolve_transfer(&record.transfer, group.currency).unwrap(),
            record.amount
        );
        delta.add_transfer(pair(&record.transfer), record.amount, 1);
    }
    let mut ledger = Ledger::new(group.currency);
    ledger.apply(&delta).unwrap();
    ledger
}

/// Asserts that `change` fails with `expected` and leaves the group,
/// including its derived totals, exactly as it was.
fn assert_unchanged<T: std::fmt::Debug>(
    group: &mut Group,
    expected: &GroupError,
    change: impl FnOnce(&mut Group) -> Result<T, GroupError>,
) {
    let before = group.clone();
    let result = change(group);
    assert_eq!(result.err().as_ref(), Some(expected));
    assert_eq!(*group, before);
    assert_eq!(group.ledger, before.ledger);
}

/// A new group starts empty, with its normalised details and an
/// identifier of its own.
#[test]
fn a_new_group_starts_empty() {
    let group = Group::builder("  Lisbon ", Currency::EUR)
        .description(" ")
        .language(Language::parse("pt-PT").unwrap())
        .build()
        .unwrap();
    let other = Group::builder("Lisbon", Currency::EUR).build().unwrap();
    assert_ne!(group.id(), other.id());
    assert_eq!(group.name(), "Lisbon");
    assert_eq!(group.description(), None);
    assert_eq!(group.currency(), Currency::EUR);
    assert_eq!(group.language().map(Language::as_str), Some("pt-PT"));
    assert_eq!(group.members().len(), 0);
    assert_eq!(group.summary().total_spent(), eur("0"));
    // A blank name is refused.
    assert!(Group::builder(" ", Currency::EUR).build().is_err());
}

/// Group details change without touching entries or totals.
#[test]
fn details_change_in_place() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    let summary = group.summary().clone();
    group.set_name("Porto").unwrap();
    group.set_description("  Weekend  ");
    group.set_language(Language::parse("en").unwrap());
    assert_eq!(group.name(), "Porto");
    assert_eq!(group.description(), Some("Weekend"));
    assert_eq!(group.language().map(Language::as_str), Some("en"));
    // Clearing works, and a blank name is refused.
    group.set_description("");
    group.set_language(None);
    assert_eq!(group.description(), None);
    assert_eq!(group.language(), None);
    assert!(group.set_name(" ").is_err());
    assert_eq!(group.name(), "Porto");
    assert_eq!(*group.summary(), summary);
}

/// Members are listed in the order they were built, a replacement keeps a
/// member's identifier, and the same member is added only once.
#[test]
fn members_are_added_replaced_and_listed_in_creation_order() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    let ids: Vec<MemberId> = group.members().map(Member::id).collect();
    assert_eq!(ids, vec![ana, bob, carol]);
    assert!(ana < bob && bob < carol);
    // Replacing returns the previous details.
    let robert = group
        .member(bob)
        .unwrap()
        .to_builder()
        .first_name("Robert")
        .build()
        .unwrap();
    let previous = group.replace_member(robert).unwrap();
    assert_eq!(previous.first_name(), "Bob");
    assert_eq!(group.member(bob).map(Member::first_name), Some("Robert"));
    // The same member is added once, under its own identifier.
    let dan = person("Dan");
    assert_eq!(group.add_member(dan.clone()), Ok(dan.id()));
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateMember(dan.id()),
        |group| group.add_member(dan),
    );
    // A member the group does not have cannot be replaced.
    let eve = person("Eve");
    assert_unchanged(&mut group, &GroupError::UnknownMember(eve.id()), |group| {
        group.replace_member(eve.clone())
    });
    assert_eq!(group.member(eve.id()), None);
}

/// Archiving keeps a member's history and balance but keeps them out of new
/// expenses, while transfers and replacements that keep them still work.
#[test]
fn an_archived_member_keeps_history_but_joins_no_new_expense() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    let dinner = group
        .add_expense(shared("30.00", ana, &[ana, bob, carol]))
        .unwrap();
    let lunch = group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    group.archive_member(carol).unwrap();
    group.archive_member(carol).unwrap();
    assert!(group.is_archived(carol));
    // New details keep her archived.
    let renamed = group
        .member(carol)
        .unwrap()
        .to_builder()
        .last_name("Ionescu")
        .build()
        .unwrap();
    group.replace_member(renamed).unwrap();
    assert!(group.is_archived(carol));
    assert!(!group.is_archived(MemberId::from_bytes([200; 16])));

    // A new expense with Carol is refused.
    assert_unchanged(&mut group, &GroupError::ArchivedMember(carol), |group| {
        group.add_expense(shared("5.00", ana, &[ana, carol]))
    });
    // Replacing an expense she was in may keep her, in any role.
    let dearer = edited_expense(&group, dinner, |builder| builder.cost(eur("36.00")));
    group.replace_expense(dearer).unwrap();
    let paid_by_her = edited_expense(&group, dinner, |builder| builder.paid_by(carol));
    group.replace_expense(paid_by_her).unwrap();
    let paid_back = edited_expense(&group, dinner, |builder| builder.paid_by(ana));
    group.replace_expense(paid_back).unwrap();
    // Replacing one she was not in cannot add her.
    let with_her = edited_expense(&group, lunch, |builder| {
        builder.split(Split::equal([ana, bob, carol]))
    });
    assert_unchanged(&mut group, &GroupError::ArchivedMember(carol), |group| {
        group.replace_expense(with_her)
    });
    group.remove_expense(lunch).unwrap();
    // She can still settle up.
    group.add_transfer(transfer(carol, ana, "12.00")).unwrap();
    assert_eq!(
        group.summary().member(carol).balance(),
        eranos_ledger::Balance::Settled
    );
    // Unarchived, she joins expenses again.
    group.unarchive_member(carol).unwrap();
    group
        .add_expense(shared("5.00", ana, &[ana, carol]))
        .unwrap();
    assert_eq!(group.ledger, rebuilt(&group));
}

/// A referenced member cannot be removed, and the error lists what still
/// references them; an unreferenced one can.
#[test]
fn only_an_unreferenced_member_can_be_removed() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    let dinner = group
        .add_expense(shared("20.00", ana, &[ana, bob]))
        .unwrap();
    let repayment = group.add_transfer(transfer(bob, carol, "1.00")).unwrap();

    assert_unchanged(
        &mut group,
        &GroupError::MemberInUse {
            member: bob,
            expenses: vec![dinner],
            transfers: vec![repayment],
        },
        |group| group.remove_member(bob),
    );
    assert_unchanged(
        &mut group,
        &GroupError::MemberInUse {
            member: carol,
            expenses: vec![],
            transfers: vec![repayment],
        },
        |group| group.remove_member(carol),
    );
    // Once the transfer is gone, Carol can be removed.
    group.remove_transfer(repayment).unwrap();
    assert_eq!(
        group
            .remove_member(carol)
            .map(|member| member.first_name().to_owned()),
        Ok("Carol".to_owned())
    );
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.remove_member(stranger)
    });
}

/// Category names are unique under canonical caseless matching, renames
/// keep that rule, and an unused category is removed.
#[test]
fn categories_keep_unique_names() {
    let Trip { mut group, .. } = trip();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let fuel = group.add_category(Category::new("Fuel").unwrap()).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: food },
        |group| group.add_category(Category::new("FOOD").unwrap()),
    );
    // Renaming to its own name in another case is fine; to another's is not.
    let renamed = group.category(food).unwrap().renamed("food").unwrap();
    group.replace_category(renamed).unwrap();
    let clashing = group.category(fuel).unwrap().renamed("Food").unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: food },
        |group| group.replace_category(clashing),
    );
    // The same category is added once.
    let again = group.category(fuel).unwrap().renamed("Other").unwrap();
    assert_unchanged(&mut group, &GroupError::DuplicateCategory(fuel), |group| {
        group.add_category(again)
    });

    // An unused one goes, and unknown ones are reported.
    let removed = group.remove_category(fuel).unwrap();
    assert_eq!(removed.name(), "Fuel");
    assert_unchanged(&mut group, &GroupError::UnknownCategory(fuel), |group| {
        group.remove_category(fuel)
    });
    assert_unchanged(&mut group, &GroupError::UnknownCategory(fuel), |group| {
        group.replace_category(removed)
    });
    assert_eq!(
        group.categories().map(Category::id).collect::<Vec<_>>(),
        vec![CategoryId::UNCATEGORISED, food]
    );
    assert_eq!(group.category(food).map(Category::name), Some("food"));
}

/// A group would hold `Straße` and `STRASSE`, or a composed and a
/// decomposed `Café`, as two categories that users see as one.
#[test]
fn category_names_compare_by_canonical_caseless_matching() {
    let Trip { mut group, .. } = trip();
    let street = group
        .add_category(Category::new("Straße").unwrap())
        .unwrap();
    let cafe = group.add_category(Category::new("Café").unwrap()).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: street },
        |group| group.add_category(Category::new("STRASSE").unwrap()),
    );
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: cafe },
        |group| group.add_category(Category::new("Cafe\u{301}").unwrap()),
    );
}

/// The built-in category would be removed, renamed, or added a second time,
/// leaving expenses with a category the group does not have.
#[test]
fn the_built_in_category_stays() {
    let Trip { mut group, .. } = trip();
    let built_in = group.category(CategoryId::UNCATEGORISED).unwrap().clone();
    assert_eq!(built_in.name(), "Uncategorised");
    assert_eq!(group.categories().len(), 1);
    assert_unchanged(&mut group, &GroupError::BuiltInCategory, |group| {
        group.remove_category(CategoryId::UNCATEGORISED)
    });
    let renamed = built_in.renamed("Other").unwrap();
    assert_unchanged(&mut group, &GroupError::BuiltInCategory, |group| {
        group.replace_category(renamed)
    });
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategory(CategoryId::UNCATEGORISED),
        |group| group.add_category(built_in),
    );
    // Its name is taken, in any case.
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName {
            existing: CategoryId::UNCATEGORISED,
        },
        |group| group.add_category(Category::new("uncategorised").unwrap()),
    );
}

/// Removing a category would lose its expenses' totals, change balances, or
/// drop cached settlements that do not depend on categories.
#[test]
fn removing_a_category_moves_its_expenses_to_uncategorised() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let lunch = Expense::builder("Lunch", eur("20.00"), ana, Split::equal([ana, bob]), day())
        .category(food)
        .build()
        .unwrap();
    let lunch = group.add_expense(lunch).unwrap();
    group
        .add_expense(shared("10.00", bob, &[ana, bob]))
        .unwrap();
    let balances: Vec<_> = group.summary().members().collect();
    let cached: *const Settlement = group.settlement(SettlementStrategy::Greedy).unwrap();
    assert_eq!(group.summary().category_total(food), eur("20.00"));

    group.remove_category(food).unwrap();
    // The expense and its totals move, per member too.
    assert_eq!(
        group.expense(lunch).map(Expense::category),
        Some(CategoryId::UNCATEGORISED)
    );
    let summary = group.summary();
    assert_eq!(
        summary.category_total(CategoryId::UNCATEGORISED),
        eur("30.00")
    );
    assert_eq!(summary.category_total(food), eur("0"));
    assert_eq!(
        summary.member_category_total(bob, CategoryId::UNCATEGORISED),
        eur("15.00")
    );
    assert_eq!(summary.member_category_totals(bob).count(), 1);
    // Balances, and with them the cached settlement, stay.
    assert_eq!(summary.members().collect::<Vec<_>>(), balances);
    assert!(std::ptr::eq(
        cached,
        group.settlement(SettlementStrategy::Greedy).unwrap()
    ));
    assert_eq!(group.ledger, rebuilt(&group));
}

/// An expense must reference existing members and categories and carry a
/// rate exactly when it is foreign; every refusal leaves the group as it
/// was.
#[test]
fn an_expense_must_fit_the_group() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.add_expense(shared("5.00", ana, &[ana, stranger]))
    });
    let missing = CategoryId::from_bytes([200; 16]);
    let categorised = shared("5.00", ana, &[ana])
        .to_builder()
        .category(missing)
        .build()
        .unwrap();
    assert_unchanged(&mut group, &GroupError::UnknownCategory(missing), |group| {
        group.add_expense(categorised)
    });
    // A foreign expense needs a converted cost; one in the group currency
    // must not have one, even at a rate of 1.
    let foreign = Expense::builder(
        "Taxi",
        Money::parse("50.00", Currency::RON).unwrap(),
        ana,
        Split::equal([ana, bob]),
        day(),
    )
    .build()
    .unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::MissingExchangeRate {
            currency: Currency::RON,
        },
        |group| group.add_expense(foreign.clone()),
    );
    let rated = shared("5.00", ana, &[ana])
        .to_builder()
        .cost(eur("5.00").at(ExchangeRate::ONE))
        .build()
        .unwrap();
    assert_unchanged(&mut group, &GroupError::UnexpectedExchangeRate, |group| {
        group.add_expense(rated)
    });
    // With its rate, the foreign expense counts in the group currency.
    let converted = foreign
        .to_builder()
        .cost(foreign.amount().at(ExchangeRate::parse("0.2").unwrap()))
        .build()
        .unwrap();
    group.add_expense(converted).unwrap();
    assert_eq!(group.summary().total_spent(), eur("10.00"));
}

/// Adding, replacing, and removing an expense move the totals by exactly
/// the difference, and the incremental totals always equal a rebuild.
#[test]
fn expenses_move_the_totals_incrementally() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    let empty = group.ledger.clone();
    let hotel = group
        .add_expense(shared("90.00", ana, &[ana, bob, carol]))
        .unwrap();
    assert_eq!(
        group.summary().member(bob).balance(),
        eranos_ledger::Balance::Owes(eur("30.00"))
    );
    assert_eq!(group.expense(hotel).map(Expense::name), Some("Shared"));
    assert_eq!(
        group.expense_shares(hotel).map(ExpenseShares::total),
        Some(eur("90.00"))
    );

    // Replacing moves the totals to the new version and returns the old one.
    let cheaper = edited_expense(&group, hotel, |builder| {
        builder
            .cost(eur("60.00"))
            .paid_by(bob)
            .split(Split::equal([ana, bob]))
    });
    let previous = group.replace_expense(cheaper).unwrap();
    assert_eq!(previous.amount(), eur("90.00"));
    assert_eq!(group.summary().total_spent(), eur("60.00"));
    assert_eq!(
        group.summary().member(ana).balance(),
        eranos_ledger::Balance::Owes(eur("30.00"))
    );
    assert_eq!(group.ledger, rebuilt(&group));

    // Removing restores the empty ledger exactly.
    let removed = group.remove_expense(hotel).unwrap();
    assert_eq!(removed.amount(), eur("60.00"));
    assert_eq!(group.ledger, empty);
    assert_unchanged(&mut group, &GroupError::UnknownExpense(hotel), |group| {
        group.remove_expense(hotel)
    });
    assert_unchanged(&mut group, &GroupError::UnknownExpense(hotel), |group| {
        group.replace_expense(removed.clone())
    });
    // An entry with a known identifier, such as one synced from another
    // device, is added once.
    group.add_expense(removed.clone()).unwrap();
    assert_unchanged(&mut group, &GroupError::DuplicateExpense(hotel), |group| {
        group.add_expense(removed)
    });
    assert_eq!(
        group.expenses().map(Expense::id).collect::<Vec<_>>(),
        vec![hotel]
    );
}

/// Transfers move balances, may involve archived members, convert like
/// expenses, and are replaced and removed exactly.
#[test]
fn transfers_move_balances_exactly() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    group
        .add_expense(shared("30.00", ana, &[ana, bob, carol]))
        .unwrap();
    group.archive_member(bob).unwrap();
    let repayment = group.add_transfer(transfer(bob, ana, "10.00")).unwrap();
    assert_eq!(
        group.summary().member(bob).balance(),
        eranos_ledger::Balance::Settled
    );
    assert_eq!(group.transfer_amount(repayment), Some(eur("10.00")));
    assert_eq!(group.transfer(repayment).map(Transfer::from), Some(bob));

    // A cash repayment in another currency converts with its rate.
    let lei = Money::parse("50.00", Currency::RON).unwrap();
    let cash = edited_transfer(&group, repayment, |builder| {
        builder
            .cost(lei.at(ExchangeRate::parse("0.2").unwrap()))
            .from(carol)
    });
    let previous = group.replace_transfer(cash).unwrap();
    assert_eq!(previous.amount(), eur("10.00"));
    assert_eq!(group.transfer_amount(repayment), Some(eur("10.00")));
    assert_eq!(
        group.summary().member(carol).balance(),
        eranos_ledger::Balance::Settled
    );
    assert_eq!(
        group.summary().member(bob).balance(),
        eranos_ledger::Balance::Owes(eur("10.00"))
    );
    assert_eq!(group.ledger, rebuilt(&group));

    // Unknown members, unknown and duplicate transfers are refused.
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.add_transfer(transfer(ana, stranger, "1.00"))
    });
    let copy = group.transfer(repayment).unwrap().clone();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateTransfer(repayment),
        |group| group.add_transfer(copy),
    );
    let removed = group.remove_transfer(repayment).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::UnknownTransfer(repayment),
        |group| group.remove_transfer(repayment),
    );
    assert_unchanged(
        &mut group,
        &GroupError::UnknownTransfer(repayment),
        |group| group.replace_transfer(removed),
    );
    assert_eq!(group.transfers().len(), 0);
    assert_eq!(group.transfer_amount(repayment), None);
}

/// A cached settlement is reused until a balance changes, and changes that
/// do not touch balances keep it.
#[test]
fn settlements_are_cached_until_a_balance_changes() {
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    group
        .add_expense(shared("30.00", ana, &[ana, bob, carol]))
        .unwrap();
    let first: *const Settlement = group.settlement(SettlementStrategy::Greedy).unwrap();
    assert!(std::ptr::eq(
        first,
        group.settlement(SettlementStrategy::Greedy).unwrap()
    ));
    assert_eq!(
        group.settlement(SettlementStrategy::Greedy).unwrap().len(),
        2
    );

    // Members and categories do not move balances, so the cache stays.
    group.add_member(person("Dan")).unwrap();
    group.add_category(Category::new("Food").unwrap()).unwrap();
    assert!(std::ptr::eq(
        first,
        group.settlement(SettlementStrategy::Greedy).unwrap()
    ));

    // A transfer does, so the next request recomputes.
    group.add_transfer(transfer(bob, ana, "10.00")).unwrap();
    assert_eq!(
        group.settlement(SettlementStrategy::Greedy).unwrap().len(),
        1
    );
    for strategy in SettlementStrategy::all() {
        assert_eq!(
            group.settlement(strategy).unwrap().payments()[0].from(),
            carol
        );
    }
}

/// A plan computed asynchronously would be computed again by the
/// synchronous call, or kept after the balances change.
#[test]
fn an_asynchronous_settlement_fills_the_cache_the_synchronous_call_reads() {
    // Sixteen members each paid a different amount for a seventeenth:
    // seventeen open balances, so the exact solver yields once.
    let mut group = Group::builder("Club", Currency::EUR).build().unwrap();
    let hub = group.add_member(person("Hub")).unwrap();
    let mut last = hub;
    for minor_units in 1..=16 {
        last = group.add_member(person("Member")).unwrap();
        group
            .add_expense(paid_for(cents(minor_units), last, hub))
            .unwrap();
    }
    let strategy = SettlementStrategy::MinimumTransfers;
    {
        let mut context = Context::from_waker(Waker::noop());
        let mut call = pin!(group.settlement_async(strategy).into_future());
        assert!(call.as_mut().poll(&mut context).is_pending());
        let Poll::Ready(Ok(computed)) = call.as_mut().poll(&mut context) else {
            panic!("the plan did not complete after its one yield");
        };
        assert_eq!(computed.len(), 16);
        assert!(std::ptr::eq(computed, group.settlement(strategy).unwrap()));
    }

    // Settling the last member's balance clears the cache.
    group.add_transfer(sent(hub, last, cents(16))).unwrap();
    assert_eq!(
        group
            .settlement_async(strategy)
            .complete()
            .map(Settlement::len),
        Ok(15)
    );
}

/// A currency change re-converts every entry and rewrites the rates, and a
/// missing or repeated rate changes nothing.
#[test]
fn a_currency_change_reconverts_every_entry() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    let local = group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    let foreign = Expense::builder(
        "Taxi",
        Money::parse("50.00", Currency::RON)
            .unwrap()
            .at(ExchangeRate::parse("0.2").unwrap()),
        bob,
        Split::equal([ana, bob]),
        day(),
    )
    .build()
    .unwrap();
    let foreign = group.add_expense(foreign).unwrap();
    let repayment = group.add_transfer(transfer(bob, ana, "5.00")).unwrap();
    let eur_to_ron = ExchangeRate::parse("5").unwrap();

    // Moving to the current currency does nothing.
    group.change_currency(Currency::EUR, &[]).unwrap();
    // Every currency in use needs exactly one rate.
    assert_unchanged(
        &mut group,
        &GroupError::MissingExchangeRate {
            currency: Currency::EUR,
        },
        |group| group.change_currency(Currency::RON, &[]),
    );
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateExchangeRate {
            currency: Currency::EUR,
        },
        |group| {
            group.change_currency(
                Currency::RON,
                &[
                    CurrencyRate::new(Currency::EUR, eur_to_ron),
                    CurrencyRate::new(Currency::EUR, eur_to_ron),
                ],
            )
        },
    );

    group
        .change_currency(
            Currency::RON,
            &[CurrencyRate::new(Currency::EUR, eur_to_ron)],
        )
        .unwrap();
    assert_eq!(group.currency(), Currency::RON);
    // 10.00 EUR became 50.00 RON; the RON taxi is now local.
    let ron = |text| Money::parse(text, Currency::RON).unwrap();
    assert_eq!(group.summary().total_spent(), ron("100.00"));
    assert_eq!(
        group.expense(local).map(Expense::cost),
        Some(eur("10.00").at(eur_to_ron))
    );
    assert_eq!(
        group.expense(foreign).map(Expense::cost),
        Some(Cost::from(ron("50.00")))
    );
    assert_eq!(group.transfer_amount(repayment), Some(ron("25.00")));
    assert_eq!(group.ledger, rebuilt(&group));
}

/// A currency change dropped at one of its yields would leave some entries
/// converted and others not: it changes nothing until it completes.
#[test]
fn a_currency_change_changes_nothing_until_it_completes() {
    // 2 100 entries: the change yields after the 1 024th and the 2 048th.
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    for _ in 0..2_000 {
        group.add_expense(shared("1.00", ana, &[ana, bob])).unwrap();
    }
    for _ in 0..100 {
        group.add_transfer(transfer(bob, ana, "0.50")).unwrap();
    }
    let rates = [CurrencyRate::new(Currency::EUR, two())];
    let mut context = Context::from_waker(Waker::noop());
    for yields in 1..=2 {
        let mut changed = group.clone();
        {
            let call = changed.change_currency_async(Currency::RON, &rates);
            let mut call = pin!(call.into_future());
            for _ in 0..yields {
                assert!(call.as_mut().poll(&mut context).is_pending());
            }
        }
        assert_eq!(changed, group);
        assert_eq!(changed.ledger, group.ledger);
    }

    // Polled to the end, it yields twice and converts every entry.
    {
        let call = group.change_currency_async(Currency::RON, &rates);
        let mut call = pin!(call.into_future());
        for _ in 0..2 {
            assert!(call.as_mut().poll(&mut context).is_pending());
        }
        assert_eq!(call.as_mut().poll(&mut context), Poll::Ready(Ok(())));
    }
    assert_eq!(group.currency(), Currency::RON);
    assert_eq!(
        group.summary().total_spent(),
        Money::parse("4000.00", Currency::RON).unwrap()
    );
    assert_eq!(group.ledger, rebuilt(&group));
}

/// A transfer would be totalled in the wrong currency if its cost were not
/// checked as an expense's is: a foreign transfer needs its rate, and one in
/// the group currency must not carry one, when it is added or replaced.
#[test]
fn a_transfer_cost_must_suit_the_group_currency() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    let lei = || {
        Transfer::builder(
            ana,
            bob,
            Money::parse("100.00", Currency::RON).unwrap(),
            day(),
        )
        .build()
        .unwrap()
    };
    let missing = GroupError::MissingExchangeRate {
        currency: Currency::RON,
    };
    assert_unchanged(&mut group, &missing, |group| group.add_transfer(lei()));
    let rated = Transfer::builder(
        ana,
        bob,
        eur("25.00").at(ExchangeRate::parse("2").unwrap()),
        day(),
    )
    .build()
    .unwrap();
    assert_unchanged(&mut group, &GroupError::UnexpectedExchangeRate, |group| {
        group.add_transfer(rated)
    });
    // A replacement is checked the same way.
    let cash = group.add_transfer(transfer(ana, bob, "25.00")).unwrap();
    let unrated = edited_transfer(&group, cash, |builder| builder.cost(lei().cost()));
    assert_unchanged(&mut group, &missing, |group| {
        group.replace_transfer(unrated)
    });
}

/// A currency pegged at par, such as the Bermudian dollar to the US dollar,
/// is still another currency: a local cost in it would be counted as
/// dollars.
#[test]
fn a_pegged_currency_needs_its_rate_even_at_par() {
    let mut group = Group::builder("Bermuda", Currency::USD).build().unwrap();
    let ana = group.add_member(person("Ana")).unwrap();
    let bermudian_dollars = Money::from_minor_units(5_000, Currency::BMD);
    let ferry = |cost: Cost| {
        Expense::builder("Ferry", cost, ana, Split::equal([ana]), day())
            .build()
            .unwrap()
    };
    assert_unchanged(
        &mut group,
        &GroupError::MissingExchangeRate {
            currency: Currency::BMD,
        },
        |group| group.add_expense(ferry(Cost::from(bermudian_dollars))),
    );
    group
        .add_expense(ferry(bermudian_dollars.at(ExchangeRate::ONE)))
        .unwrap();
    assert_eq!(
        group.summary().total_spent(),
        Money::from_minor_units(5_000, Currency::USD)
    );
}

/// Groups compare by their details and entries, never by their caches.
#[test]
fn equality_ignores_the_caches() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    let copy = group.clone();
    group.settlement(SettlementStrategy::Direct).unwrap();
    assert_eq!(group, copy);
    // Any entry difference is a difference.
    let mut other = copy.clone();
    other.add_transfer(transfer(bob, ana, "5.00")).unwrap();
    assert_ne!(group, other);
    let mut archived = copy.clone();
    archived.archive_member(bob).unwrap();
    assert_ne!(copy, archived);
}

/// A change whose totals would overflow, or whose conversion leaves the
/// range of an amount, is refused and leaves the group unchanged.
#[test]
fn an_overflowing_expense_is_refused() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    let huge = || {
        let amount = Money::from_minor_units(i64::MAX, Currency::EUR);
        Expense::builder("Huge", amount, ana, Split::equal([bob]), day())
            .build()
            .unwrap()
    };
    group.add_expense(huge()).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.add_expense(huge())
    });
    let foreign = Expense::builder(
        "Foreign",
        huge_in_lei().at(two()),
        ana,
        Split::equal([bob]),
        day(),
    )
    .build()
    .unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.add_expense(foreign)
    });
}

/// Archiving or unarchiving someone who is not a member is an error, not a
/// silent no-op that would hide a stale identifier.
#[test]
fn only_members_are_archived_or_unarchived() {
    let Trip { mut group, .. } = trip();
    let stranger = MemberId::from_bytes([200; 16]);
    let unknown = GroupError::UnknownMember(stranger);
    assert_unchanged(&mut group, &unknown, |group| group.archive_member(stranger));
    assert_unchanged(&mut group, &unknown, |group| {
        group.unarchive_member(stranger)
    });
}

/// Amounts for the range tests, in euro cents: two of `LARGE` fit in an
/// amount, and `HUGE` leaves room for less than `EXTRA`.
const LARGE: i64 = 4_000_000_000_000_000_000;
const HUGE: i64 = 9_000_000_000_000_000_000;
const EXTRA: i64 = 1_000_000_000_000_000_000;

fn cents(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, Currency::EUR)
}

fn paid_for(amount: Money, payer: MemberId, participant: MemberId) -> Expense {
    Expense::builder("Large", amount, payer, Split::equal([participant]), day())
        .build()
        .unwrap()
}

fn sent(from: MemberId, to: MemberId, amount: Money) -> Transfer {
    Transfer::builder(from, to, amount, day()).build().unwrap()
}

/// `HUGE` in lei: out of range once converted at two euros per leu.
fn huge_in_lei() -> Money {
    Money::from_minor_units(HUGE, Currency::RON)
}

fn two() -> ExchangeRate {
    ExchangeRate::parse("2").unwrap()
}

/// Moves the group to lei at 2 lei per euro.
fn doubled(group: &mut Group) -> Result<(), GroupError> {
    group.change_currency(Currency::RON, &[CurrencyRate::new(Currency::EUR, two())])
}

/// A currency change needs a rate for the currency of every transfer as
/// well as every expense, or that transfer could not be converted.
#[test]
fn a_currency_change_needs_a_rate_for_every_transfer_too() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    let cash = Transfer::builder(
        bob,
        ana,
        Money::parse("100", Currency::HUF)
            .unwrap()
            .at(ExchangeRate::parse("0.0025").unwrap()),
        day(),
    )
    .build()
    .unwrap();
    group.add_transfer(cash).unwrap();
    // A rate for the euro expense, none for the forint transfer.
    assert_unchanged(
        &mut group,
        &GroupError::MissingExchangeRate {
            currency: Currency::HUF,
        },
        |group| {
            group.change_currency(
                Currency::RON,
                &[CurrencyRate::new(
                    Currency::EUR,
                    ExchangeRate::parse("5").unwrap(),
                )],
            )
        },
    );
}

/// A currency change that would convert an expense out of range changes
/// nothing.
#[test]
fn a_currency_change_converting_an_expense_out_of_range_changes_nothing() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group.add_expense(paid_for(cents(HUGE), ana, bob)).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A currency change that would convert a transfer out of range changes
/// nothing.
#[test]
fn a_currency_change_converting_a_transfer_out_of_range_changes_nothing() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group.add_transfer(sent(ana, bob, cents(HUGE))).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A currency change whose entries convert in range but whose totals do
/// not changes nothing.
#[test]
fn a_currency_change_with_totals_out_of_range_changes_nothing() {
    // Each doubles to 8e18; together they are past i64::MAX.
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    for _ in 0..2 {
        group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    }
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A replacement that would push a total out of range, or whose conversion
/// leaves the range, is refused and leaves the original in place.
#[test]
fn a_replacement_out_of_range_is_refused() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    let second = group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    // LARGE and HUGE together are past i64::MAX.
    let huge = edited_expense(&group, second, |builder| builder.cost(cents(HUGE)));
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_expense(huge)
    });
    let foreign = edited_expense(&group, second, |builder| {
        builder.cost(huge_in_lei().at(two()))
    });
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_expense(foreign)
    });
}

/// Removing an expense can overflow too, when it is what keeps two large
/// balances in range; the removal is then refused.
#[test]
fn removing_an_expense_out_of_range_is_refused() {
    // Ana and Bob pay LARGE for each other and Bob sends Ana HUGE: without
    // Ana's payment for Bob, Ana would owe 1.3e19.
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group.add_expense(paid_for(cents(LARGE), bob, ana)).unwrap();
    let for_bob = group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    group.add_transfer(sent(bob, ana, cents(HUGE))).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.remove_expense(for_bob)
    });
}

/// Removing a transfer can overflow too, when it is what keeps a large
/// balance in range; the removal is then refused.
#[test]
fn removing_a_transfer_out_of_range_is_refused() {
    // Ana is owed HUGE and passes on to Carol what Bob paid her back:
    // without Bob's repayment, Ana would be owed 1e19.
    let Trip {
        mut group,
        ana,
        bob,
        carol,
    } = trip();
    group.add_expense(paid_for(cents(HUGE), ana, bob)).unwrap();
    let repayment = group.add_transfer(sent(bob, ana, cents(EXTRA))).unwrap();
    group.add_transfer(sent(ana, carol, cents(EXTRA))).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.remove_transfer(repayment)
    });
}

/// A transfer that would push a balance out of range, or whose conversion
/// leaves the range, is refused; so is a replacement with either problem
/// or with a stranger in it.
#[test]
fn a_transfer_out_of_range_is_refused() {
    let Trip {
        mut group,
        ana,
        bob,
        ..
    } = trip();
    group.add_expense(paid_for(cents(HUGE), ana, bob)).unwrap();
    // Ana is owed HUGE; sending Bob EXTRA would put her past i64::MAX.
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.add_transfer(sent(ana, bob, cents(EXTRA)))
    });
    let foreign = Transfer::builder(bob, ana, huge_in_lei().at(two()), day())
        .build()
        .unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.add_transfer(foreign.clone())
    });

    let repayment = group.add_transfer(sent(bob, ana, cents(EXTRA))).unwrap();
    // The other way round, Ana would be owed 1e19.
    let reversed = edited_transfer(&group, repayment, |builder| builder.from(ana).to(bob));
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_transfer(reversed)
    });
    let converted = edited_transfer(&group, repayment, |builder| builder.cost(foreign.cost()));
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_transfer(converted)
    });
    let stranger = MemberId::from_bytes([200; 16]);
    let from_a_stranger = edited_transfer(&group, repayment, |builder| builder.from(stranger));
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.replace_transfer(from_a_stranger)
    });
}

/// A participant drawn for a random expense: the position of the member,
/// and their parts in hundredths.
#[derive(Debug, Clone, Copy)]
struct DrawnParts {
    member: usize,
    hundredths: u32,
}

/// Draws one change to apply to a five-member group.
#[derive(Debug, Clone)]
enum Operation {
    Archive {
        member: usize,
    },
    Unarchive {
        member: usize,
    },
    RemoveCategory {
        category: usize,
    },
    AddExpense {
        amount: i64,
        payer: usize,
        participants: Vec<DrawnParts>,
        category: usize,
    },
    ReplaceExpense {
        position: usize,
        amount: i64,
        payer: usize,
    },
    RemoveExpense {
        position: usize,
    },
    AddTransfer {
        from: usize,
        to: usize,
        amount: i64,
    },
    RemoveTransfer {
        position: usize,
    },
}

/// Draws a participant of a random expense.
fn drawn_parts() -> impl Strategy<Value = DrawnParts> {
    (0_usize..5, 1_u32..400).prop_map(|(member, hundredths)| DrawnParts { member, hundredths })
}

fn operation() -> impl Strategy<Value = Operation> {
    prop_oneof![
        4 => (1_i64..100_000, 0_usize..3, 0_usize..5, proptest::collection::vec(drawn_parts(), 1..5))
            .prop_map(|(amount, category, payer, participants)| Operation::AddExpense { amount, payer, participants, category }),
        2 => (0_usize..8, 1_i64..100_000, 0_usize..5)
            .prop_map(|(position, amount, payer)| Operation::ReplaceExpense { position, amount, payer }),
        1 => (0_usize..8).prop_map(|position| Operation::RemoveExpense { position }),
        2 => (0_usize..5, 0_usize..5, 1_i64..50_000).prop_map(|(from, to, amount)| Operation::AddTransfer { from, to, amount }),
        1 => (0_usize..8).prop_map(|position| Operation::RemoveTransfer { position }),
        1 => (0_usize..5).prop_map(|member| Operation::Archive { member }),
        1 => (0_usize..5).prop_map(|member| Operation::Unarchive { member }),
        1 => (0_usize..3).prop_map(|category| Operation::RemoveCategory { category }),
    ]
}

/// Applies `operation`, returning whether the group accepted it;
/// `categories` starts with the built-in one.
fn apply(
    group: &mut Group,
    members: &[MemberId],
    categories: &[CategoryId],
    operation: &Operation,
) -> bool {
    let money = |amount| Money::from_minor_units(amount, Currency::EUR);
    let expense_at =
        |current: &Group, position: usize| current.expenses().map(Expense::id).nth(position);
    let transfer_at =
        |current: &Group, position: usize| current.transfers().map(Transfer::id).nth(position);
    match operation.clone() {
        Operation::Archive { member } => group.archive_member(members[member]).is_ok(),
        Operation::Unarchive { member } => group.unarchive_member(members[member]).is_ok(),
        Operation::RemoveCategory { category } => {
            group.remove_category(categories[category]).is_ok()
        }
        Operation::AddExpense {
            amount,
            payer,
            participants,
            category,
        } => {
            let mut chosen: Vec<MemberParts> = Vec::new();
            for participant in participants {
                let member = members[participant.member];
                if chosen.iter().all(|share| share.member() != member) {
                    chosen.push(MemberParts::new(
                        member,
                        Parts::from_hundredths(participant.hundredths).unwrap(),
                    ));
                }
            }
            let expense = Expense::builder(
                "Random",
                money(amount),
                members[payer],
                Split::parts(chosen),
                day(),
            )
            .category(categories[category])
            .build()
            .unwrap();
            group.add_expense(expense).is_ok()
        }
        Operation::ReplaceExpense {
            position,
            amount,
            payer,
        } => {
            let Some(id) = expense_at(group, position) else {
                return false;
            };
            let expense = group
                .expense(id)
                .unwrap()
                .to_builder()
                .cost(money(amount))
                .paid_by(members[payer]);
            let Ok(expense) = expense.build() else {
                return false;
            };
            group.replace_expense(expense).is_ok()
        }
        Operation::RemoveExpense { position } => {
            expense_at(group, position).is_some_and(|id| group.remove_expense(id).is_ok())
        }
        Operation::AddTransfer { from, to, amount } => {
            let Ok(transfer) =
                Transfer::builder(members[from], members[to], money(amount), day()).build()
            else {
                return false;
            };
            group.add_transfer(transfer).is_ok()
        }
        Operation::RemoveTransfer { position } => {
            transfer_at(group, position).is_some_and(|id| group.remove_transfer(id).is_ok())
        }
    }
}

proptest! {
    /// Whatever sequence of changes the apps make, the incremental totals
    /// equal a full rebuild, balances sum to zero, a refused change leaves
    /// the group untouched, and every settlement clears every balance.
    #[test]
    fn random_changes_keep_the_group_consistent(operations in proptest::collection::vec(operation(), 1..40)) {
        let mut group = Group::builder("Random", Currency::EUR).build().unwrap();
        let members: Vec<MemberId> = ["Ana", "Bob", "Carol", "Dan", "Eve"].iter().map(|name| group.add_member(person(name)).unwrap()).collect();
        let mut categories = vec![CategoryId::UNCATEGORISED];
        for name in ["Food", "Fuel"] {
            categories.push(group.add_category(Category::new(name).unwrap()).unwrap());
        }
        for operation in &operations {
            let before = group.clone();
            if !apply(&mut group, &members, &categories, operation) {
                prop_assert_eq!(&group, &before);
                prop_assert_eq!(&group.ledger, &before.ledger);
            }
            prop_assert_eq!(&group.ledger, &rebuilt(&group));
            let nets: i64 = group.summary().members().map(|summary| summary.net().minor_units()).sum();
            prop_assert_eq!(nets, 0);
        }
        for strategy in SettlementStrategy::all() {
            let mut settled = group.clone();
            let payments = settled.settlement(strategy).unwrap().payments().to_vec();
            for payment in payments {
                settled.add_transfer(Transfer::from_payment(&payment, day())).unwrap();
            }
            prop_assert!(settled.summary().members().all(|summary| summary.net().is_zero()));
        }
    }
}
