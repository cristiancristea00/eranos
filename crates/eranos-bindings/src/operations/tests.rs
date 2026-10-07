//! Every group call over a group, once. A call that converted its answer or
//! its entry another way than the records do, took an identifier from
//! outside the group, or let a refusal of the library through as a success
//! would give every binding the same wrong answer.

use eranos::{CategoryId, CurrencyCode, ExpenseGroup, ExpenseId, MemberId, TransferId};

use crate::calendar::CalendarDate;
use crate::expense::{Expense, ExpenseSplit, Split};
use crate::ledger::{ExpenseShares, Summary};
use crate::member::{Category, Member};
use crate::money::{Cost, CurrencyRate, Money};
use crate::operations;
use crate::settlement::{SettlementPlan, SettlementStrategy};
use crate::transfer::Transfer;
use crate::{
    CategoryFailure, EranosError, ExpenseFailure, ExpenseGroupFailure, IdFailure, LanguageFailure,
    MemberFailure, MoneyFailure, TransferFailure,
};

/// A group of two with a category, an expense, and a transfer, and the
/// identifiers the group gave them.
struct Flat {
    ana: String,
    bob: String,
    dinner: String,
    food: String,
    group: ExpenseGroup,
    repayment: String,
}

fn euros(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "EUR".to_owned(),
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
    let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR)
        .build()
        .unwrap();
    let ana = operations::add_member(&mut group, &new_member("Ana")).unwrap();
    let bob = operations::add_member(&mut group, &new_member("Bob")).unwrap();
    let food = operations::add_category(&mut group, &new_category("Food")).unwrap();
    let dinner = Expense {
        category: Some(food.clone()),
        ..new_expense("Dinner", &ana, &[&ana, &bob])
    };
    let dinner = operations::add_expense(&mut group, &dinner).unwrap();
    let repayment = operations::add_transfer(&mut group, &new_transfer(&bob, &ana, 1_000)).unwrap();
    Flat {
        ana,
        bob,
        dinner,
        food,
        group,
        repayment,
    }
}

/// A read would answer another way than the records' conversions of what
/// the group holds, or find an entry for an identifier the group does not
/// hold.
#[test]
fn every_read_converts_what_the_group_holds() {
    let flat = flat();
    let group = &flat.group;
    assert_eq!(operations::id(group), group.id().to_string());
    let categories = operations::categories(group);
    assert_eq!(
        categories[1],
        Category {
            id: Some(flat.food.clone()),
            name: "Food".to_owned(),
        }
    );
    assert_eq!(
        operations::category(group, &flat.food),
        Ok(Some(categories[1].clone()))
    );
    assert_eq!(operations::currency(group), "EUR");
    let expenses = operations::expenses(group);
    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0].id.as_ref(), Some(&flat.dinner));
    assert_eq!(
        operations::expense(group, &flat.dinner),
        Ok(Some(expenses[0].clone()))
    );
    let dinner: ExpenseId = flat.dinner.parse().unwrap();
    assert_eq!(
        operations::expense_shares(group, &flat.dinner),
        Ok(group
            .expense_shares(dinner)
            .map(ExpenseShares::from_library))
    );
    assert_eq!(
        operations::summary(group),
        Summary::from_library(group.summary())
    );
    let members = operations::members(group);
    assert_eq!(
        members
            .iter()
            .map(|member| member.id.clone())
            .collect::<Vec<_>>(),
        [Some(flat.ana.clone()), Some(flat.bob.clone())]
    );
    assert_eq!(
        operations::member(group, &flat.ana),
        Ok(Some(members[0].clone()))
    );
    assert_eq!(operations::is_archived(group, &flat.ana), Ok(false));
    assert_eq!(operations::name(group), "Flat");
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::Direct,
        SettlementStrategy::Greedy,
        SettlementStrategy::MinimumTransfers,
    ] {
        let settled = group.settlement(strategy.to_library()).unwrap();
        assert_eq!(
            operations::settlement(group, strategy),
            Ok(SettlementPlan::from_library(settled))
        );
    }
    let transfers = operations::transfers(group);
    assert_eq!(transfers.len(), 1);
    assert_eq!(
        operations::transfer(group, &flat.repayment),
        Ok(Some(transfers[0].clone()))
    );
    assert_eq!(
        operations::transfer_amount(group, &flat.repayment),
        Ok(Some(euros(1_000)))
    );
    assert_eq!(operations::description(group), None);
    assert_eq!(operations::language(group), None);
    let saved: ExpenseGroup = serde_json::from_str(&operations::save(group)).unwrap();
    assert_eq!(&saved, group);

    let (category, expense) = (
        CategoryId::from_bytes([9; 16]),
        ExpenseId::from_bytes([9; 16]),
    );
    let (member, transfer) = (
        MemberId::from_bytes([9; 16]),
        TransferId::from_bytes([9; 16]),
    );
    assert_eq!(operations::category(group, &category.to_string()), Ok(None));
    assert_eq!(operations::expense(group, &expense.to_string()), Ok(None));
    assert_eq!(
        operations::expense_shares(group, &expense.to_string()),
        Ok(None)
    );
    assert_eq!(operations::member(group, &member.to_string()), Ok(None));
    assert_eq!(
        operations::is_archived(group, &member.to_string()),
        Ok(false)
    );
    assert_eq!(operations::transfer(group, &transfer.to_string()), Ok(None));
    assert_eq!(
        operations::transfer_amount(group, &transfer.to_string()),
        Ok(None)
    );
}

/// A call given text that is not an identifier would read or change another
/// entry, or answer as if no entry had it.
#[test]
fn every_call_by_identifier_refuses_text_that_is_not_one() {
    let calls: [fn(&mut ExpenseGroup, &str) -> Option<EranosError>; 13] = [
        |group, id| operations::category(group, id).err(),
        |group, id| operations::remove_category(group, id).err(),
        |group, id| operations::expense(group, id).err(),
        |group, id| operations::expense_shares(group, id).err(),
        |group, id| operations::remove_expense(group, id).err(),
        |group, id| operations::member(group, id).err(),
        |group, id| operations::is_archived(group, id).err(),
        |group, id| operations::archive_member(group, id).err(),
        |group, id| operations::unarchive_member(group, id).err(),
        |group, id| operations::remove_member(group, id).err(),
        |group, id| operations::transfer(group, id).err(),
        |group, id| operations::transfer_amount(group, id).err(),
        |group, id| operations::remove_transfer(group, id).err(),
    ];
    let mut flat = flat();
    let before = flat.group.clone();
    for call in calls {
        let refusal = call(&mut flat.group, "nobody");
        assert!(
            matches!(
                refusal,
                Some(EranosError::Id {
                    failure: IdFailure::InvalidId,
                    ..
                })
            ),
            "{refusal:?}"
        );
    }
    assert_eq!(flat.group, before);
}

/// A category change would take an identifier from outside the group,
/// change another category, or let a refusal through.
#[test]
fn categories_change_through_the_records() {
    let mut flat = flat();
    let group = &mut flat.group;
    let travel = operations::add_category(group, &new_category("Travel")).unwrap();
    let renamed = Category {
        id: Some(travel.clone()),
        name: "Trips".to_owned(),
    };
    assert_eq!(
        operations::replace_category(group, &renamed),
        Ok(Category {
            id: Some(travel.clone()),
            name: "Travel".to_owned(),
        })
    );
    assert_eq!(
        operations::category(group, &travel),
        Ok(Some(renamed.clone()))
    );

    let refusals = [
        operations::add_category(group, &renamed).unwrap_err(),
        operations::add_category(group, &new_category("FOOD")).unwrap_err(),
        operations::replace_category(group, &new_category("Trips")).unwrap_err(),
        operations::replace_category(
            group,
            &Category {
                id: Some(CategoryId::from_bytes([9; 16]).to_string()),
                name: "Trips".to_owned(),
            },
        )
        .unwrap_err(),
        operations::replace_category(
            group,
            &Category {
                id: Some(travel.clone()),
                name: " ".to_owned(),
            },
        )
        .unwrap_err(),
        operations::replace_category(
            group,
            &Category {
                id: Some(travel.clone()),
                name: "food".to_owned(),
            },
        )
        .unwrap_err(),
        operations::replace_category(
            group,
            &Category {
                id: Some(CategoryId::UNCATEGORISED.to_string()),
                name: "Other".to_owned(),
            },
        )
        .unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::IdentifierGiven { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateCategoryName { .. },
                    ..
                },
                EranosError::IdentifierMissing { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownCategory { .. },
                    ..
                },
                EranosError::Category {
                    failure: CategoryFailure::EmptyName,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::DuplicateCategoryName { .. },
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::BuiltInCategory,
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );

    assert_eq!(operations::remove_category(group, &travel), Ok(renamed));
    assert!(matches!(
        operations::remove_category(group, &travel),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownCategory { .. },
            ..
        })
    ));
}

/// An expense change would take an identifier from outside the group,
/// replace another expense, or let a refusal through.
#[test]
fn expenses_change_through_the_records() {
    let mut flat = flat();
    let (ana, bob) = (flat.ana.as_str(), flat.bob.as_str());
    let group = &mut flat.group;
    let lunch = operations::add_expense(group, &new_expense("Lunch", bob, &[ana, bob])).unwrap();
    let replacement = Expense {
        id: Some(lunch.clone()),
        ..new_expense("Late lunch", ana, &[ana, bob])
    };
    let replaced = operations::replace_expense(group, &replacement).unwrap();
    assert_eq!(
        replaced,
        Expense {
            id: Some(lunch.clone()),
            ..new_expense("Lunch", bob, &[ana, bob])
        }
    );
    assert_eq!(
        operations::expense(group, &lunch),
        Ok(Some(replacement.clone()))
    );

    let nobody = MemberId::from_bytes([9; 16]).to_string();
    let refusals = [
        operations::add_expense(group, &replacement).unwrap_err(),
        operations::add_expense(group, &new_expense("Lunch", &nobody, &[ana])).unwrap_err(),
        operations::replace_expense(group, &new_expense("Lunch", ana, &[ana])).unwrap_err(),
        operations::replace_expense(
            group,
            &Expense {
                id: Some(ExpenseId::from_bytes([9; 16]).to_string()),
                ..replacement.clone()
            },
        )
        .unwrap_err(),
        operations::replace_expense(
            group,
            &Expense {
                name: " ".to_owned(),
                ..replacement.clone()
            },
        )
        .unwrap_err(),
        operations::replace_expense(
            group,
            &Expense {
                id: Some(lunch.clone()),
                ..new_expense("Lunch", &nobody, &[ana])
            },
        )
        .unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::IdentifierGiven { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::IdentifierMissing { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownExpense { .. },
                    ..
                },
                EranosError::Expense {
                    failure: ExpenseFailure::EmptyName,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );

    assert_eq!(operations::remove_expense(group, &lunch), Ok(replacement));
    assert!(matches!(
        operations::remove_expense(group, &lunch),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownExpense { .. },
            ..
        })
    ));
}

/// A member change would take an identifier from outside the group, change
/// another member, or let a refusal through.
#[test]
fn members_change_through_the_records() {
    let mut flat = flat();
    let group = &mut flat.group;
    let carol = operations::add_member(group, &new_member("Carol")).unwrap();
    let renamed = Member {
        id: Some(carol.clone()),
        last_name: Some("Ionescu".to_owned()),
        ..new_member("Carol")
    };
    assert_eq!(
        operations::replace_member(group, &renamed),
        Ok(Member {
            id: Some(carol.clone()),
            ..new_member("Carol")
        })
    );
    assert_eq!(operations::member(group, &carol), Ok(Some(renamed.clone())));
    operations::archive_member(group, &carol).unwrap();
    assert_eq!(operations::is_archived(group, &carol), Ok(true));
    operations::unarchive_member(group, &carol).unwrap();
    assert_eq!(operations::is_archived(group, &carol), Ok(false));

    let nobody = MemberId::from_bytes([9; 16]).to_string();
    let unknown = Member {
        id: Some(nobody.clone()),
        ..new_member("Dan")
    };
    let refusals = [
        operations::add_member(group, &renamed).unwrap_err(),
        operations::add_member(group, &new_member(" ")).unwrap_err(),
        operations::replace_member(group, &new_member("Dan")).unwrap_err(),
        operations::replace_member(group, &unknown).unwrap_err(),
        operations::replace_member(
            group,
            &Member {
                first_name: " ".to_owned(),
                ..renamed.clone()
            },
        )
        .unwrap_err(),
        operations::archive_member(group, &nobody).unwrap_err(),
        operations::unarchive_member(group, &nobody).unwrap_err(),
        operations::remove_member(group, &nobody).unwrap_err(),
        operations::remove_member(group, &flat.ana).unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::IdentifierGiven { .. },
                EranosError::Member {
                    failure: MemberFailure::EmptyFirstName,
                    ..
                },
                EranosError::IdentifierMissing { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::Member {
                    failure: MemberFailure::EmptyFirstName,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::MemberInUse { .. },
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );

    assert_eq!(operations::remove_member(group, &carol), Ok(renamed));
}

/// A transfer change would take an identifier from outside the group,
/// replace another transfer, or let a refusal through.
#[test]
fn transfers_change_through_the_records() {
    let mut flat = flat();
    let (ana, bob) = (flat.ana.as_str(), flat.bob.as_str());
    let group = &mut flat.group;
    let payment = operations::add_transfer(group, &new_transfer(ana, bob, 500)).unwrap();
    let replacement = Transfer {
        id: Some(payment.clone()),
        ..new_transfer(ana, bob, 700)
    };
    assert_eq!(
        operations::replace_transfer(group, &replacement),
        Ok(Transfer {
            id: Some(payment.clone()),
            ..new_transfer(ana, bob, 500)
        })
    );
    assert_eq!(
        operations::transfer(group, &payment),
        Ok(Some(replacement.clone()))
    );

    let nobody = MemberId::from_bytes([9; 16]).to_string();
    let refusals = [
        operations::add_transfer(group, &replacement).unwrap_err(),
        operations::add_transfer(group, &new_transfer(ana, &nobody, 500)).unwrap_err(),
        operations::replace_transfer(group, &new_transfer(ana, bob, 500)).unwrap_err(),
        operations::replace_transfer(
            group,
            &Transfer {
                id: Some(TransferId::from_bytes([9; 16]).to_string()),
                ..replacement.clone()
            },
        )
        .unwrap_err(),
        operations::replace_transfer(
            group,
            &Transfer {
                to: ana.to_owned(),
                ..replacement.clone()
            },
        )
        .unwrap_err(),
        operations::replace_transfer(
            group,
            &Transfer {
                to: nobody.clone(),
                ..replacement.clone()
            },
        )
        .unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::IdentifierGiven { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
                EranosError::IdentifierMissing { .. },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownTransfer { .. },
                    ..
                },
                EranosError::Transfer {
                    failure: TransferFailure::SameMember,
                    ..
                },
                EranosError::ExpenseGroup {
                    failure: ExpenseGroupFailure::UnknownMember { .. },
                    ..
                },
            ]
        ),
        "{refusals:?}"
    );

    assert_eq!(
        operations::remove_transfer(group, &payment),
        Ok(replacement)
    );
    assert!(matches!(
        operations::remove_transfer(group, &payment),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::UnknownTransfer { .. },
            ..
        })
    ));
}

/// The name, description, or language would change to text the library
/// refuses, or a blank description or an absent language would stay set.
#[test]
fn the_name_description_and_language_change_through_text() {
    let mut flat = flat();
    let group = &mut flat.group;
    operations::set_name(group, "Home").unwrap();
    assert_eq!(operations::name(group), "Home");
    assert!(matches!(
        operations::set_name(group, " "),
        Err(EranosError::ExpenseGroup {
            failure: ExpenseGroupFailure::EmptyName,
            ..
        })
    ));
    assert_eq!(operations::name(group), "Home");

    operations::set_description(group, "Rent and food");
    assert_eq!(
        operations::description(group).as_deref(),
        Some("Rent and food")
    );
    operations::set_description(group, " ");
    assert_eq!(operations::description(group), None);

    operations::set_language(group, Some("pt-pt")).unwrap();
    assert_eq!(operations::language(group).as_deref(), Some("pt-PT"));
    assert!(matches!(
        operations::set_language(group, Some("not a tag")),
        Err(EranosError::Language {
            failure: LanguageFailure::InvalidLanguage,
            ..
        })
    ));
    assert_eq!(operations::language(group).as_deref(), Some("pt-PT"));
    operations::set_language(group, None).unwrap();
    assert_eq!(operations::language(group), None);
}

/// A currency change would reach the library with a currency or a rate it
/// refuses, or with a rate missing, and change the group anyway.
#[test]
fn the_currency_changes_at_the_given_rates() {
    let mut flat = flat();
    let (ana, bob) = (flat.ana.as_str(), flat.bob.as_str());
    let group = &mut flat.group;
    let taxi = Expense {
        cost: Cost::Converted {
            amount: Money {
                minor_units: 5_000,
                currency: "RON".to_owned(),
            },
            rate: "0.2".to_owned(),
        },
        payers: Split::Equal {
            members: vec![ana.to_owned()],
        },
        ..new_expense("Taxi", ana, &[ana, bob])
    };
    operations::add_expense(group, &taxi).unwrap();
    let rate = |currency: &str, rate: &str| CurrencyRate {
        currency: currency.to_owned(),
        rate: rate.to_owned(),
    };

    let before = group.clone();
    let refusals = [
        operations::change_currency(group, "XYZ", &[rate("EUR", "5")]).unwrap_err(),
        operations::change_currency(group, "RON", &[rate("EUR", "zero")]).unwrap_err(),
        operations::change_currency(group, "RON", &[rate("XYZ", "5")]).unwrap_err(),
        operations::change_currency(group, "USD", &[rate("EUR", "1.1")]).unwrap_err(),
    ];
    assert!(
        matches!(
            refusals,
            [
                EranosError::Money {
                    failure: MoneyFailure::UnknownCurrency,
                    ..
                },
                EranosError::Money {
                    failure: MoneyFailure::InvalidExchangeRate,
                    ..
                },
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
    assert_eq!(*group, before);

    operations::change_currency(group, "RON", &[rate("EUR", "5")]).unwrap();
    assert_eq!(operations::currency(group), "RON");
    assert_eq!(
        operations::summary(group).total_spent,
        Money {
            minor_units: 20_000,
            currency: "RON".to_owned(),
        }
    );
}
