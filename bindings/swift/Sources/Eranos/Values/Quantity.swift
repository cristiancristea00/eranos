internal import EranosGenerated

/// How many units a line of a bill counts: a positive decimal with four decimal places, 0.452 kg
/// as 4 520 ten-thousandths.
///
/// A quantity multiplies an item's unit price into its line total, which
/// ``ExpenseItem/lineTotal()`` reads from the library.
public struct Quantity: Hashable, Sendable, CustomStringConvertible {
    // One unit is 10 000 ten-thousandths, the scale of every quantity the core's records hold;
    // the core has no helper that answers it.
    /// One unit, 10 000 ten-thousandths: the quantity of an item that is not given another.
    public static let one = Self(known: 10_000)

    /// The quantity in ten-thousandths: 15 000 for 1.5.
    public let tenThousandths: Int64

    /// The quantity as the library writes it, without trailing zeros: "0.452", "1.5", or "3".
    public var description: String {
        EranosGenerated.quantityFormat(tenThousandths: tenThousandths)
    }

    /// The quantity of `tenThousandths` ten-thousandths of a unit.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidQuantity`` for a
    ///   number that is not positive.
    public init(tenThousandths: Int64) throws(EranosError) {
        try converting { try EranosGenerated.quantityCheck(tenThousandths: tenThousandths) }
        self.tenThousandths = tenThousandths
    }

    /// The quantity of `units` whole units: 3 is 30 000 ten-thousandths.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidQuantity`` for a
    ///   number that is not positive, and for one whose ten-thousandths do not fit an `Int64`.
    public init(whole units: Int64) throws(EranosError) {
        tenThousandths = try converting { try EranosGenerated.quantityFromWhole(units: units) }
    }

    /// The quantity written `text`: "2", "1.5", or "0.452".
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/invalidQuantity`` for text
    ///   that is not a positive decimal with at most four decimal places, without a sign or an
    ///   exponent, and for a quantity whose ten-thousandths do not fit an `Int64`.
    public init(parsing text: String) throws(EranosError) {
        tenThousandths = try converting { try EranosGenerated.quantityParse(text: text) }
    }

    /// The quantity of the double `value`: 0.45205 is 0.4521.
    ///
    /// The library writes the double in its shortest decimal form, rounds it half away from zero
    /// to four decimal places, and parses it, as ``Money/init(amount:currency:)`` converts an
    /// amount.
    ///
    /// - Throws: ``EranosError/money(_:detail:)`` with ``MoneyFailure/notFinite`` for NaN and the
    ///   infinities, and ``MoneyFailure/invalidQuantity`` for a value that rounds to zero or below,
    ///   as 0.00004 does, and one whose ten-thousandths do not fit an `Int64`.
    public init(_ value: Double) throws(EranosError) {
        tenThousandths = try converting { try EranosGenerated.quantityFromF64(value: value) }
    }

    /// The quantity of `tenThousandths`, a number the library gave.
    init(known tenThousandths: Int64) {
        self.tenThousandths = tenThousandths
    }
}
