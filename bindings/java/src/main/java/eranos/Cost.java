package eranos;

import java.util.Objects;

/**
 * What an entry cost: an amount in the group currency, or one in another currency with the rate
 * that converts it into the group currency.
 */
public sealed interface Cost {
    /**
     * The amount, in its own currency.
     *
     * @return the amount
     */
    Money amount();

    /**
     * An amount in another currency than the group's, converted at {@code rate}.
     *
     * @param amount the amount, in its own currency
     * @param rate the rate that converts it into the group currency
     */
    record Converted(Money amount, ExchangeRate rate) implements Cost {
        /**
         * The cost of {@code amount}, converted at {@code rate}.
         *
         * @param amount the amount, in its own currency
         * @param rate the rate that converts it into the group currency
         */
        public Converted {
            Objects.requireNonNull(amount, "amount");
            Objects.requireNonNull(rate, "rate");
        }
    }

    /**
     * An amount in the group currency.
     *
     * @param amount the amount
     */
    record Local(Money amount) implements Cost {
        /**
         * The cost of {@code amount}.
         *
         * @param amount the amount
         */
        public Local {
            Objects.requireNonNull(amount, "amount");
        }
    }
}
