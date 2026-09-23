//! Members are the people every expense and balance names. These tests keep
//! the required first name present and trimmed, keep optional details absent
//! rather than blank, keep contact details and accounts settable and
//! clearable, and keep edits made through `to_builder` from touching the
//! member they were copied from.

use super::*;

fn email() -> Email {
    Email::parse("ana@example.com").unwrap()
}

fn phone() -> PhoneNumber {
    PhoneNumber::parse("+40712345678").unwrap()
}

fn bank_account() -> Account {
    Account::new("ING", Iban::parse("RO49AAAA1B31007593840000").unwrap()).unwrap()
}

fn app_account() -> Account {
    Account::new("Revolut", Username::new("@ana").unwrap()).unwrap()
}

/// A member with every optional detail set.
fn ana() -> Member {
    Member::builder("Ana")
        .last_name("Popescu")
        .email(email())
        .phone(phone())
        .account(bank_account())
        .account(app_account())
        .build()
        .unwrap()
}

/// Names typed with a stray space would be stored and shown with it.
#[test]
fn the_first_name_is_trimmed() {
    // Surrounding whitespace.
    let member = Member::builder("  Ana \n").build().unwrap();
    assert_eq!(member.first_name(), "Ana");
    // Inner whitespace stays.
    let ana_maria = Member::builder("Ana Maria").build().unwrap();
    assert_eq!(ana_maria.first_name(), "Ana Maria");
}

/// A member without a name would appear as a blank line in every list.
#[test]
fn a_blank_first_name_is_rejected() {
    // Empty and whitespace-only names.
    for first_name in ["", "  ", "\t\n"] {
        assert_eq!(
            Member::builder(first_name).build(),
            Err(ValueError::Empty {
                field: Field::FirstName
            }),
            "{first_name:?}"
        );
    }
}

/// A new member would show details nobody entered.
#[test]
fn a_member_starts_with_only_a_first_name() {
    // Every optional detail is absent.
    let member = Member::builder("Ana").build().unwrap();
    assert_eq!(member.last_name(), None);
    assert_eq!(member.email(), None);
    assert_eq!(member.phone(), None);
    assert!(member.accounts().is_empty());
}

/// A last name left blank would be stored as an empty string and shown as
/// a trailing space.
#[test]
fn the_last_name_is_trimmed_and_blank_clears_it() {
    // Surrounding whitespace.
    let member = Member::builder("Ana")
        .last_name(" Popescu ")
        .build()
        .unwrap();
    assert_eq!(member.last_name(), Some("Popescu"));
    // Blank text is no last name.
    let blank_last_name_member = Member::builder("Ana").last_name("   ").build().unwrap();
    assert_eq!(blank_last_name_member.last_name(), None);
    // Blank text clears a last name set before.
    let cleared_last_name_member = ana().to_builder().last_name("").build().unwrap();
    assert_eq!(cleared_last_name_member.last_name(), None);
}

/// Contact details could not be added, or not removed once added.
#[test]
fn email_and_phone_are_set_and_cleared_with_none() {
    // Set.
    let member = Member::builder("Ana")
        .email(email())
        .phone(phone())
        .build()
        .unwrap();
    assert_eq!(member.email(), Some(&email()));
    assert_eq!(member.phone(), Some(&phone()));
    // Cleared.
    let member = member.to_builder().email(None).phone(None).build().unwrap();
    assert_eq!(member.email(), None);
    assert_eq!(member.phone(), None);
}

/// Accounts would be lost when another is added, or kept when the list is
/// replaced.
#[test]
fn account_appends_and_accounts_replaces() {
    // Appending keeps the order of addition.
    let member = Member::builder("Ana")
        .account(bank_account())
        .account(app_account())
        .build()
        .unwrap();
    assert_eq!(member.accounts(), [bank_account(), app_account()]);
    // Replacing drops the accounts added before.
    let member = member
        .to_builder()
        .accounts([app_account()])
        .build()
        .unwrap();
    assert_eq!(member.accounts(), [app_account()]);
    // Replacing with nothing removes every account.
    let member = member.to_builder().accounts(Vec::new()).build().unwrap();
    assert!(member.accounts().is_empty());
}

/// Editing a member would silently drop details that were not edited.
#[test]
fn to_builder_copies_every_field() {
    // Rebuilt unchanged.
    let member = ana();
    assert_eq!(member.to_builder().build().unwrap(), member);
}

/// Editing a copy of a member would change the member shown elsewhere.
#[test]
fn edits_through_to_builder_leave_the_original_unchanged() {
    let original = ana();
    // Every field edited.
    let edited = original
        .to_builder()
        .first_name(" Anna ")
        .last_name("")
        .email(None)
        .phone(None)
        .accounts(Vec::new())
        .build()
        .unwrap();
    assert_eq!(edited.first_name(), "Anna");
    assert_eq!(edited.last_name(), None);
    assert_eq!(edited.email(), None);
    assert_eq!(edited.phone(), None);
    assert!(edited.accounts().is_empty());
    // The original keeps its values.
    assert_eq!(original, ana());
    assert_eq!(original.first_name(), "Ana");
}

/// A rename to a blank name would be accepted.
#[test]
fn a_renamed_member_is_validated_again() {
    // Blank replacement.
    assert_eq!(
        ana().to_builder().first_name(" ").build(),
        Err(ValueError::Empty {
            field: Field::FirstName
        })
    );
}

/// Saved members would fail to load, or a member with a blank name would
/// load.
#[cfg(feature = "serde")]
#[test]
fn members_round_trip_through_serde() {
    // Every field.
    let member = ana();
    let json = serde_json::to_string(&member).unwrap();
    assert_eq!(serde_json::from_str::<Member>(&json).unwrap(), member);
    // Only the first name; absent details are left out.
    let minimal_member = Member::builder("Ana").build().unwrap();
    let minimal_json = serde_json::to_string(&minimal_member).unwrap();
    assert_eq!(minimal_json, r#"{"first_name":"Ana"}"#);
    assert_eq!(
        serde_json::from_str::<Member>(&minimal_json).unwrap(),
        minimal_member
    );
    // Loading trims and validates like the builder.
    let loaded: Member = serde_json::from_str(r#"{"first_name":" Ana ","last_name":" "}"#).unwrap();
    assert_eq!(loaded, minimal_member);
    assert!(serde_json::from_str::<Member>(r#"{"first_name":"  "}"#).is_err());
}
