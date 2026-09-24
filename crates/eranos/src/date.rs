//! Calendar dates and times of day, without time zones.
//!
//! The library never reads the clock: every date comes from the caller, which
//! keeps the core deterministic and usable on wasm, where the system clock is
//! unavailable.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::error::ValueError;

/// A calendar date in the proleptic Gregorian calendar, between years 1 and
/// 9999.
///
/// The text form is ISO 8601 `YYYY-MM-DD`, and ordering is chronological.
///
/// # Examples
///
/// ```
/// use eranos::Date;
///
/// let date = Date::new(2026, 9, 23)?;
/// assert_eq!(date.to_string(), "2026-09-23");
/// assert_eq!("2026-09-23".parse::<Date>()?, date);
/// assert!(Date::new(2026, 2, 29).is_err()); // 2026 is not a leap year
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Date {
    day: u8,
    month: u8,
    year: u16,
}

// Written out because the fields are in alphabetical order: a derived
// ordering would compare days first.
impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.year, self.month, self.day).cmp(&(other.year, other.month, other.day))
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Date {
    /// Returns the date `year`-`month`-`day`.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidDate`] when the year is outside 1..=9999 or the
    /// day does not exist in that month.
    pub const fn new(year: u16, month: u8, day: u8) -> Result<Self, ValueError> {
        if year < 1 || year > 9999 || month < 1 || month > 12 {
            return Err(ValueError::InvalidDate);
        }
        if day < 1 || day > days_in_month(year, month) {
            return Err(ValueError::InvalidDate);
        }
        Ok(Self { day, month, year })
    }

    /// Parses an ISO 8601 `YYYY-MM-DD` date.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidDate`] when the text is not exactly four digits,
    /// `-`, two digits, `-`, two digits, or names a day that does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::Date;
    ///
    /// assert_eq!(Date::parse("2028-02-29")?.day(), 29);
    /// assert!(Date::parse("2026-9-23").is_err());
    /// # Ok::<(), eranos::error::ValueError>(())
    /// ```
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let bytes = text.trim().as_bytes();
        let [y1, y2, y3, y4, b'-', m1, m2, b'-', d1, d2] = *bytes else {
            return Err(ValueError::InvalidDate);
        };
        let year = four_digits([y1, y2, y3, y4]).ok_or(ValueError::InvalidDate)?;
        let month = two_digits(m1, m2).ok_or(ValueError::InvalidDate)?;
        let day = two_digits(d1, d2).ok_or(ValueError::InvalidDate)?;
        Self::new(year, month, day)
    }

    /// The day of the month, starting at 1.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    /// The month, 1 (January) to 12 (December).
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The year, 1 to 9999.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

impl FromStr for Date {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// A time of day to the minute, between 00:00 and 23:59.
///
/// The text form is `hh:mm`, and ordering is chronological.
///
/// # Examples
///
/// ```
/// use eranos::Time;
///
/// let time = Time::new(18, 30)?;
/// assert_eq!(time.to_string(), "18:30");
/// assert_eq!(Time::parse("07:05")?.hour(), 7);
/// assert!(Time::new(24, 0).is_err());
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
}

// Written out, like the ordering of `Date`, so that it does not depend on
// the order of the fields.
impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.hour, self.minute).cmp(&(other.hour, other.minute))
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Time {
    /// Returns the time `hour`:`minute`.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidTime`] when the hour is above 23 or the minute
    /// above 59.
    pub const fn new(hour: u8, minute: u8) -> Result<Self, ValueError> {
        if hour > 23 || minute > 59 {
            return Err(ValueError::InvalidTime);
        }
        Ok(Self { hour, minute })
    }

    /// Parses a `hh:mm` time, two digits each.
    ///
    /// # Errors
    ///
    /// [`ValueError::InvalidTime`] when the text is not two digits, `:`, two
    /// digits, or names a time that does not exist.
    pub fn parse(text: &str) -> Result<Self, ValueError> {
        let bytes = text.trim().as_bytes();
        let [h1, h2, b':', m1, m2] = *bytes else {
            return Err(ValueError::InvalidTime);
        };
        let hour = two_digits(h1, h2).ok_or(ValueError::InvalidTime)?;
        let minute = two_digits(m1, m2).ok_or(ValueError::InvalidTime)?;
        Self::new(hour, minute)
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }
}

impl fmt::Display for Time {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl FromStr for Time {
    type Err = ValueError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

/// The value of four ASCII decimal digits; `None` for anything else.
fn four_digits(bytes: [u8; 4]) -> Option<u16> {
    bytes.iter().try_fold(0_u16, |value, &byte| {
        byte.is_ascii_digit()
            .then(|| value * 10 + u16::from(byte - b'0'))
    })
}

/// The value of two ASCII decimal digits; `None` for anything else.
fn two_digits(tens: u8, ones: u8) -> Option<u8> {
    (tens.is_ascii_digit() && ones.is_ascii_digit()).then(|| (tens - b'0') * 10 + (ones - b'0'))
}

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{Date, Time};

    impl Serialize for Date {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Date {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialise(deserialiser, "a `YYYY-MM-DD` date")
        }
    }

    impl Serialize for Time {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Time {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            crate::serde_text::deserialise(deserialiser, "an `hh:mm` time")
        }
    }
}

#[cfg(test)]
mod tests;
