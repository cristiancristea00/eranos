//! Every double an app hands the library becomes an amount, a percentage,
//! parts, a quantity, or a rate through `rounded`, so every binding gets the
//! same cents from the same double. These tests pin the text it rounds,
//! std's shortest decimal form, and the rule it rounds by, half away from
//! zero on the digits of that text, and the fixed-point text that amounts,
//! rates, percentages, parts, and quantities are read from and written in.

use std::fmt;

use super::*;

/// `units` at `places`, as `write_fixed` writes it with no unit.
fn written(units: i64, places: u8) -> String {
    fmt::from_fn(|formatter| write_fixed(formatter, units, places, "")).to_string()
}

/// A fixed-point reader that padded a short fraction at the wrong end,
/// accepted a digit beyond its places, or gave the wrong reason for a
/// refusal would read 1.5 parts as 1.05, round a fifth decimal of a
/// quantity away unseen, or report an amount with too many decimals as one
/// that is not a number.
#[test]
fn fixed_point_text_reads_as_units_of_its_places() {
    assert_eq!(parse_fixed("1.5", 2), Ok(150));
    assert_eq!(parse_fixed("1.05", 2), Ok(105));
    assert_eq!(parse_fixed("0.452", 4), Ok(4_520));
    assert_eq!(parse_fixed("12", 4), Ok(120_000));
    assert_eq!(parse_fixed("007", 0), Ok(7));
    // Text that is not digits, checked before the places.
    for text in [
        "1.", ".5", ".", "", "-1", "+1", "1e2", " 1", "1.2.3", "1.23x",
    ] {
        assert_eq!(
            parse_fixed(text, 2),
            Err(FixedRefusal::NotDecimal),
            "{text:?}"
        );
    }
    // More decimals than the places.
    assert_eq!(parse_fixed("1.234", 2), Err(FixedRefusal::TooManyPlaces));
    // The largest value, and one more.
    assert_eq!(parse_fixed("18446744073709551615", 0), Ok(u64::MAX));
    assert_eq!(
        parse_fixed("1844674407370955.1616", 4),
        Err(FixedRefusal::TooLarge)
    );
}

/// A writer that kept trailing zeros or dropped leading ones would show
/// 1.50 parts or 0.5 for 0.05.
#[test]
fn fixed_point_text_is_written_without_trailing_zeros() {
    assert_eq!(written(150, 2), "1.5");
    assert_eq!(written(105, 2), "1.05");
    assert_eq!(written(4_520, 4), "0.452");
    assert_eq!(written(1, 4), "0.0001");
    assert_eq!(written(30_000, 4), "3");
    assert_eq!(written(-5, 2), "-0.05");
    assert_eq!(written(0, 4), "0");
    assert_eq!(written(i64::MIN, 4), "-922337203685477.5808");
}

/// Rounding the double instead of its shortest text would turn 1.005 and
/// 2.675, whose nearest doubles lie below them, into 1.00 and 2.67: the
/// cents an app typed would not be the cents it stored.
#[test]
fn a_double_rounds_from_its_shortest_decimal_form() {
    assert_eq!(rounded(1.005, 2).as_deref(), Some("1.01"));
    assert_eq!(rounded(2.675, 2).as_deref(), Some("2.68"));
    assert_eq!(rounded(-2.675, 2).as_deref(), Some("-2.68"));
    // `0.1 + 0.2` is 0.30000000000000004, and its third digit decides.
    assert_eq!(rounded(0.1 + 0.2, 2).as_deref(), Some("0.30"));
}

/// Halves rounded towards zero or to even would make a refund of -0.125
/// and a payment of 0.125 differ by more than their sign.
#[test]
fn halves_round_away_from_zero_and_the_first_dropped_digit_decides() {
    assert_eq!(rounded(0.125, 2).as_deref(), Some("0.13"));
    assert_eq!(rounded(-0.125, 2).as_deref(), Some("-0.13"));
    assert_eq!(rounded(0.135, 2).as_deref(), Some("0.14"));
    // Only the first dropped digit counts: 1.00499 does not round twice.
    assert_eq!(rounded(1.004_99, 2).as_deref(), Some("1.00"));
    assert_eq!(rounded(1.004, 2).as_deref(), Some("1.00"));
}

/// A carry that stopped at the point, or at the first digit, would turn
/// 9.995 into 9.00 or 0.00.
#[test]
fn a_carry_runs_leftwards_through_every_nine() {
    assert_eq!(rounded(9.995, 2).as_deref(), Some("10.00"));
    assert_eq!(rounded(99.995, 2).as_deref(), Some("100.00"));
    assert_eq!(rounded(0.999, 2).as_deref(), Some("1.00"));
    assert_eq!(rounded(-9.5, 0).as_deref(), Some("-10"));
    assert_eq!(rounded(199.6, 0).as_deref(), Some("200"));
}

/// A rounded zero that kept its sign would read as a negative amount, or be
/// refused by a parser that takes no sign.
#[test]
fn a_result_of_zero_has_no_sign() {
    assert_eq!(rounded(-0.0, 2).as_deref(), Some("0"));
    assert_eq!(rounded(0.0, 2).as_deref(), Some("0"));
    assert_eq!(rounded(-0.004, 2).as_deref(), Some("0.00"));
    assert_eq!(rounded(-0.4, 0).as_deref(), Some("0"));
    // The smallest subnormal double is 0.000…0005, 323 zeros after the point.
    assert_eq!(rounded(5e-324, 2).as_deref(), Some("0.00"));
    assert_eq!(
        rounded(-5e-324, 18).as_deref(),
        Some("0.000000000000000000")
    );
}

/// An exponent in the text, as `{:e}` or a debugger writes it, would reach
/// a parser that reads digits only and be refused.
#[test]
fn large_and_small_doubles_are_written_without_an_exponent() {
    assert_eq!(rounded(1e21, 2).as_deref(), Some("1000000000000000000000"));
    assert_eq!(rounded(1e-7, 18).as_deref(), Some("0.0000001"));
    assert_eq!(rounded(1e-7, 2).as_deref(), Some("0.00"));
}

/// Padding a short fraction would give a rate of 25.5 eighteen decimal
/// places, and trimming a long one would lose the digits a rate keeps.
#[test]
fn a_fraction_shorter_than_the_places_stays_as_it_is() {
    assert_eq!(rounded(25.5, 18).as_deref(), Some("25.5"));
    assert_eq!(
        rounded(0.1 + 0.2, 18).as_deref(),
        Some("0.30000000000000004")
    );
    assert_eq!(rounded(12.0, 2).as_deref(), Some("12"));
    assert_eq!(rounded(1234.5, 0).as_deref(), Some("1235"));
}

/// NaN and the infinities have no decimal form: as text they are `NaN` and
/// `inf`, which no parser should be handed.
#[test]
fn nan_and_the_infinities_are_refused() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(rounded(value, 2), None, "{value}");
    }
}
