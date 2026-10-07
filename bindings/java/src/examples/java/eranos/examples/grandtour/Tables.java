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
import eranos.ExchangeRate;
import eranos.Expense;
import eranos.ExpenseGroup;
import eranos.ExpenseShares;
import eranos.Member;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Summary;
import eranos.Transfer;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Predicate;

/**
 * The rows of every table of the tour, and the tables several sections print: the balances, the
 * categories, and the entries.
 */
final class Tables {
    private Tables() {}

    /** A detail of the group, a row of its details' table. */
    record DetailRow(String detail, String value) implements Row {
        static final List<String> HEADERS = List.of("Detail", "Value");

        @Override
        public List<String> cells() {
            return List.of(detail, value);
        }
    }

    /** A member with their contact details and accounts, a row of the members' table. */
    record MemberRow(String name, String email, String phone, String accounts) implements Row {
        static final List<String> HEADERS = List.of("Member", "Email", "Phone", "Accounts");

        @Override
        public List<String> cells() {
            return List.of(name, email, phone, accounts);
        }
    }

    /** An input with what it was checked as and the outcome, a row of the checks' table. */
    record CheckRow(String input, String kind, String result) implements Row {
        static final List<String> HEADERS = List.of("Input", "Checked as", "Result");

        @Override
        public List<String> cells() {
            return List.of(input, kind, result);
        }
    }

    /** A change the group refused and the message it refused it with, a row of the refusals. */
    record RefusalRow(String change, String error) implements Row {
        static final List<String> HEADERS = List.of("Change", "Refused with");

        @Override
        public List<String> cells() {
            return List.of(change, error);
        }
    }

    /**
     * An expense with every involved member's payment and consumption, one per line, a row of the
     * shares' table.
     */
    record ShareRow(String expense, String how, String members, String paid, String consumed)
            implements Row {
        static final List<String> HEADERS =
                List.of("Expense", "Split", "Member", "Paid", "Consumed");

        @Override
        public List<String> cells() {
            return List.of(expense, how, members, paid, consumed);
        }
    }

    /** A line of a receipt, a row of the items' table. */
    record ItemRow(String name, String sharedBy, int quantity, Money unitPrice, Money lineTotal)
            implements Row {
        static final List<String> HEADERS =
                List.of("Item", "Shared by", "Quantity", "Unit price", "Line total");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    sharedBy,
                    String.valueOf(quantity),
                    unitPrice.toString(),
                    lineTotal.toString());
        }
    }

    /** An expense or a transfer as entered and in the group currency, a row of the entries. */
    record EntryRow(String name, String payer, Money original, ExchangeRate rate, Money converted)
            implements Row {
        static final List<String> HEADERS =
                List.of("Entry", "Paid by", "Entered as", "Rate", "In the group currency");

        @Override
        public List<String> cells() {
            return List.of(name, payer, original.toString(), rate.toString(), converted.toString());
        }
    }

    /** An entry and the edit made to it, a row of the edits' table. */
    record EditRow(String entry, String edit) implements Row {
        static final List<String> HEADERS = List.of("Entry", "Edit");

        @Override
        public List<String> cells() {
            return List.of(entry, edit);
        }
    }

    /** A member's totals, a row of the balances' table. */
    record BalanceRow(
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

    /** A member's status and balance, a row of the statuses' table. */
    record StatusRow(String name, String status, Balance balance) implements Row {
        static final List<String> HEADERS = List.of("Member", "Status", "Balance");

        @Override
        public List<String> cells() {
            return List.of(name, status, balance.toString());
        }
    }

    /** A category with its expenses and their total, a row of the categories' table. */
    record CategoryRow(String name, long expenses, Money total) implements Row {
        static final List<String> HEADERS = List.of("Category", "Expenses", "Total");

        @Override
        public List<String> cells() {
            return List.of(name, String.valueOf(expenses), total.toString());
        }
    }

    /** A member's consumption in each category, a row of the members' categories. */
    record MemberCategoryRow(
            String name, Money food, Money lodging, Money transport, Money uncategorised)
            implements Row {
        static final List<String> HEADERS =
                List.of("Member", "Food", "Lodging", "Getting around", "Uncategorised");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    food.toString(),
                    lodging.toString(),
                    transport.toString(),
                    uncategorised.toString());
        }
    }

    /**
     * A strategy asked for, the strategy whose plan it chose, and the plan's payments, a row of the
     * strategies' table.
     */
    record StrategyRow(String strategy, String computedBy, int payments) implements Row {
        static final List<String> HEADERS = List.of("Strategy", "Computed by", "Payments");

        @Override
        public List<String> cells() {
            return List.of(strategy, computedBy, String.valueOf(payments));
        }
    }

    /** A payment of a plan, a row of the payments' table. */
    record PaymentRow(String from, String to, Money amount) implements Row {
        static final List<String> HEADERS = List.of("From", "To", "Amount");

        @Override
        public List<String> cells() {
            return List.of(from, to, amount.toString());
        }
    }

    /** A change to the saved file and the message loading it fails with, a row of the loads. */
    record LoadRow(String change, String message) implements Row {
        static final List<String> HEADERS = List.of("Change to the file", "Message");

        @Override
        public List<String> cells() {
            return List.of(change, message);
        }
    }

    /** A currency with its details, a row of the currencies' table. */
    record CurrencyRow(String code, String name, int numericCode, int decimalPlaces)
            implements Row {
        static final List<String> HEADERS =
                List.of("Code", "Name", "Numeric code", "Decimal places");

        @Override
        public List<String> cells() {
            return List.of(code, name, String.valueOf(numericCode), String.valueOf(decimalPlaces));
        }
    }

    /** An input and what the library makes of it, a row of a values' table. */
    record ValueRow(String input, String result) implements Row {
        static final List<String> HEADERS = List.of("Input", "Result");

        @Override
        public List<String> cells() {
            return List.of(input, result);
        }
    }

    /** The identifier of the built-in category, which the group lists first. */
    static Optional<CategoryId> builtInCategory(ExpenseGroup group) {
        return group.categories().stream().findFirst().flatMap(Category::id);
    }

    static void printBalances(ExpenseGroup group) {
        Summary summary = group.summary();
        Money zero = new Money(0, group.currency());
        List<BalanceRow> rows = new ArrayList<>();
        for (Member member : group.members()) {
            // The summary lists the members with a total; every total of the others is zero.
            Optional<MemberSummary> totals = member.id().flatMap(id -> totals(id, summary));
            rows.add(
                    totals.map(
                                    found ->
                                            new BalanceRow(
                                                    member.firstName(),
                                                    found.paid(),
                                                    found.share(),
                                                    found.sent(),
                                                    found.received(),
                                                    found.balance()))
                            .orElse(
                                    new BalanceRow(
                                            member.firstName(),
                                            zero,
                                            zero,
                                            zero,
                                            zero,
                                            new Balance.Settled())));
        }
        printTable(BalanceRow.HEADERS, rows, 1);
    }

    static void printCategories(ExpenseGroup group) throws EranosException {
        List<Expense> expenses = group.expenses();
        Optional<CategoryId> builtIn = builtInCategory(group);
        List<CategoryRow> rows = new ArrayList<>();
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
                    new CategoryRow(
                            group.category(total.category()).map(Category::name).orElse("?"),
                            count,
                            total.total()));
        }
        printTable(CategoryRow.HEADERS, rows, 1);
    }

    /**
     * Every expense and transfer in a currency {@code shown} accepts, as entered and in the group
     * currency.
     */
    static void printEntries(ExpenseGroup group, Predicate<Currency> shown) throws EranosException {
        Money zero = new Money(0, group.currency());
        List<EntryRow> rows = new ArrayList<>();
        for (Expense expense : group.expenses()) {
            if (shown.test(expense.cost().amount().currency())) {
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
        }
        for (Transfer transfer : group.transfers()) {
            if (shown.test(transfer.cost().amount().currency())) {
                Optional<Money> amount =
                        transfer.id().isPresent()
                                ? group.transferAmount(transfer.id().orElseThrow())
                                : Optional.empty();
                rows.add(
                        new EntryRow(
                                "Transfer to " + name(group, transfer.to()),
                                name(group, transfer.from()),
                                transfer.cost().amount(),
                                rate(transfer.cost()),
                                amount.orElse(zero)));
            }
        }
        printTable(EntryRow.HEADERS, rows, 2);
    }
}
