//! Calendar dates and times of day, as the bindings carry them.

use eranos::calendar as library;
use eranos::error::CalendarError;

use crate::EranosError;

/// A day of the calendar, without a time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarDate {
    /// The day of the month, from 1.
    pub day: i32,
    /// The month, from 1 for January.
    pub month: i32,
    /// The year, from 1 to 9999.
    pub year: i32,
}

impl CalendarDate {
    /// The record of `date`.
    pub(crate) fn from_library(date: library::CalendarDate) -> Self {
        Self {
            day: i32::from(date.day()),
            month: i32::from(date.month()),
            year: i32::from(date.year()),
        }
    }

    /// The library's date.
    ///
    /// # Errors
    ///
    /// [`CalendarFailure::InvalidDate`](crate::CalendarFailure::InvalidDate)
    /// for a day that does not exist, a number too large for its field
    /// included.
    pub(crate) fn to_library(self) -> Result<library::CalendarDate, EranosError> {
        let invalid = |_| CalendarError::InvalidDate;
        Ok(library::CalendarDate::new(
            u8::try_from(self.day).map_err(invalid)?,
            u8::try_from(self.month).map_err(invalid)?,
            u16::try_from(self.year).map_err(invalid)?,
        )?)
    }
}

/// A time of day, to the minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeOfDay {
    /// The hour, from 0 to 23.
    pub hour: i32,
    /// The minute, from 0 to 59.
    pub minute: i32,
}

impl TimeOfDay {
    /// The record of `time`.
    pub(crate) fn from_library(time: library::TimeOfDay) -> Self {
        Self {
            hour: i32::from(time.hour()),
            minute: i32::from(time.minute()),
        }
    }

    /// The library's time of day.
    ///
    /// # Errors
    ///
    /// [`CalendarFailure::InvalidTime`](crate::CalendarFailure::InvalidTime)
    /// for an hour or a minute out of range.
    pub(crate) fn to_library(self) -> Result<library::TimeOfDay, EranosError> {
        let invalid = |_| CalendarError::InvalidTime;
        Ok(library::TimeOfDay::new(
            u8::try_from(self.hour).map_err(invalid)?,
            u8::try_from(self.minute).map_err(invalid)?,
        )?)
    }
}

#[cfg(test)]
mod tests;
