//! Email addresses and phone numbers are how members reach each other to
//! settle up. These tests keep addresses to the RFC 5321 and 5322 rules,
//! which refuse the typos mobile keyboards make, keep obviously broken
//! numbers out, keep surrounding whitespace from making two entries of the
//! same contact differ, and keep phone numbers stored in the one `+digits`
//! form that diallers and messaging apps accept.

use super::*;

/// Correct addresses would be refused when typed or pasted.
#[test]
fn a_well_formed_email_is_accepted() {
    // A typical address, kept as entered.
    assert_eq!(
        Email::parse("Ana.Popescu@Example.com").unwrap().as_str(),
        "Ana.Popescu@Example.com"
    );
    // The shortest shape: a domain needs no dot.
    assert_eq!(Email::parse("a@b").unwrap().as_str(), "a@b");
    // A subaddress, capitals, and every atext symbol.
    for text in [
        "ana+splitwise@example.com",
        "ANA@EXAMPLE.COM",
        "!#$%&'*+-/=?^_`{|}~@example.com",
    ] {
        assert_eq!(Email::parse(text).unwrap().as_str(), text);
    }
    // `FromStr` parses the same way.
    assert_eq!(
        "ana@example.com".parse::<Email>(),
        Email::parse("ana@example.com")
    );
}

/// An address without a mailbox or a domain would be stored and every
/// message sent to it would bounce.
#[test]
fn an_email_needs_one_at_with_text_on_both_sides() {
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
        "\"ana popescu\"@example.com",
    ] {
        assert_eq!(
            Email::parse(text),
            Err(ValueError::InvalidEmail),
            "{text:?}"
        );
    }
}

/// A leading, trailing, or doubled dot before the `@`, a common typo, would
/// be stored and every message sent to it would bounce.
#[test]
fn dots_separate_non_empty_atoms_in_a_local_part() {
    // Dots between atoms.
    assert!(Email::parse("ana.popescu@example.com").is_ok());
    // A doubled dot, and a dot at either end.
    for text in [
        "ana..popescu@example.com",
        ".ana@example.com",
        "ana.@example.com",
    ] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
}

/// A comma typed for a dot, a pasted `mailto:` or angle brackets, or a
/// comment would be stored as part of the address.
#[test]
fn symbols_outside_atext_are_rejected_in_a_local_part() {
    // Symbols RFC 5322 allows only inside quotes.
    for text in [
        "ana,bob@example.com",
        "ana;bob@gmail.com",
        "ana<b>@example.com",
        "ana\\@example.com",
        "<ana@example.com>",
        "mailto:ana@example.com",
        "ana(comment)@example.com",
        "(comment)ana@example.com",
    ] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
}

/// Addresses RFC 5322 allows in quotes would be refused, or broken quoting
/// stored.
#[test]
fn a_quoted_local_part_may_hold_any_printable_character() {
    // Plain text, an `@`, doubled dots, quoted pairs, and letters outside
    // ASCII.
    for text in [
        "\"ana\"@example.com",
        "\"a@b\"@example.com",
        "\"ana..popescu\"@example.com",
        "\"a\\\"b\"@example.com",
        "\"a\\\\b\"@example.com",
        "\"ană\"@example.com",
    ] {
        assert_eq!(Email::parse(text).unwrap().as_str(), text);
    }
    // Empty quotes, a bare quote inside, an escaped closing quote, no closing
    // quote, a control character, a quoted control character, and a control
    // character outside ASCII.
    for text in [
        "\"\"@example.com",
        "\"a\"b\"@example.com",
        "\"a\\\"@example.com",
        "\"ana@example.com",
        "\"a\u{1}\"@example.com",
        "\"a\\\u{1}\"@example.com",
        "\"a\u{80}\"@example.com",
    ] {
        assert_eq!(
            Email::parse(text),
            Err(ValueError::InvalidEmail),
            "{text:?}"
        );
    }
}

/// Addresses in other scripts or with emoji, which RFC 6531 allows, would be
/// refused, or an invisible character pasted from a web page stored.
#[test]
fn characters_outside_ascii_are_accepted_unless_invisible() {
    // Emoji, a decomposed accent, letters, and other scripts.
    for text in [
        "🙂@example.com",
        "ane\u{301}@example.com",
        "anä@example.com",
        "用户@例子.广告",
    ] {
        assert_eq!(Email::parse(text).unwrap().as_str(), text);
    }
    // A format character, a control, private use, and a noncharacter in the
    // local part, and a format character in the domain.
    for text in [
        "ana\u{200b}@example.com",
        "ana\u{80}@example.com",
        "ana\u{e000}@example.com",
        "ana\u{ffff}@example.com",
        "ana@exa\u{200b}mple.com",
    ] {
        assert_eq!(
            Email::parse(text),
            Err(ValueError::InvalidEmail),
            "{text:?}"
        );
    }
}

/// An empty label, a trailing full stop (which iOS inserts after a double
/// space), or a stray symbol would be stored and every message would bounce.
#[test]
fn a_domain_is_labels_of_letters_digits_and_hyphens() {
    // Empty labels, symbols, hyphens at either end, and emoji.
    for text in [
        "ana@example..com",
        "ana@.gmail.com",
        "ana@example.com.",
        "ana@gmail.com.",
        "ana@gmail,com",
        "ana@gmail.com>",
        "ana@gmail.com;",
        "ana@-example.com",
        "ana@example-.com",
        "ana@example_domain.com",
        "ana@ex!ample.com",
        "ana@🙂.com",
    ] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
    // Hyphens inside, letters outside ASCII, punycode, a single label, and
    // digits.
    for text in [
        "ana@ex-ample.com",
        "ana@müller.de",
        "ana@MÜLLER.de",
        "ana@xn--mller-kva.de",
        "ana@localhost",
        "ana@123",
        "ana@192.168.0.1",
    ] {
        assert_eq!(Email::parse(text).unwrap().as_str(), text);
    }
}

/// An address literal that is not an IP address would be stored and could
/// never receive mail.
#[test]
fn a_bracketed_domain_is_an_ip_address() {
    // IPv4, and IPv6 behind its tag in either case.
    for text in [
        "ana@[192.168.0.1]",
        "ana@[IPv6:2001:db8::1]",
        "ana@[ipv6:2001:db8::1]",
    ] {
        assert_eq!(Email::parse(text).unwrap().as_str(), text);
    }
    // An impossible IPv4 address, IPv6 without its tag, an IPv4 address behind
    // it, a name, and unbalanced brackets.
    for text in [
        "ana@[300.1.1.1]",
        "ana@[2001:db8::1]",
        "ana@[IPv6:192.168.0.1]",
        "ana@[example.com]",
        "ana@[192.168.0.1",
        "ana@192.168.0.1]",
    ] {
        assert_eq!(Email::parse(text), Err(ValueError::InvalidEmail), "{text}");
    }
}

/// An address longer than SMTP carries would be stored and every message
/// refused by the sending server.
#[test]
fn an_email_fits_the_smtp_length_limits() {
    let address = |local_part: &str, domain: &str| format!("{local_part}@{domain}");
    // A local part of 64 octets, but not 65.
    assert!(Email::parse(&address(&"a".repeat(64), "example.com")).is_ok());
    assert!(Email::parse(&address(&"a".repeat(65), "example.com")).is_err());
    // A label of 63 octets, but not 64.
    assert!(Email::parse(&address("ana", &format!("{}.com", "b".repeat(63)))).is_ok());
    assert!(Email::parse(&address("ana", &format!("{}.com", "b".repeat(64)))).is_err());
    // An address of 254 octets, but not 255.
    let domain =
        |last: usize| format!("{}.{}.{}", "b".repeat(63), "c".repeat(63), "d".repeat(last));
    let longest = address(&"a".repeat(64), &domain(61));
    assert_eq!(longest.len(), 254);
    assert!(Email::parse(&longest).is_ok());
    assert!(Email::parse(&address(&"a".repeat(64), &domain(62))).is_err());
    // The limits count octets: 32 two-octet letters fill a local part.
    assert!(Email::parse(&address(&"ä".repeat(32), "example.com")).is_ok());
    assert!(Email::parse(&address(&"ä".repeat(33), "example.com")).is_err());
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
    assert!(serde_json::from_str::<Email>("\"ana..popescu@example.com\"").is_err());
    // Phone round trip, normalised on the way in.
    let phone = PhoneNumber::parse("+40712345678").unwrap();
    assert_eq!(serde_json::to_string(&phone).unwrap(), "\"+40712345678\"");
    assert_eq!(
        serde_json::from_str::<PhoneNumber>("\"+40 712 345 678\"").unwrap(),
        phone
    );
    assert!(serde_json::from_str::<PhoneNumber>("\"0712345678\"").is_err());
}
