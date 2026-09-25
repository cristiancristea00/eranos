// A group the tests share the shape of, and the check every value type's equality takes.

package eranos;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

import java.util.List;

/**
 * A group of two with a category, an expense, and a transfer, and the identifiers the group gave
 * them.
 */
final class Flat {
    final Currency euro;
    final CalendarDate day;
    final Group group;
    final MemberId ana;
    final MemberId bob;
    final CategoryId food;
    final ExpenseId dinner;
    final TransferId repayment;

    Flat() throws EranosException {
        euro = Currency.of("EUR");
        day = CalendarDate.of(23, 9, 2026);
        group =
                Group.builder(euro, "Flat")
                        .description("Rent")
                        .language(Language.parse("pt-pt"))
                        .build();
        ana = group.addMember(Member.builder("Ana").build());
        bob = group.addMember(Member.builder("Bob").build());
        food = group.addCategory(new Category("Food"));
        dinner =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(euros(3_000)),
                                        day,
                                        "Dinner",
                                        new Split.Equal(List.of(ana)),
                                        new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob))))
                                .category(food)
                                .build());
        repayment =
                group.addTransfer(
                        Transfer.builder(new Cost.Local(euros(1_000)), day, bob, ana).build());
    }

    /** {@code minorUnits} cents. */
    Money euros(long minorUnits) {
        return new Money(euro, minorUnits);
    }

    /**
     * An expense of 30.00 EUR, called {@code name}, that {@code payer} paid and {@code members}
     * share equally.
     */
    Expense expense(String name, MemberId payer, List<MemberId> members) {
        return Expense.builder(
                        new Cost.Local(euros(3_000)),
                        day,
                        name,
                        new Split.Equal(List.of(payer)),
                        new ExpenseSplit.Whole(new Split.Equal(members)))
                .build();
    }

    /**
     * Checks that {@code value} equals {@code same}, with the same hash and text, and differs from
     * {@code other}: a class that wrote its equality out with a field left out would merge two
     * values an app holds apart.
     */
    static <T> void assertValue(T value, T same, T other) {
        assertEquals(value, same);
        assertEquals(value.hashCode(), same.hashCode());
        assertEquals(value.toString(), same.toString());
        assertNotEquals(value, other);
    }
}
