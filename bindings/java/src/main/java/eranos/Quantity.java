package eranos;

import eranos.internal.Eranos;

/**
 * How many units a line of a bill counts, to a ten-thousandth: 0.452 kg is 4 520 ten-thousandths.
 */
public final class Quantity {
    /** One unit: the quantity of an item that does not set one. */
    public static final Quantity ONE = new Quantity(10_000);

    private final long tenThousandths;

    private Quantity(long tenThousandths) {
        this.tenThousandths = tenThousandths;
    }

    /**
     * The quantity of {@code tenThousandths} ten-thousandths.
     *
     * @param tenThousandths the quantity in ten-thousandths: 4 520 for 0.452
     * @return the quantity
     * @throws EranosException.Money with {@link MoneyFailure.InvalidQuantity} for a number that is
     *     not positive
     */
    public static Quantity ofTenThousandths(long tenThousandths) throws EranosException {
        Converting.run(() -> Eranos.quantityCheck(tenThousandths));
        return new Quantity(tenThousandths);
    }

    /**
     * The quantity of {@code units} whole units: 3 is 30 000 ten-thousandths.
     *
     * @param units the number of units
     * @return the quantity
     * @throws EranosException.Money with {@link MoneyFailure.InvalidQuantity} for a number that is
     *     not positive, and one whose ten-thousandths do not fit a {@code long}
     */
    public static Quantity ofWhole(long units) throws EranosException {
        return Converting.call(() -> new Quantity(Eranos.quantityFromWhole(units)));
    }

    /**
     * The quantity written {@code text}: "2", "1.5", or "0.452".
     *
     * @param text the quantity's text
     * @return the quantity
     * @throws EranosException.Money with {@link MoneyFailure.InvalidQuantity} for text that is not
     *     a positive decimal with at most four decimal places, and a quantity whose ten-thousandths
     *     do not fit a {@code long}
     */
    public static Quantity parse(String text) throws EranosException {
        return Converting.call(() -> new Quantity(Eranos.quantityParse(text)));
    }

    /**
     * The quantity of the double {@code value}: 0.45205 is 0.4521.
     *
     * <p>The library writes the double in its shortest decimal form, rounds it half away from zero
     * to four decimal places, and parses it.
     *
     * @param value the quantity as a double: 1.5 for one and a half units
     * @return the quantity
     * @throws EranosException.Money with {@link MoneyFailure.NotFinite} for NaN and the infinities,
     *     and {@link MoneyFailure.InvalidQuantity} for a value that rounds to zero or below and one
     *     whose ten-thousandths do not fit a {@code long}
     */
    public static Quantity fromDouble(double value) throws EranosException {
        return Converting.call(() -> new Quantity(Eranos.quantityFromF64(value)));
    }

    /**
     * The quantity of {@code tenThousandths}, a number the library gave.
     *
     * @param tenThousandths the quantity in ten-thousandths
     * @return the quantity
     */
    static Quantity known(long tenThousandths) {
        return new Quantity(tenThousandths);
    }

    /**
     * The quantity in ten-thousandths.
     *
     * @return 4 520 for 0.452
     */
    public long tenThousandths() {
        return tenThousandths;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Quantity quantity && tenThousandths == quantity.tenThousandths;
    }

    @Override
    public int hashCode() {
        return Long.hashCode(tenThousandths);
    }

    /**
     * The quantity as the library writes it.
     *
     * @return the text: "0.452", "1.5", or "3"
     */
    @Override
    public String toString() {
        return Eranos.quantityFormat(tenThousandths);
    }
}
