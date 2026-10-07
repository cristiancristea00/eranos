internal import EranosGenerated

/// An amount: a whole number of minor units of a currency, cents for the euro.
///
/// An amount prints as the library writes it, "12.50 EUR", and adds and subtracts with its
/// checks: both amounts in one currency, and the result within range.
public struct Money: Hashable, Sendable, CustomStringConvertible {
    /// The amount in minor units: cents for the euro, none for the yen.
    public let minorUnits: Int64

    /// The currency.
    public let currency: CurrencyCode

    /// The amount as the library writes it: "12.50 EUR".
    public var description: String {
        (try? EranosGenerated.moneyFormat(money: generated)) ?? "\(minorUnits) \(currency.code)"
    }

    var generated: EranosGenerated.Money {
        .init(minorUnits: minorUnits, currency: currency.code)
    }

    /// The amount of `minorUnits` minor units of `currency`.
    public init(minorUnits: Int64, currency: CurrencyCode) {
        self.minorUnits = minorUnits
        self.currency = currency
    }

    /// The amount written `text` in `currency`: "12.50", "12.5", or "-3".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidAmount`` for text
    ///   that is not a decimal number, and ``MoneyFailure/tooManyDecimals(currency:allowed:)``
    ///   for more decimal places than the currency has.
    public init(parsing text: String, in currency: CurrencyCode) throws(EranosError) {
        let money = try converting { try EranosGenerated.moneyParse(text: text, code: currency.code) }
        self.init(money)
    }

    init(_ money: EranosGenerated.Money) {
        self.init(minorUnits: money.minorUnits, currency: CurrencyCode(known: money.currency))
    }

    /// This amount plus `other`.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with
    ///   ``MoneyFailure/currencyMismatch(left:right:)`` for amounts in two currencies, and
    ///   ``MoneyFailure/overflow`` for a sum out of range.
    public func adding(_ other: Self) throws(EranosError) -> Self {
        let sum = try converting { try EranosGenerated.moneyAdd(left: generated, right: other.generated) }
        return Self(sum)
    }

    /// This amount minus `other`.
    ///
    /// - Throws: As ``adding(_:)`` does.
    public func subtracting(_ other: Self) throws(EranosError) -> Self {
        let difference = try converting { try EranosGenerated.moneySubtract(left: generated, right: other.generated) }
        return Self(difference)
    }
}
