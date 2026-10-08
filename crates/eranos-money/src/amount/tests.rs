//! Every price, share, and balance is a `Money`. These tests keep typed
//! amounts exact to the minor unit, refuse anything ambiguous such as `1,50`
//! instead of guessing, keep the text form one that reads back as the same
//! amount, and keep arithmetic from mixing currencies or silently wrapping.

use super::*;

use proptest::prelude::*;

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(minor_units, CurrencyCode::EUR)
}

fn parse_euros(text: &str) -> Result<Money, MoneyError> {
    Money::parse(text, CurrencyCode::EUR)
}

/// Whole prices such as `12` would be stored off by a power of ten.
#[test]
fn whole_numbers_parse_to_minor_units() {
    // Two decimals.
    assert_eq!(parse_euros("12"), Ok(euros(1_200)));
    assert_eq!(parse_euros("0"), Ok(euros(0)));
    // No decimals.
    assert_eq!(
        Money::parse("1500", CurrencyCode::JPY),
        Ok(Money::from_minor_units(1_500, CurrencyCode::JPY))
    );
    // Three decimals.
    assert_eq!(
        Money::parse("7", CurrencyCode::BHD),
        Ok(Money::from_minor_units(7_000, CurrencyCode::BHD))
    );
}

/// A price typed as `12.3` would be stored as 12.03 or 1.23.
#[test]
fn short_fractions_are_padded_to_the_minor_unit() {
    // Two decimals.
    assert_eq!(parse_euros("12.3"), Ok(euros(1_230)));
    assert_eq!(parse_euros("12.34"), Ok(euros(1_234)));
    assert_eq!(parse_euros("0.05"), Ok(euros(5)));
    // Three decimals.
    assert_eq!(
        Money::parse("1.2", CurrencyCode::BHD),
        Ok(Money::from_minor_units(1_200, CurrencyCode::BHD))
    );
    assert_eq!(
        Money::parse("1.234", CurrencyCode::BHD),
        Ok(Money::from_minor_units(1_234, CurrencyCode::BHD))
    );
    // Four decimals.
    assert_eq!(
        Money::parse("1.2345", CurrencyCode::CLF),
        Ok(Money::from_minor_units(12_345, CurrencyCode::CLF))
    );
}

/// Refunds and corrections typed with a minus sign would be refused or
/// stored as positive amounts.
#[test]
fn negative_amounts_parse() {
    // Negative amounts.
    assert_eq!(parse_euros("-12.34"), Ok(euros(-1_234)));
    assert_eq!(parse_euros("-0.05"), Ok(euros(-5)));
    // Negative zero is zero.
    assert_eq!(parse_euros("-0"), Ok(euros(0)));
}

/// Amounts pasted with a stray space or newline would be refused.
#[test]
fn surrounding_whitespace_is_ignored() {
    // Spaces, tabs, and newlines.
    assert_eq!(parse_euros(" 12.34 "), Ok(euros(1_234)));
    assert_eq!(parse_euros("\t-12.34\n"), Ok(euros(-1_234)));
    // Leading zeros.
    assert_eq!(parse_euros("00012.34"), Ok(euros(1_234)));
}

/// Ambiguous input such as `1,50` would be guessed at instead of refused.
#[test]
fn malformed_amounts_are_rejected() {
    // Other separators and notations.
    for text in ["1,50", "1 000", "1e3", "0x10", "1.2.3", "12.3a", "abc"] {
        assert_eq!(parse_euros(text), Err(MoneyError::InvalidAmount), "{text}");
    }
    // Missing digits on either side of the point.
    for text in [".5", "5.", ".", "-.5", "-5."] {
        assert_eq!(parse_euros(text), Err(MoneyError::InvalidAmount), "{text}");
    }
    // Signs other than one leading minus.
    for text in ["+5", "--1", "-", "- 1", "1-", "−1"] {
        assert_eq!(parse_euros(text), Err(MoneyError::InvalidAmount), "{text}");
    }
    // Digits outside ASCII.
    assert_eq!(parse_euros("١٢"), Err(MoneyError::InvalidAmount));
    // Nothing at all.
    for text in ["", "   "] {
        assert_eq!(
            parse_euros(text),
            Err(MoneyError::InvalidAmount),
            "{text:?}"
        );
    }
}

/// A price with more decimals than the currency has would be rounded
/// silently.
#[test]
fn extra_decimals_are_rejected_with_the_allowed_count() {
    // Two decimals.
    assert_eq!(
        parse_euros("12.345"),
        Err(MoneyError::TooManyDecimals {
            allowed: 2,
            currency: CurrencyCode::EUR
        })
    );
    // Trailing zeros count.
    assert_eq!(
        parse_euros("12.340"),
        Err(MoneyError::TooManyDecimals {
            allowed: 2,
            currency: CurrencyCode::EUR
        })
    );
    // No decimals: any fraction is too many.
    for text in ["1.0", "1.5"] {
        assert_eq!(
            Money::parse(text, CurrencyCode::JPY),
            Err(MoneyError::TooManyDecimals {
                allowed: 0,
                currency: CurrencyCode::JPY
            })
        );
    }
    // Three and four decimals.
    assert_eq!(
        Money::parse("1.2345", CurrencyCode::BHD),
        Err(MoneyError::TooManyDecimals {
            allowed: 3,
            currency: CurrencyCode::BHD
        })
    );
    assert_eq!(
        Money::parse("1.23456", CurrencyCode::CLF),
        Err(MoneyError::TooManyDecimals {
            allowed: 4,
            currency: CurrencyCode::CLF
        })
    );
}

/// An amount too large for 64-bit minor units would wrap around to a wrong
/// value.
#[test]
fn amounts_beyond_the_i64_range_overflow() {
    // The largest amounts that fit.
    assert_eq!(parse_euros("92233720368547758.07"), Ok(euros(i64::MAX)));
    assert_eq!(parse_euros("-92233720368547758.07"), Ok(euros(-i64::MAX)));
    assert_eq!(
        Money::parse("9223372036854775807", CurrencyCode::JPY),
        Ok(Money::from_minor_units(i64::MAX, CurrencyCode::JPY))
    );
    // One minor unit more.
    assert_eq!(
        parse_euros("92233720368547758.08"),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        Money::parse("9223372036854775808", CurrencyCode::JPY),
        Err(MoneyError::Overflow)
    );
    // Far beyond.
    assert_eq!(
        parse_euros("99999999999999999999"),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        parse_euros("-99999999999999999999.99"),
        Err(MoneyError::Overflow)
    );
    // Beyond even the 128-bit accumulator.
    assert_eq!(
        parse_euros("1000000000000000000000000000000000000000"),
        Err(MoneyError::Overflow)
    );
}

/// Amounts would be shown or saved without their decimals or currency.
#[test]
fn display_shows_the_decimal_amount_and_code() {
    // Two decimals.
    assert_eq!(euros(1_234).to_string(), "12.34 EUR");
    assert_eq!(euros(-5).to_string(), "-0.05 EUR");
    assert_eq!(euros(0).to_string(), "0.00 EUR");
    assert_eq!(euros(10_000).to_string(), "100.00 EUR");
    // No decimals.
    assert_eq!(
        Money::from_minor_units(1_500, CurrencyCode::JPY).to_string(),
        "1500 JPY"
    );
    assert_eq!(
        Money::from_minor_units(-1_500, CurrencyCode::JPY).to_string(),
        "-1500 JPY"
    );
    // Three and four decimals.
    assert_eq!(
        Money::from_minor_units(1_234, CurrencyCode::BHD).to_string(),
        "1.234 BHD"
    );
    assert_eq!(
        Money::from_minor_units(1, CurrencyCode::BHD).to_string(),
        "0.001 BHD"
    );
    assert_eq!(
        Money::from_minor_units(12_345, CurrencyCode::CLF).to_string(),
        "1.2345 CLF"
    );
}

/// Showing the smallest or largest amount would crash the app.
#[test]
fn display_handles_the_extremes() {
    // Two decimals.
    assert_eq!(euros(i64::MIN).to_string(), "-92233720368547758.08 EUR");
    assert_eq!(euros(i64::MAX).to_string(), "92233720368547758.07 EUR");
    // No decimals.
    assert_eq!(
        Money::from_minor_units(i64::MIN, CurrencyCode::JPY).to_string(),
        "-9223372036854775808 JPY"
    );
}

/// The text of an amount, beside its minor units and decimal places.
struct Formatted {
    minor_units: i64,
    decimal_places: u8,
    text: &'static str,
}

/// An amount of a currency with more decimal places than ISO 4217 lists
/// today would overflow while it is written, or be written with the point
/// in another place.
#[test]
fn minor_units_format_at_any_number_of_decimal_places() {
    for Formatted {
        minor_units,
        decimal_places,
        text,
    } in [
        Formatted {
            minor_units: 0,
            decimal_places: 0,
            text: "0",
        },
        Formatted {
            minor_units: 0,
            decimal_places: 2,
            text: "0.00",
        },
        Formatted {
            minor_units: -5,
            decimal_places: 3,
            text: "-0.005",
        },
        Formatted {
            minor_units: 12_345,
            decimal_places: 4,
            text: "1.2345",
        },
        Formatted {
            minor_units: i64::MAX,
            decimal_places: 18,
            text: "9.223372036854775807",
        },
        // From 19 places, `10^places` no longer fits an `i64`.
        Formatted {
            minor_units: i64::MIN,
            decimal_places: 19,
            text: "-0.9223372036854775808",
        },
        Formatted {
            minor_units: i64::MAX,
            decimal_places: 20,
            text: "0.09223372036854775807",
        },
    ] {
        assert_eq!(
            format_minor_units(minor_units, decimal_places).to_string(),
            text,
            "{minor_units} at {decimal_places} places"
        );
    }
    let most = format_minor_units(1, u8::MAX).to_string();
    assert_eq!(most, format!("0.{}1", "0".repeat(254)));

    // At every number of places, the digits are the amount's, the point
    // leaves that many after it, and one digit stands before it.
    for decimal_places in 0..=u8::MAX {
        for minor_units in [0, 1, -1, 1_234, i64::MIN, i64::MAX] {
            let text = format_minor_units(minor_units, decimal_places).to_string();
            let unsigned = text.strip_prefix('-').unwrap_or(&text);
            assert_eq!(text.starts_with('-'), 0 > minor_units, "{text}");
            let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
            assert_eq!(fraction.len(), usize::from(decimal_places), "{text}");
            assert!(1 == whole.len() || !whole.starts_with('0'), "{text}");
            let digits = format!("{whole}{fraction}");
            assert_eq!(
                digits.parse::<u128>(),
                Ok(u128::from(minor_units.unsigned_abs())),
                "{text}"
            );
        }
    }
}

/// The text form of an amount and the formatter the bindings write it with
/// would drift apart.
#[test]
fn display_writes_the_amount_as_the_formatter_does() {
    for &currency in CurrencyCode::all() {
        for minor_units in [0, 1, -1, 1_234, -1_234, i64::MIN, i64::MAX] {
            let amount = format_minor_units(minor_units, currency.decimal_places());
            assert_eq!(
                Money::from_minor_units(minor_units, currency).to_string(),
                format!("{amount} {}", currency.code())
            );
        }
    }
}

/// Balances would be classified as owed, owing, or settled wrongly.
#[test]
fn sign_checks_follow_the_minor_units() {
    // Zero.
    let zero = Money::zero(CurrencyCode::EUR);
    assert_eq!(zero, euros(0));
    assert!(zero.is_zero() && !zero.is_positive() && !zero.is_negative());
    // One minor unit either side of zero.
    assert!(euros(1).is_positive() && !euros(1).is_zero() && !euros(1).is_negative());
    assert!(euros(-1).is_negative() && !euros(-1).is_zero() && !euros(-1).is_positive());
}

/// Amounts would lose their value or currency when read back.
#[test]
fn accessors_return_the_parts() {
    // Minor units and currency.
    let amount = Money::from_minor_units(1_234, CurrencyCode::RON);
    assert_eq!(amount.minor_units(), 1_234);
    assert_eq!(amount.currency(), CurrencyCode::RON);
    // The same minor units in another currency is another amount.
    assert_ne!(amount, Money::from_minor_units(1_234, CurrencyCode::EUR));
}

/// Totals and balances would be computed wrongly.
#[test]
fn checked_arithmetic_adds_and_subtracts_same_currency_amounts() {
    // Addition.
    assert_eq!(euros(1_234).checked_add(euros(166)), Ok(euros(1_400)));
    assert_eq!(euros(100).checked_add(euros(-300)), Ok(euros(-200)));
    // Subtraction.
    assert_eq!(euros(1_400).checked_sub(euros(166)), Ok(euros(1_234)));
    assert_eq!(euros(100).checked_sub(euros(300)), Ok(euros(-200)));
    // The limits themselves are reachable.
    assert_eq!(
        euros(i64::MAX - 1).checked_add(euros(1)),
        Ok(euros(i64::MAX))
    );
    assert_eq!(
        euros(i64::MIN + 1).checked_sub(euros(1)),
        Ok(euros(i64::MIN))
    );
}

/// Euros and dollars would be added as if they were the same currency.
#[test]
fn mixed_currencies_are_rejected() {
    let dollars = Money::from_minor_units(100, CurrencyCode::USD);
    // Addition and subtraction name both currencies in order.
    assert_eq!(
        euros(100).checked_add(dollars),
        Err(MoneyError::CurrencyMismatch {
            left: CurrencyCode::EUR,
            right: CurrencyCode::USD
        })
    );
    assert_eq!(
        dollars.checked_sub(euros(100)),
        Err(MoneyError::CurrencyMismatch {
            left: CurrencyCode::USD,
            right: CurrencyCode::EUR
        })
    );
    // The currencies are checked before the range.
    assert_eq!(
        euros(i64::MAX).checked_add(dollars),
        Err(MoneyError::CurrencyMismatch {
            left: CurrencyCode::EUR,
            right: CurrencyCode::USD
        })
    );
}

/// A total past the 64-bit range would wrap around to a wrong balance.
#[test]
fn results_out_of_range_overflow() {
    // Addition.
    assert_eq!(
        euros(i64::MAX).checked_add(euros(1)),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        euros(i64::MIN).checked_add(euros(-1)),
        Err(MoneyError::Overflow)
    );
    // Subtraction.
    assert_eq!(
        euros(i64::MIN).checked_sub(euros(1)),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        euros(0).checked_sub(euros(i64::MIN)),
        Err(MoneyError::Overflow)
    );
}

/// Saved amounts would fail to load or load with another value.
#[cfg(feature = "serde")]
#[test]
fn amounts_serialise_as_minor_units_and_currency() {
    // Round trip.
    let amount = euros(1_234);
    let json = serde_json::to_string(&amount).unwrap();
    assert_eq!(json, r#"{"minor_units":1234,"currency":"EUR"}"#);
    assert_eq!(serde_json::from_str::<Money>(&json).unwrap(), amount);
    // An unknown currency.
    assert!(serde_json::from_str::<Money>(r#"{"minor_units":1,"currency":"XYZ"}"#).is_err());
}

proptest! {
    /// An amount saved or shown as text would read back as another amount.
    #[test]
    fn display_then_parse_round_trips(
        minor_units in any::<i64>(),
        index in 0..CurrencyCode::all().len(),
    ) {
        let amount = Money::from_minor_units(minor_units, CurrencyCode::all()[index]);
        let text = amount.to_string();
        let (number, code) = text.rsplit_once(' ').unwrap();
        prop_assert_eq!(code, amount.currency().code());
        prop_assert_eq!(Money::parse(number, amount.currency()), Ok(amount));
    }
}

/// The smallest amount could be shown and stored but never typed back in.
#[test]
fn the_smallest_amount_parses() {
    // Two decimals and no decimals.
    assert_eq!(parse_euros("-92233720368547758.08"), Ok(euros(i64::MIN)));
    assert_eq!(
        Money::parse("-9223372036854775808", CurrencyCode::JPY),
        Ok(Money::from_minor_units(i64::MIN, CurrencyCode::JPY))
    );
    // One unit further is out of range on both sides.
    assert_eq!(
        parse_euros("-92233720368547758.09"),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        parse_euros("92233720368547758.08"),
        Err(MoneyError::Overflow)
    );
}
