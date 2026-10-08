package eranos;

import java.util.Objects;

/** What the library's {@code ExpenseError} reports: a type per failure, with what it holds. */
public sealed interface ExpenseFailure {
    /** An expense whose name is blank. */
    record EmptyName() implements ExpenseFailure {}

    /** The amount or unit price is zero or negative. */
    record NonPositiveAmount() implements ExpenseFailure {}

    /**
     * An amount inside the expense is in a different currency.
     *
     * @param expected the expense's currency
     * @param found the currency of the amount inside it
     */
    record CurrencyMismatch(CurrencyCode expected, CurrencyCode found) implements ExpenseFailure {
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

    /** A split lists no member. */
    record EmptySplit() implements ExpenseFailure {}

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

    /**
     * The amounts of a split do not add up to the amount being split.
     *
     * @param expected the amount being split
     * @param actual what the amounts add up to
     */
    record AmountsDoNotAddUp(Money expected, Money actual) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param expected the amount being split
         * @param actual what the amounts add up to
         */
        public AmountsDoNotAddUp {
            Objects.requireNonNull(expected, "expected");
            Objects.requireNonNull(actual, "actual");
        }
    }

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

    /** An itemised expense without items. */
    record NoItems() implements ExpenseFailure {}

    /** An item whose name is blank. */
    record EmptyItemName() implements ExpenseFailure {}

    /**
     * The item line totals do not add up to the expense amount.
     *
     * @param expected the expense amount
     * @param actual what the line totals add up to
     */
    record ItemsDoNotAddUp(Money expected, Money actual) implements ExpenseFailure {
        /**
         * The failure of these components.
         *
         * @param expected the expense amount
         * @param actual what the line totals add up to
         */
        public ItemsDoNotAddUp {
            Objects.requireNonNull(expected, "expected");
            Objects.requireNonNull(actual, "actual");
        }
    }

    /** A fixed discount below zero. */
    record NegativeDiscount() implements ExpenseFailure {}

    /** A percentage discount above 100 %. */
    record DiscountAboveHundredPercent() implements ExpenseFailure {}

    /**
     * A fixed discount larger than what it reduces: the line total before the discount, or the unit
     * price for a discount off each unit.
     */
    record DiscountAbovePrice() implements ExpenseFailure {}

    /** A fixed tip below zero. */
    record NegativeTip() implements ExpenseFailure {}

    /** A total that does not fit the range of an amount. */
    record Overflow() implements ExpenseFailure {}
}
