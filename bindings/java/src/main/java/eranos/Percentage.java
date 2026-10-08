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
