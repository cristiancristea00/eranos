package eranos

import eranos.internal.CurrencyDetails
import eranos.internal.currencies
import eranos.internal.currencyDetails

/**
 * A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
 *
 * A currency comes from the library's list, so an amount in one always formats: look one up by its code with [of], or pick
 * one from [all].
 */
public class Currency internal constructor(
    /** The three-letter code: "EUR". */
    public val code: String,
    /** How many decimal places its minor unit has: 2 for the euro, 0 for the yen. */
    public val decimalPlaces: Int,
    /** The English name: "Euro". */
    public val name: String,
    /** The three-digit numeric code: 978 for the euro. */
    public val numericCode: Int,
) {
    /** Whether [other] is of the same class, with the same [code], [decimalPlaces], [name], and [numericCode]. */
    override fun equals(other: Any?): Boolean =
        other is Currency &&
            code == other.code &&
            decimalPlaces == other.decimalPlaces &&
            name == other.name &&
            numericCode == other.numericCode

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(code, decimalPlaces, name, numericCode)

    /** The code: "EUR". */
    override fun toString(): String = code

    /** The currencies of the library's list, and the lookup of one by its code. */
    public companion object {
        /** Every currency with minor units, by code. */
        public val all: List<Currency> by lazy { currencies().map { it.converted() } }

        /** The currencies of [all] by their codes. */
        private val byCode: Map<String, Currency> by lazy { all.associateBy { it.code } }

        /**
         * The currency with the ISO 4217 code [code].
         *
         * @throws EranosException.Money with [MoneyFailure.UnknownCurrency] for a code that names no currency with minor
         *   units.
         */
        @Throws(EranosException::class)
        public fun of(code: String): Currency = converting { currencyDetails(code).converted() }

        /**
         * The currency of [code], a code the library gave, so always one of [all]; any other code stands for itself, so
         * that an amount in it keeps the code it came with.
         */
        internal fun known(code: String): Currency = byCode[code] ?: Currency(code, 0, code, 0)
    }
}

/** The currency the library describes as [this]. */
internal fun CurrencyDetails.converted(): Currency = Currency(code, decimalPlaces, name, numericCode)
