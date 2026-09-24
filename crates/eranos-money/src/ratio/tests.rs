//! Percentage and parts splits divide real money by the ratios users type.
//! These tests keep typed ratios exact to two decimals, refuse anything
//! ambiguous instead of rounding it, keep zero parts out (a member with zero
//! parts would silently pay nothing), and keep the text form one that reads
//! back as the same ratio.

use super::*;

use proptest::prelude::*;

fn percentage(text: &str) -> Percentage {
    Percentage::parse(text).unwrap()
}

fn parts(text: &str) -> Parts {
    Parts::parse(text).unwrap()
}

/// Percentages typed in a split form would be refused or misread.
#[test]
fn percentages_parse_to_basis_points() {
    // Two, one, and no decimals.
    assert_eq!(percentage("33.33").basis_points(), 3_333);
    assert_eq!(percentage("12.5").basis_points(), 1_250);
    assert_eq!(percentage("0.05").basis_points(), 5);
    assert_eq!(percentage("100").basis_points(), 10_000);
    assert_eq!(percentage("0").basis_points(), 0);
    // Above 100 parses; splits check the total.
    assert_eq!(percentage("150").basis_points(), 15_000);
    // `FromStr` parses the same way.
    assert_eq!("12.5".parse::<Percentage>(), Ok(percentage("12.5")));
}

/// Percentages typed with their sign or stray spaces would be refused.
#[test]
fn a_per_cent_sign_and_whitespace_are_accepted() {
    // A trailing sign, with or without a space.
    assert_eq!(percentage("12.5%").basis_points(), 1_250);
    assert_eq!(percentage("12.5 %").basis_points(), 1_250);
    // Surrounding whitespace.
    assert_eq!(percentage(" 7 % ").basis_points(), 700);
    assert_eq!(percentage("\t33.33\n").basis_points(), 3_333);
}

/// An ambiguous or negative percentage would be guessed at instead of
/// refused.
#[test]
fn malformed_percentages_are_rejected() {
    // Too many decimals.
    for text in ["1.234", "33.333", "0.001"] {
        assert_eq!(
            Percentage::parse(text),
            Err(MoneyError::InvalidPercentage),
            "{text}"
        );
    }
    // Signs, letters, and other separators.
    for text in ["-1", "+1", "abc", "1,5", "12.5%%", "% 12", "12%5", "1e2"] {
        assert_eq!(
            Percentage::parse(text),
            Err(MoneyError::InvalidPercentage),
            "{text}"
        );
    }
    // Missing digits.
    for text in [".5", "5.", ".", "", "%", " % "] {
        assert_eq!(
            Percentage::parse(text),
            Err(MoneyError::InvalidPercentage),
            "{text:?}"
        );
    }
}

/// A percentage too large for 32-bit basis points would wrap around to a
/// small one.
#[test]
fn percentages_beyond_u32_basis_points_are_rejected() {
    // The largest percentage that fits.
    assert_eq!(percentage("42949672.95").basis_points(), u32::MAX);
    // One basis point more.
    for text in ["42949672.96", "42949673", "99999999999"] {
        assert_eq!(
            Percentage::parse(text),
            Err(MoneyError::InvalidPercentage),
            "{text}"
        );
    }
}

/// Percentages would be shown with noise such as `12.50%` or lose decimals.
#[test]
fn percentage_display_drops_trailing_zeros() {
    // Two, one, and no decimals.
    assert_eq!(Percentage::from_basis_points(3_333).to_string(), "33.33%");
    assert_eq!(Percentage::from_basis_points(1_250).to_string(), "12.5%");
    assert_eq!(Percentage::from_basis_points(1_200).to_string(), "12%");
    // Small percentages keep their leading zeros.
    assert_eq!(Percentage::from_basis_points(5).to_string(), "0.05%");
    assert_eq!(Percentage::from_basis_points(10).to_string(), "0.1%");
    assert_eq!(Percentage::from_basis_points(105).to_string(), "1.05%");
    // The bounds.
    assert_eq!(Percentage::ZERO.to_string(), "0%");
    assert_eq!(Percentage::HUNDRED.to_string(), "100%");
    assert_eq!(
        Percentage::from_basis_points(u32::MAX).to_string(),
        "42949672.95%"
    );
}

/// Percentage splits would be checked against the wrong total.
#[test]
fn percentage_constants_are_zero_and_one_hundred() {
    // Basis points.
    assert_eq!(Percentage::ZERO.basis_points(), 0);
    assert_eq!(Percentage::HUNDRED.basis_points(), 10_000);
    // The default is zero.
    assert_eq!(Percentage::default(), Percentage::ZERO);
    // Ordering follows the value.
    assert!(Percentage::ZERO < percentage("0.01"));
    assert!(percentage("99.99") < Percentage::HUNDRED);
}

/// Parts typed in a split form would be refused or misread.
#[test]
fn parts_parse_to_hundredths() {
    // Two, one, and no decimals.
    assert_eq!(parts("2.25").hundredths(), 225);
    assert_eq!(parts("0.5").hundredths(), 50);
    assert_eq!(parts("2").hundredths(), 200);
    assert_eq!(parts("0.01").hundredths(), 1);
    // Surrounding whitespace.
    assert_eq!(parts(" 3 ").hundredths(), 300);
    // `FromStr` parses the same way.
    assert_eq!("1.5".parse::<Parts>(), Ok(parts("1.5")));
    // One part.
    assert_eq!(Parts::ONE.hundredths(), 100);
    assert_eq!(Parts::ONE, parts("1"));
}

/// A member with zero parts would be listed in a split but pay nothing.
#[test]
fn zero_parts_are_rejected() {
    // Typed zeros.
    for text in ["0", "0.0", "0.00", "000"] {
        assert_eq!(Parts::parse(text), Err(MoneyError::InvalidParts), "{text}");
    }
    // Zero hundredths and zero whole parts.
    assert_eq!(Parts::from_hundredths(0), Err(MoneyError::InvalidParts));
    assert_eq!(Parts::from_whole(0), Err(MoneyError::InvalidParts));
    // The smallest positive value.
    assert_eq!(Parts::from_hundredths(1).map(Parts::hundredths), Ok(1));
}

/// An ambiguous or negative number of parts would be guessed at instead of
/// refused.
#[test]
fn malformed_parts_are_rejected() {
    // Too many decimals, signs, and letters.
    for text in ["1.234", "0.001", "-1", "+1", "abc", "1,5", "1%"] {
        assert_eq!(Parts::parse(text), Err(MoneyError::InvalidParts), "{text}");
    }
    // Missing digits.
    for text in [".5", "5.", "", "  "] {
        assert_eq!(
            Parts::parse(text),
            Err(MoneyError::InvalidParts),
            "{text:?}"
        );
    }
    // More than 32-bit hundredths hold.
    assert_eq!(
        Parts::parse("42949672.95").map(Parts::hundredths),
        Ok(u32::MAX)
    );
    assert_eq!(Parts::parse("42949672.96"), Err(MoneyError::InvalidParts));
}

/// Whole parts such as "a couple counts as 2" would be stored off by a
/// factor of a hundred, or wrap around.
#[test]
fn whole_parts_are_scaled_to_hundredths() {
    // Small counts.
    assert_eq!(Parts::from_whole(2).map(Parts::hundredths), Ok(200));
    assert_eq!(Parts::from_whole(1), Ok(Parts::ONE));
    // The largest count that fits, and one more.
    assert_eq!(
        Parts::from_whole(42_949_672).map(Parts::hundredths),
        Ok(4_294_967_200)
    );
    assert_eq!(Parts::from_whole(42_949_673), Err(MoneyError::InvalidParts));
    assert_eq!(Parts::from_whole(u32::MAX), Err(MoneyError::InvalidParts));
}

/// Parts would be shown with noise such as `1.50` or lose decimals.
#[test]
fn parts_display_drops_trailing_zeros() {
    // Two, one, and no decimals.
    assert_eq!(parts("0.25").to_string(), "0.25");
    assert_eq!(parts("1.5").to_string(), "1.5");
    assert_eq!(parts("2").to_string(), "2");
    // Leading zeros of the fraction stay.
    assert_eq!(parts("2.05").to_string(), "2.05");
    assert_eq!(parts("0.01").to_string(), "0.01");
}

/// Stored parts would load as a zero share or lose their value.
#[test]
fn parts_convert_to_and_from_hundredths() {
    // From hundredths.
    assert_eq!(Parts::try_from(150), Ok(parts("1.5")));
    assert_eq!(Parts::try_from(0), Err(MoneyError::InvalidParts));
    // Into hundredths.
    assert_eq!(u32::from(parts("1.5")), 150);
    // Ordering follows the value.
    assert!(parts("0.5") < Parts::ONE);
}

/// Saved ratios would fail to load, or zero parts would load.
#[cfg(feature = "serde")]
#[test]
fn ratios_serialise_as_integers() {
    // Percentages as basis points.
    assert_eq!(serde_json::to_string(&percentage("12.5")).unwrap(), "1250");
    assert_eq!(
        serde_json::from_str::<Percentage>("1250").unwrap(),
        percentage("12.5")
    );
    // Parts as hundredths, never zero.
    assert_eq!(serde_json::to_string(&parts("1.5")).unwrap(), "150");
    assert_eq!(serde_json::from_str::<Parts>("150").unwrap(), parts("1.5"));
    assert!(serde_json::from_str::<Parts>("0").is_err());
}

proptest! {
    /// A saved or shown percentage would read back as a different one.
    #[test]
    fn percentage_display_then_parse_round_trips(basis_points in any::<u32>()) {
        let ratio = Percentage::from_basis_points(basis_points);
        prop_assert_eq!(Percentage::parse(&ratio.to_string()), Ok(ratio));
    }

    /// Saved or shown parts would read back as different parts.
    #[test]
    fn parts_display_then_parse_round_trips(hundredths in 1..=u32::MAX) {
        let ratio = Parts::from_hundredths(hundredths).unwrap();
        prop_assert_eq!(Parts::parse(&ratio.to_string()), Ok(ratio));
    }
}
