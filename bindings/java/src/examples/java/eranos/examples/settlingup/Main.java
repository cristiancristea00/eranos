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
import eranos.ExpenseGroup;
import eranos.ExpenseSplit;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Payment;
import eranos.SettlementPlan;
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
    private record ExpenseRow(String name, String payer, String participants, Money amount)
            implements Row {
        static final List<String> HEADERS = List.of("Expense", "Paid by", "Shared by", "Amount");

        @Override
        public List<String> cells() {
            return List.of(name, payer, participants, amount.toString());
        }
    }

    /** A member's totals, a row of the balances' table. */
    private record BalanceRow(String name, Money paid, Money consumed, Balance balance)
            implements Row {
        static final List<String> HEADERS = List.of("Member", "Paid", "Consumed", "Balance");

        @Override
        public List<String> cells() {
            return List.of(name, paid.toString(), consumed.toString(), balance.toString());
        }
    }

    /** A payment of a plan, a row of the payments' table. */
    private record PaymentRow(String from, String to, Money amount) implements Row {
        static final List<String> HEADERS = List.of("From", "To", "Amount");

        @Override
        public List<String> cells() {
            return List.of(from, to, amount.toString());
        }
    }

    /** A strategy with what it promises and what it costs, a row of the strategies' table. */
    private record StrategyRow(String strategy, String guarantee, String cost, int payments)
            implements Row {
        static final List<String> HEADERS =
                List.of("Strategy", "Guarantee", "Cost", "Payments here");

        @Override
        public List<String> cells() {
            return List.of(strategy, guarantee, cost, String.valueOf(payments));
        }
    }

    /** The number of payments of each strategy's plan. */
    private record Counts(int adaptive, int minimum, int greedy, int direct) {}

    /** The name of {@code strategy} as the apps show it. */
    private static String strategyName(SettlementStrategy strategy) {
        return switch (strategy) {
            case ADAPTIVE -> "Adaptive";
            case MINIMUM_TRANSFERS -> "MinimumTransfers";
            case GREEDY -> "Greedy";
            case DIRECT -> "Direct";
        };
    }

    /** Records an expense of {@code amount} EUR paid by {@code payer} and split equally. */
    private static void spend(
            ExpenseGroup group,
            String name,
            String amount,
            MemberId payer,
            List<MemberId> participants)
            throws EranosException {
        group.addExpense(
                Expense.builder(
                                name,
                                new Cost.Local(Money.parse(amount, Currency.of("EUR"))),
                                new Split.Equal(List.of(payer)),
                                new ExpenseSplit.Whole(new Split.Equal(participants)),
                                CalendarDate.of(2026, 9, 20))
                        .build());
    }

    private static void printBalances(ExpenseGroup group) {
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
                                            totals.balance())));
        }
        printTable(BalanceRow.HEADERS, rows, 1);
    }

    private static void printPayments(ExpenseGroup group, SettlementPlan settlement)
            throws EranosException {
        List<PaymentRow> rows = new ArrayList<>();
        for (Payment payment : settlement.payments()) {
            rows.add(
                    new PaymentRow(
                            names(group, List.of(payment.from())),
                            names(group, List.of(payment.to())),
                            payment.amount()));
        }
        printTable(PaymentRow.HEADERS, rows, 2);
    }

    /**
     * Prints each strategy's plan with what it promises; returns the number of payments of each.
     */
    private static Counts strategies(ExpenseGroup group) throws EranosException {
        section("3. Minimum transfers");
        SettlementPlan minimum = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
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
        SettlementPlan greedy = group.settlement(SettlementStrategy.GREEDY);
        System.out.print(
                """
                The largest debtor pays the largest creditor, again and again: fast, at most one
                payment fewer than the members with a balance, but not always the minimum. %d
                payments here:
                """
                        .formatted(greedy.payments().size()));
        printPayments(group, greedy);

        section("5. Direct");
        SettlementPlan direct = group.settlement(SettlementStrategy.DIRECT);
        System.out.print(
                """
                No simplification: everyone pays back the people who paid for them, with the debts
                between each pair of members netted. The most payments (%d), but each one traces
                back to shared expenses:
                """
                        .formatted(direct.payments().size()));
        printPayments(group, direct);

        section("6. Adaptive");
        SettlementPlan adaptive = group.settlement(SettlementStrategy.ADAPTIVE);
        System.out.print(
                """
                The fewest payments while they are quick to find, greedy's beyond: up to 20 open
                balances the minimum solver settles them, past that greedy does, and the plan
                names the strategy whose plan it is. Six balances are well within the limit, so
                the plan is the one %s gives, the same %d payments.
                """
                        .formatted(strategyName(adaptive.strategy()), adaptive.payments().size()));

        section("7. The strategies side by side");
        printTable(
                StrategyRow.HEADERS,
                List.of(
                        new StrategyRow(
                                "MinimumTransfers",
                                "the fewest payments",
                                "O(2ⁿ·n) for n open balances",
                                minimum.payments().size()),
                        new StrategyRow(
                                "Greedy",
                                "at most n − 1 payments",
                                "O(n log n)",
                                greedy.payments().size()),
                        new StrategyRow(
                                "Direct",
                                "one payment per pair in debt",
                                "O(pairs)",
                                direct.payments().size()),
                        new StrategyRow(
                                "Adaptive",
                                "the fewest up to 20 balances",
                                "as MinimumTransfers, then Greedy",
                                adaptive.payments().size())),
                3);
        return new Counts(
                adaptive.payments().size(),
                minimum.payments().size(),
                greedy.payments().size(),
                direct.payments().size());
    }

    /**
     * Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly,
     * and prints which strategy's plan it took; returns the number of payments.
     */
    private static int festival() throws EranosException {
        section("8. A larger group");
        try (ExpenseGroup group = ExpenseGroup.builder("Festival", Currency.of("EUR")).build()) {
            List<MemberId> friends = new ArrayList<>();
            for (int number = 1; 21 >= number; number++) {
                friends.add(group.addMember(Member.builder("Friend " + number).build()));
            }
            // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with
            // nothing to round, and friend i's balance is 21·i² − 3311 cents: never zero, and
            // never the opposite of another's.
            for (int number = 1; 21 >= number; number++) {
                group.addExpense(
                        Expense.builder(
                                        "Round",
                                        new Cost.Local(
                                                new Money(
                                                        21L * number * number, Currency.of("EUR"))),
                                        new Split.Equal(List.of(friends.get(number - 1))),
                                        new ExpenseSplit.Whole(new Split.Equal(friends)),
                                        CalendarDate.of(2026, 9, 18))
                                .build());
            }
            long open =
                    group.summary().members().stream()
                            .filter(member -> 0 != member.net().minorUnits())
                            .count();
            SettlementPlan adaptive = group.settlement(SettlementStrategy.ADAPTIVE);
            SettlementPlan greedy = group.settlement(SettlementStrategy.GREEDY);
            System.out.print(
                    """
                    Twenty-one friends at a festival each bought one round for everyone, each round
                    at another price, so all %d end with a balance and no two of them cancel out.
                    That is more than the 20 the adaptive strategy settles exactly, so it takes the
                    plan %s gives and says so: the same %d payments.
                    """
                            .formatted(
                                    open,
                                    strategyName(adaptive.strategy()),
                                    adaptive.payments().size()));
            check(21 == open);
            check(SettlementStrategy.GREEDY == adaptive.strategy());
            check(adaptive.payments().equals(greedy.payments()));
            return adaptive.payments().size();
        }
    }

    /**
     * Prints the report of the ski week.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (ExpenseGroup group = ExpenseGroup.builder("Ski week", Currency.of("EUR")).build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            MemberId carol = group.addMember(Member.builder("Carol").build());
            MemberId dan = group.addMember(Member.builder("Dan").build());
            MemberId eva = group.addMember(Member.builder("Eva").build());
            MemberId filip = group.addMember(Member.builder("Filip").build());

            System.out.println("Ski week: four ways to settle up");
            System.out.println("════════════════════════════════");
            System.out.print(
                    """
                    Six friends spent a week in the mountains. The library can turn their balances
                    into payments in four ways, and the app picks one.
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
                                expense.name(),
                                names(group, expense.payers().members()),
                                names(group, participants(expense.split())),
                                expense.total()));
            }
            printTable(ExpenseRow.HEADERS, expenses, 3);

            section("2. Balances");
            System.out.println("Four members are owed money and two owe it:");
            printBalances(group);

            Counts payments = strategies(group);
            int festivalPayments = festival();

            section("9. Settling up for real");
            SettlementPlan plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
            for (Payment payment : plan.payments()) {
                group.addTransfer(
                        Transfer.builder(
                                        payment.from(),
                                        payment.to(),
                                        new Cost.Local(payment.amount()),
                                        CalendarDate.of(2026, 9, 27))
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

            // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice,
            // and the adaptive plan is that one; greedy pays the largest amounts first and needs
            // five; the unsimplified debts between pairs need seven.
            check(new Counts(4, 4, 5, 7).equals(payments));
            // Greedy settles the festival's 21 balances in 19 payments.
            check(19 == festivalPayments);
            Summary summary = group.summary();
            for (MemberId member : List.of(ana, bob, carol, dan, eva, filip)) {
                check(
                        Optional.of(new Balance.Settled())
                                .equals(totals(member, summary).map(MemberSummary::balance)));
            }
        }
    }
}
