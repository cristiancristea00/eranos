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
//! the wildcard arm a `match` outside this crate would need, nor
//! [`format_calendar_date`] and [`format_time_of_day`], with which the
//! binding core writes a date and a time of day from the fields its records
//! hold.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;

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
/// let date = CalendarDate::new(2026, 9, 23)?;
/// assert_eq!(date.to_string(), "2026-09-23");
/// assert_eq!("2026-09-23".parse::<CalendarDate>()?, date);
/// assert!(CalendarDate::new(2026, 2, 29).is_err()); // 2026 is not a leap year
/// # Ok::<(), eranos::error::CalendarError>(())
/// ```
// Derived: jiff orders dates chronologically.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Always in years 1..=9999: jiff's dates start at year -9999, and
    /// `from_civil` refuses the years before 1.
    inner: civil::Date,
}

impl CalendarDate {
    /// Returns the date `year`-`month`-`day`, in the order ISO 8601 writes
    /// it: `new(2026, 9, 24)` is 24 September 2026.
    ///
    /// # Errors
    ///
    /// [`CalendarError::InvalidDate`] when the year is outside 1..=9999 or the
    /// day does not exist in that month.
    pub fn new(year: u16, month: u8, day: u8) -> Result<Self, CalendarError> {
        // A year above `i16::MAX`, or a month or a day above `i8::MAX`, is in
        // no calendar jiff represents.
        let (Ok(year), Ok(month), Ok(day)) =
            (i16::try_from(year), i8::try_from(month), i8::try_from(day))
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
    /// assert_eq!(CalendarDate::parse("2026-09-24T18:30+02:00")?, CalendarDate::new(2026, 9, 24)?);
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
        (1 <= inner.year()).then_some(Self { inner })
    }

    /// The year, 1 to 9999.
    #[must_use]
    pub fn year(self) -> u16 {
        // `from_civil` refuses the years before 1 and jiff's end at 9999, so
        // the cast keeps the value.
        self.inner.year().cast_unsigned()
    }

    /// The month, 1 (January) to 12 (December).
    #[must_use]
    pub fn month(self) -> u8 {
        // jiff's months run from 1 to 12, so the cast keeps the value.
        self.inner.month().cast_unsigned()
    }

    /// The day of the month, starting at 1.
    #[must_use]
    pub fn day(self) -> u8 {
        // jiff's days run from 1 to 31, so the cast keeps the value.
        self.inner.day().cast_unsigned()
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
        fmt::Display::fmt(
            &format_calendar_date(
                i32::from(self.year()),
                i32::from(self.month()),
                i32::from(self.day()),
            ),
            formatter,
        )
    }
}

/// Formats a date's fields as the text form of [`CalendarDate`] writes
/// them: the year in four digits, the month and the day in two, padded with
/// zeros and joined by hyphens.
///
/// The text form of a date writes its fields with it. It takes any numbers,
/// those of no date included, and writes them the same way: the binding
/// core writes with it the fields of a record it has not checked, so that
/// its text for a date is this crate's. A number wider than its field is
/// written whole, and a negative one keeps its sign within the width.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes a date through its `CalendarDate`.
/// use eranos_calendar::format_calendar_date;
///
/// assert_eq!(format_calendar_date(2026, 9, 3).to_string(), "2026-09-03");
/// assert_eq!(format_calendar_date(12, 13, 32).to_string(), "0012-13-32");
/// assert_eq!(format_calendar_date(-1, 0, 100).to_string(), "-001-00-100");
/// ```
#[must_use]
pub fn format_calendar_date(year: i32, month: i32, day: i32) -> impl fmt::Display {
    fmt::from_fn(move |formatter| write!(formatter, "{year:04}-{month:02}-{day:02}"))
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
// Derived: jiff orders times chronologically.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeOfDay {
    /// Always on a whole minute: `new` sets no seconds, and `parse` refuses a
    /// time with seconds or a fraction of one.
    inner: civil::Time,
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
            .filter(|inner| 0 == inner.second() && 0 == inner.subsec_nanosecond())
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
        fmt::Display::fmt(
            &format_time_of_day(i32::from(self.hour()), i32::from(self.minute())),
            formatter,
        )
    }
}

/// Formats a time's fields as the text form of [`TimeOfDay`] writes them:
/// the hour and the minute in two digits each, padded with zeros and joined
/// by a colon.
///
/// The text form of a time writes its fields with it. It takes any numbers,
/// those of no time included, and writes them the same way: the binding
/// core writes with it the fields of a record it has not checked, so that
/// its text for a time is this crate's. A number wider than its field is
/// written whole, and a negative one keeps its sign within the width.
///
/// # Examples
///
/// ```
/// # // The facade does not re-export it: a caller writes a time through its `TimeOfDay`.
/// use eranos_calendar::format_time_of_day;
///
/// assert_eq!(format_time_of_day(7, 5).to_string(), "07:05");
/// assert_eq!(format_time_of_day(24, 60).to_string(), "24:60");
/// assert_eq!(format_time_of_day(-1, 100).to_string(), "-1:100");
/// ```
#[must_use]
pub fn format_time_of_day(hour: i32, minute: i32) -> impl fmt::Display {
    fmt::from_fn(move |formatter| write!(formatter, "{hour:02}:{minute:02}"))
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
