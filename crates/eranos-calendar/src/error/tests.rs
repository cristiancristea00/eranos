//! Every failure of this crate converts through `FromCalendarError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

/// One variant in a table of conversions: the error, what the conversion
/// finds it holding, and the method it converts through.
#[derive(Debug)]
struct Case {
    error: CalendarError,
    held: String,
    method: &'static str,
}

/// A [`CalendarError`] as a crate outside this one converts it: the method
/// that built the value, what the variant held, and the message.
#[derive(Debug, PartialEq, Eq)]
struct ConvertedCalendarError {
    held: String,
    message: String,
    method: &'static str,
}

impl FromCalendarError for ConvertedCalendarError {
    fn invalid_date(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_date",
        }
    }

    fn invalid_time(message: String) -> Self {
        Self {
            held: String::new(),
            message,
            method: "invalid_time",
        }
    }
}

/// A conversion that called another variant's method, or dropped what the
/// variant holds or its message, would show the apps another failure than the
/// library's.
#[test]
fn every_failure_of_calendar_error_converts_through_the_method_of_its_variant() {
    let cases = [
        Case {
            error: CalendarError::InvalidDate,
            held: String::new(),
            method: "invalid_date",
        },
        Case {
            error: CalendarError::InvalidTime,
            held: String::new(),
            method: "invalid_time",
        },
    ];
    for case in cases {
        assert_eq!(
            ConvertedCalendarError::from_calendar_error(case.error),
            ConvertedCalendarError {
                held: case.held,
                message: case.error.to_string(),
                method: case.method,
            }
        );
    }
}
