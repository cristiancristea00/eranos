package eranos.examples.multicurrency;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.totals;

import eranos.Balance;
import eranos.CalendarDate;
import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.Expense;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Split;
import eranos.Summary;
import eranos.Transfer;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON
 * and HUF carry a rate fixed when they are entered, and the group later moves its books to RON. The
 * port of the Rust example {@code multi_currency}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** An expense or a transfer as entered and in the group currency, a row of the entries. */
    private record EntryRow(
            String name, String payer, Money original, ExchangeRate rate, Money converted)
            implements Row {
        static final List<String> HEADERS =
                List.of("Entry", "Paid by", "Entered as", "Rate", "In the group currency");

        @Override
        public List<String> cells() {
            return List.of(name, payer, original.toString(), rate.toString(), converted.toString());
        }
    }

    /** A member's totals, a row of the balances' table. */
    private record BalanceRow(
            String name, Money paid, Money consumed, Money sent, Money received, Balance balance)
            implements Row {
        static final List<String> HEADERS =
                List.of("Member", "Paid", "Consumed", "Sent", "Received", "Balance");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    paid.toString(),
                    consumed.toString(),
                    sent.toString(),
                    received.toString(),
                    balance.toString());
        }
    }

    /** Every expense and transfer, as entered and in the group currency. */
    private static void printEntries(Group group) throws EranosException {
        Money zero = new Money(0, group.currency());
        List<EntryRow> rows = new ArrayList<>();
        for (Expense expense : group.expenses()) {
            Optional<ExpenseShares> shares =
                    expense.id().isPresent()
                            ? group.expenseShares(expense.id().orElseThrow())
                            : Optional.empty();
            rows.add(
                    new EntryRow(
                            expense.name(),
                            names(group, expense.payers().members()),
                            expense.total(),
                            rate(expense.cost()),
                            shares.map(ExpenseShares::total).orElse(zero)));
        }
        for (Transfer transfer : group.transfers()) {
            Optional<Money> amount =
                    transfer.id().isPresent()
                            ? group.transferAmount(transfer.id().orElseThrow())
                            : Optional.empty();
            rows.add(
                    new EntryRow(
                            "Transfer to " + names(group, List.of(transfer.to())),
                            names(group, List.of(transfer.from())),
                            transfer.cost().amount(),
                            rate(transfer.cost()),
                            amount.orElse(zero)));
        }
        printTable(EntryRow.HEADERS, rows, 2);
    }

    /** The rate {@code cost} was entered at: 1 for a cost in the group currency. */
    private static ExchangeRate rate(Cost cost) throws EranosException {
        return switch (cost) {
            case Cost.Local _ -> ExchangeRate.parse("1");
            case Cost.Converted converted -> converted.rate();
        };
    }

    private static void printBalances(Group group) {
        Summary summary = group.summary();
        List<BalanceRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            Optional<MemberSummary> found = member.id().flatMap(id -> totals(id, summary));
            found.ifPresent(
                    totals ->
                            rows.add(
                                    new BalanceRow(
                                            member.firstName(),
                                            totals.paid(),
                                            totals.share(),
                                            totals.sent(),
                                            totals.received(),
                                            totals.balance())));
        }
        printTable(BalanceRow.HEADERS, rows, 1);
    }

    private static void expenses(Group group, MemberId ana, MemberId bob) throws EranosException {
        section("1. Expenses in three currencies");
        ExpenseSplit both = new ExpenseSplit.Whole(new Split.Equal(List.of(ana, bob)));
        group.addExpense(
                Expense.builder(
                                "Fuel",
                                new Cost.Converted(
                                        Money.parse("400.00", Currency.of("RON")),
                                        ExchangeRate.parse("0.2")),
                                new Split.Equal(List.of(ana)),
                                both,
                                CalendarDate.of(2026, 8, 1))
                        .build());
        group.addExpense(
                Expense.builder(
                                "Dinner",
                                new Cost.Converted(
                                        Money.parse("24000.00", Currency.of("HUF")),
                                        ExchangeRate.parse("0.0025")),
                                new Split.Equal(List.of(bob)),
                                both,
                                CalendarDate.of(2026, 8, 2))
                        .build());
        group.addExpense(
                Expense.builder(
                                "Ferry",
                                new Cost.Local(Money.parse("60.00", Currency.of("EUR"))),
                                new Split.Equal(List.of(bob)),
                                both,
                                CalendarDate.of(2026, 8, 3))
                        .build());
        printEntries(group);
        System.out.print(
                """
                The rate says how many %s one unit of the entry's currency is worth: 400.00 RON at
                0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.
                """
                        .formatted(group.currency()));
    }

    private static void repayment(Group group, MemberId ana, MemberId bob) throws EranosException {
        section("3. A repayment in cash, in another currency");
        group.addTransfer(
                Transfer.builder(
                                ana,
                                bob,
                                new Cost.Converted(
                                        Money.parse("100.00", Currency.of("RON")),
                                        ExchangeRate.parse("0.2")),
                                CalendarDate.of(2026, 8, 4))
                        .note("Cash at the border")
                        .build());
        System.out.print(
                """
                Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.
                Transfers move balances but are not spending, so the total stays at %s:
                """
                        .formatted(group.summary().totalSpent()));
        printBalances(group);
    }

    private static void newCurrency(Group group) throws EranosException {
        section("4. Moving the books to another currency");
        System.out.print(
                """
                Changing the group currency needs a rate into the new currency for every other
                currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies
                take the new rate; entries already in RON become local costs, at 1.
                """);
        group.changeCurrency(
                Currency.of("RON"),
                Map.of(
                        Currency.of("EUR"), ExchangeRate.parse("5"),
                        Currency.of("HUF"), ExchangeRate.parse("0.0125")));
        printEntries(group);
        System.out.printf(
                "In %s, the trip cost %s, and everyone is still settled:%n",
                group.currency(), group.summary().totalSpent());
        printBalances(group);
    }

    /**
     * Prints the report of the road trip across currencies.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        Currency euro = Currency.of("EUR");
        Currency leu = Currency.of("RON");
        try (Group group = Group.builder("Danube road trip", euro).build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());

            System.out.println(group.name() + ": expenses in several currencies");
            System.out.println("════════════════════════════════════════════════");
            System.out.print(
                    """
                    Ana and Bob drive from Romania to Hungary. The group keeps its books in %s; an
                    expense in another currency carries the exchange rate used when it was entered,
                    and is converted once, rounding half away from zero to the cent.
                    """
                            .formatted(group.currency()));
            expenses(group, ana, bob);

            section("2. Balances");
            System.out.print(
                    """
                    The group spent %s. Bob paid 120.00 of it but consumed half, so he gets back the
                    difference:
                    """
                            .formatted(group.summary().totalSpent()));
            printBalances(group);
            check(
                    Optional.of(new Balance.GetsBack(Money.parse("20.00", euro)))
                            .equals(totals(bob, group.summary()).map(MemberSummary::balance)));

            repayment(group, ana, bob);
            check(
                    Optional.of(new Balance.Settled())
                            .equals(totals(bob, group.summary()).map(MemberSummary::balance)));

            newCurrency(group);
            // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
            Summary summary = group.summary();
            check(Money.parse("1000.00", leu).equals(summary.totalSpent()));
            check(
                    Optional.of(new Balance.Settled())
                            .equals(totals(bob, summary).map(MemberSummary::balance)));
        }
    }
}
