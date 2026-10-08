//! Transfers cross the bindings as records. A record converted back and
//! forth would change who paid whom or when, a field the library refuses
//! would reach it, and a transfer to add would lose the identifier its app
//! gives it.

use eranos::{CurrencyCode, MemberId};

use super::*;
use crate::money::{CurrencyDetails, Money};
use crate::{CalendarFailure, IdFailure, MoneyFailure, TransferFailure, identifier_generate};

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

/// The details of a currency the library does not list.
fn unlisted() -> CurrencyDetails {
    CurrencyDetails {
        code: "XYZ".to_owned(),
        name: "Unlisted".to_owned(),
        numeric_code: 999,
        decimal_places: 2,
    }
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
        id: identifier_generate(),
        sender: bob(),
        receiver: ana(),
        cost: Cost::Converted {
            amount: Money {
                minor_units: 10_000,
                currency: CurrencyDetails::from_library(CurrencyCode::RON),
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
        id: identifier_generate(),
        sender: ana(),
        receiver: bob(),
        cost: Cost::Local {
            amount: Money {
                minor_units: 1_500,
                currency: CurrencyDetails::from_library(CurrencyCode::EUR),
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
                        currency: unlisted(),
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
                sender: " nobody".to_owned(),
                ..payment()
            },
            is_expected: surrounding_whitespace,
        },
        Refusal {
            record: Transfer {
                receiver: " nobody".to_owned(),
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
                receiver: bob(),
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
        assert_eq!(Transfer::from_library(&record.to_new().unwrap()), record);
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

/// A replacement would lose the stored identifier, or keep a field that
/// the record changes or clears.
#[test]
fn a_replacement_keeps_the_identifier_and_takes_every_field() {
    let stored = repayment().to_new().unwrap();
    let id = stored.id().to_string();
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
}

/// A transfer to add or a replacement whose note is absent or blank would
/// read back with an empty note instead of none, a replacement would keep
/// the stored one, or a note would keep stray whitespace.
#[test]
fn an_absent_or_blank_note_reads_back_absent() {
    let stored = repayment().to_new().unwrap();
    for note in [None, Some(" ".to_owned())] {
        let record = Transfer {
            note,
            ..repayment()
        };
        let added = Transfer::from_library(&record.to_new().unwrap());
        assert_eq!(added.note, None);
        let replaced = Transfer::from_library(&record.replacing(&stored).unwrap());
        assert_eq!(replaced.note, None);
    }
    let padded = Transfer {
        note: Some(" Cash\n".to_owned()),
        ..payment()
    };
    let added = Transfer::from_library(&padded.to_new().unwrap());
    assert_eq!(added.note.as_deref(), Some("Cash"));
}

/// A transfer to add would lose the identifier its app gives it, its own or
/// a generated one, or take one the rules refuse.
#[test]
fn a_transfer_to_add_keeps_the_identifier_it_carries() {
    let named = Transfer {
        id: "zz".to_owned(),
        ..payment()
    };
    let transfer = named.to_new().unwrap();
    assert_eq!(transfer.id().as_str(), "zz");
    assert_eq!(Transfer::from_library(&transfer), named);
    let generated = payment();
    assert_eq!(generated.to_new().unwrap().id().as_str(), generated.id);
    let padded = Transfer {
        id: " x".to_owned(),
        ..payment()
    };
    assert!(matches!(
        padded.to_new(),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));
}
