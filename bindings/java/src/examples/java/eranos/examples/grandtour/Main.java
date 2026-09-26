package eranos.examples.grandtour;

import eranos.EranosException;
import eranos.ExpenseId;
import eranos.Group;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

/**
 * Every feature of the library in one story: a group and its details, members with contact details
 * and accounts, categories, an expense for every way of dividing one, another currency, edits and
 * refusals, members who come and go, the summary, a new group currency, settling up, saving and
 * loading, the values on their own, and the group shared between threads. Every section checks its
 * key numbers. The port of the Rust example {@code grand_tour}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /**
     * Prints the report of the trip to Lisbon and Porto.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        System.out.println("Lisbon and Porto: a grand tour of eranos");
        System.out.println("════════════════════════════════════════");
        System.out.print(
                """
                Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each
                section below uses another part of the library, from creating the group to saving
                it, and checks its key numbers.
                """);
        try (Group group = Members.theGroup()) {
            People people = Members.members(group);
            Categories categories = Members.addCategories(group);
            ExpenseId seafood = Expenses.expenses(group, people, categories);
            Edits.anotherCurrency(group, people, categories);
            Edits.edits(group, people, categories, seafood);
            Edits.comingsAndGoings(group, people);
            Totals.summary(group, people, categories);
            Totals.removingACategory(group, categories);
            Totals.newCurrency(group, people);
            Totals.settlingUp(group);
            Saving.savingAndLoading(group, people, categories);
            Values.values(group, people);
            Sharing.sharing(group, people);
        }
    }
}
