package eranos.examples.persistence;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.title;
import static eranos.examples.Report.totals;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Percentage;
import eranos.Settlement;
import eranos.SettlementStrategy;
import eranos.Split;
import eranos.Summary;
import eranos.Tip;
import eranos.TipValue;

import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;

/**
 * Saving and loading a group as JSON. The saved form holds the entries only; loading replays them
 * through the group's checks and recomputes every total. The port of the Rust example {@code
 * persistence}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** The balance of {@code member} in {@code summary}, as the library writes it. */
    private static String balance(MemberId member, Summary summary) {
        return totals(member, summary)
                .map(MemberSummary::balance)
                .map(Object::toString)
                .orElse("?");
    }

    /**
     * Prints the report of the road trip saved and loaded.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        Currency euro = Currency.of("EUR");
        CalendarDate day = CalendarDate.of(23, 9, 2026);
        try (Group group = Group.builder(euro, "Road trip").build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            ExpenseSplit both = new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob)));
            group.addExpense(
                    Expense.builder(
                                    new Cost.Local(Money.parse("70.00", euro)),
                                    day,
                                    "Fuel",
                                    new Split.Equal(List.of(ana)),
                                    both)
                            .build());
            group.addExpense(
                    Expense.builder(
                                    new Cost.Local(Money.parse("30.00", euro)),
                                    day,
                                    "Lunch",
                                    new Split.Equal(List.of(bob)),
                                    both)
                            .tip(new Tip(new TipValue.Percentage(Percentage.parse("10"))))
                            .build());

            title("Road trip: saving and loading a group");
            System.out.print(
                    """
                    With the `serde` feature a group serialises, and so does every type it stores.
                    It saves only what was entered: its details, members, categories, expenses,
                    and transfers. The totals, balances, and settlements are recomputed when it is
                    loaded.
                    """);

            section("1. The saved form");
            // Account details would be in clear text: store this in the platform's secure storage
            // (Keychain, Keystore, or an encrypted database).
            String saved = group.save();
            System.out.println(saved);
            System.out.print(
                    """
                    Amounts are minor units with their currency, dates are YYYY-MM-DD, and the
                    `format` number lets a future release refuse a file it cannot read instead of
                    dropping fields.
                    """);

            section("2. Loading it back");
            try (Group loaded = Group.load(saved)) {
                System.out.print(
                        """
                        Loading replays every entry through the same checks as adding it, then rebuilds
                        the totals. The loaded group equals the saved one, and so do its balances:
                        """);
                Summary before = group.summary();
                Summary after = loaded.summary();
                List<List<String>> rows = new ArrayList<>();
                for (MemberId member : List.of(ana, bob)) {
                    rows.add(
                            List.of(
                                    group.member(member).map(Member::firstName).orElse("?"),
                                    balance(member, before),
                                    balance(member, after)));
                }
                printTable(
                        List.of("Member", "Balance before saving", "Balance after loading"),
                        rows,
                        1);
                Settlement plan = loaded.settlement(SettlementStrategy.GREEDY);
                System.out.printf(
                        "Payments needed to settle the loaded group: %d.%n",
                        plan.payments().size());

                section("3. A corrupted file is refused");
                System.out.print(
                        """
                        Here the fuel's payer was edited to an identifier no member has. Reading the file
                        as a SavedGroup and loading it with into_group replays every entry through the
                        same checks as adding it, and refuses the file instead of producing balances that
                        do not add up:
                        """);
                // The fuel is the first entry that names Ana: the expenses come before the members
                // in
                // the saved form.
                String corrupted =
                        saved.replaceFirst(
                                Pattern.quote(ana.value()), "0199a000-0000-7000-8000-00000000ffff");
                try {
                    Group.load(corrupted).close();
                    throw new IllegalStateException("the file must be refused");
                } catch (EranosException refusal) {
                    System.out.println("  error: " + refusal.getMessage());
                }

                check(saved.equals(loaded.save()));
                check(before.equals(after));
                check(1 == plan.payments().size());
            }
        }
    }
}
