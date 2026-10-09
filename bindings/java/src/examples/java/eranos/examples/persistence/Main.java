package eranos.examples.persistence;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.title;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.CurrencyCode;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseGroup;
import eranos.ExpenseSplit;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;
import eranos.Percentage;
import eranos.SettlementPlan;
import eranos.SettlementStrategy;
import eranos.Split;
import eranos.Summary;
import eranos.TipRule;
import eranos.TipValue;
import eranos.examples.Report.Row;

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

    /** A member's balance before saving and after loading, a row of the comparison's table. */
    private record ComparisonRow(String name, String before, String after) implements Row {
        static final List<String> HEADERS =
                List.of("Member", "Balance before saving", "Balance after loading");

        @Override
        public List<String> cells() {
            return List.of(name, before, after);
        }
    }

    /** The balance of {@code member} in {@code summary}, as the library writes it. */
    private static String balance(MemberId member, Summary summary) {
        return summary.member(member).balance().toString();
    }

    /**
     * Prints the report of the road trip saved and loaded.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        CurrencyCode euro = CurrencyCode.of("EUR");
        CalendarDate day = CalendarDate.of(2026, 9, 23);
        try (ExpenseGroup group = ExpenseGroup.builder("Road trip", euro).build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            ExpenseSplit both = new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob)));
            group.addExpense(
                    Expense.builder(
                                    "Fuel",
                                    new Cost.Local(Money.parse("70.00", euro)),
                                    new Split.Equal(List.of(ana)),
                                    both,
                                    day)
                            .build());
            group.addExpense(
                    Expense.builder(
                                    "Lunch",
                                    new Cost.Local(Money.parse("30.00", euro)),
                                    new Split.Equal(List.of(bob)),
                                    both,
                                    day)
                            .tip(new TipRule(new TipValue.Percentage(Percentage.parse("10"))))
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
            try (ExpenseGroup loaded = ExpenseGroup.load(saved)) {
                System.out.print(
                        """
                        Loading replays every entry through the same checks as adding it, then rebuilds
                        the totals. The loaded group equals the saved one, and so do its balances:
                        """);
                Summary before = group.summary();
                Summary after = loaded.summary();
                List<ComparisonRow> rows = new ArrayList<>();
                for (MemberId member : List.of(ana, bob)) {
                    rows.add(
                            new ComparisonRow(
                                    group.member(member).map(Member::firstName).orElse("?"),
                                    balance(member, before),
                                    balance(member, after)));
                }
                printTable(ComparisonRow.HEADERS, rows, 1);
                SettlementPlan plan = loaded.settlement(SettlementStrategy.GREEDY);
                System.out.printf(
                        "Payments needed to settle the loaded group: %d.%n",
                        plan.payments().size());

                section("3. A corrupted file is refused");
                System.out.print(
                        """
                        Here Ana's identifier was edited in her member entry, so the fuel's payer is an
                        identifier no member has. Reading the file as a SavedExpenseGroup and loading it
                        with into_expense_group replays every entry through the same checks as adding it,
                        and refuses the file instead of producing balances that do not add up:
                        """);
                // Ana's own entry is the first to name her: the members come before the expenses
                // in the saved form, so the fuel she paid is left naming an identifier no member
                // has.
                String corrupted =
                        saved.replaceFirst(
                                Pattern.quote(ana.value()), "0199a000-0000-7000-8000-00000000ffff");
                try {
                    ExpenseGroup.load(corrupted).close();
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
