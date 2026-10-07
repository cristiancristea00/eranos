package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tour.october;
import static eranos.examples.grandtour.Tour.ron;

import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.Expense;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.GroupReader;
import eranos.Member;
import eranos.MemberId;
import eranos.Money;
import eranos.Payment;
import eranos.Settlement;
import eranos.SettlementStrategy;
import eranos.Split;
import eranos.Summary;
import eranos.examples.grandtour.Tables.PaymentRow;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;

/**
 * Section 14 of the tour: the group shared between threads, a settlement computed with the
 * asynchronous calls, and a change that waits for a report's read and is cancelled.
 */
final class Sharing {
    private Sharing() {}

    static void sharing(Group group, People people) throws EranosException {
        section("14. Sharing the group");
        // Another handle on the same group, as another thread holds: a group is shared as it is.
        Group other = group;
        List<MemberId> everyone =
                List.of(people.ana(), people.bob(), people.carol(), people.dan(), people.eva());
        Expense coffee =
                Expense.builder(
                                "Airport coffee",
                                new Cost.Local(ron("75.00")),
                                new Split.Equal(List.of(people.bob())),
                                new ExpenseSplit.Whole(new Split.Equal(everyone)),
                                october(8))
                        .build();
        other.addExpenseAsync(coffee).join();
        Settlement plan = group.settlementAsync(SettlementStrategy.MINIMUM_TRANSFERS).join();
        System.out.print(
                """
                An app shares the group between its tasks and threads through a SharedGroup,
                whose calls are futures any executor runs; here the library's block_on runs
                each one on this thread. Bob adds the airport coffee through another handle,
                and the plan that settles it comes through the first:
                """);
        List<PaymentRow> rows = new ArrayList<>();
        try (GroupReader reader = group.readAsync().join()) {
            for (Payment payment : plan.payments()) {
                rows.add(
                        new PaymentRow(
                                reader.member(payment.from()).map(Member::firstName).orElse("?"),
                                reader.member(payment.to()).map(Member::firstName).orElse("?"),
                                payment.amount()));
            }
        }
        printTable(PaymentRow.HEADERS, rows, 2);

        Summary before = group.summaryAsync().join();
        Map<Currency, ExchangeRate> rates = Map.of(Currency.of("RON"), ExchangeRate.parse("0.2"));
        // A report reads the group, holding the shared lock meanwhile, so the change waits for
        // it; a new read would wait behind the change, so the report reads through its own
        // handle.
        GroupReader report = group.readAsync().join();
        CompletableFuture<Void> change = group.changeCurrencyAsync(Currency.of("EUR"), rates);
        Money reported = report.summary().totalSpent();
        // Cancelling the future cancels the waiting change, which leaves the group as it was.
        check(change.cancel(true));
        report.close();
        Summary after = group.summaryAsync().join();
        Currency currency = group.currencyAsync().join();
        System.out.print(
                """
                While a report reads the group, a change of its currency to EUR waits for the
                report. The app drops the waiting change, which cancels it: the group still
                keeps its books in %s, and the trip cost %s, as the report read.
                """
                        .formatted(currency, after.totalSpent()));
        System.out.printf(
                "With the other handle gone, the app takes the group back: %d expenses.%n",
                group.expensesAsync().join().size());

        check(4 == plan.payments().size());
        for (Payment payment : plan.payments()) {
            check(people.bob().equals(payment.to()) && ron("15.00").equals(payment.amount()));
        }
        check(change.isCancelled());
        check(before.equals(after));
        check(reported.equals(after.totalSpent()));
        check(Currency.of("RON").equals(currency));
    }
}
