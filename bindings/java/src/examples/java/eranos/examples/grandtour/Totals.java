package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tables.printBalances;
import static eranos.examples.grandtour.Tables.printCategories;
import static eranos.examples.grandtour.Tables.printEntries;
import static eranos.examples.grandtour.Tour.eur;
import static eranos.examples.grandtour.Tour.groupFailure;
import static eranos.examples.grandtour.Tour.message;
import static eranos.examples.grandtour.Tour.name;
import static eranos.examples.grandtour.Tour.october;
import static eranos.examples.grandtour.Tour.refusal;
import static eranos.examples.grandtour.Tour.ron;
import static eranos.examples.grandtour.Tour.sum;

import eranos.Balance;
import eranos.Category;
import eranos.CategoryId;
import eranos.CategoryTotal;
import eranos.CurrencyCode;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.ExpenseGroup;
import eranos.ExpenseGroupFailure;
import eranos.Member;
import eranos.MemberCategoryTotals;
import eranos.MemberId;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Payment;
import eranos.SettlementPlan;
import eranos.SettlementStrategy;
import eranos.Summary;
import eranos.Transfer;
import eranos.examples.grandtour.Tables.MemberCategoryRow;
import eranos.examples.grandtour.Tables.PaymentRow;
import eranos.examples.grandtour.Tables.StrategyRow;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and
 * settling up.
 */
final class Totals {
    private Totals() {}

    /** The totals per category of {@code member} in {@code summary}. */
    private static List<CategoryTotal> byCategory(Summary summary, MemberId member) {
        return summary.memberCategoryTotals().stream()
                .filter(totals -> totals.member().equals(member))
                .findFirst()
                .map(MemberCategoryTotals::totals)
                .orElse(List.of());
    }

    static void summary(ExpenseGroup group, People people, Categories categories)
            throws EranosException {
        section("8. The summary");
        Summary summary = group.summary();
        System.out.print(
                """
                The group spent %s in all. For each member, what they paid, what they
                consumed, the transfers they sent and received, and what is left:
                """
                        .formatted(summary.totalSpent()));
        printBalances(group);
        System.out.println("The spending per category:");
        printCategories(group);
        System.out.println("And each member's consumption per category:");
        Money zero = Money.zero(group.currency());
        List<MemberCategoryRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            rows.add(
                    new MemberCategoryRow(
                            member.firstName(),
                            summary.memberCategoryTotal(member.id(), categories.food()),
                            summary.memberCategoryTotal(member.id(), categories.lodging()),
                            summary.memberCategoryTotal(member.id(), categories.transport()),
                            summary.memberCategoryTotal(member.id(), CategoryId.UNCATEGORISED)));
        }
        printTable(MemberCategoryRow.HEADERS, rows, 1);

        check(eur("1222.90").equals(summary.totalSpent()));
        check(eur("890.00").equals(summary.member(people.ana()).paid()));
        check(eur("140.00").equals(summary.member(people.ana()).received()));
        check(eur("20.00").equals(summary.member(people.bob()).sent()));
        check(new Balance.GetsBack(eur("12.55")).equals(summary.member(people.dan()).balance()));
        check(new Balance.Owes(eur("42.60")).equals(summary.member(people.eva()).balance()));
        check(eur("174.40").equals(summary.categoryTotal(categories.food())));
        check(eur("800.00").equals(summary.categoryTotal(categories.lodging())));
        check(eur("188.50").equals(summary.categoryTotal(categories.transport())));
        check(eur("60.00").equals(summary.categoryTotal(CategoryId.UNCATEGORISED)));
        for (MemberSummary totals : summary.members()) {
            List<Money> amounts =
                    byCategory(summary, totals.member()).stream()
                            .map(CategoryTotal::total)
                            .toList();
            check(sum(zero.currency(), amounts).equals(totals.share()));
        }
        List<Money> nets = summary.members().stream().map(MemberSummary::net).toList();
        check(0 == sum(zero.currency(), nets).minorUnits());
    }

    static void removingACategory(ExpenseGroup group, Categories categories)
            throws EranosException {
        section("9. Removing a category");
        Category removed = group.removeCategory(categories.transport());
        Optional<EranosException> builtIn =
                refusal(() -> group.removeCategory(CategoryId.UNCATEGORISED));
        System.out.print(
                """
                Removing %s moves its expenses to Uncategorised and the category
                totals with them; no balance changes. The built-in category cannot be removed:
                  error: %s
                """
                        .formatted(removed.name(), message(builtIn)));
        printCategories(group);

        check(Optional.of(new ExpenseGroupFailure.BuiltInCategory()).equals(groupFailure(builtIn)));
        check(group.category(categories.transport()).isEmpty());
        check(eur("248.50").equals(group.summary().categoryTotal(CategoryId.UNCATEGORISED)));
        check(
                group.expenses().stream()
                        .noneMatch(expense -> categories.transport().equals(expense.category())));
    }

    static void newCurrency(ExpenseGroup group, People people) throws EranosException {
        section("10. A new currency");
        CurrencyCode euro = CurrencyCode.of("EUR");
        CurrencyCode leu = CurrencyCode.of("RON");
        Optional<EranosException> missing = refusal(() -> group.changeCurrency(leu, Map.of()));
        group.changeCurrency(leu, Map.of(euro, ExchangeRate.parse("5")));
        System.out.print(
                """
                Home again, the group moves its books to RON. The change needs a rate into RON
                for every other currency in use, here only EUR, or it is refused:
                  error: %s
                Entries in EUR take the new rate, and the RON parking and cash become local:
                """
                        .formatted(message(missing)));
        printEntries(group, currency -> true);
        System.out.print(
                """
                Each entry keeps its currency and converts at the new rate. At 5 every share
                converts exactly, so each balance is five times what it was. The trip cost
                %s:
                """
                        .formatted(group.summary().totalSpent()));
        printBalances(group);

        Summary summary = group.summary();
        check(
                Optional.of(new ExpenseGroupFailure.MissingExchangeRate(euro))
                        .equals(groupFailure(missing)));
        check(leu.equals(group.currency()));
        check(ron("6114.50").equals(summary.totalSpent()));
        check(new Balance.GetsBack(ron("62.75")).equals(summary.member(people.dan()).balance()));
        check(new Balance.Owes(ron("213.00")).equals(summary.member(people.eva()).balance()));
    }

    /** The name of {@code strategy} as the apps show it. */
    static String strategyName(SettlementStrategy strategy) {
        return switch (strategy) {
            case ADAPTIVE -> "Adaptive";
            case MINIMUM_TRANSFERS -> "MinimumTransfers";
            case GREEDY -> "Greedy";
            case DIRECT -> "Direct";
        };
    }

    static void settlingUp(ExpenseGroup group) throws EranosException {
        section("11. Settling up");
        List<StrategyRow> counts = new ArrayList<>();
        for (SettlementStrategy strategy : SettlementStrategy.values()) {
            SettlementPlan settlement = group.settlement(strategy);
            // The group's open balances are well within the adaptive strategy's limit, so it
            // settles them exactly.
            SettlementStrategy computedBy =
                    SettlementStrategy.ADAPTIVE == strategy
                            ? SettlementStrategy.MINIMUM_TRANSFERS
                            : strategy;
            check(computedBy == settlement.strategy());
            counts.add(
                    new StrategyRow(
                            strategyName(strategy),
                            strategyName(settlement.strategy()),
                            settlement.payments().size()));
        }
        System.out.print(
                """
                Each strategy turns the balances into payments: the direct one pays back every
                pair's own debts, greedy pays the largest amounts first, the minimum one finds
                the fewest payments, and the adaptive one finds the fewest while that is quick
                and greedy's beyond, naming the strategy whose plan it chose.
                """);
        printTable(StrategyRow.HEADERS, counts, 2);
        SettlementPlan plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
        System.out.println("The group follows the minimum plan:");
        List<PaymentRow> rows = new ArrayList<>();
        for (Payment payment : plan.payments()) {
            rows.add(
                    new PaymentRow(
                            name(group, payment.sender()),
                            name(group, payment.receiver()),
                            payment.amount()));
        }
        printTable(PaymentRow.HEADERS, rows, 2);
        for (Payment payment : plan.payments()) {
            group.addTransfer(Transfer.builder(payment, october(7)).build());
        }
        System.out.print(
                """
                Each payment, once made, is recorded as a transfer, and every balance is settled:
                """);
        printBalances(group);

        check(group.settlement(SettlementStrategy.GREEDY).payments().isEmpty());
        check(4 == plan.payments().size());
        check(
                group.summary().members().stream()
                        .allMatch(totals -> new Balance.Settled().equals(totals.balance())));
    }
}
