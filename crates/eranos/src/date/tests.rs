//! Expense and transfer dates come from user input and from files synced
//! between devices. These tests keep impossible dates and times out, keep the
//! text form exact so a saved date reads back as the same day, and keep
//! ordering chronological, which every list sorted by date relies on.

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

fn date(year: u16, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

/// Any existing date between years 1 and 9999.
fn any_date() -> impl Strategy<Value = Date> {
    (1_u16..=9999, 1_u8..=12, 1_u8..=31)
        .prop_filter_map("the day must exist in its month", |(year, month, day)| {
            Date::new(year, month, day).ok()
        })
}

/// An impossible date such as 31 April would be stored and break every
/// calculation by day.
#[test]
fn a_day_exists_only_within_its_month() {
    // The last day of every month exists, the day after it does not.
    for (month, last_day) in MONTH_LENGTHS {
        assert!(Date::new(2026, month, last_day).is_ok(), "{month}");
        assert_eq!(
            Date::new(2026, month, last_day + 1),
            Err(ValueError::InvalidDate),
            "{month}"
        );
    }
    // Day zero exists in no month.
    for (month, _) in MONTH_LENGTHS {
        assert_eq!(Date::new(2026, month, 0), Err(ValueError::InvalidDate));
    }
    // The first day of every month exists.
    for (month, _) in MONTH_LENGTHS {
        assert!(Date::new(2026, month, 1).is_ok());
    }
}

/// A date in month 0 or 13 would be stored and shown as garbage.
#[test]
fn months_run_from_1_to_12() {
    // Inside the range.
    assert!(Date::new(2026, 1, 1).is_ok());
    assert!(Date::new(2026, 12, 31).is_ok());
    // Outside the range.
    assert_eq!(Date::new(2026, 0, 1), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(2026, 13, 1), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(2026, u8::MAX, 1), Err(ValueError::InvalidDate));
}

/// Expenses on 29 February would be rejected in leap years or accepted in
/// common ones.
#[test]
fn a_leap_day_exists_only_in_leap_years() {
    // Years divisible by 4.
    assert!(Date::new(2028, 2, 29).is_ok());
    assert!(Date::new(2024, 2, 29).is_ok());
    assert!(Date::new(4, 2, 29).is_ok());
    // Years not divisible by 4.
    assert_eq!(Date::new(2026, 2, 29), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(2027, 2, 29), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(1, 2, 29), Err(ValueError::InvalidDate));
    // Centuries are common years unless divisible by 400.
    assert_eq!(Date::new(1900, 2, 29), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(2100, 2, 29), Err(ValueError::InvalidDate));
    assert!(Date::new(2000, 2, 29).is_ok());
    assert!(Date::new(2400, 2, 29).is_ok());
    // No leap year has a 30 February.
    assert_eq!(Date::new(2028, 2, 30), Err(ValueError::InvalidDate));
}

/// A year outside four digits would not fit the `YYYY-MM-DD` text form.
#[test]
fn years_run_from_1_to_9999() {
    // Inside the range.
    assert!(Date::new(1, 1, 1).is_ok());
    assert!(Date::new(9999, 12, 31).is_ok());
    // Outside the range.
    assert_eq!(Date::new(0, 1, 1), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(10_000, 1, 1), Err(ValueError::InvalidDate));
    assert_eq!(Date::new(u16::MAX, 1, 1), Err(ValueError::InvalidDate));
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
    assert_eq!(Date::parse("2026-09-23"), Ok(date(2026, 9, 23)));
    assert_eq!(Date::parse("0001-01-01"), Ok(date(1, 1, 1)));
    assert_eq!(Date::parse("9999-12-31"), Ok(date(9999, 12, 31)));
    // Surrounding whitespace is ignored.
    assert_eq!(Date::parse(" \t2026-09-23\n"), Ok(date(2026, 9, 23)));
    // `FromStr` parses the same way.
    assert_eq!("2028-02-29".parse::<Date>(), Ok(date(2028, 2, 29)));
}

/// An ambiguous date such as `2026-9-2` or `09/02/2026` would be read as
/// the wrong day.
#[test]
fn parse_rejects_other_shapes() {
    // Missing padding.
    for text in ["2026-9-23", "2026-09-3", "26-09-23", "2026-9-3"] {
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text}");
    }
    // Other separators.
    for text in [
        "2026/09/23",
        "2026.09.23",
        "20260923",
        "2026 09 23",
        "2026-09 23",
    ] {
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text}");
    }
    // Extra characters.
    for text in [
        "2026-09-23x",
        "x2026-09-23",
        "2026-09-23T18:30",
        "+2026-09-23",
    ] {
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text}");
    }
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
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text}");
    }
    // Nothing at all.
    for text in ["", "   "] {
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text:?}");
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
        assert_eq!(Date::parse(text), Err(ValueError::InvalidDate), "{text}");
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
    assert_eq!(
        date(2026, 9, 23).cmp(&date(2026, 9, 23)),
        std::cmp::Ordering::Equal
    );
}

/// A time of day such as 24:00 or 18:60 would be stored.
#[test]
fn a_time_runs_from_midnight_to_23_59() {
    // The first and the last minute.
    assert!(Time::new(0, 0).is_ok());
    assert!(Time::new(23, 59).is_ok());
    // One past either bound.
    assert_eq!(Time::new(24, 0), Err(ValueError::InvalidTime));
    assert_eq!(Time::new(0, 60), Err(ValueError::InvalidTime));
    assert_eq!(Time::new(23, 60), Err(ValueError::InvalidTime));
    // Far outside.
    assert_eq!(Time::new(u8::MAX, u8::MAX), Err(ValueError::InvalidTime));
}

/// Times typed or synced as `hh:mm` would fail to load.
#[test]
fn time_parse_reads_two_digit_hours_and_minutes() {
    // Exact format.
    let time = Time::parse("07:05").unwrap();
    assert_eq!((time.hour(), time.minute()), (7, 5));
    assert_eq!(Time::parse("23:59"), Time::new(23, 59));
    // Surrounding whitespace is ignored.
    assert_eq!(Time::parse(" 18:30\n"), Time::new(18, 30));
    // `FromStr` parses the same way.
    assert_eq!("00:00".parse::<Time>(), Time::new(0, 0));
}

/// A time such as `7:5` or `07:05:00` would be read as the wrong minute.
#[test]
fn time_parse_rejects_other_shapes() {
    // Missing padding or other separators.
    for text in ["7:05", "07:5", "0705", "07-05", "07.05", "07 05"] {
        assert_eq!(Time::parse(text), Err(ValueError::InvalidTime), "{text}");
    }
    // Extra characters.
    for text in ["07:05:00", "07:05 pm", "T07:05"] {
        assert_eq!(Time::parse(text), Err(ValueError::InvalidTime), "{text}");
    }
    // Non-digits.
    for text in ["ab:cd", "0a:05", "07:0a", "/7:05", "07:5:"] {
        assert_eq!(Time::parse(text), Err(ValueError::InvalidTime), "{text}");
    }
    // Times that do not exist.
    for text in ["24:00", "23:60", "99:99"] {
        assert_eq!(Time::parse(text), Err(ValueError::InvalidTime), "{text}");
    }
    // Nothing at all.
    assert_eq!(Time::parse(""), Err(ValueError::InvalidTime));
}

/// Saved times would not be `hh:mm` and would not read back.
#[test]
fn time_display_pads_both_components() {
    // Single digits.
    assert_eq!(Time::new(7, 5).unwrap().to_string(), "07:05");
    // Midnight and the last minute.
    assert_eq!(Time::new(0, 0).unwrap().to_string(), "00:00");
    assert_eq!(Time::new(23, 59).unwrap().to_string(), "23:59");
}

/// A saved time would read back as another minute of the day.
#[test]
fn every_time_of_day_round_trips_through_its_text_form() {
    // All 1440 minutes.
    for hour in 0..24 {
        for minute in 0..60 {
            let time = Time::new(hour, minute).unwrap();
            assert_eq!(Time::parse(&time.to_string()), Ok(time));
        }
    }
}

/// Expenses on the same day would be listed out of order.
#[test]
fn times_order_chronologically() {
    // The minute decides within an hour.
    assert!(Time::new(7, 5).unwrap() < Time::new(7, 6).unwrap());
    // The hour decides before the minute.
    assert!(Time::new(7, 59).unwrap() < Time::new(8, 0).unwrap());
    // The whole range.
    assert!(Time::new(0, 0).unwrap() < Time::new(23, 59).unwrap());
}

/// A saved date would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn dates_serialize_as_their_text_form() {
    // Round trip.
    let september_23 = date(2026, 9, 23);
    assert_eq!(
        serde_json::to_string(&september_23).unwrap(),
        "\"2026-09-23\""
    );
    assert_eq!(
        serde_json::from_str::<Date>("\"2026-09-23\"").unwrap(),
        september_23
    );
    // Deserializing validates like parsing.
    assert!(serde_json::from_str::<Date>("\"2026-02-30\"").is_err());
    // Only strings are dates.
    assert!(serde_json::from_str::<Date>("20260923").is_err());
}

/// A saved time would fail to load, or an impossible one would load.
#[cfg(feature = "serde")]
#[test]
fn times_serialize_as_their_text_form() {
    // Round trip.
    let time = Time::new(7, 5).unwrap();
    assert_eq!(serde_json::to_string(&time).unwrap(), "\"07:05\"");
    assert_eq!(serde_json::from_str::<Time>("\"07:05\"").unwrap(), time);
    // Deserializing validates like parsing.
    assert!(serde_json::from_str::<Time>("\"24:00\"").is_err());
    // Only strings are times.
    assert!(serde_json::from_str::<Time>("705").is_err());
}

proptest! {
    /// A saved date would read back as another day.
    #[test]
    fn every_valid_date_round_trips_through_its_text_form(date in any_date()) {
        let text = date.to_string();
        prop_assert_eq!(text.len(), 10);
        prop_assert_eq!(Date::parse(&text), Ok(date));
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
