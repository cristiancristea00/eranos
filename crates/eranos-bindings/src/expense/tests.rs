//! Expenses cross the bindings as records. A record converted back and forth
//! would change how its cost divides; a split, an item, a tip, or an expense
//! with a value the library refuses, or a number too large for its field,
//! would reach the library; and an entry to add would take its identifier
//! from outside the group.

use eranos::ExpenseId;

use super::*;
use crate::{CalendarFailure, ExpenseFailure, IdFailure, MoneyFailure};

/// A record the library refuses, beside a check of how it refuses it.
struct Refusal<Record> {
    record: Record,
    is_expected: fn(&EranosError) -> bool,
}

fn ana() -> MemberId {
    MemberId::from_bytes([1; 16])
}

fn bob() -> MemberId {
    MemberId::from_bytes([2; 16])
}

fn euros(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "EUR".to_owned(),
    }
}

fn lei(minor_units: i64) -> Money {
    Money {
        minor_units,
        currency: "RON".to_owned(),
    }
}

fn unknown_money() -> Money {
    Money {
        minor_units: 3_000,
        currency: "XYZ".to_owned(),
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
        members: vec!["nobody".to_owned()],
    }
}

fn invalid_id(error: &EranosError) -> bool {
    matches!(
        error,
        EranosError::Id {
            failure: IdFailure::InvalidId,
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
fn fare() -> Item {
    Item {
        name: "Fare".to_owned(),
        unit_price: lei(2_500),
        split: equal(&[ana(), bob()]),
        quantity: 2,
    }
}

/// A dinner that Ana paid, tip included, which Ana and Bob share: every
/// optional field set.
fn dinner() -> Expense {
    Expense {
        id: None,
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
        tip: Some(Tip {
            value: TipValue::Percentage {
                basis_points: 1_000,
            },
            distribution: TipDistribution::Proportional,
        }),
        time: Some(TimeOfDay {
            hour: 20,
            minute: 15,
        }),
        category: Some(CategoryId::from_bytes([3; 16]).to_string()),
        description: Some("A birthday".to_owned()),
    }
}

/// A taxi fare in lei that Bob paid, itemised: no optional field set.
fn taxi() -> Expense {
    Expense {
        id: None,
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
        tip: None,
        time: None,
        category: None,
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
            is_expected: invalid_id,
        },
        Refusal {
            record: Expense {
                split: ExpenseSplit::Whole {
                    split: unreadable_split(),
                },
                ..dinner()
            },
            is_expected: invalid_id,
        },
        Refusal {
            record: Expense {
                split: ExpenseSplit::Itemised {
                    items: vec![Item {
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
                category: Some("nobody".to_owned()),
                ..dinner()
            },
            is_expected: invalid_id,
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
                tip: Some(Tip {
                    value: TipValue::Percentage { basis_points: -1 },
                    distribution: TipDistribution::Equal,
                }),
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
                    member: "nobody".to_owned(),
                    amount: euros(1),
                }],
            },
            is_expected: invalid_id,
        },
        Refusal {
            record: unreadable_split(),
            is_expected: invalid_id,
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
                    member: "nobody".to_owned(),
                    hundredths: 100,
                }],
            },
            is_expected: invalid_id,
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
                    member: "nobody".to_owned(),
                    basis_points: 100,
                }],
            },
            is_expected: invalid_id,
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
            .map(|converted| Item::from_library(&converted)),
        Ok(fare())
    );
    let refusals = [
        Refusal {
            record: Item {
                quantity: -1,
                ..fare()
            },
            is_expected: zero_quantity,
        },
        Refusal {
            record: Item {
                quantity: 0,
                ..fare()
            },
            is_expected: zero_quantity,
        },
        Refusal {
            record: Item {
                name: " ".to_owned(),
                ..fare()
            },
            is_expected: empty_item_name,
        },
        Refusal {
            record: Item {
                split: unreadable_split(),
                ..fare()
            },
            is_expected: invalid_id,
        },
        Refusal {
            record: Item {
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
        Tip {
            value: TipValue::Fixed { amount: euros(500) },
            distribution: TipDistribution::Equal,
        },
        Tip {
            value: TipValue::Percentage {
                basis_points: 1_250,
            },
            distribution: TipDistribution::Proportional,
        },
    ];
    for tip in tips {
        assert_eq!(tip.to_library().map(Tip::from_library), Ok(tip));
    }
    let negative = Tip {
        value: TipValue::Percentage { basis_points: -1 },
        distribution: TipDistribution::Equal,
    };
    assert!(invalid_percentage(&negative.to_library().unwrap_err()));
    let unknown = Tip {
        value: TipValue::Fixed {
            amount: unknown_money(),
        },
        distribution: TipDistribution::Equal,
    };
    assert!(unknown_currency(&unknown.to_library().unwrap_err()));
}

/// An expense added through the bindings would reach the library with a
/// field changed or dropped, come back without its identifier, or show the
/// built-in category and the absent tip as values.
#[test]
fn an_expense_to_add_converts_both_ways() {
    for record in [dinner(), taxi()] {
        let expense = record.to_new().unwrap();
        assert_eq!(
            Expense::from_library(&expense),
            Expense {
                id: Some(expense.id().to_string()),
                ..record
            }
        );
    }
}

/// An entry to add that carried an identifier would take it from outside
/// the group.
#[test]
fn an_expense_to_add_carries_no_identifier() {
    let named = Expense {
        id: Some(ExpenseId::from_bytes([4; 16]).to_string()),
        ..dinner()
    };
    assert_eq!(named.to_new().unwrap_err(), EranosError::identifier_given());
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
                id: Some(stored.id().to_string()),
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
