package eranos;

import java.util.List;
import java.util.Objects;

/** How an expense's cost is shared: as a whole, or line by line of an itemised bill. */
public sealed interface ExpenseSplit {
    /**
     * The whole amount is divided by one split.
     *
     * @param split how the amount divides
     */
    record Whole(Split split) implements ExpenseSplit {
        /**
         * The amount divided by {@code split}.
         *
         * @param split how the amount divides
         */
        public Whole {
            Objects.requireNonNull(split, "split");
        }
    }

    /**
     * Every line is divided by its own split; the lines add up to the amount.
     *
     * @param items the lines of the bill
     */
    record Itemised(List<Item> items) implements ExpenseSplit {
        /**
         * The bill of {@code items}, which it copies.
         *
         * @param items the lines of the bill
         */
        public Itemised {
            items = List.copyOf(items);
        }
    }
}
