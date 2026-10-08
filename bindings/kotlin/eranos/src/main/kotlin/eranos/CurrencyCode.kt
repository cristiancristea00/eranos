package eranos

import eranos.internal.CurrencyDetails
import eranos.internal.currencies
import eranos.internal.currencyDetails

/**
 * A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
 *
 * A currency comes from the library's list and holds the details the library gives of it, which an amount in it crosses
 * with: look one up by its code with [of], or pick one from [all].
 */
public class CurrencyCode internal constructor(
    /** The three-letter code: "EUR". */
    public val code: String,
    /** The English name: "Euro". */
    public val name: String,
    /** The three-digit numeric code: 978 for the euro. */
    public val numericCode: Int,
    /** How many decimal places its minor unit has: 2 for the euro, 0 for the yen. */
    public val decimalPlaces: Int,
) {
    /**
     * The currency as the generated code holds it.
     *
     * [decimalPlaces] holds the value of a `UByte`, the one the library gives for every currency an app can reach, so it
     * converts back unchanged.
     */
    internal val generated: CurrencyDetails
        get() = CurrencyDetails(code, name, numericCode, decimalPlaces.toUByte())

    /** Whether [other] is of the same class, with the same [code], [name], [numericCode], and [decimalPlaces]. */
    override fun equals(other: Any?): Boolean =
        other is CurrencyCode &&
            code == other.code &&
            name == other.name &&
            numericCode == other.numericCode &&
            decimalPlaces == other.decimalPlaces

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(code, name, numericCode, decimalPlaces)

    /** The code: "EUR". */
    override fun toString(): String = code

    /** The currencies of the library's list, and the lookup of one by its code. */
    public companion object {
        /** Every currency with minor units, by code. */
        public val all: List<CurrencyCode> by lazy { currencies().map { it.converted() } }

        /**
         * The currency with the ISO 4217 code [code].
         *
         * @throws EranosException.Money with [MoneyFailure.UnknownCurrency] for a code that names no currency with minor
         *   units.
         */
        @Throws(EranosException::class)
        public fun of(code: String): CurrencyCode = converting { currencyDetails(code).converted() }
    }
}

/** The currency the library describes as [this]. */
internal fun CurrencyDetails.converted(): CurrencyCode = CurrencyCode(code, name, numericCode, decimalPlaces.toInt())
