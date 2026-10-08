//! The decimal text of a double, rounded to a number of places: the one way
//! a float becomes an amount, a percentage, parts, or an exchange rate.

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
