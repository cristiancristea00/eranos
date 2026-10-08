package eranos;

import eranos.internal.Eranos;

/**
 * A number of parts, to a hundredth: a member who pays half a part pays half of what a member with
 * one part pays.
 */
public final class Parts {
    private final int hundredths;

    private Parts(int hundredths) {
        this.hundredths = hundredths;
    }

    /**
     * The parts of {@code hundredths} hundredths.
     *
     * @param hundredths the parts in hundredths: 50 for half a part
     * @return the parts
     * @throws EranosException.Money with {@link MoneyFailure.InvalidParts} for a number that is not
     *     positive
     */
    public static Parts ofHundredths(int hundredths) throws EranosException {
        Converting.run(() -> Eranos.partsCheck(hundredths));
        return new Parts(hundredths);
    }

    /**
     * The parts written {@code text}: "1", "0.5", or "2.25".
     *
     * @param text the parts' text
     * @return the parts
     * @throws EranosException.Money with {@link MoneyFailure.InvalidParts} for text that is not a
     *     positive decimal with at most two decimal places
     */
    public static Parts parse(String text) throws EranosException {
        return Converting.call(() -> new Parts(Eranos.partsParse(text)));
    }

    /**
     * The parts of the double {@code value}: 1.005 is 1.01 parts.
     *
     * <p>The library writes the double in its shortest decimal form, rounds it half away from zero
     * to two decimal places, and parses it.
     *
     * @param value the parts as a double: 0.5 for half a part
     * @return the parts
     * @throws EranosException.Money with {@link MoneyFailure.NotFinite} for NaN and the infinities,
     *     and {@link MoneyFailure.InvalidParts} for a value that rounds to zero or below and one
     *     whose hundredths do not fit an {@code int}
     */
    public static Parts fromDouble(double value) throws EranosException {
        return Converting.call(() -> new Parts(Eranos.partsFromF64(value)));
    }

    /**
     * The parts of {@code hundredths}, a number the library gave.
     *
     * @param hundredths the parts in hundredths
     * @return the parts
     */
    static Parts known(int hundredths) {
        return new Parts(hundredths);
    }

    /**
     * The parts in hundredths.
     *
     * @return 50 for half a part
     */
    public int hundredths() {
        return hundredths;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Parts parts && hundredths == parts.hundredths;
    }

    @Override
    public int hashCode() {
        return Integer.hashCode(hundredths);
    }

    /**
     * The parts as the library writes them.
     *
     * @return the text: "0.5"
     */
    @Override
    public String toString() {
        return Eranos.partsFormat(hundredths);
    }
}
