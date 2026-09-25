//! Calendar dates and times of day, without time zones.
//!
//! The library never reads the clock: every date comes from the caller, which
//! keeps the core deterministic and usable on wasm, where the system clock is
//! unavailable. Both types wrap jiff's civil types and read text with jiff's
//! ISO 8601 parser; the jiff types stay private.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromCalendarError`],
//! through which a crate that tells every failure apart converts one without
//! the wildcard arm a `match` outside this crate would need.
#![doc(test(attr(deny(warnings))))]

mod error;

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use jiff::civil;

pub use error::{CalendarError, FromCalendarError};

/// A calendar date in the proleptic Gregorian calendar, between years 1 and
/// 9999.
///
/// The text form is ISO 8601 `YYYY-MM-DD`, and ordering is chronological.
///
/// # Examples
///
/// ```
/// use eranos::CalendarDate;
///
/// let date = CalendarDate::new(23, 9, 2026)?;
/// assert_eq!(date.to_string(), "2026-09-23");
/// assert_eq!("2026-09-23".parse::<CalendarDate>()?, date);
/// assert!(CalendarDate::new(29, 2, 2026).is_err()); // 2026 is not a leap year
/// # Ok::<(), eranos::error::CalendarError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarDate {
    /// Always in years 1..=9999: jiff's dates start at year -9999, and
    /// `from_civil` refuses the years before 1.
    inner: civil::Date,
}

// Written out, like every ordering here, so that it never depends on the
// order of the fields; jiff orders dates chronologically.
impl Ord for CalendarDate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl PartialOrd for CalendarDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl CalendarDate {
    /// Returns the date `day` `month` `year`: `new(24, 9, 2026)` is 24
    /// September 2026.
    ///
    /// # Errors
    ///
    /// [`CalendarError::InvalidDate`] when the year is outside 1..=9999 or the
    /// day does not exist in that month.
    pub fn new(day: u8, month: u8, year: u16) -> Result<Self, CalendarError> {
        // A day or a month above `i8::MAX`, or a year above `i16::MAX`, is in
        // no calendar jiff represents.
        let (Ok(day), Ok(month), Ok(year)) =
            (i8::try_from(day), i8::try_from(month), i16::try_from(year))
        else {
            return Err(CalendarError::InvalidDate);
        };
        civil::Date::new(year, month, day)
            .ok()
            .and_then(Self::from_civil)
            .ok_or(CalendarError::InvalidDate)
    }

    /// Parses an ISO 8601 date such as `2026-09-24`, ignoring surrounding
    /// whitespace.
    ///
    /// jiff's parser also reads the basic form `20260924`, an expanded year
    /// such as `+002026-09-24`, and a date followed by a time, an offset, and
    /// annotations, such as `2026-09-24T18:30+02:00[Europe/Bucharest]`, of
    /// which only the date is kept. A calendar annotation such as
    /// `[u-ca=hebrew]` is read but not applied: the date stays Gregorian.
    ///
    /// # Errors
    ///
    /// [`CalendarError::InvalidDate`] when the text is not such a date, names a
    /// day that does not exist or a year before 1, or carries the `Z` of a UTC
    /// time.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::CalendarDate;
    ///
    /// assert_eq!(CalendarDate::parse("2028-02-29")?.day(), 29);
    /// assert_eq!(CalendarDate::parse("2026-09-24T18:30+02:00")?, CalendarDate::new(24, 9, 2026)?);
    /// assert!(CalendarDate::parse("2026-9-23").is_err());
    /// assert!(CalendarDate::parse("0000-01-01").is_err());
    /// # Ok::<(), eranos::error::CalendarError>(())
    /// ```
    pub fn parse(text: &str) -> Result<Self, CalendarError> {
        text.trim()
            .parse::<civil::Date>()
            .ok()
            .and_then(Self::from_civil)
            .ok_or(CalendarError::InvalidDate)
    }

    /// Keeps a jiff date from year 1 on.
    fn from_civil(inner: civil::Date) -> Option<Self> {
        (inner.year() >= 1).then_some(Self { inner })
    }

    /// The day of the month, starting at 1.
    #[must_use]
    pub fn day(self) -> u8 {
        // jiff's days run from 1 to 31, so the cast keeps the value.
        self.inner.day().cast_unsigned()
    }

    /// The month, 1 (January) to 12 (December).
    #[must_use]
    pub fn month(self) -> u8 {
        // jiff's months run from 1 to 12, so the cast keeps the value.
        self.inner.month().cast_unsigned()
    }

    /// The year, 1 to 9999.
    #[must_use]
    pub fn year(self) -> u16 {
        // `from_civil` refuses the years before 1 and jiff's end at 9999, so
        // the cast keeps the value.
        self.inner.year().cast_unsigned()
    }
}

// The text form, not the jiff value inside.
impl fmt::Debug for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

// Written here rather than taken from jiff: it is the saved form, so it must
// not change with a dependency's release.
impl fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year(),
            self.month(),
            self.day()
        )
    }
}

impl FromStr for CalendarDate {
    type Err = CalendarError;

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
/// use eranos::TimeOfDay;
///
/// let time = TimeOfDay::new(18, 30)?;
/// assert_eq!(time.to_string(), "18:30");
/// assert_eq!(TimeOfDay::parse("07:05")?.hour(), 7);
/// assert!(TimeOfDay::new(24, 0).is_err());
/// # Ok::<(), eranos::error::CalendarError>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeOfDay {
    /// Always on a whole minute: `new` sets no seconds, and `parse` refuses a
    /// time with seconds or a fraction of one.
    inner: civil::Time,
}

// Written out, like the ordering of `CalendarDate`; jiff orders times
// chronologically.
impl Ord for TimeOfDay {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl PartialOrd for TimeOfDay {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TimeOfDay {
    /// Returns the time `hour`:`minute`.
    ///
    /// # Errors
    ///
    /// [`CalendarError::InvalidTime`] when the hour is above 23 or the minute
    /// above 59.
    pub fn new(hour: u8, minute: u8) -> Result<Self, CalendarError> {
        // An hour or a minute above `i8::MAX` is in no day.
        let (Ok(hour), Ok(minute)) = (i8::try_from(hour), i8::try_from(minute)) else {
            return Err(CalendarError::InvalidTime);
        };
        civil::Time::new(hour, minute, 0, 0)
            .ok()
            .map(|inner| Self { inner })
            .ok_or(CalendarError::InvalidTime)
    }

    /// Parses an ISO 8601 time of day such as `18:30`, ignoring surrounding
    /// whitespace.
    ///
    /// jiff's parser also reads zero seconds (`18:30:00`), the `T` designator
    /// (`T1830`), the hour alone (`18` for 18:00), and the time of a date and
    /// time, or one followed by an offset or annotations
    /// (`2026-09-24T18:30+02:00`).
    ///
    /// # Errors
    ///
    /// [`CalendarError::InvalidTime`] when the text is not such a time, names one
    /// that does not exist, has seconds or a fraction of a second, or carries
    /// the `Z` of a UTC time.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::TimeOfDay;
    ///
    /// assert_eq!(TimeOfDay::parse("07:05")?, TimeOfDay::new(7, 5)?);
    /// assert_eq!(TimeOfDay::parse("T07:05:00")?, TimeOfDay::new(7, 5)?);
    /// assert!(TimeOfDay::parse("07:05:30").is_err());
    /// # Ok::<(), eranos::error::CalendarError>(())
    /// ```
    pub fn parse(text: &str) -> Result<Self, CalendarError> {
        text.trim()
            .parse::<civil::Time>()
            .ok()
            .filter(|inner| inner.second() == 0 && inner.subsec_nanosecond() == 0)
            .map(|inner| Self { inner })
            .ok_or(CalendarError::InvalidTime)
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub fn hour(self) -> u8 {
        // jiff's hours run from 0 to 23, so the cast keeps the value.
        self.inner.hour().cast_unsigned()
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub fn minute(self) -> u8 {
        // jiff's minutes run from 0 to 59, so the cast keeps the value.
        self.inner.minute().cast_unsigned()
    }
}

// The text form, not the jiff value inside.
impl fmt::Debug for TimeOfDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

// Written here rather than taken from jiff, whose form has seconds: it is the
// saved form, so it must not change with a dependency's release.
impl fmt::Display for TimeOfDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour(), self.minute())
    }
}

impl FromStr for TimeOfDay {
    type Err = CalendarError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse(text)
    }
}

#[cfg(feature = "serde")]
mod serde_impls {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{CalendarDate, TimeOfDay};

    impl Serialize for CalendarDate {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for CalendarDate {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            eranos_core::serde_text::deserialise(
                deserialiser,
                "an ISO 8601 date such as `2026-09-24`",
            )
        }
    }

    impl Serialize for TimeOfDay {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            serialiser.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for TimeOfDay {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            eranos_core::serde_text::deserialise(deserialiser, "an ISO 8601 time such as `18:30`")
        }
    }
}

#[cfg(test)]
mod tests;
