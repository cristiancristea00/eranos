//! A group's language tag picks the apps' display language. These tests keep
//! tags to the Unicode BCP 47 locale identifiers (UTS #35) that the apps'
//! locale APIs accept, stored in canonical form, and keep surrounding
//! whitespace out of the stored tag.

use super::*;

/// Common tags would be refused when a group picks its language.
#[test]
fn well_formed_tags_are_accepted() {
    // Language, script, region, and numeric region subtags, already in
    // canonical form, stay as typed.
    for text in ["en", "fil", "ro-RO", "zh-Hant-TW", "es-419", "sr-Latn-RS"] {
        assert_eq!(Language::parse(text).unwrap().as_str(), text);
    }
    // Variants at their length bounds: four characters starting with a
    // digit, and eight letters or digits.
    for text in ["de-1996", "en-abcdefgh", "en-12345678"] {
        assert_eq!(Language::parse(text).unwrap().as_str(), text);
    }
    // A Unicode extension and a private-use extension.
    for text in ["th-TH-u-nu-thai", "de-x-private"] {
        assert_eq!(Language::parse(text).unwrap().as_str(), text);
    }
}

/// The same language typed in two letter cases would be stored as two tags,
/// and the apps would compare or look up the wrong one.
#[test]
fn tags_are_stored_in_canonical_form() {
    // Canonical case for each subtag.
    assert_eq!(Language::parse("EN-gb").unwrap().as_str(), "en-GB");
    assert_eq!(
        Language::parse("zh-hant-tw").unwrap().as_str(),
        "zh-Hant-TW"
    );
    // Sorted variants and sorted `-u-` keywords.
    assert_eq!(
        Language::parse("sl-rozaj-biske-1994").unwrap().as_str(),
        "sl-1994-biske-rozaj"
    );
    assert_eq!(
        Language::parse("en-u-nu-latn-ca-gregory").unwrap().as_str(),
        "en-u-ca-gregory-nu-latn"
    );
    // Aliases stay as typed: mapping `iw` to `he` is the apps' job.
    assert_eq!(Language::parse("iw").unwrap().as_str(), "iw");
    // Tags that differ only in case are equal.
    assert_eq!(Language::parse("EN-gb"), Language::parse("en-GB"));
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
    // is a private-use tag in BCP 47, but UTS #35 requires a language first.
    for text in ["r", "x-private", "abcdefghi", "123", "e1", "é-FR"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // Languages of 4 to 8 letters, none of which was ever registered.
    for text in ["abcd", "abcde", "abcdefgh", "root"] {
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

/// A tag the apps' locale APIs throw on would be stored, and the group's
/// display language would break when it loads.
#[test]
fn misplaced_or_repeated_subtags_are_rejected() {
    // Extensions without subtags, or with a one-character subtag.
    for text in ["en-a", "en-u", "en-x", "en-a-b"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
    // A repeated region, a script after the region, a repeated variant, and
    // a repeated extension.
    for text in ["en-US-US", "en-US-Latn", "de-1901-1901", "en-a-bbb-a-ccc"] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
        );
    }
}

/// Legacy BCP 47 forms that UTS #35 drops would reach locale APIs that no
/// longer read them.
#[test]
fn extended_and_grandfathered_tags_are_rejected() {
    // Extended language subtags, and tags grandfathered from RFC 3066.
    for text in [
        "zh-yue-HK",
        "zh-cmn-Hans-CN",
        "en-GB-oed",
        "sgn-BE-FR",
        "zh-min-nan",
        "no-bok",
        "i-klingon",
    ] {
        assert_eq!(
            Language::parse(text),
            Err(ValueError::InvalidLanguage),
            "{text}"
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
fn tags_serialise_as_their_text_form() {
    // Round trip.
    let language = Language::parse("ro-RO").unwrap();
    assert_eq!(serde_json::to_string(&language).unwrap(), "\"ro-RO\"");
    assert_eq!(
        serde_json::from_str::<Language>("\"ro-RO\"").unwrap(),
        language
    );
    // Loading validates like parsing and keeps the canonical form.
    assert_eq!(
        serde_json::from_str::<Language>("\"EN-gb\"")
            .unwrap()
            .as_str(),
        "en-GB"
    );
    assert!(serde_json::from_str::<Language>("\"ro_RO\"").is_err());
    assert!(serde_json::from_str::<Language>("\"en-US-US\"").is_err());
}
