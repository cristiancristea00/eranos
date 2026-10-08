//! Expenses cross the bindings as records. A record converted back and forth
//! would change how its cost divides; a split, an item, a tip, or an expense
//! with a value the library refuses, or a number too large for its field,
//! would reach the library; and an entry to add would lose the identifier
//! its app gives it.

use eranos::{CategoryId, CurrencyCode, ExpenseId};

use super::*;
use crate::{
    CalendarFailure, CurrencyDetails, ExpenseFailure, IdFailure, MoneyFailure,
    category_id_uncategorised, identifier_generate, tip_rule_none,
};

/// A record the library refuses, beside a check of how it refuses it.
struct Refusal<Record> {
    record: Record,
    is_expected: fn(&EranosError) -> bool,
}

fn ana() -> MemberId {
    MemberId::new("member-1").unwrap()
}

fn bob() -> MemberId {
    MemberId::new("member-2").unwrap()
}

fn euros(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: CurrencyDetails::from_library(CurrencyCode::EUR),
    }
}

fn lei(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: CurrencyDetails::from_library(CurrencyCode::RON),
    }
}

/// The details of a currency the library does not list.
fn unlisted() -> CurrencyDetails {
    CurrencyDetails {
        code: "XYZ".to_owned(),
        name: "Unlisted".to_owned(),
        numeric_code: 999,
        decimal_places: 2,
    }
}

fn unknown_money() -> Money {
    Money {
        minor_units: 3_000,
        currency: unlisted(),
    }
}

fn equal(members: &[MemberId]) -> Split {
    Split::Equal {
        members: members.iter().map(ToString::to_string).collect(),
    }
}

/// A split naming a member by text that is not an identifier.
fn unreadable_split() -> Split {
    Split::Equal {
        members: vec![" nobody".to_owned()],
    }
}

fn surrounding_whitespace(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        }
    )
}

fn invalid_parts(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Money {
            failure: MoneyFailure::InvalidParts,
            ..
        }
    )
}

fn invalid_percentage(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Money {
            failure: MoneyFailure::InvalidPercentage,
            ..
        }
    )
}

fn unknown_currency(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Money {
            failure: MoneyFailure::UnknownCurrency,
            ..
        }
    )
}

fn empty_item_name(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Expense {
            failure: ExpenseFailure::EmptyItemName,
            ..
        }
    )
}

fn empty_name(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Expense {
            failure: ExpenseFailure::EmptyName,
            ..
        }
    )
}

fn invalid_date(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Calendar {
            failure: CalendarFailure::InvalidDate,
            ..
        }
    )
}

fn invalid_time(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Calendar {
            failure: CalendarFailure::InvalidTime,
            ..
        }
    )
}

fn zero_quantity(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Expense {
            failure: ExpenseFailure::ZeroQuantity,
            ..
        }
    )
}

/// One split of every kind, each between Ana and Bob.
fn splits() -> [Split; 4] {
    [
        Split::Amounts {
            shares: vec![
                MemberAmount {
                    member: ana().to_string(),
                    amount: euros(1_000),
                },
                MemberAmount {
                    member: bob().to_string(),
                    amount: euros(2_000),
                },
            ],
        },
        equal(&[ana(), bob()]),
        Split::Parts {
            shares: vec![
                MemberParts {
                    member: ana().to_string(),
                    hundredths: 100,
                },
                MemberParts {
                    member: bob().to_string(),
                    hundredths: 50,
                },
            ],
        },
        Split::Percentages {
            shares: vec![
                MemberPercentage {
                    member: ana().to_string(),
                    basis_points: 6_000,
                },
                MemberPercentage {
                    member: bob().to_string(),
                    basis_points: 4_000,
                },
            ],
        },
    ]
}

/// The one line of the taxi's bill.
fn fare() -> ExpenseItem {
    ExpenseItem {
        name: "Fare".to_owned(),
        unit_price: lei(2_500),
        split: equal(&[ana(), bob()]),
        quantity: 2,
    }
}

/// A dinner that Ana paid, tip included, which Ana and Bob share: every
/// field set, the tip and the category to other values than their defaults.
fn dinner() -> Expense {
    Expense {
        id: identifier_generate(),
        name: "Dinner".to_owned(),
        cost: Cost::Local {
            amount: euros(3_000),
        },
        payers: Split::Amounts {
            shares: vec![MemberAmount {
                member: ana().to_string(),
                amount: euros(3_300),
            }],
        },
        split: ExpenseSplit::Whole {
            split: equal(&[ana(), bob()]),
        },
        date: CalendarDate {
            year: 2026,
            month: 9,
            day: 23,
        },
        tip: TipRule {
            value: TipValue::Percentage {
                basis_points: 1_000,
            },
            distribution: TipDistribution::Proportional,
        },
        time: Some(TimeOfDay {
            hour: 20,
            minute: 15,
        }),
        category: CategoryId::new("category-3").unwrap().to_string(),
        description: Some("A birthday".to_owned()),
    }
}

/// A taxi fare in lei that Bob paid, itemised: no optional field set, no
/// tip, and the built-in category.
fn taxi() -> Expense {
    Expense {
        id: identifier_generate(),
        name: "Taxi".to_owned(),
        cost: Cost::Converted {
            amount: lei(5_000),
            rate: "0.2".to_owned(),
        },
        payers: equal(&[bob()]),
        split: ExpenseSplit::Itemised {
            items: vec![fare()],
        },
        date: CalendarDate {
            year: 2026,
            month: 9,
            day: 24,
        },
        tip: tip_rule_none(),
        time: None,
        category: category_id_uncategorised(),
        description: None,
    }
}

/// Expenses with one field the library refuses.
fn refused_expenses() -> Vec<Refusal<Expense>> {
    vec![
        Refusal {
            record: Expense {
                cost: Cost::Local {
                    amount: unknown_money(),
                },
                ..dinner()
            },
            is_expected: unknown_currency,
        },
        Refusal {
            record: Expense {
                date: CalendarDate {
                    year: 2026,
                    month: 9,
                    day: 31,
                },
                ..dinner()
            },
            is_expected: invalid_date,
        },
        Refusal {
            record: Expense {
                payers: unreadable_split(),
                ..dinner()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Expense {
                split: ExpenseSplit::Whole {
                    split: unreadable_split(),
                },
                ..dinner()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Expense {
                split: ExpenseSplit::Itemised {
                    items: vec![ExpenseItem {
                        quantity: -1,
                        ..fare()
                    }],
                },
                ..taxi()
            },
            is_expected: zero_quantity,
        },
        Refusal {
            record: Expense {
                category: " nobody".to_owned(),
                ..dinner()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Expense {
                time: Some(TimeOfDay {
                    hour: 24,
                    minute: 0,
                }),
                ..dinner()
            },
            is_expected: invalid_time,
        },
        Refusal {
            record: Expense {
                tip: TipRule {
                    value: TipValue::Percentage { basis_points: -1 },
                    distribution: TipDistribution::Equal,
                },
                ..dinner()
            },
            is_expected: invalid_percentage,
        },
        Refusal {
            record: Expense {
                name: " ".to_owned(),
                ..dinner()
            },
            is_expected: empty_name,
        },
    ]
}

/// A split converted back and forth would change who pays or how much, and
/// its members would come back in another order than it lists them.
#[test]
fn every_split_converts_both_ways() {
    for split in splits() {
        let converted = split.to_library().unwrap();
        assert_eq!(
            Split::members_of(&converted),
            [ana(), bob()].map(|member| member.to_string())
        );
        assert_eq!(Split::from_split(&converted), split);
    }
    let listed = equal(&[bob(), ana()]).to_library().unwrap();
    assert_eq!(
        Split::members_of(&listed),
        [bob(), ana()].map(|member| member.to_string())
    );
}

/// A split with a value the library refuses, or a number too large for its
/// field, would reach the library, or wrap into another split.
#[test]
fn a_split_refuses_what_does_not_convert() {
    let refusals = [
        Refusal {
            record: Split::Amounts {
                shares: vec![MemberAmount {
                    member: ana().to_string(),
                    amount: unknown_money(),
                }],
            },
            is_expected: unknown_currency,
        },
        Refusal {
            record: Split::Amounts {
                shares: vec![MemberAmount {
                    member: " nobody".to_owned(),
                    amount: euros(1),
                }],
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: unreadable_split(),
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Split::Parts {
                shares: vec![MemberParts {
                    member: ana().to_string(),
                    hundredths: -1,
                }],
            },
            is_expected: invalid_parts,
        },
        Refusal {
            record: Split::Parts {
                shares: vec![MemberParts {
                    member: ana().to_string(),
                    hundredths: 0,
                }],
            },
            is_expected: invalid_parts,
        },
        Refusal {
            record: Split::Parts {
                shares: vec![MemberParts {
                    member: " nobody".to_owned(),
                    hundredths: 100,
                }],
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Split::Percentages {
                shares: vec![MemberPercentage {
                    member: ana().to_string(),
                    basis_points: -1,
                }],
            },
            is_expected: invalid_percentage,
        },
        Refusal {
            record: Split::Percentages {
                shares: vec![MemberPercentage {
                    member: " nobody".to_owned(),
                    basis_points: 100,
                }],
            },
            is_expected: surrounding_whitespace,
        },
    ];
    for refusal in refusals {
        let error = refusal.record.to_library().unwrap_err();
        assert!((refusal.is_expected)(&error), "{error:?}");
    }
}

/// An item converted back and forth would change its line, and one the
/// library refuses would reach it.
#[test]
fn an_item_converts_both_ways_and_refuses_bad_lines() {
    let item = fare();
    assert_eq!(
        item.to_library()
            .map(|converted| ExpenseItem::from_library(&converted)),
        Ok(fare())
    );
    let refusals = [
        Refusal {
            record: ExpenseItem {
                quantity: -1,
                ..fare()
            },
            is_expected: zero_quantity,
        },
        Refusal {
            record: ExpenseItem {
                quantity: 0,
                ..fare()
            },
            is_expected: zero_quantity,
        },
        Refusal {
            record: ExpenseItem {
                name: " ".to_owned(),
                ..fare()
            },
            is_expected: empty_item_name,
        },
        Refusal {
            record: ExpenseItem {
                split: unreadable_split(),
                ..fare()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: ExpenseItem {
                unit_price: unknown_money(),
                ..fare()
            },
            is_expected: unknown_currency,
        },
    ];
    for refusal in refusals {
        let error = refusal.record.to_library().unwrap_err();
        assert!((refusal.is_expected)(&error), "{error:?}");
    }
}

/// A tip converted back and forth would change its value or how it is
/// shared, and a negative percentage would wrap into a large one.
#[test]
fn tips_convert_both_ways_and_refuse_what_does_not_convert() {
    let tips = [
        TipRule {
            value: TipValue::Fixed { amount: euros(500) },
            distribution: TipDistribution::Equal,
        },
        TipRule {
            value: TipValue::Percentage {
                basis_points: 1_250,
            },
            distribution: TipDistribution::Proportional,
        },
    ];
    for tip in tips {
        assert_eq!(tip.to_library().map(TipRule::from_library), Ok(tip));
    }
    let negative = TipRule {
        value: TipValue::Percentage { basis_points: -1 },
        distribution: TipDistribution::Equal,
    };
    assert!(invalid_percentage(&negative.to_library().unwrap_err()));
    let unknown = TipRule {
        value: TipValue::Fixed {
            amount: unknown_money(),
        },
        distribution: TipDistribution::Equal,
    };
    assert!(unknown_currency(&unknown.to_library().unwrap_err()));
}

/// An expense added through the bindings would reach the library with a
/// field changed or dropped, or come back without its identifier, its tip,
/// or its category.
#[test]
fn an_expense_to_add_converts_both_ways() {
    for record in [dinner(), taxi()] {
        assert_eq!(Expense::from_library(&record.to_new().unwrap()), record);
    }
}

/// An expense the library holds without a tip or a category would read
/// back with either left out, or with another value than the library's no
/// tip and built-in category, so the apps could not tell them from a value
/// they set.
#[test]
fn an_expense_reads_back_no_tip_and_the_built_in_category_by_name() {
    let expense = library::Expense::builder(
        "Taxi",
        money::Money::from_minor_units(5_000, CurrencyCode::EUR),
        ana(),
        library::Split::equal([ana(), bob()]),
        eranos::CalendarDate::new(2026, 9, 24).unwrap(),
    )
    .build()
    .unwrap();
    let record = Expense::from_library(&expense);
    assert_eq!(
        record.tip,
        TipRule {
            value: TipValue::Percentage { basis_points: 0 },
            distribution: TipDistribution::Proportional,
        }
    );
    assert_eq!(record.tip, tip_rule_none());
    assert_eq!(record.category, "00000000-0000-0000-0000-000000000000");
    assert_eq!(record.category, category_id_uncategorised());
}

/// A tip of 0 % set on purpose would read back as another rule than the
/// no-tip rule when it is shared proportionally, or be taken for no tip when
/// it is shared equally.
#[test]
fn a_zero_tip_reads_back_as_set() {
    let proportional = Expense {
        tip: TipRule {
            value: TipValue::Percentage { basis_points: 0 },
            distribution: TipDistribution::Proportional,
        },
        ..taxi()
    };
    let stored = proportional.to_new().unwrap();
    assert_eq!(stored.tip(), library::TipRule::NONE);
    assert_eq!(Expense::from_library(&stored).tip, tip_rule_none());
    let equal = Expense {
        tip: TipRule {
            value: TipValue::Percentage { basis_points: 0 },
            distribution: TipDistribution::Equal,
        },
        ..taxi()
    };
    let read = Expense::from_library(&equal.to_new().unwrap());
    assert_ne!(read.tip, tip_rule_none());
    assert_eq!(read, equal);
}

/// An expense to add or a replacement whose description is absent or blank
/// would read back with an empty description instead of none, a
/// replacement would keep the stored one, or a description would keep stray
/// whitespace.
#[test]
fn an_absent_or_blank_description_reads_back_absent() {
    let stored = dinner().to_new().unwrap();
    for description in [None, Some(" ".to_owned())] {
        let record = Expense {
            description,
            ..dinner()
        };
        let added = Expense::from_library(&record.to_new().unwrap());
        assert_eq!(added.description, None);
        let replaced = Expense::from_library(&record.replacing(&stored).unwrap());
        assert_eq!(replaced.description, None);
    }
    let padded = Expense {
        description: Some(" A birthday\n".to_owned()),
        ..taxi()
    };
    let added = Expense::from_library(&padded.to_new().unwrap());
    assert_eq!(added.description.as_deref(), Some("A birthday"));
}

/// An expense to add would lose the identifier its app gives it, its own or
/// a generated one, or take one the rules refuse.
#[test]
fn an_expense_to_add_keeps_the_identifier_it_carries() {
    let id = ExpenseId::new("Ωmega").unwrap();
    let named = Expense {
        id: id.to_string(),
        ..dinner()
    };
    let expense = named.to_new().unwrap();
    assert_eq!(expense.id(), id);
    assert_eq!(Expense::from_library(&expense), named);
    let generated = dinner();
    assert_eq!(generated.to_new().unwrap().id().as_str(), generated.id);
    let padded = Expense {
        id: " x".to_owned(),
        ..dinner()
    };
    assert!(matches!(
        padded.to_new(),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));
}

/// A replacement would lose the stored expense's identifier, or keep a
/// field of the stored expense that the record changes or clears.
#[test]
fn a_replacement_keeps_the_identifier_and_takes_every_field() {
    let stored = dinner().to_new().unwrap();
    for record in [taxi(), dinner()] {
        let replacement = record.replacing(&stored).unwrap();
        assert_eq!(
            Expense::from_library(&replacement),
            Expense {
                id: stored.id().to_string(),
                ..record
            }
        );
    }
}

/// An expense with a field the library refuses would reach the group, as an
/// entry to add or as a replacement.
#[test]
fn an_expense_refuses_every_field_that_does_not_convert() {
    let stored = dinner().to_new().unwrap();
    for refusal in refused_expenses() {
        let errors = [
            refusal.record.to_new().unwrap_err(),
            refusal.record.replacing(&stored).unwrap_err(),
        ];
        for error in errors {
            assert!((refusal.is_expected)(&error), "{error:?}");
        }
    }
}
