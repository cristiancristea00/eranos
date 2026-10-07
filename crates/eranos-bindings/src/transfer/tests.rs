//! Transfers cross the bindings as records. A record converted back and
//! forth would change who paid whom or when, a field the library refuses
//! would reach it, and a transfer to add would take its identifier from
//! outside the group.

use eranos::MemberId;

use super::*;
use crate::money::Money;
use crate::{CalendarFailure, IdFailure, MoneyFailure, TransferFailure};

/// A record the library refuses, beside a check of how it refuses it.
struct Refusal {
    record: Transfer,
    is_expected: fn(&EranosError) -> bool,
}

fn ana() -> String {
    MemberId::new("member-1").unwrap().to_string()
}

fn bob() -> String {
    MemberId::new("member-2").unwrap().to_string()
}

fn date() -> CalendarDate {
    CalendarDate {
        year: 2026,
        month: 9,
        day: 25,
    }
}

/// Bob paying Ana back in lei, with a note and a time.
fn repayment() -> Transfer {
    Transfer {
        id: None,
        from: bob(),
        to: ana(),
        cost: Cost::Converted {
            amount: Money {
                minor_units: 10_000,
                currency: "RON".to_owned(),
            },
            rate: "0.2".to_owned(),
        },
        date: date(),
        time: Some(TimeOfDay { hour: 9, minute: 5 }),
        note: Some("Cash".to_owned()),
    }
}

/// Ana paying Bob in euros, with no optional field.
fn payment() -> Transfer {
    Transfer {
        id: None,
        from: ana(),
        to: bob(),
        cost: Cost::Local {
            amount: Money {
                minor_units: 1_500,
                currency: "EUR".to_owned(),
            },
        },
        date: date(),
        time: None,
        note: None,
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

/// Transfers with one field the library refuses.
fn refused_transfers() -> [Refusal; 6] {
    [
        Refusal {
            record: Transfer {
                cost: Cost::Local {
                    amount: Money {
                        minor_units: 1,
                        currency: "XYZ".to_owned(),
                    },
                },
                ..payment()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Money {
                        failure: MoneyFailure::UnknownCurrency,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Transfer {
                date: CalendarDate { day: 0, ..date() },
                ..payment()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Calendar {
                        failure: CalendarFailure::InvalidDate,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Transfer {
                from: " nobody".to_owned(),
                ..payment()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Transfer {
                to: " nobody".to_owned(),
                ..payment()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Transfer {
                time: Some(TimeOfDay {
                    hour: 9,
                    minute: 60,
                }),
                ..payment()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Calendar {
                        failure: CalendarFailure::InvalidTime,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Transfer {
                to: bob(),
                ..repayment()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Transfer {
                        failure: TransferFailure::SameMember,
                        ..
                    }
                )
            },
        },
    ]
}

/// A transfer added through the bindings would reach the library with a
/// field changed or dropped, or come back without its identifier.
#[test]
fn a_transfer_to_add_converts_both_ways() {
    for record in [repayment(), payment()] {
        let transfer = record.to_new().unwrap();
        assert_eq!(
            Transfer::from_library(&transfer),
            Transfer {
                id: Some(transfer.id().to_string()),
                ..record
            }
        );
    }
}

/// A transfer with a field the library refuses would reach the group, as an
/// entry to add or as a replacement.
#[test]
fn a_transfer_refuses_every_field_that_does_not_convert() {
    let stored = repayment().to_new().unwrap();
    for refusal in refused_transfers() {
        let errors = [
            refusal.record.to_new().unwrap_err(),
            refusal.record.replacing(&stored).unwrap_err(),
        ];
        for error in errors {
            assert!((refusal.is_expected)(&error), "{error:?}");
        }
    }
}

/// An entry to add that carried an identifier would take it from outside
/// the group, and a replacement would lose the stored identifier, keep a
/// field that the record changes or clears, or name no transfer.
#[test]
fn a_replacement_keeps_the_identifier_and_takes_every_field() {
    let stored = repayment().to_new().unwrap();
    let id = Some(stored.id().to_string());
    for record in [payment(), repayment()] {
        let replacement = record.replacing(&stored).unwrap();
        assert_eq!(
            Transfer::from_library(&replacement),
            Transfer {
                id: id.clone(),
                ..record
            }
        );
    }
    let named = Transfer { id, ..payment() };
    assert_eq!(named.to_new().unwrap_err(), EranosError::identifier_given());
}
