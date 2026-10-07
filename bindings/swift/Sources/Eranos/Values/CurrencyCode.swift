internal import EranosGenerated

/// A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
///
/// A currency comes from the library's list, so an amount in one always formats: create one from
/// its code with ``init(code:)``, or pick one from ``all``.
public struct CurrencyCode: Hashable, Sendable, CustomStringConvertible {
    /// Every currency with minor units, by code.
    public static let all: [Self] = EranosGenerated.currencies().map(Self.init)

    /// The currencies of ``all`` by their codes.
    private static let byCode: [String: Self] = Dictionary(uniqueKeysWithValues: all.map { ($0.code, $0) })

    /// The three-letter code: "EUR".
    public let code: String

    /// The English name: "Euro".
    public let name: String

    /// The three-digit numeric code: 978 for the euro.
    public let numericCode: Int

    /// How many decimal places its minor unit has: 2 for the euro, 0 for the yen.
    public let decimalPlaces: Int

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

    init(_ details: CurrencyDetails) {
        code = details.code
        name = details.name
        numericCode = Int(details.numericCode)
        decimalPlaces = Int(details.decimalPlaces)
    }

    /// The currency of `code`, a code the library gave, so always one of ``all``; any other code
    /// stands for itself, so that an amount in it keeps the code it came with.
    init(known code: String) {
        self = Self.byCode[code] ?? Self(unlisted: code)
    }

    /// A currency that is not one of ``all``, named by its code alone.
    init(unlisted code: String) {
        self.code = code
        name = code
        numericCode = 0
        decimalPlaces = 0
    }
}
