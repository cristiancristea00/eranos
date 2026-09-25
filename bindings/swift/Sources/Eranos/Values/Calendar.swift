internal import EranosGenerated

/// A day of the calendar, without a time zone: the day an entry happened wherever it happened.
public struct CalendarDate: Hashable, Sendable, CustomStringConvertible {
    /// The day of the month, from 1.
    public let day: Int32

    /// The month, from 1 for January.
    public let month: Int32

    /// The year, from 1 to 9999.
    public let year: Int32

    /// The date as the library writes it, in ISO 8601: "2026-09-23".
    public var description: String {
        (try? EranosGenerated.calendarDateFormat(date: generated)) ?? "\(year)/\(month)/\(day)"
    }

    var generated: EranosGenerated.CalendarDate {
        .init(day: day, month: month, year: year)
    }

    /// The date of `day`, `month`, and `year`.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidDate`` for a
    ///   day that does not exist, such as 30 February.
    public init(day: Int32, month: Int32, year: Int32) throws(EranosError) {
        let date = EranosGenerated.CalendarDate(day: day, month: month, year: year)
        _ = try converting { try EranosGenerated.calendarDateFormat(date: date) }
        self.init(date)
    }

    /// The date written `text` in ISO 8601: "2026-09-23".
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidDate``.
    public init(parsing text: String) throws(EranosError) {
        let date = try converting { try EranosGenerated.calendarDateParse(text: text) }
        self.init(date)
    }

    init(_ date: EranosGenerated.CalendarDate) {
        day = date.day
        month = date.month
        year = date.year
    }
}

/// A time of day, to the minute, without a time zone.
public struct TimeOfDay: Hashable, Sendable, CustomStringConvertible {
    /// The hour, from 0 to 23.
    public let hour: Int32

    /// The minute, from 0 to 59.
    public let minute: Int32

    /// The time as the library writes it, in ISO 8601: "18:30".
    public var description: String {
        (try? EranosGenerated.timeOfDayFormat(time: generated)) ?? "\(hour)h\(minute)"
    }

    var generated: EranosGenerated.TimeOfDay {
        .init(hour: hour, minute: minute)
    }

    /// The time of `hour` and `minute`.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidTime`` for an
    ///   hour or a minute out of range.
    public init(hour: Int32, minute: Int32) throws(EranosError) {
        let time = EranosGenerated.TimeOfDay(hour: hour, minute: minute)
        _ = try converting { try EranosGenerated.timeOfDayFormat(time: time) }
        self.init(time)
    }

    /// The time written `text` in ISO 8601: "18:30".
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidTime``.
    public init(parsing text: String) throws(EranosError) {
        let time = try converting { try EranosGenerated.timeOfDayParse(text: text) }
        self.init(time)
    }

    init(_ time: EranosGenerated.TimeOfDay) {
        hour = time.hour
        minute = time.minute
    }
}
