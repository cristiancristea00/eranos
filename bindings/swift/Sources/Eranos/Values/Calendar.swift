internal import EranosGenerated

/// The library's refusal of a date, in its words, which a test holds to the library's.
func invalidDate() -> EranosError {
    .calendar(.invalidDate, detail: "invalid date: expected an existing ISO 8601 date between years 1 and 9999")
}

/// The library's refusal of a time of day, in its words, which a test holds to the library's.
func invalidTime() -> EranosError {
    .calendar(.invalidTime, detail: "invalid time: expected an ISO 8601 time between 00:00 and 23:59 with zero seconds")
}

/// A day of the calendar, without a time zone: the day an entry happened wherever it happened.
///
/// Dates order as the library orders them: by year, then month, then day. A date encodes as its
/// three numbers, and decodes through ``init(year:month:day:)``, so a decoded date exists.
public struct CalendarDate: Hashable, Sendable, Comparable, Codable, CustomStringConvertible {
    /// The year, from 1 to 9999.
    public let year: Int

    /// The month, from 1 for January.
    public let month: Int

    /// The day of the month, from 1.
    public let day: Int

    /// The date as the library writes it, in ISO 8601: "2026-09-23".
    public var description: String {
        EranosGenerated.calendarDateFormat(date: generated)
    }

    // Every initialiser keeps the numbers in the range of `Int32`.
    var generated: EranosGenerated.CalendarDate {
        .init(year: Int32(year), month: Int32(month), day: Int32(day))
    }

    /// The date of `year`, `month`, and `day`.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidDate`` for a
    ///   day that does not exist, such as 30 February.
    public init(year: Int, month: Int, day: Int) throws(EranosError) {
        let date = EranosGenerated.CalendarDate(
            year: try checked32Bits(year, invalidDate),
            month: try checked32Bits(month, invalidDate),
            day: try checked32Bits(day, invalidDate)
        )
        try converting { try EranosGenerated.calendarDateCheck(date: date) }
        self.init(date)
    }

    /// The date `decoder` holds as its year, month, and day.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of a day that does not
    ///   exist.
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let year = try container.decode(Int.self, forKey: .year)
        let month = try container.decode(Int.self, forKey: .month)
        let day = try container.decode(Int.self, forKey: .day)
        self = try decoded(decoder) { () throws(EranosError) in try Self(year: year, month: month, day: day) }
    }

    /// Whether `left` is a day before `right`.
    public static func < (left: Self, right: Self) -> Bool {
        if left.year != right.year {
            return left.year < right.year
        }
        if left.month != right.month {
            return left.month < right.month
        }
        return left.day < right.day
    }

    /// The date written `text` in ISO 8601: "2026-09-23".
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidDate``.
    public init(parsing text: String) throws(EranosError) {
        let date = try converting { try EranosGenerated.calendarDateParse(text: text) }
        self.init(date)
    }

    init(_ date: EranosGenerated.CalendarDate) {
        year = Int(date.year)
        month = Int(date.month)
        day = Int(date.day)
    }
}

/// A time of day, to the minute, without a time zone.
///
/// Times order by hour, then minute. A time encodes as its two numbers, and decodes through
/// ``init(hour:minute:)``.
public struct TimeOfDay: Hashable, Sendable, Comparable, Codable, CustomStringConvertible {
    /// The hour, from 0 to 23.
    public let hour: Int

    /// The minute, from 0 to 59.
    public let minute: Int

    /// The time as the library writes it, in ISO 8601: "18:30".
    public var description: String {
        EranosGenerated.timeOfDayFormat(time: generated)
    }

    // Every initialiser keeps the numbers in the range of `Int32`.
    var generated: EranosGenerated.TimeOfDay {
        .init(hour: Int32(hour), minute: Int32(minute))
    }

    /// The time of `hour` and `minute`.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidTime`` for an
    ///   hour or a minute out of range.
    public init(hour: Int, minute: Int) throws(EranosError) {
        let time = EranosGenerated.TimeOfDay(hour: try checked32Bits(hour, invalidTime), minute: try checked32Bits(minute, invalidTime))
        try converting { try EranosGenerated.timeOfDayCheck(time: time) }
        self.init(time)
    }

    /// The time `decoder` holds as its hour and minute.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of an hour or a minute
    ///   out of range.
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let hour = try container.decode(Int.self, forKey: .hour)
        let minute = try container.decode(Int.self, forKey: .minute)
        self = try decoded(decoder) { () throws(EranosError) in try Self(hour: hour, minute: minute) }
    }

    /// Whether `left` is a time before `right`.
    public static func < (left: Self, right: Self) -> Bool {
        if left.hour != right.hour {
            return left.hour < right.hour
        }
        return left.minute < right.minute
    }

    /// The time written `text` in ISO 8601: "18:30".
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidTime``.
    public init(parsing text: String) throws(EranosError) {
        let time = try converting { try EranosGenerated.timeOfDayParse(text: text) }
        self.init(time)
    }

    init(_ time: EranosGenerated.TimeOfDay) {
        hour = Int(time.hour)
        minute = Int(time.minute)
    }
}
