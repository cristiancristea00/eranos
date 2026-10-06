package eranos.examples.restaurantbill;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.title;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseId;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.Item;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberParts;
import eranos.MemberShare;
import eranos.Money;
import eranos.Parts;
import eranos.Percentage;
import eranos.Split;
import eranos.Tip;
import eranos.TipDistribution;
import eranos.TipValue;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * An itemised restaurant bill: every line is split among the people who had it, quantities multiply
 * the unit price, and the tip is shared either in proportion to what each person ordered or
 * equally. The port of the Rust example {@code restaurant_bill}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** A line of the receipt, a row of the receipt's table. */
    private record LineRow(
            Money lineTotal, String name, String participants, int quantity, Money unitPrice)
            implements Row {
        static final List<String> HEADERS =
                List.of("Item", "Shared by", "Quantity", "Unit price", "Line total");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    participants,
                    String.valueOf(quantity),
                    unitPrice.toString(),
                    lineTotal.toString());
        }
    }

    /** What a member ordered and pays with each way of sharing the tip, a row of the tips. */
    private record TipRow(
            Money equalTip,
            Money equalTotal,
            String name,
            Money proportionalTip,
            Money proportionalTotal,
            Money subtotal)
            implements Row {
        static final List<String> HEADERS =
                List.of(
                        "Member",
                        "Ordered",
                        "Tip by consumption",
                        "Pays",
                        "Tip shared equally",
                        "Pays");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    subtotal.toString(),
                    proportionalTip.toString(),
                    proportionalTotal.toString(),
                    equalTip.toString(),
                    equalTotal.toString());
        }
    }

    /** The amount written {@code text}, in euros. */
    private static Money eur(String text) throws EranosException {
        return Money.parse(text, Currency.of("EUR"));
    }

    /** The receipt: every line with the members who had it. */
    private static List<Item> receipt(MemberId ana, MemberId bob, MemberId carol)
            throws EranosException {
        return List.of(
                new Item("Ciorbă", new Split.Equal(List.of(ana)), eur("8.00")),
                new Item("Sarmale", new Split.Equal(List.of(bob, carol)), eur("14.00"), 2),
                new Item(
                        "Papanași",
                        new Split.Parts(
                                List.of(
                                        new MemberParts(ana, Parts.ofHundredths(100)),
                                        new MemberParts(carol, Parts.ofHundredths(200)))),
                        eur("9.00")),
                new Item("Lemonade", new Split.Equal(List.of(ana, bob, carol)), eur("3.00"), 3));
    }

    /** The dinner, its tip of 10 % shared by {@code distribution}. */
    private static Expense bill(
            MemberId ana,
            MemberId bob,
            MemberId carol,
            CalendarDate day,
            TipDistribution distribution)
            throws EranosException {
        return Expense.builder(
                        new Cost.Local(eur("54.00")),
                        day,
                        "Dinner",
                        new Split.Equal(List.of(ana)),
                        new ExpenseSplit.Itemised(receipt(ana, bob, carol)))
                .tip(new Tip(new TipValue.Percentage(Percentage.parse("10")), distribution))
                .build();
    }

    /**
     * How {@code expense} would divide if it were added to {@code group}, computed on a copy loaded
     * from the group's saved form, so the group itself is unchanged.
     */
    private static Optional<ExpenseShares> preview(Group group, Expense expense)
            throws EranosException {
        try (Group copy = Group.load(group.save())) {
            return copy.expenseShares(copy.addExpense(expense));
        }
    }

    /** What {@code member} consumed of the expense that {@code shares} divides. */
    private static Optional<Money> consumed(Optional<ExpenseShares> shares, MemberId member) {
        return shares.flatMap(
                found ->
                        found.members().stream()
                                .filter(share -> share.member().equals(member))
                                .findFirst()
                                .map(MemberShare::share));
    }

    /**
     * Prints the report of the dinner.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (Group group = Group.builder(Currency.of("EUR"), "Dinner at Caru' cu Bere").build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            MemberId carol = group.addMember(Member.builder("Carol").build());
            CalendarDate day = CalendarDate.of(23, 9, 2026);

            title(group.name() + ": an itemised bill");
            System.out.print(
                    """
                    Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
                    every line of the receipt is split among the people who had it.
                    """);

            section("1. The receipt");
            List<LineRow> lines = new ArrayList<>();
            for (Item item : receipt(ana, bob, carol)) {
                Money total =
                        new Money(
                                item.unitPrice().currency(),
                                item.unitPrice().minorUnits() * item.quantity());
                lines.add(
                        new LineRow(
                                total,
                                item.name(),
                                names(group, item.split().members()),
                                item.quantity(),
                                item.unitPrice()));
            }
            printTable(LineRow.HEADERS, lines, 2);
            System.out.print(
                    """
                    The papanași are split by parts: Carol ate two of the three. The line totals add up
                    to 54.00 EUR, and the expense's amount must match them exactly.
                    """);

            section("2. Who ordered what, and the tip");
            Expense untipped =
                    Expense.builder(
                                    new Cost.Local(eur("54.00")),
                                    day,
                                    "Dinner",
                                    new Split.Equal(List.of(ana)),
                                    new ExpenseSplit.Itemised(receipt(ana, bob, carol)))
                            .build();
            Optional<ExpenseShares> ordered = preview(group, untipped);
            ExpenseId proportional =
                    group.addExpense(bill(ana, bob, carol, day, TipDistribution.PROPORTIONAL));
            Optional<ExpenseShares> byConsumption = group.expenseShares(proportional);
            Optional<ExpenseShares> even =
                    preview(group, bill(ana, bob, carol, day, TipDistribution.EQUAL));
            System.out.print(
                    """
                    Ana paid the bill plus a 10 %% tip (%s). A tip can be shared in proportion to what
                    each person ordered, or equally; both are shown:
                    """
                            .formatted(
                                    byConsumption
                                            .map(ExpenseShares::tip)
                                            .map(Money::toString)
                                            .orElse("?")));
            List<TipRow> rows = new ArrayList<>();
            for (MemberId member : List.of(ana, bob, carol)) {
                Money subtotal = consumed(ordered, member).orElseThrow();
                Money proportionalTotal = consumed(byConsumption, member).orElseThrow();
                Money equalTotal = consumed(even, member).orElseThrow();
                rows.add(
                        new TipRow(
                                equalTotal.minus(subtotal),
                                equalTotal,
                                names(group, List.of(member)),
                                proportionalTotal.minus(subtotal),
                                proportionalTotal,
                                subtotal));
            }
            printTable(TipRow.HEADERS, rows, 1);
            System.out.print(
                    """
                    By consumption, Carol, who ordered the most, adds the largest part of the tip;
                    shared equally, everyone adds the same 1.80.
                    """);

            section("3. A total that does not match is refused");
            try {
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("55.00")),
                                        day,
                                        "Dinner",
                                        new Split.Equal(List.of(ana)),
                                        new ExpenseSplit.Itemised(receipt(ana, bob, carol)))
                                .build());
                throw new IllegalStateException("the mismatch must be refused");
            } catch (EranosException refusal) {
                System.out.print(
                        """
                        An amount of 55.00 EUR with these lines is refused when the expense is built:
                          error: %s
                        """
                                .formatted(refusal.getMessage()));
            }

            // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A
            // proportional tip adds 10 % to each; an equal one 1.80 to everyone.
            check(Optional.of(eur("15.40")).equals(consumed(byConsumption, ana)));
            check(Optional.of(eur("18.70")).equals(consumed(byConsumption, bob)));
            check(Optional.of(eur("25.30")).equals(consumed(byConsumption, carol)));
            check(Optional.of(eur("24.80")).equals(consumed(even, carol)));
            check(Optional.of(eur("59.40")).equals(even.map(ExpenseShares::total)));
        }
    }
}
