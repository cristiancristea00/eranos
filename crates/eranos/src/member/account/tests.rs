//! Accounts tell other members where to send a repayment. These tests keep
//! the account label present and trimmed, keep IBANs and usernames
//! convertible into account details of the right kind, and keep usernames
//! out of debug output, and therefore out of logs and crash reports, however
//! deeply they are nested.

use super::*;

use crate::member::Member;

/// The username every test holds; distinctive, so a leak is easy to find.
const USERNAME: &str = "@ana.popescu.secret";

fn iban() -> Iban {
    Iban::parse("RO49AAAA1B31007593840000").unwrap()
}

fn username() -> Username {
    Username::new(USERNAME).unwrap()
}

/// Account labels typed with a stray space would be stored and shown with
/// it.
#[test]
fn the_account_name_is_trimmed() {
    // Surrounding whitespace.
    let account = Account::new(iban(), "  ING \n").unwrap();
    assert_eq!(account.name(), "ING");
    // The details are kept as given.
    assert_eq!(account.details(), &AccountDetails::Iban(iban()));
}

/// An account without a label would appear as a blank line in the payment
/// screen.
#[test]
fn a_blank_account_name_is_rejected() {
    // Empty and whitespace-only labels.
    for name in ["", "   ", "\t"] {
        assert_eq!(
            Account::new(username(), name),
            Err(ValueError::Empty {
                field: Field::AccountName
            }),
            "{name:?}"
        );
    }
}

/// An IBAN or username would be stored as the wrong kind of details.
#[test]
fn ibans_and_usernames_convert_into_account_details() {
    // IBANs.
    assert_eq!(AccountDetails::from(iban()), AccountDetails::Iban(iban()));
    // Usernames.
    assert_eq!(
        AccountDetails::from(username()),
        AccountDetails::Username(username())
    );
}

/// The apps would offer a bank transfer for a payment-app account, or the
/// other way around.
#[test]
fn kind_reports_the_kind_of_details() {
    // IBAN accounts.
    let bank = Account::new(iban(), "ING").unwrap();
    assert_eq!(bank.kind(), AccountKind::Iban);
    assert_eq!(bank.details().kind(), AccountKind::Iban);
    // Username accounts.
    let app = Account::new(username(), "Revolut").unwrap();
    assert_eq!(app.kind(), AccountKind::Username);
    assert_eq!(app.details().kind(), AccountKind::Username);
}

/// Usernames typed with a stray space would not be found in the payment app.
#[test]
fn usernames_are_trimmed() {
    // Surrounding whitespace.
    assert_eq!(Username::new(" \t@ana \n").unwrap().expose_secret(), "@ana");
    // `FromStr` parses the same way.
    assert_eq!("@ana".parse::<Username>(), Username::new("@ana"));
}

/// An account with an empty username would be stored and every repayment
/// to it would fail.
#[test]
fn a_blank_username_is_rejected() {
    // Empty and whitespace-only usernames.
    for text in ["", "   ", "\n"] {
        assert_eq!(
            Username::new(text),
            Err(ValueError::Empty {
                field: Field::Username
            }),
            "{text:?}"
        );
    }
}

/// A username would leak into logs through the debug output of anything
/// that holds it.
#[test]
fn usernames_never_appear_in_debug_output() {
    // The username itself.
    assert_eq!(format!("{:?}", username()), "Username([REDACTED])");
    // The account details and the account.
    let details = AccountDetails::from(username());
    let account = Account::new(username(), "Revolut").unwrap();
    // A member holding the account.
    let member = Member::builder("Ana")
        .account(account.clone())
        .build()
        .unwrap();
    for debug in [
        format!("{details:?}"),
        format!("{account:?}"),
        format!("{account:#?}"),
        format!("{member:?}"),
        format!("{member:#?}"),
    ] {
        assert!(!debug.contains(USERNAME), "{debug}");
        assert!(!debug.contains("ana.popescu"), "{debug}");
        assert!(debug.contains("[REDACTED]"), "{debug}");
    }
}

/// Payment screens would show a different username than the one entered.
#[test]
fn expose_secret_returns_the_username() {
    // The stored text.
    assert_eq!(username().expose_secret(), USERNAME);
    // Equality compares the text.
    assert_eq!(username(), Username::new(USERNAME).unwrap());
    assert_ne!(username(), Username::new("@bob").unwrap());
}

/// Saved accounts would fail to load, or an account with a blank label or
/// username would load.
#[cfg(feature = "serde")]
#[test]
fn accounts_round_trip_through_serde() {
    // IBAN accounts.
    let bank = Account::new(iban(), "ING").unwrap();
    let json = serde_json::to_string(&bank).unwrap();
    assert_eq!(
        json,
        r#"{"details":{"iban":"RO49AAAA1B31007593840000"},"name":"ING"}"#
    );
    assert_eq!(serde_json::from_str::<Account>(&json).unwrap(), bank);
    // Username accounts, trimmed on the way in.
    let app = Account::new(Username::new("@ana").unwrap(), "Revolut").unwrap();
    let loaded: Account =
        serde_json::from_str(r#"{"name":" Revolut ","details":{"username":" @ana "}}"#).unwrap();
    assert_eq!(loaded, app);
    // Blank labels and usernames are rejected.
    assert!(
        serde_json::from_str::<Account>(r#"{"name":" ","details":{"username":"@ana"}}"#).is_err()
    );
    assert!(
        serde_json::from_str::<Account>(r#"{"name":"Revolut","details":{"username":" "}}"#)
            .is_err()
    );
    // The kind alone.
    assert_eq!(
        serde_json::to_string(&AccountKind::Username).unwrap(),
        "\"username\""
    );
}
