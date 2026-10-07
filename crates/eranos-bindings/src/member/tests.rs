//! Members, their accounts, and categories cross the bindings as records. A
//! record converted back and forth would change a detail, a detail the
//! library refuses would reach it, and an entry to add would lose the
//! identifier its app gives it.

use eranos::MemberId;

use super::*;
use crate::{CategoryFailure, ContactFailure, IdFailure, MemberFailure, PaymentAccountFailure};

/// A record the library refuses, beside a check of how it refuses it.
struct Refusal {
    record: Member,
    is_expected: fn(&EranosError) -> bool,
}

fn iban_account() -> PaymentAccount {
    PaymentAccount {
        name: "Main account".to_owned(),
        details: PaymentAccountDetails::Iban {
            iban: "RO49AAAA1B31007593840000".to_owned(),
        },
    }
}

/// Ana with every detail, as the group holds them.
fn ana() -> Member {
    Member {
        id: None,
        first_name: "Ana".to_owned(),
        last_name: Some("Popescu".to_owned()),
        email: Some("ana@example.com".to_owned()),
        phone: Some("+40712345678".to_owned()),
        accounts: vec![
            iban_account(),
            PaymentAccount {
                name: "Payment app".to_owned(),
                details: PaymentAccountDetails::Username {
                    username: "ana.pays".to_owned(),
                },
            },
        ],
    }
}

/// Bob with a first name only.
fn bob() -> Member {
    Member {
        id: None,
        first_name: "Bob".to_owned(),
        last_name: None,
        email: None,
        phone: None,
        accounts: Vec::new(),
    }
}

fn with_account(details: PaymentAccountDetails, name: &str) -> Member {
    Member {
        accounts: vec![PaymentAccount {
            name: name.to_owned(),
            details,
        }],
        ..ana()
    }
}

/// Members with one detail the library refuses.
fn refused_members() -> [Refusal; 6] {
    [
        Refusal {
            record: with_account(
                PaymentAccountDetails::Iban {
                    iban: "RO49AAAA1B31007593840001".to_owned(),
                },
                "Main account",
            ),
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::PaymentAccount {
                        failure: PaymentAccountFailure::InvalidIban { .. },
                        ..
                    }
                )
            },
        },
        Refusal {
            record: with_account(
                PaymentAccountDetails::Username {
                    username: " ".to_owned(),
                },
                "Payment app",
            ),
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::PaymentAccount {
                        failure: PaymentAccountFailure::EmptyUsername,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: with_account(iban_account().details, " "),
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::PaymentAccount {
                        failure: PaymentAccountFailure::EmptyName,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Member {
                email: Some("ana at example.com".to_owned()),
                ..ana()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Contact {
                        failure: ContactFailure::InvalidEmail,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Member {
                phone: Some("0712345678".to_owned()),
                ..ana()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Contact {
                        failure: ContactFailure::InvalidPhoneNumber,
                        ..
                    }
                )
            },
        },
        Refusal {
            record: Member {
                first_name: " ".to_owned(),
                ..ana()
            },
            is_expected: |error| {
                matches!(
                    error,
                    EranosError::Member {
                        failure: MemberFailure::EmptyFirstName,
                        ..
                    }
                )
            },
        },
    ]
}

/// A member added through the bindings would reach the library with a
/// detail changed or dropped, or come back without their identifier; the
/// details come back in the form the group holds them.
#[test]
fn a_member_to_add_converts_both_ways() {
    for record in [ana(), bob()] {
        let member = record.to_new().unwrap();
        assert_eq!(
            Member::from_library(&member),
            Member {
                id: Some(member.id().to_string()),
                ..record
            }
        );
    }
    let entered = Member {
        email: Some(" ana@example.com ".to_owned()),
        phone: Some("+40 712 345 678".to_owned()),
        accounts: vec![PaymentAccount {
            name: " Main account ".to_owned(),
            details: PaymentAccountDetails::Iban {
                iban: "ro49 aaaa 1b31 0075 9384 0000".to_owned(),
            },
        }],
        ..ana()
    };
    let held = Member::from_library(&entered.to_new().unwrap());
    assert_eq!(held.accounts, [iban_account()]);
    assert_eq!(held.email, ana().email);
    assert_eq!(held.phone, ana().phone);
}

/// A member with a detail the library refuses would reach the group, as an
/// entry to add or as a replacement.
#[test]
fn a_member_refuses_every_detail_that_does_not_convert() {
    let stored = ana().to_new().unwrap();
    for refusal in refused_members() {
        let errors = [
            refusal.record.to_new().unwrap_err(),
            refusal.record.replacing(&stored).unwrap_err(),
        ];
        for error in errors {
            assert!((refusal.is_expected)(&error), "{error:?}");
        }
    }
}

/// A member to add would lose the identifier their app gives them, take one
/// the rules refuse, or, without one, not get a generated one.
#[test]
fn a_member_to_add_keeps_the_identifier_they_carry() {
    let id = MemberId::new("!a").unwrap();
    let named = Member {
        id: Some(id.to_string()),
        ..ana()
    };
    let member = named.to_new().unwrap();
    assert_eq!(member.id(), id);
    assert_eq!(Member::from_library(&member), named);
    assert_ne!(ana().to_new().unwrap().id(), id);
    let padded = Member {
        id: Some(" x".to_owned()),
        ..ana()
    };
    assert!(matches!(
        padded.to_new(),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));
}

/// A replacement would lose the stored member's identifier, or keep a
/// detail of the stored member that the record changes or clears.
#[test]
fn a_replacement_keeps_the_identifier_and_takes_every_detail() {
    let stored = ana().to_new().unwrap();
    for record in [bob(), ana()] {
        let replacement = record.replacing(&stored).unwrap();
        assert_eq!(
            Member::from_library(&replacement),
            Member {
                id: Some(stored.id().to_string()),
                ..record
            }
        );
    }
}

/// A category would lose the identifier its app gives it, reach the group
/// with an identifier the rules refuse or a blank name, or, renamed, keep
/// another identifier.
#[test]
fn a_category_converts_and_keeps_its_identifier_when_renamed() {
    let food = Category {
        id: None,
        name: "Food".to_owned(),
    };
    let stored = food.to_new().unwrap();
    let id = Some(stored.id().to_string());
    assert_eq!(
        Category::from_library(&stored),
        Category {
            id: id.clone(),
            ..food
        }
    );
    let renamed = Category {
        id: id.clone(),
        name: "Groceries".to_owned(),
    };
    assert_eq!(
        renamed
            .replacing(&stored)
            .map(|category| Category::from_library(&category)),
        Ok(renamed.clone())
    );
    // A category to add under an identifier keeps it.
    assert_eq!(
        renamed
            .to_new()
            .map(|category| Category::from_library(&category)),
        Ok(renamed.clone())
    );
    let padded = Category {
        id: Some(" x".to_owned()),
        ..renamed
    };
    assert!(matches!(
        padded.to_new(),
        Err(EranosError::Id {
            failure: IdFailure::SurroundingWhitespace,
            ..
        })
    ));
    let blank = Category {
        id: None,
        name: " ".to_owned(),
    };
    let blank_rename = Category {
        id,
        ..blank.clone()
    };
    for error in [
        blank.to_new().unwrap_err(),
        blank_rename.replacing(&stored).unwrap_err(),
    ] {
        assert!(
            matches!(
                error,
                EranosError::Category {
                    failure: CategoryFailure::EmptyName,
                    ..
                }
            ),
            "{error:?}"
        );
    }
}
