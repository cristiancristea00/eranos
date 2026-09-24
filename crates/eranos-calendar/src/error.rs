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
