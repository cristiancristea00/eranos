package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * An amount: a whole number of minor units of a currency, cents for the euro.
 *
 * <p>An amount prints as the library writes it, "12.50 EUR", and adds and subtracts with its
 * checks: both amounts in one currency, and the result within range.
 *
 * @param minorUnits the amount in minor units: cents for the euro, none for the yen
 * @param currency the currency
 */
public record Money(long minorUnits, Currency currency) {
    /**
     * The amount of {@code minorUnits} minor units of {@code currency}.
     *
     * @param minorUnits the amount in minor units
     * @param currency the currency
     */
    public Money {
        Objects.requireNonNull(currency, "currency");
    }

    /**
     * The amount written {@code text} in {@code currency}: "12.50", "12.5", or "-3".
     *
     * @param text the amount's text
     * @param currency the currency
     * @return the amount
     * @throws EranosException.Money with {@link MoneyFailure.InvalidAmount} for text that is not a
     *     decimal number, and {@link MoneyFailure.TooManyDecimals} for more decimal places than the
     *     currency has
     */
    public static Money parse(String text, Currency currency) throws EranosException {
        return Converting.call(() -> converted(Eranos.moneyParse(text, currency.code())));
    }

    /**
     * The amount the generated code holds as {@code money}.
     *
     * @param money the generated amount
     * @return the amount
     */
    static Money converted(eranos.internal.Money money) {
        return new Money(money.minorUnits(), Currency.known(money.currency()));
    }

    /**
     * The amount as the generated code holds it.
     *
     * @return the generated amount
     */
    eranos.internal.Money generated() {
        return new eranos.internal.Money(minorUnits, currency.code());
    }

    /**
     * This amount plus {@code other}.
     *
     * @param other the amount to add
     * @return the sum
     * @throws EranosException.Money with {@link MoneyFailure.CurrencyMismatch} for amounts in two
     *     currencies, and {@link MoneyFailure.Overflow} for a sum out of range
     */
    public Money plus(Money other) throws EranosException {
        return Converting.call(() -> converted(Eranos.moneyAdd(generated(), other.generated())));
    }

    /**
     * This amount minus {@code other}.
     *
     * @param other the amount to subtract
     * @return the difference
     * @throws EranosException.Money with {@link MoneyFailure.CurrencyMismatch} for amounts in two
     *     currencies, and {@link MoneyFailure.Overflow} for a difference out of range
     */
    public Money minus(Money other) throws EranosException {
        return Converting.call(
                () -> converted(Eranos.moneySubtract(generated(), other.generated())));
    }

    /**
     * The amount as the library writes it.
     *
     * @return the text: "12.50 EUR"
     */
    @Override
    public String toString() {
        try {
            return Eranos.moneyFormat(generated());
        } catch (eranos.internal.EranosException exception) {
            return minorUnits + " " + currency.code();
        }
    }
}
