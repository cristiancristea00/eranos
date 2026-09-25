//! Dates and times cross the bindings as integers. A date converted back and
//! forth would change, or a number too large for its field would wrap into
//! a date or time that exists instead of being refused.

use super::*;

/// A date or time would come back changed from the library, or a number
/// too large for its field would wrap into one that exists.
#[test]
fn dates_and_times_convert_both_ways_and_refuse_what_does_not_exist() {
    let date = CalendarDate {
        day: 29,
        month: 2,
        year: 2028,
    };
    assert_eq!(date.to_library().map(CalendarDate::from_library), Ok(date));
    let time = TimeOfDay {
        hour: 23,
        minute: 59,
    };
    assert_eq!(time.to_library().map(TimeOfDay::from_library), Ok(time));

    let refused_dates = [
        // No 29 February in 2027, a day too large for any field, a
        // negative month, and a year beyond 9999 that would wrap.
        CalendarDate {
            day: 29,
            month: 2,
            year: 2027,
        },
        CalendarDate {
            day: 257,
            month: 1,
            year: 2026,
        },
        CalendarDate {
            day: 1,
            month: -1,
            year: 2026,
        },
        CalendarDate {
            day: 1,
            month: 1,
            year: 70_000,
        },
    ];
    for impossible in refused_dates {
        assert!(
            matches!(
                impossible.to_library(),
                Err(EranosError::InvalidDate { .. })
            ),
            "{impossible:?}"
        );
    }
    let refused_times = [
        TimeOfDay {
            hour: 24,
            minute: 0,
        },
        TimeOfDay {
            hour: 300,
            minute: 0,
        },
        TimeOfDay {
            hour: 0,
            minute: -1,
        },
    ];
    for impossible in refused_times {
        assert!(
            matches!(
                impossible.to_library(),
                Err(EranosError::InvalidTime { .. })
            ),
            "{impossible:?}"
        );
    }
}
