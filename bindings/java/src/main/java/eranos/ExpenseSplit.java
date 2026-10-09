package eranos;

import eranos.internal.Eranos;

import java.util.List;
import java.util.Objects;

/** How an expense's cost is shared: as a whole, or line by line of an itemised bill. */
public sealed interface ExpenseSplit {
    /**
     * Every member who shares the cost, once each, in identifier order.
     *
     * @return the members
     * @throws EranosException.Id for a member whose identifier is not one, and the failures of an
     *     item that does not build
     */
    default List<MemberId> participants() throws EranosException {
        return Converting.call(
                () ->
                        Eranos.expenseSplitParticipants(Conversions.generated(this)).stream()
                                .map(MemberId::new)
                                .toList());
    }

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
    record Itemised(List<ExpenseItem> items) implements ExpenseSplit {
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
