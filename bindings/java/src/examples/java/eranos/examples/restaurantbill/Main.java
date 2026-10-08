package eranos.examples.restaurantbill;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.title;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.CurrencyCode;
import eranos.Discount;
import eranos.DiscountScope;
import eranos.DiscountValue;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseGroup;
import eranos.ExpenseId;
import eranos.ExpenseItem;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Member;
import eranos.MemberId;
import eranos.MemberParts;
import eranos.MemberShare;
import eranos.Money;
import eranos.Parts;
import eranos.Percentage;
import eranos.Quantity;
import eranos.Split;
import eranos.TipDistribution;
import eranos.TipRule;
import eranos.TipValue;
import eranos.examples.Report.Row;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * An itemised restaurant bill: every line is split among the people who had it, quantities multiply
 * the unit price, and the tip is shared either in proportion to what each person ordered or
 * equally; then a line weighed on a scale, and discounts off the line and off each unit. The port
 * of the Rust example {@code restaurant_bill}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** A line of the receipt, a row of the receipt's table. */
    private record LineRow(
            String name, String participants, Quantity quantity, Money unitPrice, Money lineTotal)
            implements Row {
        static final List<String> HEADERS =
                List.of("Item", "Shared by", "Quantity", "Unit price", "Line total");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    participants,
                    quantity.toString(),
                    unitPrice.toString(),
                    lineTotal.toString());
        }
    }

    /** A line of the second round with its discount, a row of the discounts' table. */
    private record DiscountRow(
            String name,
            Quantity quantity,
            Money unitPrice,
            Money lineTotalBeforeDiscount,
            Money discountAmount,
            Money lineTotal)
            implements Row {
        static final List<String> HEADERS =
                List.of(
                        "Item",
                        "Quantity",
                        "Unit price",
                        "Before discount",
                        "Discount",
                        "Line total");

        @Override
        public List<String> cells() {
            return List.of(
                    name,
                    quantity.toString(),
                    unitPrice.toString(),
                    lineTotalBeforeDiscount.toString(),
                    discountAmount.toString(),
                    lineTotal.toString());
        }
    }

    /** What a member ordered and pays with each way of sharing the tip, a row of the tips. */
    private record TipRow(
            String name,
            Money subtotal,
            Money proportionalTip,
            Money proportionalTotal,
            Money equalTip,
            Money equalTotal)
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
        return Money.parse(text, CurrencyCode.of("EUR"));
    }

    /** The receipt: every line with the members who had it. */
    private static List<ExpenseItem> receipt(MemberId ana, MemberId bob, MemberId carol)
            throws EranosException {
        return List.of(
                ExpenseItem.builder("Ciorbă", eur("8.00"), new Split.Equal(List.of(ana))).build(),
                ExpenseItem.builder("Sarmale", eur("14.00"), new Split.Equal(List.of(bob, carol)))
                        .quantity(Quantity.ofWhole(2))
                        .build(),
                ExpenseItem.builder(
                                "Papanași",
                                eur("9.00"),
                                new Split.Parts(
                                        List.of(
                                                new MemberParts(ana, Parts.ofHundredths(100)),
                                                new MemberParts(carol, Parts.ofHundredths(200)))))
                        .build(),
                ExpenseItem.builder(
                                "Lemonade", eur("3.00"), new Split.Equal(List.of(ana, bob, carol)))
                        .quantity(Quantity.ofWhole(3))
                        .build());
    }

    /**
     * A second round: telemea weighed on a scale with 10 % off the line, three lemonades with 1.00
     * off each, and papanași on the house.
     */
    private static List<ExpenseItem> secondRound(MemberId ana, MemberId bob, MemberId carol)
            throws EranosException {
        // The scale reports the weight as a double.
        Quantity weight = Quantity.fromDouble(0.35);
        return List.of(
                ExpenseItem.builder("Telemea", eur("42.00"), new Split.Equal(List.of(ana, bob)))
                        .quantity(weight)
                        .discount(
                                new Discount(new DiscountValue.Percentage(Percentage.parse("10"))))
                        .build(),
                ExpenseItem.builder(
                                "Lemonade", eur("3.00"), new Split.Equal(List.of(ana, bob, carol)))
                        .quantity(Quantity.ofWhole(3))
                        .discount(
                                new Discount(
                                        new DiscountValue.Fixed(eur("1.00")), DiscountScope.UNIT))
                        .build(),
                ExpenseItem.builder("Papanași", eur("9.00"), new Split.Equal(List.of(carol)))
                        .discount(
                                new Discount(new DiscountValue.Percentage(Percentage.parse("100"))))
                        .build());
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
                        "Dinner",
                        new Cost.Local(eur("54.00")),
                        new Split.Equal(List.of(ana)),
                        new ExpenseSplit.Itemised(receipt(ana, bob, carol)),
                        day)
                .tip(new TipRule(new TipValue.Percentage(Percentage.parse("10")), distribution))
                .build();
    }

    /**
     * How {@code expense} would divide if it were added to {@code group}, computed on a copy loaded
     * from the group's saved form, so the group itself is unchanged.
     */
    private static Optional<ExpenseShares> preview(ExpenseGroup group, Expense expense)
            throws EranosException {
        try (ExpenseGroup copy = ExpenseGroup.load(group.save())) {
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
     * Section 4: the second round's lines, each with its total before the discount, the discount,
     * and its line total, and a discount refused for being larger than its line.
     */
    private static void weighedLinesAndDiscounts(MemberId ana, MemberId bob, MemberId carol)
            throws EranosException {
        section("4. Weighed lines and discounts");
        List<ExpenseItem> round = secondRound(ana, bob, carol);
        List<DiscountRow> rows = new ArrayList<>();
        for (ExpenseItem item : round) {
            rows.add(
                    new DiscountRow(
                            item.name(),
                            item.quantity(),
                            item.unitPrice(),
                            item.lineTotalBeforeDiscount(),
                            item.discountAmount(),
                            item.lineTotal()));
        }
        printTable(DiscountRow.HEADERS, rows, 1);
        ExpenseItem telemea = round.get(0);
        ExpenseItem lemonade = round.get(1);
        ExpenseItem papanasi = round.get(2);
        Money nine = eur("9.00");
        Money ten = eur("10.00");
        try {
            ExpenseItem.builder("Papanași", nine, new Split.Equal(List.of(carol)))
                    .discount(new Discount(new DiscountValue.Fixed(ten)))
                    .build()
                    .lineTotal();
            throw new IllegalStateException("the discount must be refused");
        } catch (EranosException refusal) {
            System.out.print(
                    """
                    The scale reports %s kg as a double just below it; the quantity reads its shortest
                    form, %s, so the telemea costs exactly %s before its discount. Taking %s
                    off a %s line is refused when the item is built:
                      error: %s
                    """
                            .formatted(
                                    telemea.quantity(),
                                    telemea.quantity(),
                                    telemea.lineTotalBeforeDiscount(),
                                    ten,
                                    nine,
                                    refusal.getMessage()));
        }

        // 10 % of 14.70 is 1.47; 1.00 off each of three lemonades is 3.00; the papanași on the
        // house leave a free line.
        check(eur("14.70").equals(telemea.lineTotalBeforeDiscount()));
        check(eur("1.47").equals(telemea.discountAmount()));
        check(eur("13.23").equals(telemea.lineTotal()));
        check(eur("3.00").equals(lemonade.discountAmount()));
        check(eur("6.00").equals(lemonade.lineTotal()));
        check(eur("9.00").equals(papanasi.discountAmount()));
        check(eur("0.00").equals(papanasi.lineTotal()));
    }

    /**
     * Prints the report of the dinner.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (ExpenseGroup group =
                ExpenseGroup.builder("Dinner at Caru' cu Bere", CurrencyCode.of("EUR")).build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            MemberId carol = group.addMember(Member.builder("Carol").build());
            CalendarDate day = CalendarDate.of(2026, 9, 23);

            title(group.name() + ": an itemised bill");
            System.out.print(
                    """
                    Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
                    every line of the receipt is split among the people who had it.
                    """);

            section("1. The receipt");
            List<LineRow> lines = new ArrayList<>();
            for (ExpenseItem item : receipt(ana, bob, carol)) {
                lines.add(
                        new LineRow(
                                item.name(),
                                names(group, item.split().members()),
                                item.quantity(),
                                item.unitPrice(),
                                item.lineTotal()));
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
                                    "Dinner",
                                    new Cost.Local(eur("54.00")),
                                    new Split.Equal(List.of(ana)),
                                    new ExpenseSplit.Itemised(receipt(ana, bob, carol)),
                                    day)
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
                                names(group, List.of(member)),
                                subtotal,
                                proportionalTotal.minus(subtotal),
                                proportionalTotal,
                                equalTotal.minus(subtotal),
                                equalTotal));
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
                                        "Dinner",
                                        new Cost.Local(eur("55.00")),
                                        new Split.Equal(List.of(ana)),
                                        new ExpenseSplit.Itemised(receipt(ana, bob, carol)),
                                        day)
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

            weighedLinesAndDiscounts(ana, bob, carol);

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
