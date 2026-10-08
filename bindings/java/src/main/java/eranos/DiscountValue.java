package eranos;

import java.util.Objects;

/** How a discount is expressed. */
public sealed interface DiscountValue {
    /**
     * A percentage of what the discount reduces, at most 100 %.
     *
     * @param percentage the percentage
     */
    record Percentage(eranos.Percentage percentage) implements DiscountValue {
        /**
         * The discount of {@code percentage} of what it reduces.
         *
         * @param percentage the percentage
         */
        public Percentage {
            Objects.requireNonNull(percentage, "percentage");
        }
    }

    /**
     * A fixed amount in the item's currency, at most what the discount reduces.
     *
     * @param amount the amount
     */
    record Fixed(Money amount) implements DiscountValue {
        /**
         * The discount of {@code amount}.
         *
         * @param amount the amount
         */
        public Fixed {
            Objects.requireNonNull(amount, "amount");
        }
    }
}
