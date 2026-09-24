//! Foreign-currency expenses are converted into the group currency with the
//! rate typed when they were entered. These tests keep rates exact decimals
//! that compare equal however they were typed, keep conversions rounded half
//! away from zero to the target's minor unit as EU Regulation 1103/97
//! requires, and keep out-of-range conversions from wrapping.

use super::*;

use proptest::prelude::*;

fn rate(text: &str) -> ExchangeRate {
    ExchangeRate::parse(text).unwrap()
}

fn euros(minor_units: i64) -> Money {
    Money::from_minor_units(Currency::EUR, minor_units)
}

/// Rates typed from a bank statement would be refused or misread.
#[test]
fn parse_reads_positive_decimals() {
    // Decimal rates.
    let typed = rate("4.9765");
    assert_eq!((typed.mantissa(), typed.scale()), (49_765, 4));
    let rate_below_one = rate("0.2011");
    assert_eq!(
        (rate_below_one.mantissa(), rate_below_one.scale()),
        (2_011, 4)
    );
    // Whole rates.
    let whole_rate = rate("12");
    assert_eq!((whole_rate.mantissa(), whole_rate.scale()), (12, 0));
    // Surrounding whitespace.
    assert_eq!(ExchangeRate::parse(" 4.9765\n"), Ok(rate("4.9765")));
    // `FromStr` parses the same way.
    assert_eq!("4.9765".parse::<ExchangeRate>(), Ok(rate("4.9765")));
}

/// A rate of 1 typed by hand would differ from the rate of a local cost, and
/// a conversion at par would not match it.
#[test]
fn one_is_the_rate_typed_as_1() {
    assert_eq!(ExchangeRate::ONE, rate("1"));
    assert_eq!(ExchangeRate::ONE, rate("1.000"));
    assert_eq!(ExchangeRate::ONE.to_string(), "1");
}

/// The same rate typed with and without trailing zeros would count as two
/// different rates.
#[test]
fn trailing_zeros_do_not_change_a_rate() {
    // Trailing fraction zeros.
    assert_eq!(rate("4.9700"), rate("4.97"));
    assert_eq!(
        (rate("4.9700").mantissa(), rate("4.9700").scale()),
        (497, 2)
    );
    // A whole number with and without a fraction.
    assert_eq!(rate("1"), rate("1.0"));
    assert_eq!(rate("1"), rate("1.000"));
    assert_eq!(
        (rate("100.00").mantissa(), rate("100.00").scale()),
        (100, 0)
    );
    // Zeros of the whole part stay.
    assert_eq!((rate("10").mantissa(), rate("10").scale()), (10, 0));
    assert_ne!(rate("10"), rate("1"));
}

/// A zero, negative, or ambiguous rate would be stored and convert every
/// amount wrongly.
#[test]
fn malformed_rates_are_rejected() {
    // Zero.
    for text in ["0", "0.000", "0.0", "000"] {
        assert_eq!(
            ExchangeRate::parse(text),
            Err(MoneyError::InvalidExchangeRate),
            "{text}"
        );
    }
    // Signs, separators, and notations.
    for text in ["-1", "+1", "1,5", "1 5", "1e3", "1.2.3", "abc", "4.97a"] {
        assert_eq!(
            ExchangeRate::parse(text),
            Err(MoneyError::InvalidExchangeRate),
            "{text}"
        );
    }
    // Missing digits on either side of the point.
    for text in [".5", "5.", ".", "", "  "] {
        assert_eq!(
            ExchangeRate::parse(text),
            Err(MoneyError::InvalidExchangeRate),
            "{text:?}"
        );
    }
}

/// A rate with more precision than the library keeps would be rounded
/// silently.
#[test]
fn rates_with_more_than_18_decimals_are_rejected() {
    // Exactly 18 decimals.
    let smallest = rate("0.000000000000000001");
    assert_eq!((smallest.mantissa(), smallest.scale()), (1, 18));
    // 19 decimals, even when they are zeros.
    for text in ["0.0000000000000000001", "1.0000000000000000000"] {
        assert_eq!(
            ExchangeRate::parse(text),
            Err(MoneyError::InvalidExchangeRate),
            "{text}"
        );
    }
    // More decimals than a scale can count.
    let long = format!("1.{}", "0".repeat(256));
    assert_eq!(
        ExchangeRate::parse(&long),
        Err(MoneyError::InvalidExchangeRate)
    );
}

/// A rate with more digits than 64 bits hold would wrap around to a wrong
/// rate.
#[test]
fn rates_with_more_digits_than_fit_are_rejected() {
    // The largest mantissa, whole and with 18 decimals.
    assert_eq!(rate("18446744073709551615").mantissa(), u64::MAX);
    assert_eq!(rate("18.446744073709551615").mantissa(), u64::MAX);
    // One more, and a digit more.
    for text in [
        "18446744073709551616",
        "1844674407370955161.6",
        "123456789012345678901",
    ] {
        assert_eq!(
            ExchangeRate::parse(text),
            Err(MoneyError::InvalidExchangeRate),
            "{text}"
        );
    }
}

/// Rates built from stored digits would accept a zero rate or compare
/// unequal to the same rate typed as text.
#[test]
fn from_decimal_validates_and_normalises() {
    // A zero mantissa.
    assert_eq!(
        ExchangeRate::from_decimal(0, 0),
        Err(MoneyError::InvalidExchangeRate)
    );
    assert_eq!(
        ExchangeRate::from_decimal(0, 4),
        Err(MoneyError::InvalidExchangeRate)
    );
    // The scale limit.
    assert!(ExchangeRate::from_decimal(1, 18).is_ok());
    assert_eq!(
        ExchangeRate::from_decimal(1, 19),
        Err(MoneyError::InvalidExchangeRate)
    );
    assert_eq!(
        ExchangeRate::from_decimal(10, 19),
        Err(MoneyError::InvalidExchangeRate)
    );
    // Normalisation.
    assert_eq!(ExchangeRate::from_decimal(49_765, 4), Ok(rate("4.9765")));
    assert_eq!(ExchangeRate::from_decimal(49_700, 4), Ok(rate("4.97")));
    assert_eq!(ExchangeRate::from_decimal(1_000, 3), Ok(rate("1")));
    assert_eq!(ExchangeRate::from_decimal(10, 0), Ok(rate("10")));
}

/// Saved or shown rates would lose digits or gain a wrong decimal point.
#[test]
fn display_writes_the_exact_decimal() {
    // Fractions, padded after the point.
    assert_eq!(rate("4.9765").to_string(), "4.9765");
    assert_eq!(rate("0.0001").to_string(), "0.0001");
    assert_eq!(
        rate("0.000000000000000001").to_string(),
        "0.000000000000000001"
    );
    assert_eq!(
        ExchangeRate::from_decimal(u64::MAX, 18)
            .unwrap()
            .to_string(),
        "18.446744073709551615"
    );
    // Whole numbers, without a point.
    assert_eq!(rate("12").to_string(), "12");
    assert_eq!(rate("10.000").to_string(), "10");
}

/// A converted expense would be off by a cent.
#[test]
fn convert_rounds_to_the_nearest_minor_unit() {
    // 10.00 EUR at 4.9765 is 49.765 RON, rounded up.
    let ten_euros = Money::parse("10.00", Currency::EUR).unwrap();
    assert_eq!(
        rate("4.9765").convert(ten_euros, Currency::RON),
        Ok(Money::parse("49.77", Currency::RON).unwrap())
    );
    // A refund rounds the same way, away from zero.
    assert_eq!(
        rate("4.9765").convert(euros(-1_000), Currency::RON),
        Ok(Money::parse("-49.77", Currency::RON).unwrap())
    );
    // Zero stays zero.
    assert_eq!(
        rate("4.9765").convert(euros(0), Currency::RON),
        Ok(Money::zero(Currency::RON))
    );
}

/// Conversions would round halves towards zero or towards even, which the EU
/// rules for currency conversion forbid.
#[test]
fn exact_halves_round_away_from_zero() {
    // Halves.
    let half = rate("0.5");
    assert_eq!(half.convert(euros(1), Currency::EUR), Ok(euros(1)));
    assert_eq!(half.convert(euros(-1), Currency::EUR), Ok(euros(-1)));
    assert_eq!(half.convert(euros(3), Currency::EUR), Ok(euros(2)));
    assert_eq!(half.convert(euros(-3), Currency::EUR), Ok(euros(-2)));
    // Just below and just above a half.
    assert_eq!(
        rate("0.4999").convert(euros(1), Currency::EUR),
        Ok(euros(0))
    );
    assert_eq!(
        rate("0.4999").convert(euros(-1), Currency::EUR),
        Ok(euros(0))
    );
    assert_eq!(
        rate("0.5001").convert(euros(1), Currency::EUR),
        Ok(euros(1))
    );
    assert_eq!(
        rate("0.5001").convert(euros(-1), Currency::EUR),
        Ok(euros(-1))
    );
}

/// Converting between currencies with different minor units would be off by
/// a power of ten.
#[test]
fn convert_scales_between_decimal_places() {
    // No decimals into two: 1000 JPY at 0.0061 is 6.10 EUR.
    assert_eq!(
        rate("0.0061").convert(Money::from_minor_units(Currency::JPY, 1_000), Currency::EUR),
        Ok(euros(610))
    );
    // Two decimals into none: 10.00 EUR at 162.5 is 1625 JPY, and 0.01 EUR
    // is 1.625 JPY, rounded to 2.
    assert_eq!(
        rate("162.5").convert(euros(1_000), Currency::JPY),
        Ok(Money::from_minor_units(Currency::JPY, 1_625))
    );
    assert_eq!(
        rate("162.5").convert(euros(1), Currency::JPY),
        Ok(Money::from_minor_units(Currency::JPY, 2))
    );
    // Two decimals into three: 10.00 EUR at 0.41 is 4.100 BHD.
    assert_eq!(
        rate("0.41").convert(euros(1_000), Currency::BHD),
        Ok(Money::from_minor_units(Currency::BHD, 4_100))
    );
    // Three decimals into two: 1.234 BHD at 2.4 is 2.9616 EUR, rounded down.
    assert_eq!(
        rate("2.4").convert(Money::from_minor_units(Currency::BHD, 1_234), Currency::EUR),
        Ok(euros(296))
    );
    // No decimals into four: 1 JPY at 1 is 1.0000 CLF.
    assert_eq!(
        rate("1").convert(Money::from_minor_units(Currency::JPY, 1), Currency::CLF),
        Ok(Money::from_minor_units(Currency::CLF, 10_000))
    );
}

/// A conversion too large for 64-bit minor units would wrap around to a
/// wrong amount.
#[test]
fn conversions_out_of_range_overflow() {
    // The converted amount does not fit.
    assert_eq!(
        rate("2").convert(euros(i64::MAX), Currency::EUR),
        Err(MoneyError::Overflow)
    );
    assert_eq!(
        rate("2").convert(euros(i64::MIN), Currency::EUR),
        Err(MoneyError::Overflow)
    );
    // The intermediate product does not fit.
    let largest = ExchangeRate::from_decimal(u64::MAX, 0).unwrap();
    assert_eq!(
        largest.convert(euros(i64::MAX), Currency::BHD),
        Err(MoneyError::Overflow)
    );
    // The largest amount at rate 1 fits.
    assert_eq!(
        rate("1").convert(euros(i64::MAX), Currency::USD),
        Ok(Money::from_minor_units(Currency::USD, i64::MAX))
    );
}

/// Saved rates would fail to load, or a zero rate would load.
#[cfg(feature = "serde")]
#[test]
fn rates_serialise_as_their_text_form() {
    // Round trip.
    assert_eq!(
        serde_json::to_string(&rate("4.9765")).unwrap(),
        "\"4.9765\""
    );
    assert_eq!(
        serde_json::from_str::<ExchangeRate>("\"4.9700\"").unwrap(),
        rate("4.97")
    );
    // Deserialising validates like parsing.
    assert!(serde_json::from_str::<ExchangeRate>("\"0\"").is_err());
    // Only strings are rates.
    assert!(serde_json::from_str::<ExchangeRate>("4.97").is_err());
}

proptest! {
    /// A saved rate would read back as a different rate.
    #[test]
    fn display_then_parse_round_trips(mantissa in 1..=u64::MAX, scale in 0..=MAX_SCALE) {
        let exchange_rate = ExchangeRate::from_decimal(mantissa, scale).unwrap();
        prop_assert_eq!(ExchangeRate::parse(&exchange_rate.to_string()), Ok(exchange_rate));
    }

    /// Converting at rate 1 between currencies with the same minor unit
    /// would change the amount.
    #[test]
    fn a_rate_of_one_keeps_the_amount(
        minor_units in any::<i64>(),
        source_index in 0..Currency::all().len(),
        target_choice in any::<prop::sample::Index>(),
    ) {
        let source = Currency::all()[source_index];
        let targets: Vec<Currency> = Currency::all()
            .iter()
            .copied()
            .filter(|currency| currency.decimal_places() == source.decimal_places())
            .collect();
        let target = targets[target_choice.index(targets.len())];
        prop_assert_eq!(
            rate("1").convert(Money::from_minor_units(source, minor_units), target),
            Ok(Money::from_minor_units(target, minor_units))
        );
    }
}

/// A currency change would convert with another currency's rate, or read the
/// rate as belonging to the target currency.
#[test]
fn a_currency_rate_keeps_its_currency_and_rate() {
    let rate = ExchangeRate::parse("4.97").unwrap();
    let pair = CurrencyRate::new(Currency::EUR, rate);
    assert_eq!(pair.currency(), Currency::EUR);
    assert_eq!(pair.rate(), rate);
}
