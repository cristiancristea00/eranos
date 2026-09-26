package eranos.examples.grandtour;

import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.totals;
import static eranos.examples.grandtour.Tour.name;
import static eranos.examples.grandtour.Tour.rate;

import eranos.Balance;
import eranos.Category;
import eranos.CategoryId;
import eranos.CategoryTotal;
import eranos.Currency;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseShares;
import eranos.Group;
import eranos.Member;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Summary;
import eranos.Transfer;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Predicate;

/** The tables several sections of the tour print: the balances, the categories, and the entries. */
final class Tables {
    private Tables() {}

    /** The identifier of the built-in category, which the group lists first. */
    static Optional<CategoryId> builtInCategory(Group group) {
        return group.categories().stream().findFirst().flatMap(Category::id);
    }

    static void printBalances(Group group) {
        Summary summary = group.summary();
        String zero = new Money(group.currency(), 0).toString();
        List<List<String>> rows = new ArrayList<>();
        for (Member member : group.members()) {
            // The summary lists the members with a total; every total of the others is zero.
            Optional<MemberSummary> totals = member.id().flatMap(id -> totals(id, summary));
            rows.add(
                    totals.map(
                                    found ->
                                            List.of(
                                                    member.firstName(),
                                                    found.paid().toString(),
                                                    found.share().toString(),
                                                    found.sent().toString(),
                                                    found.received().toString(),
                                                    found.balance().toString()))
                            .orElse(
                                    List.of(
                                            member.firstName(),
                                            zero,
                                            zero,
                                            zero,
                                            zero,
                                            new Balance.Settled().toString())));
        }
        printTable(List.of("Member", "Paid", "Consumed", "Sent", "Received", "Balance"), rows, 1);
    }

    static void printCategories(Group group) throws EranosException {
        List<Expense> expenses = group.expenses();
        Optional<CategoryId> builtIn = builtInCategory(group);
        List<List<String>> rows = new ArrayList<>();
        for (CategoryTotal total : group.summary().categoryTotals()) {
            long count =
                    expenses.stream()
                            .filter(
                                    expense ->
                                            expense.category()
                                                    .or(() -> builtIn)
                                                    .equals(Optional.of(total.category())))
                            .count();
            rows.add(
                    List.of(
                            group.category(total.category()).map(Category::name).orElse("?"),
                            String.valueOf(count),
                            total.total().toString()));
        }
        printTable(List.of("Category", "Expenses", "Total"), rows, 1);
    }

    /**
     * Every expense and transfer in a currency {@code shown} accepts, as entered and in the group
     * currency.
     */
    static void printEntries(Group group, Predicate<Currency> shown) throws EranosException {
        Money zero = new Money(group.currency(), 0);
        List<List<String>> rows = new ArrayList<>();
        for (Expense expense : group.expenses()) {
            if (shown.test(expense.cost().amount().currency())) {
                Optional<ExpenseShares> shares =
                        expense.id().isPresent()
                                ? group.expenseShares(expense.id().orElseThrow())
                                : Optional.empty();
                rows.add(
                        List.of(
                                expense.name(),
                                names(group, expense.payers().members()),
                                expense.total().toString(),
                                rate(expense.cost()).toString(),
                                shares.map(ExpenseShares::total).orElse(zero).toString()));
            }
        }
        for (Transfer transfer : group.transfers()) {
            if (shown.test(transfer.cost().amount().currency())) {
                Optional<Money> amount =
                        transfer.id().isPresent()
                                ? group.transferAmount(transfer.id().orElseThrow())
                                : Optional.empty();
                rows.add(
                        List.of(
                                "Transfer to " + name(group, transfer.to()),
                                name(group, transfer.from()),
                                transfer.cost().amount().toString(),
                                rate(transfer.cost()).toString(),
                                amount.orElse(zero).toString()));
            }
        }
        printTable(
                List.of("Entry", "Paid by", "Entered as", "Rate", "In the group currency"),
                rows,
                2);
    }
}
