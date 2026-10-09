internal import EranosGenerated

/// A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
///
/// A currency holds the details the library gives for it, and an amount prints from them and
/// crosses back with them. Create one from its code with ``init(code:)``, or pick one from
/// ``all``; the library gives one with every amount, as the group currency, and in a failure.
/// Currencies order by their codes, as the library orders them; a currency encodes as its code,
/// and decodes through ``init(code:)``.
public struct CurrencyCode: Hashable, Sendable, Comparable, Codable, CustomStringConvertible {
    /// Every currency with minor units, by code.
    public static let all: [Self] = EranosGenerated.currencies().map(Self.init)

    /// The details as the library gives them.
    let generated: EranosGenerated.CurrencyDetails

    /// The three-letter code: "EUR".
    public var code: String {
        generated.code
    }

    /// The English name: "Euro".
    public var name: String {
        generated.name
    }

    /// The three-digit numeric code: 978 for the euro.
    public var numericCode: Int {
        Int(generated.numericCode)
    }

    /// How many decimal places its minor unit has: 2 for the euro, 0 for the yen.
    public var decimalPlaces: Int {
        Int(generated.decimalPlaces)
    }

    /// The code: "EUR".
    public var description: String {
        code
    }

    /// The currency with the ISO 4217 code `code`.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/unknownCurrency`` for a code
    ///   that names no currency with minor units.
    public init(code: String) throws(EranosError) {
        let details = try converting { try EranosGenerated.currencyDetails(code: code) }
        self.init(details)
    }

    init(_ details: EranosGenerated.CurrencyDetails) {
        generated = details
    }

    /// The currency `decoder` holds as its code.
    ///
    /// - Throws: `DecodingError.dataCorrupted` with the library's refusal of a code that names no
    ///   currency with minor units.
    public init(from decoder: any Decoder) throws {
        let code = try decoder.singleValueContainer().decode(String.self)
        self = try decoded(decoder) { () throws(EranosError) in try Self(code: code) }
    }

    /// Writes the code to `encoder`.
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(code)
    }

    /// Whether the code of `left` comes before the code of `right`.
    public static func < (left: Self, right: Self) -> Bool {
        left.code.utf8.lexicographicallyPrecedes(right.code.utf8)
    }
}
