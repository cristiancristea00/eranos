//! Decimal text: the fixed-point numbers that amounts, exchange rates,
//! percentages, parts, and quantities are written in, and the text of a
//! double rounded to a number of places, the one way a float becomes an
//! amount, a percentage, parts, a quantity, or an exchange rate.

use std::fmt;
use std::iter;

/// Why [`parse_fixed`] refused a text, which each caller reports as its own
/// failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedRefusal {
    /// The text is not digits with an optional `.` followed by digits.
    NotDecimal,
    /// It has more decimal places than the reader was asked for.
    TooManyPlaces,
    /// Its value is above `u64::MAX`.
    TooLarge,
}

/// The number of units of `10^-places` that `text` writes: digits with an
/// optional `.` followed by at most `places` digits, so `1.5` at two places
/// is 150 and `0.452` at four is 4 520. A fraction shorter than `places` is
/// padded with zeros.
///
/// # Errors
///
/// [`FixedRefusal::NotDecimal`] for anything else, a sign, an exponent,
/// whitespace, or a point without a digit on either side included, checked
/// first; [`FixedRefusal::TooManyPlaces`] for a fraction longer than
/// `places`; and [`FixedRefusal::TooLarge`] for a value above `u64::MAX`.
pub(crate) fn parse_fixed(text: &str, places: u8) -> Result<u64, FixedRefusal> {
    let (whole, fraction) = match text.split_once('.') {
        Some((_, "")) => return Err(FixedRefusal::NotDecimal),
        Some((whole, fraction)) => (whole, fraction),
        None => (text, ""),
    };
    let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    if whole.is_empty() || !is_digits(whole) || !is_digits(fraction) {
        return Err(FixedRefusal::NotDecimal);
    }
    let padding = usize::from(places)
        .checked_sub(fraction.len())
        .ok_or(FixedRefusal::TooManyPlaces)?;
    whole
        .bytes()
        .chain(fraction.bytes())
        .chain(iter::repeat_n(b'0', padding))
        .try_fold(0_u64, |value, digit| {
            value.checked_mul(10)?.checked_add(u64::from(digit - b'0'))
        })
        .ok_or(FixedRefusal::TooLarge)
}

/// Writes `units` of `10^-places` as a decimal without trailing zeros,
/// followed by `unit`: 1 250 at two places with `%` is `12.5%`, 4 520 at
/// four places is `0.452`, and -5 at two places is `-0.05`.
///
/// It takes any `i64`, zero and negative numbers included, and writes a
/// negative one after a minus sign. Callers pass a constant of this crate
/// for `places`, two or four, so the power of ten fits in a `u64`, which
/// holds every power up to the 19th.
pub(crate) fn write_fixed(
    formatter: &mut fmt::Formatter<'_>,
    units: i64,
    places: u8,
    unit: &str,
) -> fmt::Result {
    let sign = if 0 > units { "-" } else { "" };
    let magnitude = units.unsigned_abs();
    let divisor = 10_u64.pow(u32::from(places));
    let whole = magnitude / divisor;
    let mut fraction = magnitude % divisor;
    if 0 == fraction {
        return write!(formatter, "{sign}{whole}{unit}");
    }
    // A fraction that is not zero and below `10^places` has at most
    // `places - 1` trailing zeros, so the width stays at least one.
    let mut width = usize::from(places);
    while fraction.is_multiple_of(10) {
        fraction /= 10;
        width -= 1;
    }
    write!(formatter, "{sign}{whole}.{fraction:0width$}{unit}")
}

/// `value` as the shortest decimal that reads back as the same double,
/// rounded half away from zero to at most `places` decimal places; `None`
/// for NaN and the infinities.
///
/// The text is std's `Display` of an `f64`, which never writes an exponent:
/// 1e21 is `1000000000000000000000` and 1e-7 is `0.0000001`. Rounding that
/// text rather than the double keeps 1.005, whose nearest double lies below
/// it, at `1.01` for two places. The first dropped digit decides, and a
/// carry runs leftwards, so 9.995 is `10.00`. A fraction with fewer digits
/// than `places` stays as it is, so 25.5 at 18 places is `25.5`; a result
/// of zero has no sign, and a result without decimal places has no point.
/// Beyond `is_finite` and `to_string`, it does no arithmetic on the double.
pub(crate) fn rounded(value: f64, places: u8) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let text = value.to_string();
    let (negative, magnitude) = text
        .strip_prefix('-')
        .map_or((false, text.as_str()), |rest| (true, rest));
    let (whole, fraction) = magnitude.split_once('.').unwrap_or((magnitude, ""));
    let places = usize::from(places);
    let kept = fraction.len().min(places);
    let mut digits: Vec<u8> = whole.bytes().chain(fraction.bytes().take(places)).collect();
    if fraction
        .as_bytes()
        .get(places)
        .is_some_and(|&digit| b'5' <= digit)
    {
        increment(&mut digits);
    }
    let is_zero = digits.iter().all(|&digit| b'0' == digit);
    // The digits hold the whole part's and the `kept` places', and the carry
    // only adds digits, so the point falls after the first digit or later.
    let point = digits.len() - kept;
    let mut written = String::with_capacity(digits.len() + 2);
    if negative && !is_zero {
        written.push('-');
    }
    for (position, digit) in digits.into_iter().enumerate() {
        if position == point {
            written.push('.');
        }
        written.push(char::from(digit));
    }
    Some(written)
}

/// Adds one to the decimal number `digits` writes, carrying leftwards past
/// every 9: `199` becomes `200`, and `99` becomes `100`.
fn increment(digits: &mut Vec<u8>) {
    for digit in digits.iter_mut().rev() {
        if b'9' == *digit {
            *digit = b'0';
        } else {
            // An ASCII digit below 9, so the next one is a digit too.
            *digit += 1;
            return;
        }
    }
    digits.insert(0, b'1');
}

#[cfg(test)]
mod tests;
