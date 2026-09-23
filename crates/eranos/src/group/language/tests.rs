//! A group's language tag picks the apps' display language. These tests keep
//! tags in the shape of BCP 47 (a first subtag of 2 to 8 letters, then
//! subtags of 1 to 8 letters or digits) so the apps can hand them to their
//! locale APIs, and keep surrounding whitespace out of the stored tag.

use super::*;

/// Common tags would be refused when a group picks its language.
#[test]
fn well_formed_tags_are_accepted() {
    // Language, region, script, and numeric region subtags.
    for text in ["en", "ro-RO", "zh-Hant-TW", "es-419", "sr-Latn-RS"] {
        assert_eq!(Language::parse(text).unwrap().as_str(), text);
    }
    // Letter case is kept as entered.
    assert_eq!(Language::parse("EN-gb").unwrap().as_str(), "EN-gb");
    // The first subtag at both length bounds.
    assert!(Language::parse("de").is_ok());
    assert!(Language::parse("abcdefgh").is_ok());
    // Later subtags at both length bounds.
    assert!(Language::parse("en-a").is_ok());
    assert!(Language::parse("en-abcdefgh").is_ok());
    assert!(Language::parse("en-12345678").is_ok());
}

/// A malformed tag would reach the apps' locale APIs, which reject or
/// misread it.
#[test]
fn malformed_tags_are_rejected() {
    // Other separators.
    for text in ["ro_RO", "ro RO", "ro.RO"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // A first subtag that is too short, too long, or not letters. `x-private`
    // is a private-use tag in BCP 47, but its one-letter first subtag is
    // outside the accepted shape.
    for text in ["r", "x-private", "abcdefghi", "123", "e1", "é-FR"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // Empty subtags.
    for text in ["en-", "-en", "en--US", "-"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // A later subtag that is too long or not alphanumeric.
    for text in ["en-abcdefghi", "en-U$", "en-ÜS"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // Nothing at all.
    for text in ["", "   "] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text:?}"
        );
    }
}

/// A tag typed with a stray space would be stored with it and rejected by
/// the apps' locale APIs.
#[test]
fn surrounding_whitespace_is_trimmed() {
    // Spaces, tabs, and newlines around the tag.
    let language = Language::parse(" \tro-RO\n").unwrap();
    assert_eq!(language.as_str(), "ro-RO");
    assert_eq!(language, Language::parse("ro-RO").unwrap());
    // Display shows the stored form.
    assert_eq!(language.to_string(), "ro-RO");
    // `FromStr` parses the same way.
    assert_eq!(" ro-RO ".parse::<Language>(), Ok(language));
}

/// Saved groups would fail to load their language, or a malformed tag would
/// load.
#[cfg(feature = "serde")]
#[test]
fn tags_serialize_as_their_text_form() {
    // Round trip.
    let language = Language::parse("ro-RO").unwrap();
    assert_eq!(serde_json::to_string(&language).unwrap(), "\"ro-RO\"");
    assert_eq!(
        serde_json::from_str::<Language>("\"ro-RO\"").unwrap(),
        language
    );
    // Loading validates like parsing.
    assert!(serde_json::from_str::<Language>("\"ro_RO\"").is_err());
}
