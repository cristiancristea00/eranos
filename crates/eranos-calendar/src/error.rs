//! The failure of building or reading a calendar date or a time of day.

/// A calendar date or a time of day that could not be built or read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[non_exhaustive]
pub enum CalendarError {
    /// A calendar date that does not exist, is outside years 1 to 9999, or is
    /// not ISO 8601 text.
    #[error("invalid date: expected an existing ISO 8601 date between years 1 and 9999")]
    InvalidDate,
    /// A time of day that does not exist, has seconds, or is not ISO 8601
    /// text.
    #[error("invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds")]
    InvalidTime,
}

/// Builds a value from a [`CalendarError`], one method per failure, each
/// given what its variant holds and the error's message.
pub trait FromCalendarError: Sized {
    /// The value for [`CalendarError::InvalidDate`].
    fn invalid_date(message: String) -> Self;

    /// The value for [`CalendarError::InvalidTime`].
    fn invalid_time(message: String) -> Self;

    /// The value for `error`, from the method of its variant, given the error's
    /// message.
    #[must_use]
    fn from_calendar_error(error: CalendarError) -> Self {
        let message = error.to_string();
        match error {
            CalendarError::InvalidDate => Self::invalid_date(message),
            CalendarError::InvalidTime => Self::invalid_time(message),
        }
    }
}

#[cfg(test)]
mod tests;
