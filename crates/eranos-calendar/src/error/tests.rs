//! Every failure of this crate converts through `FromCalendarError`, the
//! trait through which a crate outside this one, such as the binding core,
//! reports it. These tests keep each variant reaching its own method with
//! what it holds and its message.

use super::*;

include!("../../../eranos-core/src/error/conversions.rs");

conversion_test! {
    /// A conversion that called another variant's method, or dropped what the
    /// variant holds or its message, would show the apps another failure than the
    /// library's.
    fn every_failure_of_calendar_error_converts_through_the_method_of_its_variant: CalendarError => ConvertedCalendarError through FromCalendarError::from_calendar_error() {
        methods {
            invalid_date() => String::new();
            invalid_time() => String::new();
        }
        setup {}
        cases {
            CalendarError::InvalidDate => invalid_date, String::new();
            CalendarError::InvalidTime => invalid_time, String::new();
        }
    }
}
