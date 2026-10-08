//! An item's quantity multiplies its unit price into the line total its
//! members share. These tests keep typed quantities exact to four decimals,
//! refuse anything ambiguous instead of rounding it, keep zero out (a line
//! of zero units would be shared and cost nothing), keep every quantity
//! within the signed 64-bit integers the bindings carry, and keep the text
//! form one that reads back as the same quantity.

use super::*;

use proptest::prelude::*;

/// The largest number of ten-thousandths a quantity holds, `i64::MAX`.
const LARGEST: u64 = 9_223_372_036_854_775_807;

fn quantity(text: &str) -> Quantity {
    Quantity::parse(text).unwrap()
}

/// A quantity typed on a receipt line would be refused or misread.
#[test]
fn quantities_parse_to_ten_thousandths() {
    // Four, three, one, and no decimals.
    assert_eq!(quantity("0.0001").ten_thousandths(), 1);
    assert_eq!(quantity("0.452").ten_thousandths(), 4_520);
    assert_eq!(quantity("1.5").ten_thousandths(), 15_000);
    assert_eq!(quantity("2").ten_thousandths(), 20_000);
    // Surrounding whitespace.
    assert_eq!(quantity(" 3 ").ten_thousandths(), 30_000);
    // `FromStr` parses the same way.
    assert_eq!("1.5".parse::<Quantity>(), Ok(quantity("1.5")));
    // One unit.
    assert_eq!(Quantity::ONE.ten_thousandths(), 10_000);
    assert_eq!(Quantity::ONE, quantity("1"));
}

/// A line of zero units would be listed on the receipt and cost nothing.
#[test]
fn a_zero_quantity_is_refused() {
    // Typed zeros.
    for text in ["0", "0.0", "0.0000", "000"] {
        assert_eq!(
            Quantity::parse(text),
            Err(MoneyError::InvalidQuantity),
            "{text}"
        );
    }
    // Zero ten-thousandths and zero whole units.
    assert_eq!(
        Quantity::from_ten_thousandths(0),
        Err(MoneyError::InvalidQuantity)
    );
    assert_eq!(Quantity::from_whole(0), Err(MoneyError::InvalidQuantity));
    // The smallest positive quantity.
    assert_eq!(
        Quantity::from_ten_thousandths(1).map(Quantity::ten_thousandths),
        Ok(1)
    );
}

/// An ambiguous or negative quantity would be guessed at instead of
/// refused: a fifth decimal would be rounded away without the user seeing.
#[test]
fn malformed_quantities_are_refused() {
    // Five decimals.
    for text in ["0.00005", "1.23456", "2.00000"] {
        assert_eq!(
            Quantity::parse(text),
            Err(MoneyError::InvalidQuantity),
            "{text}"
        );
    }
    // Signs, exponents, letters, and other separators.
    for text in ["-1", "+1", "1e2", "1E2", "abc", "1,5", "1.2.3", "½"] {
        assert_eq!(
            Quantity::parse(text),
            Err(MoneyError::InvalidQuantity),
            "{text}"
        );
    }
    // Missing digits.
    for text in [".5", "5.", ".", "", "  "] {
        assert_eq!(
            Quantity::parse(text),
            Err(MoneyError::InvalidQuantity),
            "{text:?}"
        );
    }
}

/// A quantity beyond the signed 64-bit ten-thousandths the bindings carry
/// would wrap around to a negative one in another language, or a parse
/// would wrap to a small one.
#[test]
fn quantities_beyond_i64_ten_thousandths_are_refused() {
    // The largest quantity, from its ten-thousandths and from its text.
    assert_eq!(
        Quantity::from_ten_thousandths(LARGEST).map(Quantity::ten_thousandths),
        Ok(LARGEST)
    );
    assert_eq!(
        Quantity::parse("922337203685477.5807").map(Quantity::ten_thousandths),
        Ok(LARGEST)
    );
    // One ten-thousandth more, and far more.
    assert_eq!(
        Quantity::from_ten_thousandths(LARGEST + 1),
        Err(MoneyError::InvalidQuantity)
    );
    assert_eq!(
        Quantity::from_ten_thousandths(u64::MAX),
        Err(MoneyError::InvalidQuantity)
    );
    for text in [
        "922337203685477.5808",
        "1844674407370955.1615",
        "99999999999999999999999",
    ] {
        assert_eq!(
            Quantity::parse(text),
            Err(MoneyError::InvalidQuantity),
            "{text}"
        );
    }
}

/// Whole quantities such as "3 beers" would be stored off by a factor of
/// ten thousand, or wrap around.
#[test]
fn whole_quantities_are_scaled_to_ten_thousandths() {
    // Small counts.
    assert_eq!(
        Quantity::from_whole(3).map(Quantity::ten_thousandths),
        Ok(30_000)
    );
    assert_eq!(Quantity::from_whole(1), Ok(Quantity::ONE));
    // The largest count that fits, and one more.
    assert_eq!(
        Quantity::from_whole(922_337_203_685_477).map(Quantity::ten_thousandths),
        Ok(9_223_372_036_854_770_000)
    );
    assert_eq!(
        Quantity::from_whole(922_337_203_685_478),
        Err(MoneyError::InvalidQuantity)
    );
    // A count whose ten-thousandths do not fit in a `u64` at all.
    assert_eq!(
        Quantity::from_whole(u64::MAX),
        Err(MoneyError::InvalidQuantity)
    );
}

/// Quantities would be shown with noise such as `1.5000` or lose decimals.
#[test]
fn quantity_display_drops_trailing_zeros() {
    // Four, three, two, one, and no decimals.
    assert_eq!(quantity("0.0001").to_string(), "0.0001");
    assert_eq!(quantity("0.452").to_string(), "0.452");
    assert_eq!(quantity("2.05").to_string(), "2.05");
    assert_eq!(quantity("1.5000").to_string(), "1.5");
    assert_eq!(quantity("3").to_string(), "3");
    // The largest quantity.
    assert_eq!(
        Quantity::from_ten_thousandths(LARGEST).unwrap().to_string(),
        "922337203685477.5807"
    );
}

/// The bindings, which write a quantity from the ten-thousandths of their
/// record, would show a valid one otherwise than the library does, or fail
/// on zero or a negative number rather than show it.
#[test]
fn the_quantity_writer_writes_any_ten_thousandths_as_the_text_form_does() {
    // As the text form writes every quantity.
    for ten_thousandths in [1, 10, 4_520, 10_000, 15_000, 20_500, LARGEST] {
        assert_eq!(
            format_ten_thousandths(i64::try_from(ten_thousandths).unwrap()).to_string(),
            Quantity::from_ten_thousandths(ten_thousandths)
                .unwrap()
                .to_string()
        );
    }
    // Zero and a negative number, which no quantity holds.
    assert_eq!(format_ten_thousandths(0).to_string(), "0");
    assert_eq!(format_ten_thousandths(-1).to_string(), "-0.0001");
    assert_eq!(format_ten_thousandths(-15_000).to_string(), "-1.5");
    assert_eq!(format_ten_thousandths(-20_000).to_string(), "-2");
    // The bounds of an `i64`.
    assert_eq!(
        format_ten_thousandths(i64::MIN).to_string(),
        "-922337203685477.5808"
    );
    assert_eq!(
        format_ten_thousandths(i64::MAX).to_string(),
        "922337203685477.5807"
    );
}

/// A quantity held as a double would multiply the unit price by other
/// ten-thousandths than the user typed, or a double that rounds to zero
/// would make a line of no units.
#[test]
fn a_quantity_converts_from_a_double_rounded_to_four_places() {
    assert_eq!(Quantity::from_f64(0.452), Ok(quantity("0.452")));
    assert_eq!(Quantity::from_f64(0.45205), Ok(quantity("0.4521")));
    assert_eq!(Quantity::from_f64(1.5), Ok(quantity("1.5")));
    assert_eq!(Quantity::from_f64(2.0), Ok(quantity("2")));
    // Half a ten-thousandth rounds away from zero.
    assert_eq!(Quantity::from_f64(0.00005), Ok(quantity("0.0001")));
    // Zero, negative, and huge quantities are refused as the parser
    // refuses them.
    for value in [0.00004, 0.0, -0.0, -1.0, 1e21] {
        assert_eq!(
            Quantity::from_f64(value),
            Err(MoneyError::InvalidQuantity),
            "{value}"
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            Quantity::from_f64(value),
            Err(MoneyError::NotFinite),
            "{value}"
        );
    }
}

/// An item that sets no quantity would count zero units, and ordering
/// would not follow the value.
#[test]
fn the_default_quantity_is_one() {
    assert_eq!(Quantity::default(), Quantity::ONE);
    assert!(Quantity::ONE > quantity("0.9999"));
    assert!(Quantity::ONE < quantity("1.0001"));
}

/// Saved quantities would fail to load, load as another quantity, or a
/// zero quantity would load.
#[cfg(feature = "serde")]
#[test]
fn quantities_serialise_as_their_text() {
    assert_eq!(serde_json::to_string(&quantity("1.5")).unwrap(), r#""1.5""#);
    assert_eq!(serde_json::to_string(&Quantity::ONE).unwrap(), r#""1""#);
    assert_eq!(
        serde_json::from_str::<Quantity>(r#""0.452""#).unwrap(),
        quantity("0.452")
    );
    // Loading checks as parsing does.
    for json in [r#""0""#, r#""0.00005""#, r#""-1""#, "15000", "1.5"] {
        assert!(serde_json::from_str::<Quantity>(json).is_err(), "{json}");
    }
}

proptest! {
    /// A saved or shown quantity would read back as a different one.
    #[test]
    fn quantity_display_then_parse_round_trips(ten_thousandths in 1..=LARGEST) {
        let value = Quantity::from_ten_thousandths(ten_thousandths).unwrap();
        prop_assert_eq!(Quantity::parse(&value.to_string()), Ok(value));
    }

    /// The bindings would write a valid quantity otherwise than the library
    /// does.
    #[test]
    fn the_quantity_writer_writes_every_quantity_as_its_text_form(ten_thousandths in 1..=LARGEST) {
        prop_assert_eq!(
            format_ten_thousandths(i64::try_from(ten_thousandths).unwrap()).to_string(),
            Quantity::from_ten_thousandths(ten_thousandths).unwrap().to_string()
        );
    }
}
