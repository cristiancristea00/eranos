package eranos.examples.settlingup;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.participants;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.totals;

import eranos.Balance;
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
import eranos.Payment;
import eranos.Settlement;
import eranos.SettlementStrategy;
import eranos.Split;
import eranos.Summary;
import eranos.Transfer;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Settling up with each strategy on a group where they differ, then recording the chosen plan as
 * transfers until everyone is settled. The port of the Rust example {@code settling_up}, which
 * prints the same report.
 */
public final class Main {
    private Main() {}

    /** An expense with who paid it and who shares it, a row of the expenses' table. */
    private record ExpenseRow(Money amount, String name, String participants, String payer)
            implements Row {
        static final List<String> HEADERS = List.of("Expense", "Paid by", "Shared by", "Amount");

        @Override
        public List<String> cells() {
            return List.of(name, payer, participants, amount.toString());
        }
    }

    /** A member's totals, a row of the balances' table. */
    private record BalanceRow(Balance balance, Money consumed, String name, Money paid)
            implements Row {
        static final List<String> HEADERS = List.of("Member", "Paid", "Consumed", "Balance");

        @Override
        public List<String> cells() {
            return List.of(name, paid.toString(), consumed.toString(), balance.toString());
        }
    }

    /** A payment of a plan, a row of the payments' table. */
    private record PaymentRow(Money amount, String from, String to) implements Row {
        static final List<String> HEADERS = List.of("From", "To", "Amount");

        @Override
        public List<String> cells() {
            return List.of(from, to, amount.toString());
        }
    }

    /** A strategy with what it promises and what it costs, a row of the strategies' table. */
    private record StrategyRow(String cost, String guarantee, int payments, String strategy)
            implements Row {
        static final List<String> HEADERS =
                List.of("Strategy", "Guarantee", "Cost", "Payments here");

        @Override
        public List<String> cells() {
            return List.of(strategy, guarantee, cost, String.valueOf(payments));
        }
    }

    /** The number of payments of each strategy's plan. */
    private record Counts(int direct, int greedy, int minimum) {}

    /** Records an expense of {@code amount} EUR paid by {@code payer} and split equally. */
    private static void spend(
            Group group, String name, String amount, MemberId payer, List<MemberId> participants)
            throws EranosException {
        group.addExpense(
                Expense.builder(
                                new Cost.Local(Money.parse(amount, Currency.of("EUR"))),
                                CalendarDate.of(20, 9, 2026),
                                name,
                                new Split.Equal(List.of(payer)),
                                new ExpenseSplit.Whole(new Split.Equal(participants)))
                        .build());
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
                                            totals.balance(),
                                            totals.share(),
                                            member.firstName(),
                                            totals.paid())));
        }
        printTable(BalanceRow.HEADERS, rows, 1);
    }

    private static void printPayments(Group group, Settlement settlement) throws EranosException {
        List<PaymentRow> rows = new ArrayList<>();
        for (Payment payment : settlement.payments()) {
            rows.add(
                    new PaymentRow(
                            payment.amount(),
                            names(group, List.of(payment.from())),
                            names(group, List.of(payment.to()))));
        }
        printTable(PaymentRow.HEADERS, rows, 2);
    }

    /**
     * Prints each strategy's plan with what it promises; returns the number of payments of each.
     */
    private static Counts strategies(Group group) throws EranosException {
        section("3. Minimum transfers");
        Settlement minimum = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
        System.out.print(
                """
                The fewest payments possible. Finding them is NP-hard; the solver does it exactly by
                splitting the balances into as many groups that sum to zero as it can: here
                {Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than
                its size. %d payments:
                """
                        .formatted(minimum.payments().size()));
        printPayments(group, minimum);

        section("4. Greedy");
        Settlement greedy = group.settlement(SettlementStrategy.GREEDY);
        System.out.print(
                """
                The largest debtor pays the largest creditor, again and again: fast, at most one
                payment fewer than the members with a balance, but not always the minimum. %d
                payments here:
                """
                        .formatted(greedy.payments().size()));
        printPayments(group, greedy);

        section("5. Direct");
        Settlement direct = group.settlement(SettlementStrategy.DIRECT);
        System.out.print(
                """
                No simplification: everyone pays back the people who paid for them, with the debts
                between each pair of members netted. The most payments (%d), but each one traces
                back to shared expenses:
                """
                        .formatted(direct.payments().size()));
        printPayments(group, direct);

        section("6. The strategies side by side");
        printTable(
                StrategyRow.HEADERS,
                List.of(
                        new StrategyRow(
                                "O(2ⁿ·n) for n open balances",
                                "the fewest payments",
                                minimum.payments().size(),
                                "MinimumTransfers"),
                        new StrategyRow(
                                "O(n log n)",
                                "at most n − 1 payments",
                                greedy.payments().size(),
                                "Greedy"),
                        new StrategyRow(
                                "O(pairs)",
                                "one payment per pair in debt",
                                direct.payments().size(),
                                "Direct")),
                3);
        return new Counts(
                direct.payments().size(), greedy.payments().size(), minimum.payments().size());
    }

    /**
     * Prints the report of the ski week.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (Group group = Group.builder(Currency.of("EUR"), "Ski week").build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            MemberId carol = group.addMember(Member.builder("Carol").build());
            MemberId dan = group.addMember(Member.builder("Dan").build());
            MemberId eva = group.addMember(Member.builder("Eva").build());
            MemberId filip = group.addMember(Member.builder("Filip").build());

            System.out.println("Ski week: three ways to settle up");
            System.out.println("═════════════════════════════════");
            System.out.print(
                    """
                    Six friends spent a week in the mountains. The library can turn their balances
                    into payments in three ways; the app picks one, and they differ here.
                    """);

            section("1. What was spent");
            spend(group, "Cabin", "12.00", dan, List.of(dan, eva, filip));
            spend(group, "Lift passes", "9.00", bob, List.of(bob, carol, filip));
            spend(group, "Groceries", "6.00", carol, List.of(bob, dan));
            spend(group, "Coffee", "2.00", ana, List.of(dan, eva));
            spend(group, "Snacks", "1.00", filip, List.of(ana));
            List<ExpenseRow> expenses = new ArrayList<>();
            for (Expense expense : group.expenses()) {
                expenses.add(
                        new ExpenseRow(
                                expense.total(),
                                expense.name(),
                                names(group, participants(expense.split())),
                                names(group, expense.payers().members())));
            }
            printTable(ExpenseRow.HEADERS, expenses, 3);

            section("2. Balances");
            System.out.println("Four members are owed money and two owe it:");
            printBalances(group);

            Counts payments = strategies(group);

            section("7. Settling up for real");
            Settlement plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
            for (Payment payment : plan.payments()) {
                group.addTransfer(
                        Transfer.builder(
                                        new Cost.Local(payment.amount()),
                                        CalendarDate.of(27, 9, 2026),
                                        payment.from(),
                                        payment.to())
                                .build());
            }
            System.out.print(
                    """
                    Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards
                    every balance is zero and a new settlement has nothing left to pay:
                    """);
            printBalances(group);
            System.out.printf(
                    "Payments still needed: %d.%n",
                    group.settlement(SettlementStrategy.MINIMUM_TRANSFERS).payments().size());

            // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice;
            // greedy pays the largest amounts first and needs five; the unsimplified debts between
            // pairs need seven.
            check(new Counts(7, 5, 4).equals(payments));
            Summary summary = group.summary();
            for (MemberId member : List.of(ana, bob, carol, dan, eva, filip)) {
                check(
                        Optional.of(new Balance.Settled())
                                .equals(totals(member, summary).map(MemberSummary::balance)));
            }
        }
    }
}
