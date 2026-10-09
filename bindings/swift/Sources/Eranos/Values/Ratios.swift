internal import EranosGenerated

/// The library's refusal of a percentage, in its words, which a test holds to the library's.
func invalidPercentage() -> EranosError {
    .money(.invalidPercentage, detail: "invalid percentage: expected a number with at most 2 decimal places")
}

/// The library's refusal of a number of parts, in its words, which a test holds to the library's.
func invalidParts() -> EranosError {
    .money(.invalidParts, detail: "invalid parts: expected a positive number with at most 2 decimal places")
}

/// The rate that converts an amount in one currency into another: a positive decimal.
///
/// A rate encodes as its text, and decodes through ``init(parsing:)``.
public struct ExchangeRate: Hashable, Sendable, Codable, CustomStringConvertible {
    /// The rate as the library writes it: "4.97".
    public let text: String

    /// The rate as the library writes it: "4.97".
    public var description: String {
        text
    }

    /// The rate written `text`: "4.97" or "0.2012".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidExchangeRate`` for
    ///   text that is not a positive decimal with at most 18 decimal places.
    public init(parsing text: String) throws(EranosError) {
        self.text = try converting { try EranosGenerated.exchangeRateParse(text: text) }
    }

    /// The rate `decoder` holds as its text.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of text that is not a
    ///   rate.
    public init(from decoder: any Decoder) throws {
        let text = try decoder.singleValueContainer().decode(String.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(parsing: text) }
    }

    /// Writes the rate's text to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(text)
    }

    /// The rate of the double `value`: 4.9765 is "4.9765".
    ///
    /// The library writes the double in its shortest decimal form, rounds it half away from zero
    /// to 18 decimal places, and parses it, as ``Money/init(amount:currency:)`` converts an
    /// amount, so the rate keeps the digits of that form: 25.5 is "25.5", and `0.1 + 0.2` is
    /// "0.30000000000000004".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/notFinite`` for NaN and the
    ///   infinities, and ``MoneyFailure/invalidExchangeRate`` for a value that is not positive at
    ///   18 decimal places or has more significant digits than fit in 64 bits.
    public init(_ value: Double) throws(EranosError) {
        text = try converting { try EranosGenerated.exchangeRateFromF64(value: value) }
    }

    /// The rate the library wrote as `text`.
    init(known text: String) {
        self.text = text
    }

    /// `amount` converted at this rate into `currency`, rounded to its minor unit, halves away
    /// from zero.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/overflow`` for a result
    ///   out of range.
    public func convert(_ amount: Money, into currency: CurrencyCode) throws(EranosError) -> Money {
        let converted = try converting {
            try EranosGenerated.exchangeRateConvert(rate: text, amount: amount.generated, code: currency.code)
        }
        return Money(converted)
    }
}

/// A share of a whole, to a hundredth of a per cent: 12.5 % is 1 250 basis points.
///
/// Percentages order by their basis points. A percentage encodes as its basis points, and
/// decodes through ``init(basisPoints:)``.
public struct Percentage: Hashable, Sendable, Comparable, Codable, CustomStringConvertible {
    /// The percentage in basis points: 1 250 for 12.5 %.
    public let basisPoints: Int

    /// The percentage as the library writes it: "12.5%".
    public var description: String {
        EranosGenerated.percentageFormat(basisPoints: generated)
    }

    // Every initialiser keeps the basis points in the range of `Int32`.
    var generated: Int32 {
        Int32(basisPoints)
    }

    /// The percentage of `basisPoints` hundredths of a per cent.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidPercentage`` for a
    ///   negative number, and for one above 2 147 483 647, which the library does not hold.
    public init(basisPoints: Int) throws(EranosError) {
        let narrow = try checked32Bits(basisPoints, invalidPercentage)
        try converting { try EranosGenerated.percentageCheck(basisPoints: narrow) }
        self.basisPoints = basisPoints
    }

    /// The percentage `decoder` holds as its basis points.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of a number it does
    ///   not hold.
    public init(from decoder: any Decoder) throws {
        let basisPoints = try decoder.singleValueContainer().decode(Int.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(basisPoints: basisPoints) }
    }

    /// Writes the basis points to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(basisPoints)
    }

    /// Whether `left` is less than `right`.
    public static func < (left: Self, right: Self) -> Bool {
        left.basisPoints < right.basisPoints
    }

    /// The percentage written `text`: "12.5", "12.5%", or "100".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidPercentage`` for
    ///   text that is not a non-negative decimal with at most two decimal places.
    public init(parsing text: String) throws(EranosError) {
        basisPoints = Int(try converting { try EranosGenerated.percentageParse(text: text) })
    }

    /// The percentage of the double `value`, a number of per cent: 12.345 is 12.35 %.
    ///
    /// The library writes the double in its shortest decimal form, rounds it half away from zero
    /// to two decimal places, and parses it, as ``Money/init(amount:currency:)`` converts an
    /// amount: `0.1 + 0.2`, which is 0.30000000000000004 as a double, is 0.3 %.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/notFinite`` for NaN and the
    ///   infinities, and ``MoneyFailure/invalidPercentage`` for a negative value and one whose
    ///   basis points do not fit an `Int32`.
    public init(_ value: Double) throws(EranosError) {
        basisPoints = Int(try converting { try EranosGenerated.percentageFromF64(value: value) })
    }

    /// The percentage of `basisPoints`, a number the library gave.
    init(known basisPoints: Int32) {
        self.basisPoints = Int(basisPoints)
    }
}

/// A number of parts, to a hundredth: a member who pays half a part pays half of what a member
/// with one part pays.
///
/// Parts order by their hundredths. Parts encode as their hundredths, and decode through
/// ``init(hundredths:)``.
public struct Parts: Hashable, Sendable, Comparable, Codable, CustomStringConvertible {
    /// The parts in hundredths: 50 for half a part.
    public let hundredths: Int

    /// The parts as the library writes them: "0.5".
    public var description: String {
        EranosGenerated.partsFormat(hundredths: generated)
    }

    // Every initialiser keeps the hundredths in the range of `Int32`.
    var generated: Int32 {
        Int32(hundredths)
    }

    /// The parts of `hundredths` hundredths.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidParts`` for a number
    ///   that is not positive, and for one above 2 147 483 647, which the library does not hold.
    public init(hundredths: Int) throws(EranosError) {
        let narrow = try checked32Bits(hundredths, invalidParts)
        try converting { try EranosGenerated.partsCheck(hundredths: narrow) }
        self.hundredths = hundredths
    }

    /// The parts `decoder` holds as their hundredths.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of a number it does
    ///   not hold.
    public init(from decoder: any Decoder) throws {
        let hundredths = try decoder.singleValueContainer().decode(Int.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(hundredths: hundredths) }
    }

    /// Writes the hundredths to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(hundredths)
    }

    /// Whether `left` is less than `right`.
    public static func < (left: Self, right: Self) -> Bool {
        left.hundredths < right.hundredths
    }

    /// The parts written `text`: "1", "0.5", or "2.25".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidParts`` for text
    ///   that is not a positive decimal with at most two decimal places.
    public init(parsing text: String) throws(EranosError) {
        hundredths = Int(try converting { try EranosGenerated.partsParse(text: text) })
    }

    /// The parts of the double `value`: 1.005 is 1.01 parts.
    ///
    /// The library writes the double in its shortest decimal form, rounds it half away from zero
    /// to two decimal places, and parses it, as ``Money/init(amount:currency:)`` converts an
    /// amount.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/notFinite`` for NaN and the
    ///   infinities, and ``MoneyFailure/invalidParts`` for a value that rounds to zero or below,
    ///   as 0.004 does, and one whose hundredths do not fit an `Int32`.
    public init(_ value: Double) throws(EranosError) {
        hundredths = Int(try converting { try EranosGenerated.partsFromF64(value: value) })
    }

    /// The parts of `hundredths`, a number the library gave.
    init(known hundredths: Int32) {
        self.hundredths = Int(hundredths)
    }
}
