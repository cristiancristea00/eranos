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
    final CurrencyCode euro;
    final CalendarDate day;
    final ExpenseGroup group;
    final MemberId ana;
    final MemberId bob;
    final CategoryId food;
    final ExpenseId dinner;
    final TransferId repayment;

    Flat() throws EranosException {
        euro = CurrencyCode.of("EUR");
        day = CalendarDate.of(2026, 9, 23);
        group =
                ExpenseGroup.builder("Flat", euro)
                        .description("Rent")
                        .language(Language.parse("pt-pt"))
                        .build();
        ana = group.addMember(Member.builder("Ana").build());
        bob = group.addMember(Member.builder("Bob").build());
        food = group.addCategory(new Category("Food"));
        dinner =
                group.addExpense(
                        Expense.builder(
                                        "Dinner",
                                        new Cost.Local(euros(3_000)),
                                        new Split.Equal(List.of(ana)),
                                        new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob))),
                                        day)
                                .category(food)
                                .build());
        repayment =
                group.addTransfer(
                        Transfer.builder(bob, ana, new Cost.Local(euros(1_000)), day).build());
    }

    /** {@code minorUnits} cents. */
    Money euros(long minorUnits) {
        return new Money(minorUnits, euro);
    }

    /**
     * An expense of 30.00 EUR, called {@code name}, that {@code payer} paid and {@code members}
     * share equally.
     */
    Expense expense(String name, MemberId payer, List<MemberId> members) {
        return Expense.builder(
                        name,
                        new Cost.Local(euros(3_000)),
                        new Split.Equal(List.of(payer)),
                        new ExpenseSplit.Whole(new Split.Equal(members)),
                        day)
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
