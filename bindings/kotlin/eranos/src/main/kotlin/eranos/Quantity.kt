package eranos

import eranos.internal.quantityCheck
import eranos.internal.quantityFormat
import eranos.internal.quantityFromF64
import eranos.internal.quantityFromWhole
import eranos.internal.quantityParse

/** How many units of an item, to a ten-thousandth: 0.452 kg is 4 520 ten-thousandths. */
public class Quantity internal constructor(
    /** The quantity in ten-thousandths: 10 000 for one unit, 4 520 for 0.452. */
    public val tenThousandths: Long,
) {
    /** Whether [other] is of the same class, with the same [tenThousandths]. */
    override fun equals(other: Any?): Boolean = other is Quantity && tenThousandths == other.tenThousandths

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = tenThousandths.hashCode()

    /** The quantity as the library writes it: "0.452". */
    override fun toString(): String = quantityFormat(tenThousandths)

    /**
     * One unit, the quantity of a number of ten-thousandths or of whole units, the parsing of one from its text, and the
     * conversion of a double into one.
     */
    public companion object {
        /** One unit, 10 000 ten-thousandths: the quantity of an item until it is given another. */
        public val ONE: Quantity = Quantity(tenThousandths = 10_000)

        /**
         * The quantity of [tenThousandths] ten-thousandths.
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidQuantity] for a number that is not positive.
         */
        @Throws(EranosException::class)
        public fun ofTenThousandths(tenThousandths: Long): Quantity =
            converting {
                quantityCheck(tenThousandths)
                Quantity(tenThousandths)
            }

        /**
         * The quantity of [units] whole units: 3 is 30 000 ten-thousandths.
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidQuantity] for a number that is not positive, and for one
         *   whose ten-thousandths do not fit a `Long`.
         */
        @Throws(EranosException::class)
        public fun ofWhole(units: Long): Quantity = converting { Quantity(quantityFromWhole(units)) }

        /**
         * The quantity written [text]: "3", "1.5", or "0.452".
         *
         * @throws EranosException.Money with [MoneyFailure.InvalidQuantity] for text that is not a positive decimal with at
         *   most four decimal places, and for a quantity whose ten-thousandths do not fit a `Long`.
         */
        @Throws(EranosException::class)
        public fun parse(text: String): Quantity = converting { Quantity(quantityParse(text)) }

        /**
         * The quantity of the double [value]: 0.45205 is 0.4521, 4 521 ten-thousandths.
         *
         * The library writes the double in its shortest decimal form, rounds it half away from zero to four decimal places,
         * and parses it.
         *
         * @throws EranosException.Money with [MoneyFailure.NotFinite] for NaN and the infinities, and
         *   [MoneyFailure.InvalidQuantity] for a value that rounds to zero or below and one whose ten-thousandths do not fit a
         *   `Long`.
         */
        @Throws(EranosException::class)
        public fun fromDouble(value: Double): Quantity = converting { Quantity(quantityFromF64(value)) }
    }
}
