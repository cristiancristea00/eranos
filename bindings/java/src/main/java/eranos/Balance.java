package eranos;

import java.util.Objects;

/** Where a member stands with the group; it prints as the library writes it, "owes 12.00 EUR". */
public sealed interface Balance {
    /**
     * The member owes the others {@code amount}.
     *
     * @param amount what the member owes
     */
    record Owes(Money amount) implements Balance {
        /**
         * The balance of a member who owes {@code amount}.
         *
         * @param amount what the member owes
         */
        public Owes {
            Objects.requireNonNull(amount, "amount");
        }

        /**
         * The balance as the library writes it.
         *
         * @return the text: "owes 12.00 EUR"
         */
        @Override
        public String toString() {
            return Conversions.formatted(this);
        }
    }

    /** The member owes nothing and is owed nothing. */
    record Settled() implements Balance {
        /**
         * The balance as the library writes it.
         *
         * @return the text: "settled"
         */
        @Override
        public String toString() {
            return Conversions.formatted(this);
        }
    }

    /**
     * The others owe the member {@code amount}.
     *
     * @param amount what the member gets back
     */
    record GetsBack(Money amount) implements Balance {
        /**
         * The balance of a member who gets {@code amount} back.
         *
         * @param amount what the member gets back
         */
        public GetsBack {
            Objects.requireNonNull(amount, "amount");
        }

        /**
         * The balance as the library writes it.
         *
         * @return the text: "gets back 12.00 EUR"
         */
        @Override
        public String toString() {
            return Conversions.formatted(this);
        }
    }
}
