//! Members share IBANs so others can repay them. These tests keep the
//! registry's own example valid for every country, catch single-digit typos,
//! letters typed for digits, and check digits no bank issues before a
//! repayment bounces, reject numbers of an unknown country or the wrong
//! length, and keep the full number out of debug output and logs.

use super::*;

/// The registry's German example, in electronic format.
const GERMAN: &str = "DE89370400440532013000";

/// `number` in print format: lowercase, in groups of four separated by
/// spaces.
fn print_format(number: &str) -> String {
    number
        .to_ascii_lowercase()
        .as_bytes()
        .chunks(4)
        .map(|group| std::str::from_utf8(group).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Account numbers from some country would be refused as invalid.
#[test]
fn every_registry_example_validates() {
    // One example per registry entry.
    assert_eq!(registry::EXAMPLES.len(), registry::COUNTRIES.len());
    // Each example has its country's code and length, and validates.
    for (example, country) in registry::EXAMPLES.iter().zip(registry::COUNTRIES) {
        assert!(example.starts_with(country.code), "{example}");
        assert_eq!(example.len(), usize::from(country.length), "{example}");
        let iban = Iban::parse(example).unwrap();
        assert_eq!(iban.expose_secret(), *example);
        assert_eq!(iban.country_code(), country.code);
    }
}

/// Lookups would miss countries, because the binary search needs sorted,
/// unique codes, or a registry typo would reject every number of a country.
#[test]
fn the_registry_is_sorted_and_unique() {
    // Strictly increasing country codes.
    for pair in registry::COUNTRIES.windows(2) {
        assert!(
            pair[0].code < pair[1].code,
            "{} before {}",
            pair[0].code,
            pair[1].code
        );
    }
    // Two uppercase letters and an ISO 13616 length.
    for country in registry::COUNTRIES {
        let code = country.code;
        assert_eq!(code.len(), 2, "{code}");
        assert!(code.bytes().all(|byte| byte.is_ascii_uppercase()), "{code}");
        assert!((15..=34).contains(&country.length), "{code}");
    }
}

/// A structure that does not add up to its country's length would check
/// characters against the wrong class, or leave some unchecked.
#[test]
fn every_structure_adds_up_to_the_country_length() {
    for country in registry::COUNTRIES {
        let code = country.code;
        let mut total = 0;
        // Segments of a length without leading zeros, `!`, and a class.
        for segment in country.structure.split_inclusive(['a', 'c', 'n']) {
            let (length, class) = segment.split_once('!').unwrap();
            assert!(["a", "c", "n"].contains(&class), "{code}: {segment}");
            assert!(!length.starts_with('0'), "{code}: {segment}");
            total += length.parse::<usize>().unwrap();
        }
        // The prefix of four plus every segment.
        assert_eq!(total + 4, usize::from(country.length), "{code}");
    }
}

/// A letter typed for a digit can still pass mod 97, and the repayment would
/// go to an account that cannot exist.
#[test]
fn a_character_of_the_wrong_kind_for_its_position_is_rejected() {
    // Letters typed for digits: each passes mod 97.
    for text in [
        "SE4550000O00058398257466",
        "PT5000020123I234567890154",
        "IT60X054Z811101000000123456",
        "IT60X05428111O1000000123456",
        "DE4637O400440532013000",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidStructure)),
            "{text}"
        );
    }
    // Digits typed for letters: the British bank code and the Italian
    // check character are letters.
    for text in [
        "GB29 1WBK 6016 1331 9268 19",
        "IT60 0054 2811 1010 0000 0123 456",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidStructure)),
            "{text}"
        );
    }
    // The class of each position ignores letter case, since parsing
    // uppercases first.
    assert!(Iban::parse("gb29 nwbk 6016 1331 9268 19").is_ok());
}

/// Check digits of 00, 01, or 99 pass mod 97 for some numbers, but no bank
/// issues them, so such a number is a typo.
#[test]
fn check_digits_outside_02_to_98_are_rejected() {
    // Each passes mod 97.
    for text in [
        "DE00370400440532000052",
        "DE01370400440532000034",
        "DE99370400440532000016",
        "IQ01NBIQ850123456789012",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidChecksum)),
            "{text}"
        );
    }
    // The same numbers with the check digits a bank issues: 97, 98, and 02.
    for text in [
        "DE97370400440532000052",
        "DE98370400440532000034",
        "DE02370400440532000016",
    ] {
        assert!(Iban::parse(text).is_ok(), "{text}");
    }
    // The country and the length are checked first.
    assert_eq!(
        Iban::parse("AA00"),
        Err(ValueError::InvalidIban(IbanError::UnknownCountry))
    );
    assert_eq!(
        Iban::parse("DE00"),
        Err(ValueError::InvalidIban(IbanError::WrongLength {
            expected: 22,
            found: 4
        }))
    );
}

/// Numbers copied from a bank statement, with spaces and in any case, would
/// be refused.
#[test]
fn print_format_and_lowercase_are_accepted() {
    // The German example.
    let iban = Iban::parse("de89 3704 0044 0532 0130 00").unwrap();
    assert_eq!(iban.expose_secret(), GERMAN);
    // Every example.
    for example in registry::EXAMPLES {
        let example_iban = Iban::parse(&print_format(example)).unwrap();
        assert_eq!(example_iban.expose_secret(), *example);
    }
    // Other whitespace between the groups.
    assert_eq!(
        Iban::parse("DE89\t3704\n0044\u{a0}0532 0130 00").unwrap(),
        Iban::parse(GERMAN).unwrap()
    );
}

/// A mistyped digit would be saved and the repayment would go to another
/// account or bounce.
#[test]
fn every_single_digit_typo_is_caught() {
    // Every digit of every example, changed by one.
    for example in registry::EXAMPLES {
        for (index, digit) in example.bytes().enumerate() {
            if !digit.is_ascii_digit() {
                continue;
            }
            let mut typo = example.as_bytes().to_vec();
            typo[index] = b'0' + (digit - b'0' + 1) % 10;
            let typo = String::from_utf8(typo).unwrap();
            assert_eq!(
                Iban::parse(&typo),
                Err(ValueError::InvalidIban(IbanError::InvalidChecksum)),
                "{typo}"
            );
        }
    }
    // Two swapped digits.
    assert_eq!(
        Iban::parse(&GERMAN.replace("0532", "0523")),
        Err(ValueError::InvalidIban(IbanError::InvalidChecksum))
    );
}

/// A number from a country without IBANs would be accepted.
#[test]
fn an_unknown_country_is_rejected() {
    // Made-up codes and countries outside the registry.
    for text in [
        "XX89370400440532013000",
        "xx89370400440532013000",
        "US64SVBKUS6S3300958879",
        "AA00",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::UnknownCountry)),
            "{text}"
        );
    }
}

/// A truncated or padded number would be accepted, or the error would state
/// the wrong lengths.
#[test]
fn the_wrong_length_for_the_country_is_rejected() {
    // One character missing or extra.
    assert_eq!(
        Iban::parse("DE8937040044053201300"),
        Err(ValueError::InvalidIban(IbanError::WrongLength {
            expected: 22,
            found: 21
        }))
    );
    assert_eq!(
        Iban::parse(&format!("{GERMAN}0")),
        Err(ValueError::InvalidIban(IbanError::WrongLength {
            expected: 22,
            found: 23
        }))
    );
    // Only the prefix.
    assert_eq!(
        Iban::parse("DE89"),
        Err(ValueError::InvalidIban(IbanError::WrongLength {
            expected: 22,
            found: 4
        }))
    );
    // Spaces do not count.
    assert_eq!(
        Iban::parse("DE89 3704 0044 0532 0130 0"),
        Err(ValueError::InvalidIban(IbanError::WrongLength {
            expected: 22,
            found: 21
        }))
    );
}

/// Separators such as `-` or stray symbols would be dropped silently or
/// accepted into the number.
#[test]
fn characters_other_than_letters_digits_and_whitespace_are_rejected() {
    // Punctuation, symbols, and letters outside ASCII.
    for text in [
        "DE89-3704-0044-0532-0130-00",
        "DE89.3704.0044.0532.0130.00",
        "DE89370400440532013000€",
        "DÉ89370400440532013000",
        "DE89_370400440532013000",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidCharacter)),
            "{text}"
        );
    }
}

/// Text that is not an IBAN at all would be looked up as a country.
#[test]
fn a_malformed_prefix_is_rejected() {
    // Digits where the country goes, letters where the check digits go.
    for text in [
        "12AB370400440532013000",
        "DEAB370400440532013000",
        "D189370400440532013000",
        "DE8X370400440532013000",
    ] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidFormat)),
            "{text}"
        );
    }
    // Too short to have a prefix.
    for text in ["", "   ", "DE", "DE8"] {
        assert_eq!(
            Iban::parse(text),
            Err(ValueError::InvalidIban(IbanError::InvalidFormat)),
            "{text:?}"
        );
    }
}

/// Screens and logs would show more of the number than the first and last
/// four characters, or the wrong ones.
#[test]
fn country_code_and_masked_show_only_the_ends() {
    // The German example.
    let iban = Iban::parse(GERMAN).unwrap();
    assert_eq!(iban.country_code(), "DE");
    assert_eq!(iban.masked(), "DE89 •••• 3000");
    // The shortest and the longest numbers in the registry.
    assert_eq!(
        Iban::parse("NO9386011117947").unwrap().masked(),
        "NO93 •••• 7947"
    );
    assert_eq!(
        Iban::parse("RU0304452522540817810538091310419")
            .unwrap()
            .masked(),
        "RU03 •••• 0419"
    );
}

/// The full account number would leak into logs through `{:?}` or `{}`.
#[test]
fn debug_and_display_never_show_the_full_number() {
    // The German example.
    let iban = Iban::parse(GERMAN).unwrap();
    assert_eq!(format!("{iban:?}"), "Iban(\"DE89 •••• 3000\")");
    assert_eq!(iban.to_string(), "DE89 •••• 3000");
    // The hidden middle of every example stays hidden.
    for example in registry::EXAMPLES {
        let example_iban = Iban::parse(example).unwrap();
        let hidden: String = example.chars().skip(4).take(example.len() - 8).collect();
        assert!(!format!("{example_iban:?}").contains(&hidden), "{example}");
        assert!(!format!("{example_iban:#?}").contains(&hidden), "{example}");
        assert!(!example_iban.to_string().contains(&hidden), "{example}");
    }
}

/// Payment screens would show the number with spaces or in lowercase, which
/// some banking apps refuse to paste.
#[test]
fn expose_secret_returns_the_electronic_format() {
    // Uppercase, without spaces.
    let iban = Iban::parse(" gb29 nwbk 6016 1331 9268 19 ").unwrap();
    assert_eq!(iban.expose_secret(), "GB29NWBK60161331926819");
}

/// The same account typed in two formats would count as two accounts, or
/// two accounts as one.
#[test]
fn equality_compares_the_electronic_format() {
    // Same number, different formats.
    assert_eq!(
        Iban::parse("de89 3704 0044 0532 0130 00").unwrap(),
        Iban::parse(GERMAN).unwrap()
    );
    // Different numbers.
    assert_ne!(
        Iban::parse(GERMAN).unwrap(),
        Iban::parse("GB29NWBK60161331926819").unwrap()
    );
    // `FromStr` parses the same way.
    assert_eq!(GERMAN.parse::<Iban>(), Iban::parse(GERMAN));
}

/// Saved accounts would fail to load, or a mistyped number would load.
#[cfg(feature = "serde")]
#[test]
fn ibans_serialise_as_the_electronic_format() {
    // Round trip, normalised on the way in.
    let iban = Iban::parse(GERMAN).unwrap();
    assert_eq!(
        serde_json::to_string(&iban).unwrap(),
        format!("\"{GERMAN}\"")
    );
    assert_eq!(
        serde_json::from_str::<Iban>("\"de89 3704 0044 0532 0130 00\"").unwrap(),
        iban
    );
    // Loading validates like parsing.
    assert!(serde_json::from_str::<Iban>("\"DE89370400440532013001\"").is_err());
    assert!(serde_json::from_str::<Iban>("\"DE00370400440532000052\"").is_err());
}
