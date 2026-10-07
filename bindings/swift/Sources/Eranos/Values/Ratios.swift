internal import EranosGenerated

/// The rate that converts an amount in one currency into another: a positive decimal.
public struct ExchangeRate: Hashable, Sendable, CustomStringConvertible {
    /// The rate as the library writes it: "4.97".
    public let description: String

    /// The rate written `text`: "4.97" or "0.2012".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidExchangeRate`` for
    ///   text that is not a positive decimal with at most 18 decimal places.
    public init(parsing text: String) throws(EranosError) {
        description = try converting { try EranosGenerated.exchangeRateParse(text: text) }
    }

    /// The rate the library wrote as `text`.
    init(known text: String) {
        description = text
    }

    /// `amount` converted at this rate into `currency`, rounded to its minor unit, halves away
    /// from zero.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/overflow`` for a result
    ///   out of range.
    public func convert(_ amount: Money, into currency: CurrencyCode) throws(EranosError) -> Money {
        let converted = try converting {
            try EranosGenerated.exchangeRateConvert(rate: description, amount: amount.generated, code: currency.code)
        }
        return Money(converted)
    }
}

/// A share of a whole, to a hundredth of a per cent: 12.5 % is 1 250 basis points.
public struct Percentage: Hashable, Sendable, CustomStringConvertible {
    /// The percentage in basis points: 1 250 for 12.5 %.
    public let basisPoints: Int32

    /// The percentage as the library writes it: "12.5%".
    public var description: String {
        (try? EranosGenerated.percentageFormat(basisPoints: basisPoints)) ?? "\(basisPoints) bp"
    }

    /// The percentage of `basisPoints` hundredths of a per cent.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidPercentage`` for a
    ///   negative number.
    public init(basisPoints: Int32) throws(EranosError) {
        _ = try converting { try EranosGenerated.percentageFormat(basisPoints: basisPoints) }
        self.basisPoints = basisPoints
    }

    /// The percentage written `text`: "12.5", "12.5%", or "100".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidPercentage`` for
    ///   text that is not a non-negative decimal with at most two decimal places.
    public init(parsing text: String) throws(EranosError) {
        basisPoints = try converting { try EranosGenerated.percentageParse(text: text) }
    }

    /// The percentage of `basisPoints`, a number the library gave.
    init(known basisPoints: Int32) {
        self.basisPoints = basisPoints
    }
}

/// A number of parts, to a hundredth: a member who pays half a part pays half of what a member
/// with one part pays.
public struct Parts: Hashable, Sendable, CustomStringConvertible {
    /// The parts in hundredths: 50 for half a part.
    public let hundredths: Int32

    /// The parts as the library writes them: "0.5".
    public var description: String {
        (try? EranosGenerated.partsFormat(hundredths: hundredths)) ?? "\(hundredths)/100"
    }

    /// The parts of `hundredths` hundredths.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidParts`` for a number
    ///   that is not positive.
    public init(hundredths: Int32) throws(EranosError) {
        _ = try converting { try EranosGenerated.partsFormat(hundredths: hundredths) }
        self.hundredths = hundredths
    }

    /// The parts written `text`: "1", "0.5", or "2.25".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidParts`` for text
    ///   that is not a positive decimal with at most two decimal places.
    public init(parsing text: String) throws(EranosError) {
        hundredths = try converting { try EranosGenerated.partsParse(text: text) }
    }

    /// The parts of `hundredths`, a number the library gave.
    init(known hundredths: Int32) {
        self.hundredths = hundredths
    }
}
