package eranos;

import java.util.Objects;

/** What the library's {@code MoneyError} reports: a type per failure, with what it holds. */
public sealed interface MoneyFailure {
    /** An amount that is not a decimal number. */
    record InvalidAmount() implements MoneyFailure {}

    /**
     * An amount with more decimal places than its currency has.
     *
     * @param currency the currency
     * @param allowed how many decimal places the currency has
     */
    record TooManyDecimals(CurrencyCode currency, long allowed) implements MoneyFailure {
        /**
         * The failure of these components.
         *
         * @param currency the currency
         * @param allowed how many decimal places the currency has
         */
        public TooManyDecimals {
            Objects.requireNonNull(currency, "currency");
        }
    }

    /**
     * Arithmetic between amounts of two different currencies.
     *
     * @param left the currency of the first amount
     * @param right the currency of the second amount
     */
    record CurrencyMismatch(CurrencyCode left, CurrencyCode right) implements MoneyFailure {
        /**
         * The failure of these components.
         *
         * @param left the currency of the first amount
         * @param right the currency of the second amount
         */
        public CurrencyMismatch {
            Objects.requireNonNull(left, "left");
            Objects.requireNonNull(right, "right");
        }
    }

    /** A currency code that is not an active ISO 4217 code. */
    record UnknownCurrency() implements MoneyFailure {}

    /** An exchange rate that is not a positive decimal number with at most 18 decimal places. */
    record InvalidExchangeRate() implements MoneyFailure {}

    /** A number of parts that is not a positive decimal number with at most two decimal places. */
    record InvalidParts() implements MoneyFailure {}

    /** A percentage that is not a decimal number with at most two decimal places. */
    record InvalidPercentage() implements MoneyFailure {}

    /**
     * A quantity that is not a positive decimal number with at most four decimal places, or whose
     * ten-thousandths do not fit a {@code long}.
     */
    record InvalidQuantity() implements MoneyFailure {}

    /**
     * A double that is NaN or infinite, which no amount, percentage, number of parts, quantity, or
     * exchange rate is.
     */
    record NotFinite() implements MoneyFailure {}

    /** A number or a result that does not fit its range. */
    record Overflow() implements MoneyFailure {}
}
