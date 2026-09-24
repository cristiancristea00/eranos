//! Email addresses and phone numbers are how members reach each other to
//! settle up. These tests keep obviously broken addresses and numbers out,
//! keep surrounding whitespace from making two entries of the same contact
//! differ, and keep phone numbers stored in the one `+digits` form that
//! diallers and messaging apps accept.

use super::*;

/// Correct addresses would be refused when typed or pasted.
#[test]
fn a_well_formed_email_is_accepted() {
    // A typical address, kept as entered.
    assert_eq!(
        Email::parse("Ana.Popescu@Example.com").unwrap().as_str(),
        "Ana.Popescu@Example.com"
    );
    // The shortest shape.
    assert_eq!(Email::parse("a@b").unwrap().as_str(), "a@b");
    // `FromStr` parses the same way.
    assert_eq!(
        "ana@example.com".parse::<Email>(),
        Email::parse("ana@example.com")
    );
}

/// An address without a mailbox or a domain would be stored and every
/// message sent to it would bounce.
#[test]
fn an_email_needs_exactly_one_at_with_text_on_both_sides() {
    // No `@`.
    assert_eq!(
        Email::parse("ana.example.com"),
        Err(ValueError::InvalidEmail)
    );
    // More than one `@`.
    for text in ["ana@@example.com", "ana@ex@ample.com", "@ana@example.com"] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
    // An empty side.
    for text in ["@example.com", "ana@", "@"] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
    // Nothing at all.
    for text in ["", "   "] {
        assert_eq!(
            Email::parse(text),
            Err(ValueError::InvalidEmail),
            "{text:?}"
        );
    }
}

/// An address with a space in it would be stored and every message sent to
/// it would bounce.
#[test]
fn whitespace_inside_an_email_is_rejected() {
    // Spaces, tabs, newlines, and non-breaking spaces between the ends.
    for text in [
        "ana @example.com",
        "ana@ example.com",
        "ana@exam ple.com",
        "ana\t@example.com",
        "ana@example.com\nbcc@example.com",
        "ana\u{a0}@example.com",
    ] {
        assert_eq!(
            Email::parse(text),
            Err(ValueError::InvalidEmail),
            "{text:?}"
        );
    }
}

/// The same address typed with a stray space would be stored as a second,
/// different address.
#[test]
fn surrounding_whitespace_is_trimmed_from_an_email() {
    // Spaces, tabs, and newlines around the address.
    let email = Email::parse(" \tana@example.com\n").unwrap();
    assert_eq!(email.as_str(), "ana@example.com");
    assert_eq!(email, Email::parse("ana@example.com").unwrap());
    // Display shows the stored form.
    assert_eq!(email.to_string(), "ana@example.com");
}

/// Numbers typed in a common local style would be refused, or stored with
/// their separators and fail in a dialler.
#[test]
fn phone_separators_are_dropped() {
    // Spaces, parentheses, dashes, and dots.
    for text in [
        "+40 (712) 345-678",
        "+40.712.345.678",
        "+40-712-345-678",
        "+40712345678",
    ] {
        assert_eq!(
            PhoneNumber::parse(text).unwrap().as_str(),
            "+40712345678",
            "{text}"
        );
    }
    // Display shows the stored form.
    assert_eq!(
        PhoneNumber::parse("+40 712 345 678").unwrap().to_string(),
        "+40712345678"
    );
    // `FromStr` parses the same way.
    assert_eq!(
        "+40 712 345 678".parse::<PhoneNumber>(),
        PhoneNumber::parse("+40712345678")
    );
}

/// A number without its country code would be stored and dialled in the
/// wrong country.
#[test]
fn a_phone_number_needs_a_leading_plus() {
    // Local and national formats.
    for text in ["0712345678", "0712 345 678", "0040712345678", "40712345678"] {
        assert_eq!(
            PhoneNumber::parse(text),
            Err(ValueError::InvalidPhoneNumber),
            "{text}"
        );
    }
    // A sign alone or a second sign.
    for text in ["+", "++40712345678", "+40+712345678"] {
        assert_eq!(
            PhoneNumber::parse(text),
            Err(ValueError::InvalidPhoneNumber),
            "{text}"
        );
    }
}

/// A number too short or too long for E.164 would be stored and never
/// connect.
#[test]
fn a_phone_number_has_7_to_15_digits() {
    // The bounds.
    assert_eq!(PhoneNumber::parse("+1234567").unwrap().as_str(), "+1234567");
    assert_eq!(
        PhoneNumber::parse("+123456789012345").unwrap().as_str(),
        "+123456789012345"
    );
    // One digit fewer or more.
    for text in ["+123456", "+1234567890123456"] {
        assert_eq!(
            PhoneNumber::parse(text),
            Err(ValueError::InvalidPhoneNumber),
            "{text}"
        );
    }
    // Separators do not count as digits.
    assert_eq!(
        PhoneNumber::parse("+1 2 3 4 5 6"),
        Err(ValueError::InvalidPhoneNumber)
    );
}

/// A number with letters or unknown symbols would be stored and fail to
/// dial.
#[test]
fn letters_and_other_symbols_are_rejected_in_a_phone_number() {
    // Letters, extensions, and symbols other than the separators.
    for text in [
        "+40712345abc",
        "+40 712 345 678 ext 9",
        "+40/712/345/678",
        "+40_712_345_678",
        "+40\t712345678",
        "+4071234567８",
    ] {
        assert_eq!(
            PhoneNumber::parse(text),
            Err(ValueError::InvalidPhoneNumber),
            "{text:?}"
        );
    }
}

/// A number pasted with a stray space or newline would be refused.
#[test]
fn surrounding_whitespace_is_trimmed_from_a_phone_number() {
    // Spaces, tabs, and newlines around the number.
    assert_eq!(
        PhoneNumber::parse(" \t+40712345678\n").unwrap().as_str(),
        "+40712345678"
    );
}

/// Saved contact details would fail to load, or broken ones would load.
#[cfg(feature = "serde")]
#[test]
fn contact_details_serialise_as_their_text_form() {
    // Email round trip.
    let email = Email::parse("ana@example.com").unwrap();
    assert_eq!(
        serde_json::to_string(&email).unwrap(),
        "\"ana@example.com\""
    );
    assert_eq!(
        serde_json::from_str::<Email>("\"ana@example.com\"").unwrap(),
        email
    );
    assert!(serde_json::from_str::<Email>("\"ana.example.com\"").is_err());
    // Phone round trip, normalised on the way in.
    let phone = PhoneNumber::parse("+40712345678").unwrap();
    assert_eq!(serde_json::to_string(&phone).unwrap(), "\"+40712345678\"");
    assert_eq!(
        serde_json::from_str::<PhoneNumber>("\"+40 712 345 678\"").unwrap(),
        phone
    );
    assert!(serde_json::from_str::<PhoneNumber>("\"0712345678\"").is_err());
}
