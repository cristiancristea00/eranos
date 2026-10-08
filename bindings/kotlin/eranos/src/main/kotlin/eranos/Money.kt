package eranos

import eranos.internal.moneyAdd
import eranos.internal.moneyFormat
import eranos.internal.moneyParse
import eranos.internal.moneySubtract

/**
 * An amount: a whole number of minor units of a currency, cents for the euro.
 *
 * An amount prints as the library writes it, "12.50 EUR", and adds and subtracts with its checks: both amounts in one
 * currency, and the result within range.
 */
public class Money(
    /** The amount in minor units: cents for the euro, none for the yen. */
    public val minorUnits: Long,
    /** The currency. */
    public val currency: CurrencyCode,
) {
    /** The amount as the generated code holds it, with the details its currency holds. */
    internal val generated: eranos.internal.Money
        get() = eranos.internal.Money(minorUnits, currency.generated)

    /**
     * This amount plus [other].
     *
     * @throws EranosException.Money with [MoneyFailure.CurrencyMismatch] for amounts in two currencies, and
     *   [MoneyFailure.Overflow] for a sum out of range.
     */
    @Throws(EranosException::class)
    public operator fun plus(other: Money): Money = converting { moneyAdd(generated, other.generated).converted() }

    /**
     * This amount minus [other].
     *
     * @throws EranosException.Money with [MoneyFailure.CurrencyMismatch] for amounts in two currencies, and
     *   [MoneyFailure.Overflow] for a difference out of range.
     */
    @Throws(EranosException::class)
    public operator fun minus(other: Money): Money = converting { moneySubtract(generated, other.generated).converted() }

    /** Whether [other] is of the same class, with the same [minorUnits] and [currency]. */
    override fun equals(other: Any?): Boolean = other is Money && minorUnits == other.minorUnits && currency == other.currency

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(minorUnits, currency)

    /** The amount as the library writes it: "12.50 EUR". */
    override fun toString(): String = moneyFormat(generated)

    /** The parsing of an amount from its text. */
    public companion object {
        /**
         * The amount written [text] in [currency]: "12.50", "12.5", or "-3".
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidAmount] for text that is not a decimal number, and
         *   [MoneyFailure.TooManyDecimals] for more decimal places than the currency has.
         */
        @Throws(EranosException::class)
        public fun parse(
            text: String,
            currency: CurrencyCode,
        ): Money = converting { moneyParse(text, currency.code).converted() }
    }
}

/** The amount the generated code holds as [this]. */
internal fun eranos.internal.Money.converted(): Money = Money(minorUnits, currency.converted())
