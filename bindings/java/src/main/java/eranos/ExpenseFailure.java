package eranos;

import java.util.Objects;

/** What the library's {@code ExpenseError} reports: a type per failure, with what it holds. */
public sealed interface ExpenseFailure {
    /**
     * The amounts of a split do not add up to the amount being split.
     *
     * @param actual what the amounts add up to
     * @param expected the amount being split
     */
    record AmountsDoNotAddUp(Money actual, Money expected) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param actual what the amounts add up to
         * @param expected the amount being split
         */
        public AmountsDoNotAddUp {
            Objects.requireNonNull(actual, "actual");
            Objects.requireNonNull(expected, "expected");
        }
    }

    /**
     * An amount inside the expense is in a different currency.
     *
     * @param expected the expense's currency
     * @param found the currency of the amount inside it
     */
    record CurrencyMismatch(Currency expected, Currency found) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param expected the expense's currency
         * @param found the currency of the amount inside it
         */
        public CurrencyMismatch {
            Objects.requireNonNull(expected, "expected");
            Objects.requireNonNull(found, "found");
        }
    }

    /**
     * A member appears more than once in one split.
     *
     * @param member the member who appears more than once
     */
    record DuplicateMember(MemberId member) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param member the member who appears more than once
         */
        public DuplicateMember {
            Objects.requireNonNull(member, "member");
        }
    }

    /** An item whose name is blank. */
    record EmptyItemName() implements ExpenseFailure {}

    /** An expense whose name is blank. */
    record EmptyName() implements ExpenseFailure {}

    /** A split lists no member. */
    record EmptySplit() implements ExpenseFailure {}

    /**
     * The item line totals do not add up to the expense amount.
     *
     * @param actual what the line totals add up to
     * @param expected the expense amount
     */
    record ItemsDoNotAddUp(Money actual, Money expected) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param actual what the line totals add up to
         * @param expected the expense amount
         */
        public ItemsDoNotAddUp {
            Objects.requireNonNull(actual, "actual");
            Objects.requireNonNull(expected, "expected");
        }
    }

    /** A fixed tip below zero. */
    record NegativeTip() implements ExpenseFailure {}

    /** An itemised expense without items. */
    record NoItems() implements ExpenseFailure {}

    /** The amount or unit price is zero or negative. */
    record NonPositiveAmount() implements ExpenseFailure {}

    /**
     * A member's percentage or amount is zero or negative.
     *
     * @param member the member whose share is zero or negative
     */
    record NonPositiveShare(MemberId member) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param member the member whose share is zero or negative
         */
        public NonPositiveShare {
            Objects.requireNonNull(member, "member");
        }
    }

    /** A total that does not fit the range of an amount. */
    record Overflow() implements ExpenseFailure {}

    /**
     * The percentages of a split do not add up to exactly 100 %.
     *
     * @param totalBasisPoints what the percentages add up to, in basis points
     */
    record PercentagesDoNotAddUp(long totalBasisPoints) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param totalBasisPoints what the percentages add up to, in basis points
         */
        public PercentagesDoNotAddUp {}
    }

    /** An item quantity of zero. */
    record ZeroQuantity() implements ExpenseFailure {}
}
