package eranos;

import eranos.internal.Eranos;

/** The rate that converts an amount in one currency into another: a positive decimal. */
public final class ExchangeRate {
    private final String text;

    private ExchangeRate(String text) {
        this.text = text;
    }

    /**
     * The rate written {@code text}: "4.97" or "0.2012".
     *
     * @param text the rate's text
     * @return the rate
     * @throws EranosException.Money with {@link MoneyFailure.InvalidExchangeRate} for text that is
     *     not a positive decimal with at most 18 decimal places
     */
    public static ExchangeRate parse(String text) throws EranosException {
        return Converting.call(() -> new ExchangeRate(Eranos.exchangeRateParse(text)));
    }

    /**
     * The rate the library wrote as {@code text}.
     *
     * @param text the rate's text
     * @return the rate
     */
    static ExchangeRate known(String text) {
        return new ExchangeRate(text);
    }

    /**
     * {@code amount} converted at this rate into {@code currency}, rounded to its minor unit,
     * halves away from zero.
     *
     * @param amount the amount to convert
     * @param currency the currency to convert it into
     * @return the converted amount
     * @throws EranosException.Money with {@link MoneyFailure.Overflow} for a result out of range
     */
    public Money convert(Money amount, Currency currency) throws EranosException {
        return Converting.call(
                () ->
                        Money.converted(
                                Eranos.exchangeRateConvert(
                                        text, amount.generated(), currency.code())));
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof ExchangeRate rate && text.equals(rate.text);
    }

    @Override
    public int hashCode() {
        return text.hashCode();
    }

    /**
     * The rate as the library writes it.
     *
     * @return the text: "4.97"
     */
    @Override
    public String toString() {
        return text;
    }
}
