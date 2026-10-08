//! Percentages and parts: the two decimal ratios splits use, both with two
//! decimal places and stored as exact integers.

use std::fmt;
use std::str::FromStr;

use crate::error::MoneyError;

/// A percentage with two decimal places, stored as basis points (1 % = 100).
///
/// The text form is the percentage without trailing zeros and with a `%`
/// sign, such as `33.33%` or `12.5%`.
///
/// # Examples
///
/// ```
/// use eranos::Percentage;
///
/// let third = Percentage::parse("33.33")?;
/// assert_eq!(third.basis_points(), 3333);
/// assert_eq!(third.to_string(), "33.33%");
/// assert_eq!(Percentage::parse("12.5%")?, Percentage::from_basis_points(1250));
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Percentage(u32);

impl Percentage {
    /// 0 %.
    pub const ZERO: Self = Self(0);

    /// 100 %: the total a percentage split must reach.
    pub const HUNDRED: Self = Self(10_000);

    /// Returns the percentage with `basis_points` hundredths of a per cent.
    #[must_use]
    pub const fn from_basis_points(basis_points: u32) -> Self {
        Self(basis_points)
    }

    /// Parses a non-negative decimal with at most two decimal places and an
    /// optional trailing `%`, such as `33.33`, `12.5%`, or `100`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidPercentage`] for anything else, including values
    /// that do not fit in 32-bit basis points.
    pub fn parse(text: &str) -> Result<Self, MoneyError> {
        let text = text.trim();
        let number = text.strip_suffix('%').map_or(text, str::trim_end);
        parse_hundredths(number)
            .map(Self)
            .ok_or(MoneyError::InvalidPercentage)
    }

    /// The percentage in basis points: 3333 for 33.33 %.
    #[must_use]
    pub const fn basis_points(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&format_basis_points(i64::from(self.0)), formatter)
    }
}

impl FromStr for Percentage {
    type Err = MoneyError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// A positive number of parts (shares) with two decimal places, stored as
/// hundredths.
///
/// In a parts split every member pays in proportion to their parts: a couple
/// counted as 2 and a child as 0.5 next to adults at 1. The text form has no
/// trailing zeros, such as `1.5` or `2`.
///
/// # Examples
///
/// ```
/// use eranos::Parts;
///
/// let half = Parts::parse("0.5")?;
/// assert_eq!(half.hundredths(), 50);
/// assert_eq!(Parts::from_whole(2)?.to_string(), "2");
/// assert!(Parts::parse("0").is_err());
/// # Ok::<(), eranos::error::MoneyError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "u32", into = "u32")
)]
pub struct Parts(u32);

impl Parts {
    /// One part.
    pub const ONE: Self = Self(100);

    /// Returns `hundredths / 100` parts.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidParts`] when `hundredths` is zero.
    pub const fn from_hundredths(hundredths: u32) -> Result<Self, MoneyError> {
        if 0 == hundredths {
            return Err(MoneyError::InvalidParts);
        }
        Ok(Self(hundredths))
    }

    /// Returns `parts` whole parts.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidParts`] when `parts` is zero or above
    /// `u32::MAX / 100`.
    pub const fn from_whole(parts: u32) -> Result<Self, MoneyError> {
        match parts.checked_mul(100) {
            Some(hundredths) => Self::from_hundredths(hundredths),
            None => Err(MoneyError::InvalidParts),
        }
    }

    /// Parses a positive decimal with at most two decimal places, such as
    /// `1`, `0.5`, or `2.25`.
    ///
    /// # Errors
    ///
    /// [`MoneyError::InvalidParts`] for anything else, zero included.
    pub fn parse(text: &str) -> Result<Self, MoneyError> {
        let hundredths = parse_hundredths(text.trim()).ok_or(MoneyError::InvalidParts)?;
        Self::from_hundredths(hundredths)
    }

    /// The parts in hundredths: 150 for 1.5 parts, as `u32::from(parts)`
    /// also gives.
    #[must_use]
    pub const fn hundredths(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Parts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&format_hundredths(i64::from(self.0)), formatter)
    }
}

impl FromStr for Parts {
    type Err = MoneyError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

impl TryFrom<u32> for Parts {
    type Error = MoneyError;

    fn try_from(hundredths: u32) -> Result<Self, Self::Error> {
        Self::from_hundredths(hundredths)
    }
}

impl From<Parts> for u32 {
    fn from(parts: Parts) -> Self {
        parts.0
    }
}

/// Parses `digits[.d[d]]` into hundredths; `None` for anything else or when
/// the value does not fit in a `u32`.
fn parse_hundredths(text: &str) -> Option<u32> {
    let (whole, fraction) = match text.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (text, "00"),
    };
    let is_digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    if !is_digits(whole) || !is_digits(fraction) || 2 < fraction.len() {
        return None;
    }
    let padding = if 1 == fraction.len() { "0" } else { "" };
    whole
        .bytes()
        .chain(fraction.bytes())
        .chain(padding.bytes())
        .try_fold(0_u32, |value, digit| {
            value.checked_mul(10)?.checked_add(u32::from(digit - b'0'))
        })
}

/// Formats `basis_points` as a percentage: the number of per cent without
/// trailing zeros, then a `%` sign.
///
/// The text form of [`Percentage`] writes its basis points with it: 3333 is
/// `33.33%`, 1250 is `12.5%`, and 1200 is `12%`. It takes any number, a
/// negative one included, which no percentage holds, and writes it the same
/// way after a minus sign: the binding core writes with it the basis points
/// of a record it has not checked, so that its text for a percentage is this
/// crate's.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes a percentage through its `Percentage`.
/// use eranos_money::format_basis_points;
///
/// assert_eq!(format_basis_points(1_250).to_string(), "12.5%");
/// assert_eq!(format_basis_points(-5).to_string(), "-0.05%");
/// ```
#[must_use]
pub fn format_basis_points(basis_points: i64) -> impl fmt::Display {
    fmt::from_fn(move |formatter| write_hundredths(formatter, basis_points, "%"))
}

/// Formats `hundredths` as a number of parts: a decimal without trailing
/// zeros.
///
/// The text form of [`Parts`] writes its hundredths with it: 150 is `1.5`,
/// 225 is `2.25`, and 200 is `2`. It takes any number, zero and negative ones
/// included, which no parts hold, and writes it the same way, a negative one
/// after a minus sign: the binding core writes with it the hundredths of a
/// record it has not checked, so that its text for parts is this crate's.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes parts through its `Parts`.
/// use eranos_money::format_hundredths;
///
/// assert_eq!(format_hundredths(150).to_string(), "1.5");
/// assert_eq!(format_hundredths(0).to_string(), "0");
/// assert_eq!(format_hundredths(-205).to_string(), "-2.05");
/// ```
#[must_use]
pub fn format_hundredths(hundredths: i64) -> impl fmt::Display {
    fmt::from_fn(move |formatter| write_hundredths(formatter, hundredths, ""))
}

/// Writes hundredths as a decimal without trailing zeros, followed by
/// `unit`: 1250 and `%` → `12.5%`, and -5 and `%` → `-0.05%`.
fn write_hundredths(
    formatter: &mut fmt::Formatter<'_>,
    hundredths: i64,
    unit: &str,
) -> fmt::Result {
    let sign = if 0 > hundredths { "-" } else { "" };
    let magnitude = hundredths.unsigned_abs();
    let (whole, fraction) = (magnitude / 100, magnitude % 100);
    match fraction {
        0 => write!(formatter, "{sign}{whole}{unit}"),
        _ if 0 == fraction % 10 => write!(formatter, "{sign}{whole}.{}{unit}", fraction / 10),
        _ => write!(formatter, "{sign}{whole}.{fraction:02}{unit}"),
    }
}

#[cfg(test)]
mod tests;
