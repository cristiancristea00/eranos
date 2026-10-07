package eranos;

import java.util.Objects;

/** How a tip is expressed. */
public sealed interface TipValue {
    /**
     * A percentage of the expense's amount.
     *
     * @param percentage the percentage
     */
    record Percentage(eranos.Percentage percentage) implements TipValue {
        /**
         * The tip of {@code percentage} of the amount.
         *
         * @param percentage the percentage
         */
        public Percentage {
            Objects.requireNonNull(percentage, "percentage");
        }
    }

    /**
     * A fixed amount in the expense's currency.
     *
     * @param amount the amount
     */
    record Fixed(Money amount) implements TipValue {
        /**
         * The tip of {@code amount}.
         *
         * @param amount the amount
         */
        public Fixed {
            Objects.requireNonNull(amount, "amount");
        }
    }
}
