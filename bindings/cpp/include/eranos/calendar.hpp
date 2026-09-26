/// @file eranos/calendar.hpp
/// Days of the calendar and times of day, which the library checks when they are made.
///
/// A part of eranos.hpp, which includes every part and then the conversions their functions call: include eranos.hpp.

#ifndef ERANOS_CALENDAR_HPP
#define ERANOS_CALENDAR_HPP

// IWYU pragma: private, include "eranos.hpp"

#include "eranos/detail/declarations.hpp"
#include "eranos/error.hpp"

#include <cstdint>
#include <expected>
#include <optional>
#include <string>
#include <string_view>

namespace eranos {

namespace detail {

[[nodiscard]] constexpr CalendarDate calendarDateOf(EranosCalendarDate const date) noexcept;
[[nodiscard]] constexpr TimeOfDay timeOfDayOf(EranosTimeOfDay const time) noexcept;

} // namespace detail

/// A day of the calendar, which the library checks when it is made.
class CalendarDate final {
  public:
    /// The date of @p day @p month @p year, in the library's order of the three.
    ///
    /// @param day The day of the month, from 1
    /// @param month The month, from 1 for January
    /// @param year The year
    /// @return The date, or an error of the kind ErrorKind::CALENDAR for a day the calendar does not have
    // NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the order is the library's, day, month, and year.
    [[nodiscard]] static std::expected<CalendarDate, Error> of(std::int32_t const day, std::int32_t const month, std::int32_t const year);

    /// The date written @p text: "2026-09-23".
    ///
    /// @param text The date's text
    /// @return The date, or an error of the kind ErrorKind::CALENDAR for text that is not a date, and ErrorKind::INVALID_TEXT for text that is not
    ///   UTF-8
    [[nodiscard]] static std::expected<CalendarDate, Error> parse(std::string_view const text);

    /// The day of the month, from 1.
    ///
    /// @return The day
    [[nodiscard]] constexpr std::int32_t day() const noexcept { return fields.day; }

    /// The month, from 1 for January.
    ///
    /// @return The month
    [[nodiscard]] constexpr std::int32_t month() const noexcept { return fields.month; }

    /// The year.
    ///
    /// @return The year
    [[nodiscard]] constexpr std::int32_t year() const noexcept { return fields.year; }

    /// The date as the library writes it: "2026-09-23".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const;

    /// Whether two dates are the same day.
    ///
    /// @param left A date
    /// @param right Another date
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(CalendarDate const &left, CalendarDate const &right) noexcept = default;

  private:
    friend constexpr CalendarDate detail::calendarDateOf(detail::EranosCalendarDate const date) noexcept;

    /// The day, the month, and the year.
    struct Fields final {
        std::int32_t day;
        std::int32_t month;
        std::int32_t year;

        [[nodiscard]] friend constexpr bool operator==(Fields const &left, Fields const &right) noexcept = default;
    };

    constexpr explicit CalendarDate(Fields const fields) noexcept : fields(fields) {}

    Fields fields;
};

/// A time of day, to the minute, which the library checks when it is made.
class TimeOfDay final {
  public:
    /// The time @p hour : @p minute.
    ///
    /// @param hour The hour, from 0 to 23
    /// @param minute The minute, from 0 to 59
    /// @return The time, or an error of the kind ErrorKind::CALENDAR for a time the day does not have
    // NOLINTNEXTLINE(bugprone-easily-swappable-parameters): the order is the library's, hour and minute.
    [[nodiscard]] static std::expected<TimeOfDay, Error> of(std::int32_t const hour, std::int32_t const minute);

    /// The time written @p text: "18:30".
    ///
    /// @param text The time's text
    /// @return The time, or an error of the kind ErrorKind::CALENDAR for text that is not a time, and ErrorKind::INVALID_TEXT for text that is not
    ///   UTF-8
    [[nodiscard]] static std::expected<TimeOfDay, Error> parse(std::string_view const text);

    /// The hour, from 0 to 23.
    ///
    /// @return The hour
    [[nodiscard]] constexpr std::int32_t hour() const noexcept { return fields.hour; }

    /// The minute, from 0 to 59.
    ///
    /// @return The minute
    [[nodiscard]] constexpr std::int32_t minute() const noexcept { return fields.minute; }

    /// The time as the library writes it: "18:30".
    ///
    /// @return The text
    [[nodiscard]] std::string toString() const;

    /// Whether two times are the same minute.
    ///
    /// @param left A time
    /// @param right Another time
    /// @return Whether they are equal
    [[nodiscard]] friend constexpr bool operator==(TimeOfDay const &left, TimeOfDay const &right) noexcept = default;

  private:
    friend constexpr TimeOfDay detail::timeOfDayOf(detail::EranosTimeOfDay const time) noexcept;

    /// The hour and the minute.
    struct Fields final {
        std::int32_t hour;
        std::int32_t minute;

        [[nodiscard]] friend constexpr bool operator==(Fields const &left, Fields const &right) noexcept = default;
    };

    constexpr explicit TimeOfDay(Fields const fields) noexcept : fields(fields) {}

    Fields fields;
};

namespace detail {

[[nodiscard]] constexpr EranosCalendarDate toDetail(CalendarDate const date) noexcept;
[[nodiscard]] constexpr EranosTimeOfDay toDetail(TimeOfDay const time) noexcept;
[[nodiscard]] constexpr std::optional<TimeOfDay> optionalTime(std::optional<EranosTimeOfDay> const time) noexcept;

} // namespace detail

inline std::expected<CalendarDate, Error> CalendarDate::of(std::int32_t const day, std::int32_t const month, std::int32_t const year) {
    detail::EranosCalendarDate const date{.day = day, .month = month, .year = year};
    return detail::expectedOf(date.format()).transform([date](std::string const & /* text */) noexcept { return detail::calendarDateOf(date); });
}

inline std::expected<CalendarDate, Error> CalendarDate::parse(std::string_view const text) {
    return detail::expectedOf(detail::EranosCalendarDate::parse(text)).transform(detail::calendarDateOf);
}

// A date is checked when it is made.
inline std::string CalendarDate::toString() const { return detail::known(detail::toDetail(*this).format()); }

inline std::expected<TimeOfDay, Error> TimeOfDay::of(std::int32_t const hour, std::int32_t const minute) {
    detail::EranosTimeOfDay const time{.hour = hour, .minute = minute};
    return detail::expectedOf(time.format()).transform([time](std::string const & /* text */) noexcept { return detail::timeOfDayOf(time); });
}

inline std::expected<TimeOfDay, Error> TimeOfDay::parse(std::string_view const text) {
    return detail::expectedOf(detail::EranosTimeOfDay::parse(text)).transform(detail::timeOfDayOf);
}

// A time is checked when it is made.
inline std::string TimeOfDay::toString() const { return detail::known(detail::toDetail(*this).format()); }

} // namespace eranos

#endif // ERANOS_CALENDAR_HPP
