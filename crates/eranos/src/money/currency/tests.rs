//! Every amount carries a currency from the ISO 4217 table. These tests keep
//! the table sorted and unique, which the binary search in `from_code`
//! depends on, keep each currency's decimal places right, since they scale
//! every amount entered, shown, and converted, and keep lookups tolerant of
//! case and whitespace but strict about codes that are not currencies.

use super::*;

/// The number of active ISO 4217 codes with a minor unit.
const CURRENCY_COUNT: usize = 165;

/// Lookups would miss currencies, because the binary search needs sorted,
/// unique codes.
#[test]
fn the_table_is_sorted_and_unique() {
    // Strictly increasing codes.
    for pair in TABLE.windows(2) {
        assert!(pair[0].0 < pair[1].0, "{} before {}", pair[0].0, pair[1].0);
    }
    // One entry per currency.
    assert_eq!(TABLE.len(), CURRENCY_COUNT);
}

/// A malformed code in the table could never be looked up or would print
/// wrongly.
#[test]
fn every_code_is_three_uppercase_ascii_letters() {
    // Shape of every code.
    for (code, _) in TABLE {
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
            Err(ValueError::UnknownCurrency),
            "{code}"
        );
    }
    // Three bytes that are not three letters.
    for code in ["€", "E1R", "E R", "12A"] {
        assert_eq!(
            Currency::from_code(code),
            Err(ValueError::UnknownCurrency),
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
            Err(ValueError::UnknownCurrency),
            "{code:?}"
        );
    }
    // Too long.
    for code in ["EURO", "EUR EUR", "E UR", "ＥＵＲ"] {
        assert_eq!(
            Currency::from_code(code),
            Err(ValueError::UnknownCurrency),
            "{code:?}"
        );
    }
}

/// Amounts would be entered, shown, or converted with the wrong number of
/// decimals.
#[test]
fn decimal_places_follow_iso_4217() {
    // Two decimals.
    for currency in [Currency::EUR, Currency::USD, Currency::RON, Currency::HUF] {
        assert_eq!(currency.decimal_places(), 2, "{currency}");
    }
    // No decimals.
    for currency in [Currency::JPY, Currency::KRW, Currency::ISK] {
        assert_eq!(currency.decimal_places(), 0, "{currency}");
    }
    // Three decimals.
    for currency in [Currency::BHD, Currency::KWD, Currency::TND] {
        assert_eq!(currency.decimal_places(), 3, "{currency}");
    }
    // Four decimals.
    for currency in [Currency::CLF, Currency::UYW] {
        assert_eq!(currency.decimal_places(), 4, "{currency}");
    }
}

/// Currency pickers built from `all` would miss currencies or list them out
/// of order.
#[test]
fn all_lists_every_currency_in_code_order() {
    let every_currency = Currency::all();
    // Every table entry, in table order, and the lookup of its code.
    assert_eq!(every_currency.len(), CURRENCY_COUNT);
    for (currency, (code, _)) in every_currency.iter().zip(TABLE) {
        assert_eq!(currency.code(), *code);
        assert_eq!(Currency::from_code(code), Ok(*currency));
    }
    // Ordering is by code.
    assert!(every_currency.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(Currency::AED < Currency::EUR && Currency::EUR < Currency::ZWG);
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
    assert_eq!("XYZ".parse::<Currency>(), Err(ValueError::UnknownCurrency));
}

/// Saved amounts would load in the wrong currency, or an unknown one.
#[cfg(feature = "serde")]
#[test]
fn currencies_serialize_as_their_code() {
    // Round trip.
    assert_eq!(serde_json::to_string(&Currency::EUR).unwrap(), "\"EUR\"");
    assert_eq!(
        serde_json::from_str::<Currency>("\"EUR\"").unwrap(),
        Currency::EUR
    );
    // Deserializing looks the code up like parsing.
    assert_eq!(
        serde_json::from_str::<Currency>("\"ron\"").unwrap(),
        Currency::RON
    );
    assert!(serde_json::from_str::<Currency>("\"XYZ\"").is_err());
    // Only strings are codes.
    assert!(serde_json::from_str::<Currency>("978").is_err());
}
