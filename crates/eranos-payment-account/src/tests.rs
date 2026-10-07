//! Accounts tell other members where to send a repayment. These tests keep
//! the account label present and trimmed, keep IBANs and usernames
//! convertible into account details of the right kind, keep usernames
//! printed as they were entered, and keep the kind of details through
//! `FromPaymentAccountDetails`.

use super::*;

/// The username most tests hold.
const USERNAME: &str = "@ana.popescu";

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
    let account = PaymentAccount::new("  ING \n", iban()).unwrap();
    assert_eq!(account.name(), "ING");
    // The details are kept as given.
    assert_eq!(account.details(), &PaymentAccountDetails::Iban(iban()));
}

/// An account without a label would appear as a blank line in the payment
/// screen.
#[test]
fn a_blank_account_name_is_rejected() {
    // Empty and whitespace-only labels.
    for name in ["", "   ", "\t"] {
        assert_eq!(
            PaymentAccount::new(name, username()),
            Err(PaymentAccountError::EmptyName),
            "{name:?}"
        );
    }
}

/// An IBAN or username would be stored as the wrong kind of details.
#[test]
fn ibans_and_usernames_convert_into_account_details() {
    // IBANs.
    assert_eq!(
        PaymentAccountDetails::from(iban()),
        PaymentAccountDetails::Iban(iban())
    );
    // Usernames.
    assert_eq!(
        PaymentAccountDetails::from(username()),
        PaymentAccountDetails::Username(username())
    );
}

/// The apps would offer a bank transfer for a payment-app account, or the
/// other way around.
#[test]
fn kind_reports_the_kind_of_details() {
    // IBAN accounts.
    let bank = PaymentAccount::new("ING", iban()).unwrap();
    assert_eq!(bank.kind(), PaymentAccountKind::Iban);
    assert_eq!(bank.details().kind(), PaymentAccountKind::Iban);
    // Username accounts.
    let app = PaymentAccount::new("Revolut", username()).unwrap();
    assert_eq!(app.kind(), PaymentAccountKind::Username);
    assert_eq!(app.details().kind(), PaymentAccountKind::Username);
}

/// Usernames typed with a stray space would not be found in the payment app.
#[test]
fn usernames_are_trimmed() {
    // Surrounding whitespace.
    assert_eq!(Username::new(" \t@ana \n").unwrap().as_str(), "@ana");
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
            Err(PaymentAccountError::EmptyUsername),
            "{text:?}"
        );
    }
}

/// Payment screens would show the username differently from how it was
/// entered, or debug output would show something else than the value.
#[test]
fn usernames_print_as_entered() {
    // Display and debug output.
    assert_eq!(username().to_string(), USERNAME);
    assert_eq!(format!("{:?}", username()), "Username(\"@ana.popescu\")");
    // Nested in account details.
    let details = PaymentAccountDetails::from(username());
    assert_eq!(
        format!("{details:?}"),
        "Username(Username(\"@ana.popescu\"))"
    );
}

/// Payment screens would show a different username than the one entered.
#[test]
fn as_str_returns_the_username() {
    // The stored text.
    assert_eq!(username().as_str(), USERNAME);
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
    let bank = PaymentAccount::new("ING", iban()).unwrap();
    let json = serde_json::to_string(&bank).unwrap();
    assert_eq!(
        json,
        r#"{"name":"ING","details":{"iban":"RO49AAAA1B31007593840000"}}"#
    );
    assert_eq!(serde_json::from_str::<PaymentAccount>(&json).unwrap(), bank);
    // Username accounts, trimmed on the way in.
    let app = PaymentAccount::new("Revolut", Username::new("@ana").unwrap()).unwrap();
    let loaded: PaymentAccount =
        serde_json::from_str(r#"{"name":" Revolut ","details":{"username":" @ana "}}"#).unwrap();
    assert_eq!(loaded, app);
    // Blank labels and usernames are rejected.
    assert!(
        serde_json::from_str::<PaymentAccount>(r#"{"name":" ","details":{"username":"@ana"}}"#)
            .is_err()
    );
    assert!(
        serde_json::from_str::<PaymentAccount>(r#"{"name":"Revolut","details":{"username":" "}}"#)
            .is_err()
    );
    // The kind alone.
    assert_eq!(
        serde_json::to_string(&PaymentAccountKind::Username).unwrap(),
        "\"username\""
    );
}

/// Account details as a crate outside this one converts them.
#[derive(Debug, PartialEq, Eq)]
enum ConvertedDetails {
    Iban(String),
    Username(String),
}

impl FromPaymentAccountDetails for ConvertedDetails {
    fn iban(iban: &Iban) -> Self {
        Self::Iban(iban.as_str().to_owned())
    }

    fn username(username: &Username) -> Self {
        Self::Username(username.as_str().to_owned())
    }
}

/// A conversion that called another variant's method would show a bank
/// account as a payment-app username.
#[test]
fn account_details_convert_through_the_method_of_their_variant() {
    assert_eq!(
        ConvertedDetails::from_payment_account_details(&PaymentAccountDetails::from(iban())),
        ConvertedDetails::Iban(iban().as_str().to_owned())
    );
    assert_eq!(
        ConvertedDetails::from_payment_account_details(&PaymentAccountDetails::from(username())),
        ConvertedDetails::Username(USERNAME.to_owned())
    );
}
