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
        EranosGenerated.moneyFormat(money: generated)
    }

    var generated: EranosGenerated.Money {
        .init(minorUnits: minorUnits, currency: currency.generated)
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

    /// The amount of the double `amount` in `currency`: 1.005 in EUR is 1.01 EUR.
    ///
    /// The library writes the double in its shortest decimal form, the shortest text that reads
    /// back as the same double, rounds it half away from zero to the currency's decimal places,
    /// and parses it. Rounding the text rather than the double keeps the cents the user typed:
    /// the double nearest 1.005 lies below it, yet it is 1.01 EUR. Every language converts
    /// through the library, so the same double is the same amount in each.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/notFinite`` for NaN and the
    ///   infinities, and ``MoneyFailure/overflow`` for an amount that does not fit in 64-bit minor
    ///   units.
    public init(amount: Double, currency: CurrencyCode) throws(EranosError) {
        let money = try converting { try EranosGenerated.moneyFromF64(value: amount, code: currency.code) }
        self.init(money)
    }

    init(_ money: EranosGenerated.Money) {
        self.init(minorUnits: money.minorUnits, currency: CurrencyCode(money.currency))
    }

    /// Zero in `currency`.
    public static func zero(in currency: CurrencyCode) -> Self {
        Self(minorUnits: 0, currency: currency)
    }

    /// Whether the amount is zero.
    public var isZero: Bool {
        0 == minorUnits
    }

    /// Whether the amount is above zero.
    public var isPositive: Bool {
        0 < minorUnits
    }

    /// Whether the amount is below zero.
    public var isNegative: Bool {
        0 > minorUnits
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
