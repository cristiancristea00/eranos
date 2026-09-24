//! Every amount carries a currency from ISO 4217, whose data comes from
//! `iso4217-static`. These tests keep the constants equal to the crate's
//! codes with a minor unit and in code order, keep each currency's decimal
//! places right, since they scale every amount entered, shown, and
//! converted, and keep lookups tolerant of case and whitespace but strict
//! about codes that are not currencies.

use std::collections::BTreeSet;

use super::*;

/// The number of active ISO 4217 codes with a minor unit.
const CURRENCY_COUNT: usize = 165;

/// A code a pin bump adds would be accepted by `from_code` but missing from
/// `all` and the constants.
#[test]
fn all_matches_every_iso4217_static_code_with_a_minor_unit() {
    let listed: BTreeSet<&str> = Currency::all()
        .iter()
        .map(|currency| currency.code())
        .collect();
    let in_crate: BTreeSet<&str> = (0..=999_u16)
        .filter_map(|number| IsoCurrency::from_u16(number).ok())
        .filter(|iso| iso.minor_unit().is_some())
        .map(|iso| iso.as_str())
        .collect();
    // Codes the crate has and the list lacks, named for iso4217.rs.
    let missing: Vec<&&str> = in_crate.difference(&listed).collect();
    assert!(missing.is_empty(), "add to iso4217.rs: {missing:?}");
    assert_eq!(listed, in_crate);
}

/// Precious metals, drawing rights, and testing codes would become
/// currencies with made-up decimal places, or text that is not an exact code
/// would be looked up.
#[test]
fn lookup_finds_only_exact_codes_with_a_minor_unit() {
    // An exact code with a minor unit.
    assert_eq!(Currency::lookup("EUR"), Some(Currency::EUR));
    // Codes without a minor unit.
    for code in ["XAU", "XDR", "XTS", "XXX"] {
        assert_eq!(Currency::lookup(code), None, "{code}");
    }
    // Text that is not an exact code; `from_code` trims and uppercases first.
    for code in ["XYZ", "eur", " EUR", "EURO", "€"] {
        assert_eq!(Currency::lookup(code), None, "{code:?}");
    }
}

/// A malformed code could never be looked up or would print wrongly.
#[test]
fn every_code_is_three_uppercase_ascii_letters() {
    // Shape of every code.
    for currency in Currency::all() {
        let code = currency.code();
        assert_eq!(code.len(), 3, "{code}");
        assert!(code.bytes().all(|byte| byte.is_ascii_uppercase()), "{code}");
    }
}

/// Amounts in a currency with an unusual minor unit could not be scaled:
/// conversions multiply by 10^4 at most without an overflow check.
#[test]
fn every_currency_has_at_most_4_decimal_places() {
    // Decimal places of every currency.
    for &currency in Currency::all() {
        assert!(currency.decimal_places() <= 4, "{currency}");
    }
}

/// Codes typed in lowercase or with stray spaces would be refused.
#[test]
fn lookup_ignores_case_and_surrounding_whitespace() {
    // Letter case.
    for code in ["EUR", "eur", "Eur", "eUr"] {
        assert_eq!(Currency::from_code(code), Ok(Currency::EUR), "{code}");
    }
    // Surrounding whitespace.
    assert_eq!(Currency::from_code(" RON "), Ok(Currency::RON));
    assert_eq!(Currency::from_code("\tusd\n"), Ok(Currency::USD));
    // The first and the last code of the table.
    assert_eq!(Currency::from_code("aed"), Ok(Currency::AED));
    assert_eq!(Currency::from_code("zwg"), Ok(Currency::ZWG));
}

/// An amount would be recorded in a currency that does not exist.
#[test]
fn unknown_codes_are_rejected() {
    // Three letters that are not a currency with a minor unit: made up,
    // precious metals, special drawing rights, and testing codes.
    for code in ["XYZ", "ABC", "XAU", "XDR", "XTS", "XXX"] {
        assert_eq!(
            Currency::from_code(code),
            Err(MoneyError::UnknownCurrency),
            "{code}"
        );
    }
    // Three bytes that are not three letters.
    for code in ["€", "E1R", "E R", "12A"] {
        assert_eq!(
            Currency::from_code(code),
            Err(MoneyError::UnknownCurrency),
            "{code}"
        );
    }
}

/// A truncated or padded code would be read as some currency.
#[test]
fn short_and_long_codes_are_rejected() {
    // Too short.
    for code in ["", "   ", "E", "EU"] {
        assert_eq!(
            Currency::from_code(code),
            Err(MoneyError::UnknownCurrency),
            "{code:?}"
        );
    }
    // Too long.
    for code in ["EURO", "EUR EUR", "E UR", "ＥＵＲ"] {
        assert_eq!(
            Currency::from_code(code),
            Err(MoneyError::UnknownCurrency),
            "{code:?}"
        );
    }
}

/// Amounts would be entered, shown, or converted with the wrong number of
/// decimals, or a pin bump would change a minor unit, and so the meaning of
/// stored amounts, unnoticed.
#[test]
fn decimal_places_follow_iso_4217() {
    let none = [
        "BIF", "CLP", "DJF", "GNF", "ISK", "JPY", "KMF", "KRW", "PYG", "RWF", "UGX", "UYI", "VND",
        "VUV", "XAF", "XOF", "XPF",
    ];
    let three = ["BHD", "IQD", "JOD", "KWD", "LYD", "OMR", "TND"];
    let four = ["CLF", "UYW"];
    // No, three, and four decimals.
    for (codes, places) in [(&none[..], 0), (&three[..], 3), (&four[..], 4)] {
        for code in codes {
            assert_eq!(
                Currency::from_code(code).map(Currency::decimal_places),
                Ok(places),
                "{code}"
            );
        }
    }
    // Every other currency has two.
    let others: Vec<&Currency> = Currency::all()
        .iter()
        .filter(|currency| {
            let code = currency.code();
            !none.contains(&code) && !three.contains(&code) && !four.contains(&code)
        })
        .collect();
    for currency in &others {
        assert_eq!(currency.decimal_places(), 2, "{currency}");
    }
    assert_eq!(
        others.len(),
        CURRENCY_COUNT - none.len() - three.len() - four.len()
    );
}

/// Currency pickers built from `all` would miss currencies or list them out
/// of order.
#[test]
fn all_lists_every_currency_in_code_order() {
    let every_currency = Currency::all();
    // One entry per currency, strictly increasing by code and by ordering.
    assert_eq!(every_currency.len(), CURRENCY_COUNT);
    assert!(
        every_currency
            .windows(2)
            .all(|pair| pair[0].code() < pair[1].code())
    );
    assert!(every_currency.windows(2).all(|pair| pair[0] < pair[1]));
    // Ordering is by code: 978 and 840 in numeric order would put USD first.
    assert!(Currency::AED < Currency::EUR && Currency::EUR < Currency::ZWG);
    assert!(Currency::EUR < Currency::USD);
    assert_eq!(Currency::EUR.cmp(&Currency::EUR), Ordering::Equal);
    // Every code looks up its own currency.
    for &currency in every_currency {
        assert_eq!(Currency::from_code(currency.code()), Ok(currency));
    }
}

/// Albanian lek amounts would be refused, since `ALL` reads like "every
/// currency" rather than a code.
#[test]
fn the_constant_all_is_the_albanian_lek() {
    // The constant and the lookup agree.
    assert_eq!(Currency::ALL.code(), "ALL");
    assert_eq!(Currency::ALL.decimal_places(), 2);
    assert_eq!(Currency::from_code("all"), Ok(Currency::ALL));
    // `all()` lists the lek once, among the other currencies.
    let leks = Currency::all()
        .iter()
        .filter(|&&currency| currency == Currency::ALL)
        .count();
    assert_eq!(leks, 1);
}

/// Logs and saved amounts would print an index instead of the code.
#[test]
fn debug_and_display_print_the_code() {
    // Display.
    assert_eq!(Currency::EUR.to_string(), "EUR");
    // Debug.
    assert_eq!(format!("{:?}", Currency::JPY), "JPY");
    assert_eq!(format!("{:?}", Some(Currency::BHD)), "Some(BHD)");
}

/// Currency pickers would show a name other than ISO 4217's, or with the
/// space the SIX list has after "Comorian Franc".
#[test]
fn names_are_the_trimmed_iso_4217_names() {
    // A few names.
    for (currency, name) in [
        (Currency::EUR, "Euro"),
        (Currency::GBP, "Pound Sterling"),
        (Currency::USD, "US Dollar"),
        (Currency::KMF, "Comorian Franc"),
        (Currency::TOP, "Pa’anga"),
    ] {
        assert_eq!(currency.name(), name, "{currency}");
    }
    // Two codes can share a name.
    assert_eq!(Currency::VED.name(), Currency::VES.name());
    // Every name is present and trimmed.
    for currency in Currency::all() {
        let name = currency.name();
        assert!(!name.is_empty(), "{currency}");
        assert_eq!(name, name.trim(), "{currency}");
    }
}

/// Exports and bank formats that name currencies by number would name the
/// wrong one.
#[test]
fn numeric_codes_are_the_iso_4217_numbers() {
    // A few numbers, including one ISO writes with leading zeros.
    for (currency, number) in [
        (Currency::EUR, 978),
        (Currency::USD, 840),
        (Currency::ALL, 8),
        (Currency::JPY, 392),
    ] {
        assert_eq!(currency.numeric_code(), number, "{currency}");
    }
    // Every number has at most three digits and leads back to its code.
    for &currency in Currency::all() {
        let number = currency.numeric_code();
        assert!((1..=999).contains(&number), "{currency}");
        assert_eq!(
            IsoCurrency::from_u16(number).map(|iso| iso.as_str()),
            Ok(currency.code()),
            "{currency}"
        );
    }
}

/// Amounts would be scaled by the wrong power of ten.
#[test]
fn minor_units_per_major_is_ten_to_the_decimal_places() {
    // One per decimal-place count.
    assert_eq!(Currency::JPY.minor_units_per_major(), 1);
    assert_eq!(Currency::EUR.minor_units_per_major(), 100);
    assert_eq!(Currency::BHD.minor_units_per_major(), 1_000);
    assert_eq!(Currency::CLF.minor_units_per_major(), 10_000);
    // Every currency.
    for &currency in Currency::all() {
        let expected = 10_i64.pow(u32::from(currency.decimal_places()));
        assert_eq!(currency.minor_units_per_major(), expected, "{currency}");
    }
}

/// Codes parsed through `str::parse` would behave differently from
/// `from_code`.
#[test]
fn from_str_parses_like_from_code() {
    // Known and unknown codes.
    assert_eq!(" chf ".parse::<Currency>(), Ok(Currency::CHF));
    assert_eq!("XYZ".parse::<Currency>(), Err(MoneyError::UnknownCurrency));
}

/// Saved amounts would load in the wrong currency, or an unknown one.
#[cfg(feature = "serde")]
#[test]
fn currencies_serialise_as_their_code() {
    // Round trip.
    assert_eq!(serde_json::to_string(&Currency::EUR).unwrap(), "\"EUR\"");
    assert_eq!(
        serde_json::from_str::<Currency>("\"EUR\"").unwrap(),
        Currency::EUR
    );
    // Deserialising looks the code up like parsing.
    assert_eq!(
        serde_json::from_str::<Currency>("\"ron\"").unwrap(),
        Currency::RON
    );
    assert!(serde_json::from_str::<Currency>("\"XYZ\"").is_err());
    // Only strings are codes.
    assert!(serde_json::from_str::<Currency>("978").is_err());
}
