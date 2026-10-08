package eranos;

import eranos.internal.Eranos;

/** A share of a whole, to a hundredth of a per cent: 12.5 % is 1 250 basis points. */
public final class Percentage {
    private final int basisPoints;

    private Percentage(int basisPoints) {
        this.basisPoints = basisPoints;
    }

    /**
     * The percentage of {@code basisPoints} hundredths of a per cent.
     *
     * @param basisPoints the percentage in basis points: 1 250 for 12.5 %
     * @return the percentage
     * @throws EranosException.Money with {@link MoneyFailure.InvalidPercentage} for a negative
     *     number
     */
    public static Percentage ofBasisPoints(int basisPoints) throws EranosException {
        Converting.run(() -> Eranos.percentageCheck(basisPoints));
        return new Percentage(basisPoints);
    }

    /**
     * The percentage written {@code text}: "12.5", "12.5%", or "100".
     *
     * @param text the percentage's text
     * @return the percentage
     * @throws EranosException.Money with {@link MoneyFailure.InvalidPercentage} for text that is
     *     not a non-negative decimal with at most two decimal places
     */
    public static Percentage parse(String text) throws EranosException {
        return Converting.call(() -> new Percentage(Eranos.percentageParse(text)));
    }

    /**
     * The percentage of the double {@code value}, a number of per cent: 12.345 is 12.35 %.
     *
     * <p>The library writes the double in its shortest decimal form, rounds it half away from zero
     * to two decimal places, and parses it: {@code 0.1 + 0.2}, which is 0.30000000000000004 as a
     * double, is 0.3 %.
     *
     * @param value the percentage as a double: 12.5 for 12.5 %
     * @return the percentage
     * @throws EranosException.Money with {@link MoneyFailure.NotFinite} for NaN and the infinities,
     *     and {@link MoneyFailure.InvalidPercentage} for a negative value and one whose basis
     *     points do not fit an {@code int}
     */
    public static Percentage fromDouble(double value) throws EranosException {
        return Converting.call(() -> new Percentage(Eranos.percentageFromF64(value)));
    }

    /**
     * The percentage of {@code basisPoints}, a number the library gave.
     *
     * @param basisPoints the percentage in basis points
     * @return the percentage
     */
    static Percentage known(int basisPoints) {
        return new Percentage(basisPoints);
    }

    /**
     * The percentage in basis points.
     *
     * @return 1 250 for 12.5 %
     */
    public int basisPoints() {
        return basisPoints;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Percentage percentage && basisPoints == percentage.basisPoints;
    }

    @Override
    public int hashCode() {
        return Integer.hashCode(basisPoints);
    }

    /**
     * The percentage as the library writes it.
     *
     * @return the text: "12.5%"
     */
    @Override
    public String toString() {
        return Eranos.percentageFormat(basisPoints);
    }
}
