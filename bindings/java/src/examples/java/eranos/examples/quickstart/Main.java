package eranos.examples.quickstart;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.participants;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.title;
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
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom.
 * The port of the Rust example {@code quickstart}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** An expense with who paid it and who shares it, a row of the expenses' table. */
    private record ExpenseRow(Money amount, String name, String participants, String payers)
            implements Row {
        static final List<String> HEADERS = List.of("Expense", "Paid by", "Shared by", "Amount");

        @Override
        public List<String> cells() {
            return List.of(name, payers, participants, amount.toString());
        }
    }

    /** A member's totals, a row of the members' table. */
    private record MemberRow(Balance balance, Money consumed, String name, Money paid)
            implements Row {
        static final List<String> HEADERS = List.of("Member", "Paid", "Consumed", "Balance");

        @Override
        public List<String> cells() {
            return List.of(name, paid.toString(), consumed.toString(), balance.toString());
        }
    }

    /** A payment of the plan, a row of the payments' table. */
    private record PaymentRow(Money amount, String from, String to) implements Row {
        static final List<String> HEADERS = List.of("From", "To", "Amount");

        @Override
        public List<String> cells() {
            return List.of(from, to, amount.toString());
        }
    }

    /**
     * Prints the report of the weekend trip.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        Currency euro = Currency.of("EUR");
        try (Group group = Group.builder(euro, "Weekend in Sinaia").build()) {
            title(group.name());
            System.out.print(
                    """
                    Three friends rent a cabin and buy groceries. The group keeps its books in %s, so
                    every total, balance, and payment below is in %s.
                    """
                            .formatted(group.currency(), group.currency()));

            section("1. Members");
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            MemberId carol = group.addMember(Member.builder("Carol").build());
            System.out.printf(
                    "%d people joined the group: %s.%n",
                    group.members().size(), names(group, List.of(ana, bob, carol)));

            section("2. Expenses");
            System.out.print(
                    """
                    Each expense records who paid and who shares it. Here everyone shares everything
                    equally.
                    """);
            CalendarDate friday = CalendarDate.of(25, 9, 2026);
            Split everyone = new Split.Equal(List.of(ana, bob, carol));
            group.addExpense(
                    Expense.builder(
                                    new Cost.Local(Money.parse("240.00", euro)),
                                    friday,
                                    "Cabin",
                                    new Split.Equal(List.of(ana)),
                                    new ExpenseSplit.Whole(everyone))
                            .build());
            group.addExpense(
                    Expense.builder(
                                    new Cost.Local(Money.parse("60.00", euro)),
                                    friday,
                                    "Groceries",
                                    new Split.Equal(List.of(bob)),
                                    new ExpenseSplit.Whole(everyone))
                            .build());
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

            section("3. Who paid and who consumed");
            Summary summary = group.summary();
            System.out.print(
                    """
                    The group spent %s in total. A member's share of an expense is what they consumed,
                    and their balance is what they paid minus what they consumed.
                    """
                            .formatted(summary.totalSpent()));
            List<MemberRow> balances = new ArrayList<>();
            for (Member member : group.members()) {
                Optional<MemberSummary> totals = member.id().flatMap(id -> totals(id, summary));
                totals.ifPresent(
                        found ->
                                balances.add(
                                        new MemberRow(
                                                found.balance(),
                                                found.share(),
                                                member.firstName(),
                                                found.paid())));
            }
            printTable(MemberRow.HEADERS, balances, 1);
            Optional<MemberSummary> anaTotals = totals(ana, summary);
            System.out.printf(
                    "Ana paid %s but consumed only %s, so the others owe her the difference.%n",
                    anaTotals.map(MemberSummary::paid).map(Money::toString).orElse("?"),
                    anaTotals.map(MemberSummary::share).map(Money::toString).orElse("?"));

            section("4. Settling up");
            Settlement settlement = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
            System.out.printf(
                    "The fewest payments that bring every balance to zero (%d in total):%n",
                    settlement.payments().size());
            List<PaymentRow> payments = new ArrayList<>();
            for (Payment payment : settlement.payments()) {
                payments.add(
                        new PaymentRow(
                                payment.amount(),
                                names(group, List.of(payment.from())),
                                names(group, List.of(payment.to()))));
            }
            printTable(PaymentRow.HEADERS, payments, 2);
            System.out.println("Once these payments are made, nobody owes anything.");

            // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
            check(Money.parse("300.00", euro).equals(summary.totalSpent()));
            check(
                    Optional.of(new Balance.GetsBack(Money.parse("140.00", euro)))
                            .equals(anaTotals.map(MemberSummary::balance)));
            check(2 == settlement.payments().size());
        }
    }
}
