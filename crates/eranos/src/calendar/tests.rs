//! Expense and transfer dates come from user input and from files synced
//! between devices. These tests keep impossible dates and times out, pin
//! which ISO 8601 forms jiff's parser reads, keep the text form exact so a
//! saved date reads back as the same day, and keep ordering chronological,
//! which every list sorted by date relies on.

use super::*;

use proptest::prelude::*;

/// The last day of every month of 2026, a common year.
const MONTH_LENGTHS: [(u8, u8); 12] = [
    (1, 31),
    (2, 28),
    (3, 31),
    (4, 30),
    (5, 31),
    (6, 30),
    (7, 31),
    (8, 31),
    (9, 30),
    (10, 31),
    (11, 30),
    (12, 31),
];

fn date(year: u16, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(day, month, year).unwrap()
}

/// Any existing date between years 1 and 9999.
fn any_date() -> impl Strategy<Value = CalendarDate> {
    (1_u16..=9999, 1_u8..=12, 1_u8..=31)
        .prop_filter_map("the day must exist in its month", |(year, month, day)| {
            CalendarDate::new(day, month, year).ok()
        })
}

/// An impossible date such as 31 April would be stored and break every
/// calculation by day.
#[test]
fn a_day_exists_only_within_its_month() {
    // The last day of every month exists, the day after it does not.
    for (month, last_day) in MONTH_LENGTHS {
        assert!(CalendarDate::new(last_day, month, 2026).is_ok(), "{month}");
        assert_eq!(
            CalendarDate::new(last_day + 1, month, 2026),
            Err(ValueError::InvalidDate),
            "{month}"
        );
    }
    // Day zero exists in no month.
    for (month, _) in MONTH_LENGTHS {
        assert_eq!(
            CalendarDate::new(0, month, 2026),
            Err(ValueError::InvalidDate)
        );
    }
    // The first day of every month exists.
    for (month, _) in MONTH_LENGTHS {
        assert!(CalendarDate::new(1, month, 2026).is_ok());
    }
}

/// A date in month 0 or 13 would be stored and shown as garbage.
#[test]
fn months_run_from_1_to_12() {
    // Inside the range.
    assert!(CalendarDate::new(1, 1, 2026).is_ok());
    assert!(CalendarDate::new(31, 12, 2026).is_ok());
    // Outside the range.
    assert_eq!(CalendarDate::new(1, 0, 2026), Err(ValueError::InvalidDate));
    assert_eq!(CalendarDate::new(1, 13, 2026), Err(ValueError::InvalidDate));
    assert_eq!(
        CalendarDate::new(1, u8::MAX, 2026),
        Err(ValueError::InvalidDate)
    );
}

/// Expenses on 29 February would be rejected in leap years or accepted in
/// common ones.
#[test]
fn a_leap_day_exists_only_in_leap_years() {
    // Years divisible by 4.
    assert!(CalendarDate::new(29, 2, 2028).is_ok());
    assert!(CalendarDate::new(29, 2, 2024).is_ok());
    assert!(CalendarDate::new(29, 2, 4).is_ok());
    // Years not divisible by 4.
    assert_eq!(CalendarDate::new(29, 2, 2026), Err(ValueError::InvalidDate));
    assert_eq!(CalendarDate::new(29, 2, 2027), Err(ValueError::InvalidDate));
    assert_eq!(CalendarDate::new(29, 2, 1), Err(ValueError::InvalidDate));
    // Centuries are common years unless divisible by 400.
    assert_eq!(CalendarDate::new(29, 2, 1900), Err(ValueError::InvalidDate));
    assert_eq!(CalendarDate::new(29, 2, 2100), Err(ValueError::InvalidDate));
    assert!(CalendarDate::new(29, 2, 2000).is_ok());
    assert!(CalendarDate::new(29, 2, 2400).is_ok());
    // No leap year has a 30 February.
    assert_eq!(CalendarDate::new(30, 2, 2028), Err(ValueError::InvalidDate));
}

/// A year outside four digits would not fit the `YYYY-MM-DD` text form.
#[test]
fn years_run_from_1_to_9999() {
    // Inside the range.
    assert!(CalendarDate::new(1, 1, 1).is_ok());
    assert!(CalendarDate::new(31, 12, 9999).is_ok());
    // Outside the range.
    assert_eq!(CalendarDate::new(1, 1, 0), Err(ValueError::InvalidDate));
    assert_eq!(
        CalendarDate::new(1, 1, 10_000),
        Err(ValueError::InvalidDate)
    );
    assert_eq!(
        CalendarDate::new(1, 1, u16::MAX),
        Err(ValueError::InvalidDate)
    );
}

/// The apps would show the wrong day, month, or year of an expense.
#[test]
fn accessors_return_the_components() {
    // Year, month, and day.
    let september_23 = date(2026, 9, 23);
    assert_eq!(september_23.year(), 2026);
    assert_eq!(september_23.month(), 9);
    assert_eq!(september_23.day(), 23);
}

/// Dates typed or synced as `YYYY-MM-DD` would fail to load.
#[test]
fn parse_reads_the_iso_format() {
    // Exact format.
    assert_eq!(CalendarDate::parse("2026-09-23"), Ok(date(2026, 9, 23)));
    assert_eq!(CalendarDate::parse("0001-01-01"), Ok(date(1, 1, 1)));
    assert_eq!(CalendarDate::parse("9999-12-31"), Ok(date(9999, 12, 31)));
    // Surrounding whitespace is ignored.
    assert_eq!(
        CalendarDate::parse(" \t2026-09-23\n"),
        Ok(date(2026, 9, 23))
    );
    // `FromStr` parses the same way.
    assert_eq!("2028-02-29".parse::<CalendarDate>(), Ok(date(2028, 2, 29)));
}

/// An ambiguous date such as `2026-9-2` or `09/02/2026` would be read as
/// the wrong day.
#[test]
fn parse_rejects_other_shapes() {
    // Missing padding.
    for text in ["2026-9-23", "2026-09-3", "26-09-23", "2026-9-3"] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
    // Other separators.
    for text in ["2026/09/23", "2026.09.23", "2026 09 23", "2026-09 23"] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
    // Extra characters, and a sign on a four-digit year.
    for text in ["2026-09-23x", "x2026-09-23", "+2026-09-23"] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
    // The UTC designator: the date would depend on the reader's time zone.
    assert_eq!(
        CalendarDate::parse("2026-09-24T18:30:00Z"),
        Err(ValueError::InvalidDate)
    );
    // A byte-order mark, which `trim` keeps.
    assert_eq!(
        CalendarDate::parse("\u{feff}2026-09-24"),
        Err(ValueError::InvalidDate)
    );
    // Non-digits, including the characters on either side of `0`-`9`.
    for text in [
        "2a26-09-23",
        "/026-09-23",
        "202:-09-23",
        "2026-0a-23",
        "2026-/9-23",
        "2026-09-2:",
        "2026--9-23",
        "２０２６-09-23",
    ] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
    // Nothing at all.
    for text in ["", "   "] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text:?}"
        );
    }
}

/// Dates pasted from calendars and other apps, which write the basic form or
/// a whole date and time, would be refused.
#[test]
fn parse_reads_the_other_iso_8601_forms() {
    let september_24 = date(2026, 9, 24);
    // The basic form, without separators.
    assert_eq!(CalendarDate::parse("20260923"), Ok(date(2026, 9, 23)));
    assert_eq!(CalendarDate::parse("20260924"), Ok(september_24));
    // A date and time, of which only the date is kept.
    for text in [
        "2026-09-24T18:30",
        "2026-09-24T00:00",
        "2026-09-24T00:00:00",
        "2026-09-24t18:30",
        "2026-09-24 18:30",
        "2026-09-24T23:59:60",
    ] {
        assert_eq!(CalendarDate::parse(text), Ok(september_24), "{text}");
    }
    // A date and time with an offset or annotations. The calendar annotation
    // is read but not applied: the date stays Gregorian.
    for text in [
        "2026-09-24T18:30:00+02:00",
        "2026-09-24T18:30:00+00",
        "2026-09-24T18:30+02:00[Europe/Bucharest]",
        "2026-09-24[u-ca=iso8601]",
        "2026-09-24[u-ca=hebrew]",
    ] {
        assert_eq!(CalendarDate::parse(text), Ok(september_24), "{text}");
    }
    // An expanded year, and the basic form with surrounding whitespace.
    assert_eq!(CalendarDate::parse("+002026-09-24"), Ok(september_24));
    assert_eq!(CalendarDate::parse(" 20260924\n"), Ok(september_24));
}

/// A date before year 1, which jiff's parser reads, would be stored although
/// the year does not fit the `YYYY-MM-DD` text form.
#[test]
fn parse_rejects_years_before_1() {
    // Year 0 and negative years, in the forms jiff reads.
    for text in [
        "0000-01-01",
        "0000-02-29",
        "+000000-01-01",
        "-000001-01-01",
        "-002026-09-24",
    ] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
}

/// A well-formed but impossible date would be stored.
#[test]
fn parse_rejects_dates_that_do_not_exist() {
    // Wrong day, month, or year.
    for text in [
        "2026-02-29",
        "2026-04-31",
        "2026-09-00",
        "2026-13-01",
        "2026-00-10",
        "0000-01-01",
    ] {
        assert_eq!(
            CalendarDate::parse(text),
            Err(ValueError::InvalidDate),
            "{text}"
        );
    }
}

/// Saved dates would not be `YYYY-MM-DD` and would not read back.
#[test]
fn display_pads_every_component() {
    // The smallest date.
    assert_eq!(date(1, 1, 1).to_string(), "0001-01-01");
    // Single-digit month and day.
    assert_eq!(date(2026, 9, 3).to_string(), "2026-09-03");
    // The largest date.
    assert_eq!(date(9999, 12, 31).to_string(), "9999-12-31");
}

/// Expenses listed by date would appear out of order.
#[test]
fn dates_order_chronologically() {
    // The day decides within a month.
    assert!(date(2026, 9, 23) < date(2026, 9, 24));
    // The month decides before the day.
    assert!(date(2026, 1, 31) < date(2026, 2, 1));
    // The year decides before the month.
    assert!(date(2026, 12, 31) < date(2027, 1, 1));
    // The whole range.
    assert!(date(1, 1, 1) < date(9999, 12, 31));
    // Equal dates.
    assert_eq!(date(2026, 9, 23).cmp(&date(2026, 9, 23)), Ordering::Equal);
}

/// A time of day such as 24:00 or 18:60 would be stored.
#[test]
fn a_time_runs_from_midnight_to_23_59() {
    // The first and the last minute.
    assert!(TimeOfDay::new(0, 0).is_ok());
    assert!(TimeOfDay::new(23, 59).is_ok());
    // One past either bound.
    assert_eq!(TimeOfDay::new(24, 0), Err(ValueError::InvalidTime));
    assert_eq!(TimeOfDay::new(0, 60), Err(ValueError::InvalidTime));
    assert_eq!(TimeOfDay::new(23, 60), Err(ValueError::InvalidTime));
    // Far outside.
    assert_eq!(
        TimeOfDay::new(u8::MAX, u8::MAX),
        Err(ValueError::InvalidTime)
    );
}

/// Times typed or synced as `hh:mm` would fail to load.
#[test]
fn time_parse_reads_two_digit_hours_and_minutes() {
    // Exact format.
    let time = TimeOfDay::parse("07:05").unwrap();
    assert_eq!((time.hour(), time.minute()), (7, 5));
    assert_eq!(TimeOfDay::parse("23:59"), TimeOfDay::new(23, 59));
    // Surrounding whitespace is ignored.
    assert_eq!(TimeOfDay::parse(" 18:30\n"), TimeOfDay::new(18, 30));
    // `FromStr` parses the same way.
    assert_eq!("00:00".parse::<TimeOfDay>(), TimeOfDay::new(0, 0));
}

/// A time such as `7:5` or `07:05 pm` would be read as the wrong minute.
#[test]
fn time_parse_rejects_other_shapes() {
    // Missing padding or other separators.
    for text in ["7:05", "07:5", "0705", "07-05", "07.05", "07 05"] {
        assert_eq!(
            TimeOfDay::parse(text),
            Err(ValueError::InvalidTime),
            "{text}"
        );
    }
    // Extra characters, and the UTC designator, which makes the time depend
    // on the reader's time zone.
    for text in ["07:05 pm", "07:05Z"] {
        assert_eq!(
            TimeOfDay::parse(text),
            Err(ValueError::InvalidTime),
            "{text}"
        );
    }
    // Non-digits.
    for text in ["ab:cd", "0a:05", "07:0a", "/7:05", "07:5:"] {
        assert_eq!(
            TimeOfDay::parse(text),
            Err(ValueError::InvalidTime),
            "{text}"
        );
    }
    // Times that do not exist.
    for text in ["24:00", "23:60", "99:99"] {
        assert_eq!(
            TimeOfDay::parse(text),
            Err(ValueError::InvalidTime),
            "{text}"
        );
    }
    // Nothing at all.
    assert_eq!(TimeOfDay::parse(""), Err(ValueError::InvalidTime));
}

/// Times pasted from other apps, which write seconds, the `T` designator, or
/// a whole date and time, would be refused.
#[test]
fn time_parse_reads_the_other_iso_8601_forms() {
    let five_past_seven = TimeOfDay::new(7, 5).unwrap();
    // Zero seconds and the `T` designator.
    for text in ["07:05:00", "T07:05", "t07:05", "T0705"] {
        assert_eq!(TimeOfDay::parse(text), Ok(five_past_seven), "{text}");
    }
    // The hour alone, which is the hour on the dot.
    for text in ["07", "T07"] {
        assert_eq!(TimeOfDay::parse(text), TimeOfDay::new(7, 0), "{text}");
    }
    // A time with an offset or an annotation, and the time of a date and
    // time.
    for text in [
        "07:05+02:00",
        "07:05[Europe/Bucharest]",
        "2026-09-24T07:05",
        "2026-09-24 07:05",
    ] {
        assert_eq!(TimeOfDay::parse(text), Ok(five_past_seven), "{text}");
    }
}

/// A time with seconds would be stored as another time than the one typed,
/// since a `TimeOfDay` holds whole minutes.
#[test]
fn time_parse_rejects_seconds() {
    // Seconds, fractions of a second, and a leap second, which jiff reads as
    // 59 seconds.
    for text in [
        "07:05:30",
        "07:05:59",
        "07:05:00.5",
        "07:05:00,5",
        "07:05:00.000000001",
        "23:59:60",
        "2026-09-24T07:05:30",
    ] {
        assert_eq!(
            TimeOfDay::parse(text),
            Err(ValueError::InvalidTime),
            "{text}"
        );
    }
}

/// Saved times would not be `hh:mm` and would not read back.
#[test]
fn time_display_pads_both_components() {
    // Single digits.
    assert_eq!(TimeOfDay::new(7, 5).unwrap().to_string(), "07:05");
    // Midnight and the last minute.
    assert_eq!(TimeOfDay::new(0, 0).unwrap().to_string(), "00:00");
    assert_eq!(TimeOfDay::new(23, 59).unwrap().to_string(), "23:59");
}

/// A saved time would read back as another minute of the day.
#[test]
fn every_time_of_day_round_trips_through_its_text_form() {
    // All 1440 minutes.
    for hour in 0..24 {
        for minute in 0..60 {
            let time = TimeOfDay::new(hour, minute).unwrap();
            assert_eq!(TimeOfDay::parse(&time.to_string()), Ok(time));
        }
    }
}

/// Expenses on the same day would be listed out of order.
#[test]
fn times_order_chronologically() {
    // The minute decides within an hour.
    assert!(TimeOfDay::new(7, 5).unwrap() < TimeOfDay::new(7, 6).unwrap());
    // The hour decides before the minute.
    assert!(TimeOfDay::new(7, 59).unwrap() < TimeOfDay::new(8, 0).unwrap());
    // The whole range.
    assert!(TimeOfDay::new(0, 0).unwrap() < TimeOfDay::new(23, 59).unwrap());
}

/// Logs and test failures would show jiff's value inside instead of the date
/// or the time.
#[test]
fn dates_and_times_debug_as_their_text_form() {
    // The same text as `Display`.
    assert_eq!(format!("{:?}", date(2026, 9, 3)), "2026-09-03");
    assert_eq!(format!("{:?}", TimeOfDay::new(7, 5).unwrap()), "07:05");
    // Inside another value.
    assert_eq!(
        format!("{:?}", Some(TimeOfDay::new(7, 5).unwrap())),
        "Some(07:05)"
    );
}

/// A saved date would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn dates_serialise_as_their_text_form() {
    // Round trip.
    let september_23 = date(2026, 9, 23);
    assert_eq!(
        serde_json::to_string(&september_23).unwrap(),
        "\"2026-09-23\""
    );
    assert_eq!(
        serde_json::from_str::<CalendarDate>("\"2026-09-23\"").unwrap(),
        september_23
    );
    // Deserialising validates like parsing.
    assert!(serde_json::from_str::<CalendarDate>("\"2026-02-30\"").is_err());
    // Only strings are dates.
    assert!(serde_json::from_str::<CalendarDate>("20260923").is_err());
}

/// A saved time would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn times_serialise_as_their_text_form() {
    // Round trip.
    let time = TimeOfDay::new(7, 5).unwrap();
    assert_eq!(serde_json::to_string(&time).unwrap(), "\"07:05\"");
    assert_eq!(
        serde_json::from_str::<TimeOfDay>("\"07:05\"").unwrap(),
        time
    );
    // Deserialising validates like parsing.
    assert!(serde_json::from_str::<TimeOfDay>("\"24:00\"").is_err());
    // Only strings are times.
    assert!(serde_json::from_str::<TimeOfDay>("705").is_err());
}

proptest! {
    /// A saved date would read back as another day.
    #[test]
    fn every_valid_date_round_trips_through_its_text_form(date in any_date()) {
        let text = date.to_string();
        prop_assert_eq!(text.len(), 10);
        prop_assert_eq!(CalendarDate::parse(&text), Ok(date));
    }

    /// Dates sorted as text, as databases and file names do, would disagree
    /// with dates sorted by the library.
    #[test]
    fn date_order_matches_text_order(first in any_date(), second in any_date()) {
        prop_assert_eq!(first.cmp(&second), first.to_string().cmp(&second.to_string()));
        let first_components = (first.year(), first.month(), first.day());
        let second_components = (second.year(), second.month(), second.day());
        prop_assert_eq!(first.cmp(&second), first_components.cmp(&second_components));
    }
}
