package eranos

import eranos.internal.exchangeRateConvert
import eranos.internal.exchangeRateFromF64
import eranos.internal.exchangeRateParse
import eranos.internal.partsCheck
import eranos.internal.partsFormat
import eranos.internal.partsFromF64
import eranos.internal.partsParse
import eranos.internal.percentageCheck
import eranos.internal.percentageFormat
import eranos.internal.percentageFromF64
import eranos.internal.percentageParse

/** The rate that converts an amount in one currency into another: a positive decimal. */
public class ExchangeRate internal constructor(
    /** The rate as the library writes it. */
    private val text: String,
) {
    /**
     * [amount] converted at this rate into [currency], rounded to its minor unit, halves away from zero.
     *
     * @throws EranosException.Money with [MoneyFailure.Overflow] for a result out of range.
     */
    @Throws(EranosException::class)
    public fun convert(
        amount: Money,
        currency: CurrencyCode,
    ): Money = converting { exchangeRateConvert(text, amount.generated, currency.code).converted() }

    /** Whether [other] is of the same class, with the same text. */
    override fun equals(other: Any?): Boolean = other is ExchangeRate && text == other.text

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = text.hashCode()

    /** The rate as the library writes it: "4.97". */
    override fun toString(): String = text

    /** The parsing of a rate from its text, and its conversion from a double. */
    public companion object {
        /**
         * The rate written [text]: "4.97" or "0.2012".
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidExchangeRate] for text that is not a positive decimal
         *   with at most 18 decimal places.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): ExchangeRate = converting { ExchangeRate(exchangeRateParse(text)) }

        /**
         * The rate of the double [value], with the digits of its shortest decimal form: 25.5 is 25.5.
         *
         * The library writes the double in its shortest decimal form, rounds it half away from zero to 18 decimal places, and
         * parses it, so `0.1 + 0.2`, which no double holds exactly, is 0.30000000000000004.
         *
         * @throws EranosException.Money with [MoneyFailure.NotFinite] for NaN and the infinities, and
         *   [MoneyFailure.InvalidExchangeRate] for a value that is not positive at 18 decimal places or has more significant
         *   digits than fit in 64 bits.
         */
        @Throws(EranosException::class)
        public fun fromDouble(value: Double): ExchangeRate = converting { ExchangeRate(exchangeRateFromF64(value)) }
    }
}

/** A share of a whole, to a hundredth of a per cent: 12.5 % is 1 250 basis points. */
public class Percentage internal constructor(
    /** The percentage in basis points: 1 250 for 12.5 %. */
    public val basisPoints: Int,
) {
    /** Whether [other] is of the same class, with the same [basisPoints]. */
    override fun equals(other: Any?): Boolean = other is Percentage && basisPoints == other.basisPoints

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = basisPoints

    /** The percentage as the library writes it: "12.5%". */
    override fun toString(): String = percentageFormat(basisPoints)

    /** The percentage of a number of basis points, the parsing of one from its text, and the conversion of a double into one. */
    public companion object {
        /**
         * The percentage of [basisPoints] hundredths of a per cent.
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidPercentage] for a negative number.
         */
        @Throws(EranosException::class)
        public fun ofBasisPoints(basisPoints: Int): Percentage =
            converting {
                percentageCheck(basisPoints)
                Percentage(basisPoints)
            }

        /**
         * The percentage written [text]: "12.5", "12.5%", or "100".
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidPercentage] for text that is not a non-negative decimal
         *   with at most two decimal places.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Percentage = converting { Percentage(percentageParse(text)) }

        /**
         * The percentage of the double [value], a number of per cent: 12.345 is 12.35 %, 1 235 basis points.
         *
         * The library writes the double in its shortest decimal form, rounds it half away from zero to two decimal places, and
         * parses it: `0.1 + 0.2`, which is 0.30000000000000004 as a double, is 0.3 %.
         *
         * @throws EranosException.Money with [MoneyFailure.NotFinite] for NaN and the infinities, and
         *   [MoneyFailure.InvalidPercentage] for a negative value and one whose basis points do not fit an `Int`.
         */
        @Throws(EranosException::class)
        public fun fromDouble(value: Double): Percentage = converting { Percentage(percentageFromF64(value)) }
    }
}

/** A number of parts, to a hundredth: a member who pays half a part pays half of what a member with one part pays. */
public class Parts internal constructor(
    /** The parts in hundredths: 50 for half a part. */
    public val hundredths: Int,
) {
    /** Whether [other] is of the same class, with the same [hundredths]. */
    override fun equals(other: Any?): Boolean = other is Parts && hundredths == other.hundredths

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hundredths

    /** The parts as the library writes them: "0.5". */
    override fun toString(): String = partsFormat(hundredths)

    /** The parts of a number of hundredths, the parsing of them from their text, and the conversion of a double into them. */
    public companion object {
        /**
         * The parts of [hundredths] hundredths.
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidParts] for a number that is not positive.
         */
        @Throws(EranosException::class)
        public fun ofHundredths(hundredths: Int): Parts =
            converting {
                partsCheck(hundredths)
                Parts(hundredths)
            }

        /**
         * The parts written [text]: "1", "0.5", or "2.25".
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidParts] for text that is not a positive decimal with at
         *   most two decimal places.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Parts = converting { Parts(partsParse(text)) }

        /**
         * The parts of the double [value]: 1.005 is 1.01 parts, 101 hundredths.
         *
         * The library writes the double in its shortest decimal form, rounds it half away from zero to two decimal places, and
         * parses it.
         *
         * @throws EranosException.Money with [MoneyFailure.NotFinite] for NaN and the infinities, and [MoneyFailure.InvalidParts]
         *   for a value that rounds to zero or below and one whose hundredths do not fit an `Int`.
         */
        @Throws(EranosException::class)
        public fun fromDouble(value: Double): Parts = converting { Parts(partsFromF64(value)) }
    }
}
