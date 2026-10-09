public import Foundation

// The values an app on Apple's platforms holds in Foundation's types: amounts as `Decimal`, and
// days and times as `DateComponents`. Each crosses through the library's own checks.

public extension Money {
    /// The amount of the decimal `amount` in `currency`: 12.5 in EUR is 12.50 EUR.
    ///
    /// The library parses the decimal's text, as ``init(parsing:in:)`` does, so a decimal with
    /// more decimal places than the currency has is refused rather than rounded.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/tooManyDecimals(currency:allowed:)``
    ///   for more decimal places than the currency has, ``MoneyFailure/invalidAmount`` for NaN,
    ///   and ``MoneyFailure/overflow`` for an amount that does not fit in 64-bit minor units.
    init(_ amount: Decimal, currency: CurrencyCode) throws(EranosError) {
        try self.init(parsing: amount.description, in: currency)
    }

    /// The amount as a decimal: 12.50 EUR is 12.5.
    ///
    /// The decimal holds the minor units with as many decimal places as the currency has, so it
    /// is exact.
    var decimalValue: Decimal {
        Decimal(sign: 0 > minorUnits ? .minus : .plus, exponent: -currency.decimalPlaces, significand: Decimal(minorUnits.magnitude))
    }
}

public extension CalendarDate {
    /// The date of the year, the month, and the day of `components`, in the calendar they were
    /// read in; the other components are ignored.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidDate`` for
    ///   components without a year, a month, or a day, and for a day that does not exist.
    init(components: DateComponents) throws(EranosError) {
        guard let year = components.year, let month = components.month, let day = components.day else {
            throw invalidDate()
        }
        try self.init(year: year, month: month, day: day)
    }

    /// The date's year, month, and day, as the components of a date with no calendar or time zone.
    var dateComponents: DateComponents {
        DateComponents(year: year, month: month, day: day)
    }
}

public extension TimeOfDay {
    /// The time of the hour and the minute of `components`; the other components are ignored.
    ///
    /// - Throws: ``EranosError/calendar(_:detail:)`` with ``CalendarFailure/invalidTime`` for
    ///   components without an hour or a minute, and for an hour or a minute out of range.
    init(components: DateComponents) throws(EranosError) {
        guard let hour = components.hour, let minute = components.minute else {
            throw invalidTime()
        }
        try self.init(hour: hour, minute: minute)
    }

    /// The time's hour and minute, as the components of a time with no calendar or time zone.
    var dateComponents: DateComponents {
        DateComponents(hour: hour, minute: minute)
    }
}
