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
public record Money(long minorUnits, CurrencyCode currency) {
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
     * Zero in {@code currency}.
     *
     * @param currency the currency
     * @return zero minor units of the currency
     */
    public static Money zero(CurrencyCode currency) {
        return new Money(0, currency);
    }

    /**
     * Whether the amount is zero.
     *
     * @return whether it is zero
     */
    public boolean isZero() {
        return 0 == minorUnits;
    }

    /**
     * Whether the amount is above zero.
     *
     * @return whether it is above zero
     */
    public boolean isPositive() {
        return 0 < minorUnits;
    }

    /**
     * Whether the amount is below zero.
     *
     * @return whether it is below zero
     */
    public boolean isNegative() {
        return 0 > minorUnits;
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
    public static Money parse(String text, CurrencyCode currency) throws EranosException {
        return Converting.call(() -> converted(Eranos.moneyParse(text, currency.code())));
    }

    /**
     * The amount of the double {@code value} in {@code currency}: 1.005 in EUR is 1.01 EUR.
     *
     * <p>The library writes the double in its shortest decimal form, rounds it half away from zero
     * to the currency's decimal places, and parses it, so every language gets the same amount from
     * the same double. The double nearest 1.005 lies below it, but its shortest form is 1.005,
     * which rounds up.
     *
     * @param value the amount as a double
     * @param currency the currency
     * @return the amount
     * @throws EranosException.Money with {@link MoneyFailure.NotFinite} for NaN and the infinities,
     *     and {@link MoneyFailure.Overflow} for an amount that does not fit in 64-bit minor units
     */
    public static Money fromDouble(double value, CurrencyCode currency) throws EranosException {
        return Converting.call(() -> converted(Eranos.moneyFromF64(value, currency.code())));
    }

    /**
     * The amount the generated code holds as {@code money}.
     *
     * @param money the generated amount
     * @return the amount
     */
    static Money converted(eranos.internal.Money money) {
        return new Money(money.minorUnits(), CurrencyCode.converted(money.currency()));
    }

    /**
     * The amount as the generated code holds it.
     *
     * @return the generated amount
     */
    eranos.internal.Money generated() {
        return new eranos.internal.Money(minorUnits, currency.generated());
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
     * The amount as the library writes it, with the decimal places of its currency.
     *
     * @return the text: "12.50 EUR"
     */
    @Override
    public String toString() {
        return Eranos.moneyFormat(generated());
    }
}
