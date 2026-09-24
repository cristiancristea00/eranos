//! Tests for the group aggregate. The group is the only way the apps change
//! data, so these pin its consistency rules: references are checked, a
//! failed change leaves the group exactly as it was, the incremental totals
//! always equal a full rebuild, archived members follow their rules, cached
//! settlements are dropped exactly when balances change, and a currency
//! change re-converts every entry. The random-operations property drives
//! all of it at once.

use proptest::prelude::*;

use super::*;
use crate::date::Date;
use crate::expense::Split;
use crate::money::{Cost, Parts};

fn eur(text: &str) -> Money {
    Money::parse(text, Currency::EUR).unwrap()
}

fn day() -> Date {
    Date::new(2026, 9, 23).unwrap()
}

fn person(name: &str) -> Member {
    Member::builder(name).build().unwrap()
}

/// A group with three members, in creation order.
fn trip() -> (Group, MemberId, MemberId, MemberId) {
    let mut group = Group::builder("Trip", Currency::EUR).build().unwrap();
    let ana = group.add_member(person("Ana"));
    let bob = group.add_member(person("Bob"));
    let carol = group.add_member(person("Carol"));
    (group, ana, bob, carol)
}

fn shared(amount: &str, payer: MemberId, participants: &[MemberId]) -> Expense {
    Expense::builder("Shared", eur(amount), day())
        .paid_by(payer)
        .split(Split::equal(participants.iter().copied()))
        .build()
        .unwrap()
}

fn transfer(from: MemberId, to: MemberId, amount: &str) -> Transfer {
    Transfer::builder(from, to, eur(amount), day())
        .build()
        .unwrap()
}

/// The ledger recomputed from scratch from the group's entries, checking
/// every stored resolution along the way.
fn rebuilt(group: &Group) -> Ledger {
    let mut delta = Delta::default();
    for (&id, record) in &group.expenses {
        let shares = resolve_expense(id, &record.expense, group.currency).unwrap();
        assert_eq!(shares, record.shares);
        delta.add_expense(&shares, record.expense.category(), 1);
    }
    for record in group.transfers.values() {
        assert_eq!(
            resolve_transfer(&record.transfer, group.currency).unwrap(),
            record.amount
        );
        delta.add_transfer(
            record.transfer.from(),
            record.transfer.to(),
            record.amount,
            1,
        );
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

const fn assert_send_sync<T: Send + Sync>() {}

/// Every public type crosses threads, which the future binding crates rely
/// on (`UniFFI` objects must be `Send + Sync`).
#[test]
fn every_public_type_is_send_and_sync() {
    const {
        assert_send_sync::<Group>();
        assert_send_sync::<Summary>();
        assert_send_sync::<Settlement>();
        assert_send_sync::<Expense>();
        assert_send_sync::<Member>();
        assert_send_sync::<Transfer>();
        assert_send_sync::<GroupError>();
    }
}

/// A new group starts empty, with its normalised details.
#[test]
fn a_new_group_starts_empty() {
    let id = GroupId::from_bytes([9; 16]);
    let group = Group::builder("  Lisbon ", Currency::EUR)
        .id(id)
        .description(" ")
        .language(Language::parse("pt-PT").unwrap())
        .build()
        .unwrap();
    assert_eq!(group.id(), id);
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
    let (mut group, ana, bob, _) = trip();
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

/// Members get fresh increasing identifiers, and an imported identifier
/// cannot be reused.
#[test]
fn members_are_added_updated_and_listed_in_creation_order() {
    let (mut group, ana, bob, carol) = trip();
    let ids: Vec<MemberId> = group.members().map(|(id, _)| id).collect();
    assert_eq!(ids, vec![ana, bob, carol]);
    assert!(ana < bob && bob < carol);
    // Updating returns the previous details.
    let previous = group.update_member(bob, person("Robert")).unwrap();
    assert_eq!(previous.first_name(), "Bob");
    assert_eq!(group.member(bob).map(Member::first_name), Some("Robert"));
    // An imported identifier works once.
    let imported = MemberId::from_bytes([7; 16]);
    group.insert_member(imported, person("Dan")).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateMember(imported),
        |group| group.insert_member(imported, person("Dan")),
    );
    // Unknown members are reported.
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.update_member(stranger, person("Eve"))
    });
    assert_eq!(group.member(stranger), None);
}

/// Archiving keeps a member's history and balance but keeps them out of new
/// expenses, while transfers and replacements that keep them still work.
#[test]
fn an_archived_member_keeps_history_but_joins_no_new_expense() {
    let (mut group, ana, bob, carol) = trip();
    let dinner = group
        .add_expense(shared("30.00", ana, &[ana, bob, carol]))
        .unwrap();
    let lunch = group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    group.archive_member(carol).unwrap();
    group.archive_member(carol).unwrap();
    assert!(group.is_archived(carol));
    assert!(!group.is_archived(MemberId::from_bytes([200; 16])));

    // A new expense with Carol is refused.
    assert_unchanged(&mut group, &GroupError::ArchivedMember(carol), |group| {
        group.add_expense(shared("5.00", ana, &[ana, carol]))
    });
    // Replacing an expense she was in may keep her, in any role.
    group
        .replace_expense(dinner, shared("36.00", ana, &[ana, bob, carol]))
        .unwrap();
    group
        .replace_expense(dinner, shared("36.00", carol, &[ana, bob, carol]))
        .unwrap();
    group
        .replace_expense(dinner, shared("36.00", ana, &[ana, bob, carol]))
        .unwrap();
    // Replacing one she was not in cannot add her.
    assert_unchanged(&mut group, &GroupError::ArchivedMember(carol), |group| {
        group.replace_expense(lunch, shared("10.00", ana, &[ana, bob, carol]))
    });
    group.remove_expense(lunch).unwrap();
    // She can still settle up.
    group.add_transfer(transfer(carol, ana, "12.00")).unwrap();
    assert_eq!(
        group.summary().member(carol).balance(),
        crate::ledger::Balance::Settled
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
    let (mut group, ana, bob, carol) = trip();
    let dinner = group
        .add_expense(shared("20.00", ana, &[ana, bob]))
        .unwrap();
    let repayment = group.add_transfer(transfer(bob, carol, "1.00")).unwrap();

    assert_unchanged(
        &mut group,
        &GroupError::MemberInUse {
            expenses: vec![dinner],
            member: bob,
            transfers: vec![repayment],
        },
        |group| group.remove_member(bob),
    );
    assert_unchanged(
        &mut group,
        &GroupError::MemberInUse {
            expenses: vec![],
            member: carol,
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

/// Category names are unique case-insensitively, renames keep that rule,
/// and a category in use cannot be removed.
#[test]
fn categories_keep_unique_names_and_their_expenses() {
    let (mut group, ana, bob, _) = trip();
    let food = group.add_category(Category::new("Food").unwrap()).unwrap();
    let fuel = group.add_category(Category::new("Fuel").unwrap()).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: food },
        |group| group.add_category(Category::new("FOOD").unwrap()),
    );
    // Renaming to its own name in another case is fine; to another's is not.
    group
        .rename_category(food, Category::new("food").unwrap())
        .unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateCategoryName { existing: food },
        |group| group.rename_category(fuel, Category::new("Food").unwrap()),
    );
    assert_unchanged(&mut group, &GroupError::DuplicateCategory(fuel), |group| {
        group.insert_category(fuel, Category::new("Other").unwrap())
    });

    // A category in use stays.
    let lunch = Expense::builder("Lunch", eur("20.00"), day())
        .paid_by(ana)
        .split(Split::equal([ana, bob]))
        .category(food)
        .build()
        .unwrap();
    let lunch = group.add_expense(lunch).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::CategoryInUse {
            category: food,
            expenses: vec![lunch],
        },
        |group| group.remove_category(food),
    );
    assert_eq!(group.summary().category_total(Some(food)), eur("20.00"));
    // An unused one goes, and unknown ones are reported.
    assert_eq!(
        group
            .remove_category(fuel)
            .map(|category| category.name().to_owned()),
        Ok("Fuel".to_owned())
    );
    assert_unchanged(&mut group, &GroupError::UnknownCategory(fuel), |group| {
        group.remove_category(fuel)
    });
    assert_unchanged(&mut group, &GroupError::UnknownCategory(fuel), |group| {
        group.rename_category(fuel, Category::new("Fuel").unwrap())
    });
    assert_eq!(
        group.categories().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![food]
    );
    assert_eq!(group.category(food).map(Category::name), Some("food"));
}

/// An expense must reference existing members and categories and carry a
/// rate exactly when it is foreign; every refusal leaves the group as it
/// was.
#[test]
fn an_expense_must_fit_the_group() {
    let (mut group, ana, bob, _) = trip();
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
    let foreign = Expense::builder("Taxi", Money::parse("50.00", Currency::RON).unwrap(), day())
        .paid_by(ana)
        .split(Split::equal([ana, bob]))
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
    let (mut group, ana, bob, carol) = trip();
    let empty = group.ledger.clone();
    let hotel = group
        .add_expense(shared("90.00", ana, &[ana, bob, carol]))
        .unwrap();
    assert_eq!(
        group.summary().member(bob).balance(),
        crate::ledger::Balance::Owes(eur("30.00"))
    );
    assert_eq!(group.expense(hotel).map(Expense::name), Some("Shared"));
    assert_eq!(
        group.expense_shares(hotel).map(ExpenseShares::total),
        Some(eur("90.00"))
    );

    // Replacing moves the totals to the new version and returns the old one.
    let previous = group
        .replace_expense(hotel, shared("60.00", bob, &[ana, bob]))
        .unwrap();
    assert_eq!(previous.amount(), eur("90.00"));
    assert_eq!(group.summary().total_spent(), eur("60.00"));
    assert_eq!(
        group.summary().member(ana).balance(),
        crate::ledger::Balance::Owes(eur("30.00"))
    );
    assert_eq!(group.ledger, rebuilt(&group));

    // Removing restores the empty ledger exactly.
    assert_eq!(
        group.remove_expense(hotel).map(|expense| expense.amount()),
        Ok(eur("60.00"))
    );
    assert_eq!(group.ledger, empty);
    assert_unchanged(&mut group, &GroupError::UnknownExpense(hotel), |group| {
        group.remove_expense(hotel)
    });
    assert_unchanged(&mut group, &GroupError::UnknownExpense(hotel), |group| {
        group.replace_expense(hotel, shared("1.00", ana, &[ana]))
    });
    // An imported identifier works once.
    group
        .insert_expense(hotel, shared("1.00", ana, &[ana]))
        .unwrap();
    assert_unchanged(&mut group, &GroupError::DuplicateExpense(hotel), |group| {
        group.insert_expense(hotel, shared("1.00", ana, &[ana]))
    });
    assert_eq!(
        group.expenses().map(|(id, _)| id).collect::<Vec<_>>(),
        vec![hotel]
    );
}

/// Transfers move balances, may involve archived members, convert like
/// expenses, and are replaced and removed exactly.
#[test]
fn transfers_move_balances_exactly() {
    let (mut group, ana, bob, carol) = trip();
    group
        .add_expense(shared("30.00", ana, &[ana, bob, carol]))
        .unwrap();
    group.archive_member(bob).unwrap();
    let repayment = group.add_transfer(transfer(bob, ana, "10.00")).unwrap();
    assert_eq!(
        group.summary().member(bob).balance(),
        crate::ledger::Balance::Settled
    );
    assert_eq!(group.transfer_amount(repayment), Some(eur("10.00")));
    assert_eq!(group.transfer(repayment).map(Transfer::from), Some(bob));

    // A cash repayment in another currency converts with its rate.
    let cash = Transfer::builder(
        carol,
        ana,
        Money::parse("50.00", Currency::RON)
            .unwrap()
            .at(ExchangeRate::parse("0.2").unwrap()),
        day(),
    )
    .build()
    .unwrap();
    let previous = group.replace_transfer(repayment, cash).unwrap();
    assert_eq!(previous.amount(), eur("10.00"));
    assert_eq!(group.transfer_amount(repayment), Some(eur("10.00")));
    assert_eq!(
        group.summary().member(carol).balance(),
        crate::ledger::Balance::Settled
    );
    assert_eq!(
        group.summary().member(bob).balance(),
        crate::ledger::Balance::Owes(eur("10.00"))
    );
    assert_eq!(group.ledger, rebuilt(&group));

    // Unknown members, unknown and duplicate transfers are refused.
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.add_transfer(transfer(ana, stranger, "1.00"))
    });
    assert_unchanged(
        &mut group,
        &GroupError::DuplicateTransfer(repayment),
        |group| group.insert_transfer(repayment, transfer(ana, bob, "1.00")),
    );
    group.remove_transfer(repayment).unwrap();
    assert_unchanged(
        &mut group,
        &GroupError::UnknownTransfer(repayment),
        |group| group.remove_transfer(repayment),
    );
    assert_unchanged(
        &mut group,
        &GroupError::UnknownTransfer(repayment),
        |group| group.replace_transfer(repayment, transfer(ana, bob, "1.00")),
    );
    assert_eq!(group.transfers().len(), 0);
    assert_eq!(group.transfer_amount(repayment), None);
}

/// A cached settlement is reused until a balance changes, and changes that
/// do not touch balances keep it.
#[test]
fn settlements_are_cached_until_a_balance_changes() {
    let (mut group, ana, bob, carol) = trip();
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
    group.add_member(person("Dan"));
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

/// A currency change re-converts every entry and rewrites the rates, and a
/// missing or repeated rate changes nothing.
#[test]
fn a_currency_change_reconverts_every_entry() {
    let (mut group, ana, bob, _) = trip();
    let local = group
        .add_expense(shared("10.00", ana, &[ana, bob]))
        .unwrap();
    let foreign = Expense::builder(
        "Taxi",
        Money::parse("50.00", Currency::RON)
            .unwrap()
            .at(ExchangeRate::parse("0.2").unwrap()),
        day(),
    )
    .paid_by(bob)
    .split(Split::equal([ana, bob]))
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
                &[(Currency::EUR, eur_to_ron), (Currency::EUR, eur_to_ron)],
            )
        },
    );

    group
        .change_currency(Currency::RON, &[(Currency::EUR, eur_to_ron)])
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

/// Groups compare by their details and entries, never by their caches.
#[test]
fn equality_ignores_the_caches() {
    let (mut group, ana, bob, _) = trip();
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

/// A change whose totals would overflow is refused and leaves the group
/// unchanged.
#[test]
fn an_overflowing_expense_is_refused() {
    let (mut group, ana, bob, _) = trip();
    let huge = Money::from_minor_units(i64::MAX, Currency::EUR);
    let first = Expense::builder("Huge", huge, day())
        .paid_by(ana)
        .split(Split::equal([bob]))
        .build()
        .unwrap();
    group.add_expense(first.clone()).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.add_expense(first)
    });
}

/// A generated identifier that is already taken is drawn again instead of
/// overwriting the entry that holds it.
#[test]
fn a_taken_identifier_is_drawn_again() {
    let taken = BTreeMap::from([(1, "first"), (2, "second")]);
    let mut draws = [1, 2, 1, 3, 4].into_iter();
    assert_eq!(unused_id(&taken, || draws.next().unwrap()), 3);
    // It stops at the first free one.
    assert_eq!(draws.next(), Some(4));
}

/// Archiving or unarchiving someone who is not a member is an error, not a
/// silent no-op that would hide a stale identifier.
#[test]
fn only_members_are_archived_or_unarchived() {
    let (mut group, ..) = trip();
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
    Expense::builder("Large", amount, day())
        .paid_by(payer)
        .split(Split::equal([participant]))
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
    group.change_currency(Currency::RON, &[(Currency::EUR, two())])
}

/// A currency change needs a rate for the currency of every transfer as
/// well as every expense, or that transfer could not be converted.
#[test]
fn a_currency_change_needs_a_rate_for_every_transfer_too() {
    let (mut group, ana, bob, _) = trip();
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
                &[(Currency::EUR, ExchangeRate::parse("5").unwrap())],
            )
        },
    );
}

/// A currency change that would convert an expense out of range changes
/// nothing.
#[test]
fn a_currency_change_converting_an_expense_out_of_range_changes_nothing() {
    let (mut group, ana, bob, _) = trip();
    group.add_expense(paid_for(cents(HUGE), ana, bob)).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A currency change that would convert a transfer out of range changes
/// nothing.
#[test]
fn a_currency_change_converting_a_transfer_out_of_range_changes_nothing() {
    let (mut group, ana, bob, _) = trip();
    group.add_transfer(sent(ana, bob, cents(HUGE))).unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A currency change whose entries convert in range but whose totals do
/// not changes nothing.
#[test]
fn a_currency_change_with_totals_out_of_range_changes_nothing() {
    // Each doubles to 8e18; together they are past i64::MAX.
    let (mut group, ana, bob, _) = trip();
    for _ in 0..2 {
        group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    }
    assert_unchanged(&mut group, &GroupError::Overflow, doubled);
}

/// A replacement that would push a total out of range, or whose conversion
/// leaves the range, is refused and leaves the original in place.
#[test]
fn a_replacement_out_of_range_is_refused() {
    let (mut group, ana, bob, _) = trip();
    group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    let second = group.add_expense(paid_for(cents(LARGE), ana, bob)).unwrap();
    // LARGE and HUGE together are past i64::MAX.
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_expense(second, paid_for(cents(HUGE), ana, bob))
    });
    let foreign = Expense::builder("Foreign", huge_in_lei().at(two()), day())
        .paid_by(ana)
        .split(Split::equal([bob]))
        .build()
        .unwrap();
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_expense(second, foreign)
    });
}

/// Removing an expense can overflow too, when it is what keeps two large
/// balances in range; the removal is then refused.
#[test]
fn removing_an_expense_out_of_range_is_refused() {
    // Ana and Bob pay LARGE for each other and Bob sends Ana HUGE: without
    // Ana's payment for Bob, Ana would owe 1.3e19.
    let (mut group, ana, bob, _) = trip();
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
    let (mut group, ana, bob, carol) = trip();
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
    let (mut group, ana, bob, _) = trip();
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
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_transfer(repayment, sent(ana, bob, cents(EXTRA)))
    });
    assert_unchanged(&mut group, &GroupError::Overflow, |group| {
        group.replace_transfer(repayment, foreign)
    });
    let stranger = MemberId::from_bytes([200; 16]);
    assert_unchanged(&mut group, &GroupError::UnknownMember(stranger), |group| {
        group.replace_transfer(repayment, sent(stranger, ana, cents(EXTRA)))
    });
}

/// Draws one change to apply to a five-member group.
#[derive(Debug, Clone)]
enum Operation {
    AddExpense {
        amount: i64,
        participants: Vec<(usize, u32)>,
        payer: usize,
    },
    ReplaceExpense {
        amount: i64,
        payer: usize,
        position: usize,
    },
    RemoveExpense {
        position: usize,
    },
    AddTransfer {
        amount: i64,
        from: usize,
        to: usize,
    },
    RemoveTransfer {
        position: usize,
    },
    Archive {
        member: usize,
    },
    Unarchive {
        member: usize,
    },
}

fn operation() -> impl Strategy<Value = Operation> {
    prop_oneof![
        4 => (1_i64..100_000, 0_usize..5, proptest::collection::vec((0_usize..5, 1_u32..400), 1..5))
            .prop_map(|(amount, payer, participants)| Operation::AddExpense { amount, payer, participants }),
        2 => (0_usize..8, 1_i64..100_000, 0_usize..5)
            .prop_map(|(position, amount, payer)| Operation::ReplaceExpense { position, amount, payer }),
        1 => (0_usize..8).prop_map(|position| Operation::RemoveExpense { position }),
        2 => (0_usize..5, 0_usize..5, 1_i64..50_000).prop_map(|(from, to, amount)| Operation::AddTransfer { from, to, amount }),
        1 => (0_usize..8).prop_map(|position| Operation::RemoveTransfer { position }),
        1 => (0_usize..5).prop_map(|member| Operation::Archive { member }),
        1 => (0_usize..5).prop_map(|member| Operation::Unarchive { member }),
    ]
}

/// Applies `operation`, returning whether the group accepted it.
fn apply(group: &mut Group, members: &[MemberId], operation: &Operation) -> bool {
    let money = |amount| Money::from_minor_units(amount, Currency::EUR);
    let expense_at =
        |current: &Group, position: usize| current.expenses().map(|(id, _)| id).nth(position);
    let transfer_at =
        |current: &Group, position: usize| current.transfers().map(|(id, _)| id).nth(position);
    match operation.clone() {
        Operation::AddExpense {
            amount,
            participants,
            payer,
        } => {
            let mut chosen: Vec<(MemberId, Parts)> = Vec::new();
            for (index, hundredths) in participants {
                if chosen.iter().all(|&(member, _)| member != members[index]) {
                    chosen.push((members[index], Parts::from_hundredths(hundredths).unwrap()));
                }
            }
            let expense = Expense::builder("Random", money(amount), day())
                .paid_by(members[payer])
                .split(Split::parts(chosen))
                .build()
                .unwrap();
            group.add_expense(expense).is_ok()
        }
        Operation::ReplaceExpense {
            amount,
            payer,
            position,
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
            group.replace_expense(id, expense).is_ok()
        }
        Operation::RemoveExpense { position } => {
            expense_at(group, position).is_some_and(|id| group.remove_expense(id).is_ok())
        }
        Operation::AddTransfer { amount, from, to } => {
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
        Operation::Archive { member } => group.archive_member(members[member]).is_ok(),
        Operation::Unarchive { member } => group.unarchive_member(members[member]).is_ok(),
    }
}

proptest! {
    /// Whatever sequence of changes the apps make, the incremental totals
    /// equal a full rebuild, balances sum to zero, a refused change leaves
    /// the group untouched, and every settlement clears every balance.
    #[test]
    fn random_changes_keep_the_group_consistent(operations in proptest::collection::vec(operation(), 1..40)) {
        let mut group = Group::builder("Random", Currency::EUR).build().unwrap();
        let members: Vec<MemberId> = ["Ana", "Bob", "Carol", "Dan", "Eve"].iter().map(|name| group.add_member(person(name))).collect();
        for operation in &operations {
            let before = group.clone();
            if !apply(&mut group, &members, operation) {
                prop_assert_eq!(&group, &before);
                prop_assert_eq!(&group.ledger, &before.ledger);
            }
            prop_assert_eq!(&group.ledger, &rebuilt(&group));
            let nets: i64 = group.summary().members().map(|(_, summary)| summary.net().minor_units()).sum();
            prop_assert_eq!(nets, 0);
        }
        for strategy in SettlementStrategy::all() {
            let mut settled = group.clone();
            let payments = settled.settlement(strategy).unwrap().payments().to_vec();
            for payment in payments {
                settled.add_transfer(payment.to_transfer(day())).unwrap();
            }
            prop_assert!(settled.summary().members().all(|(_, summary)| summary.net().is_zero()));
        }
    }
}
