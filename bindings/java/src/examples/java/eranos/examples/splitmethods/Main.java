package eranos.examples.splitmethods;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.totals;

import eranos.CalendarDate;
import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseId;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.Member;
import eranos.MemberAmount;
import eranos.MemberId;
import eranos.MemberParts;
import eranos.MemberPercentage;
import eranos.MemberShare;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Parts;
import eranos.Percentage;
import eranos.Split;
import eranos.Summary;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Every way to divide an expense: equally, by percentage, by parts (including half a part for a
 * child), by exact amounts, and with several payers. The port of the Rust example {@code
 * split_methods}, which prints the same report.
 */
public final class Main {
    private Main() {}

    /** The family and the day every expense happened. */
    private record Family(MemberId ana, MemberId bob, CalendarDate day, MemberId mia) {
        /** The rule of an expense shared equally by the three. */
        Map<MemberId, String> thirds() {
            return Map.of(ana, "an equal third", bob, "an equal third", mia, "an equal third");
        }
    }

    private static Money eur(String text) throws EranosException {
        return Money.parse(text, Currency.of("EUR"));
    }

    /**
     * Prints how one expense divides: every involved member's rule, what they paid, and what they
     * consumed.
     */
    private static void report(Group group, ExpenseId id, Map<MemberId, String> rules)
            throws EranosException {
        Optional<Expense> expense = group.expense(id);
        Optional<ExpenseShares> shares = group.expenseShares(id);
        if (expense.isEmpty() || shares.isEmpty()) {
            return;
        }
        System.out.printf("%s, %s:%n", expense.orElseThrow().name(), shares.orElseThrow().total());
        List<List<String>> rows = new ArrayList<>();
        for (MemberShare share : shares.orElseThrow().members()) {
            rows.add(
                    List.of(
                            names(group, List.of(share.member())),
                            rules.getOrDefault(share.member(), "(payer only)"),
                            share.paid().toString(),
                            share.share().toString()));
        }
        printTable(List.of("Member", "Rule", "Paid", "Consumed"), rows, 2);
    }

    private static void equally(Group group, Family family) throws EranosException {
        section("1. Equally");
        System.out.print(
                """
                Everyone pays the same. 10.00 does not divide by three, so one member pays the
                leftover cent; which member it is rotates from expense to expense, so over a trip
                nobody always pays it.
                """);
        ExpenseId taxi =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("10.00")),
                                        family.day(),
                                        "Taxi",
                                        new Split.Equal(List.of(family.ana())),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(
                                                                family.ana(),
                                                                family.bob(),
                                                                family.mia()))))
                                .build());
        report(group, taxi, family.thirds());
    }

    private static void byPercentage(Group group, Family family) throws EranosException {
        section("2. By percentage");
        System.out.print(
                """
                Everyone pays a percentage of the amount; the percentages must add up to exactly
                100 %. Mia's room is smaller, so she takes 20 %.
                """);
        ExpenseId rent =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("500.00")),
                                        family.day(),
                                        "Flat",
                                        new Split.Equal(List.of(family.bob())),
                                        new ExpenseSplit.Whole(
                                                new Split.Percentages(
                                                        List.of(
                                                                new MemberPercentage(
                                                                        family.ana(),
                                                                        Percentage.parse("40")),
                                                                new MemberPercentage(
                                                                        family.bob(),
                                                                        Percentage.parse("40")),
                                                                new MemberPercentage(
                                                                        family.mia(),
                                                                        Percentage.parse("20"))))))
                                .build());
        report(group, rent, Map.of(family.ana(), "40%", family.bob(), "40%", family.mia(), "20%"));
    }

    private static ExpenseId byParts(Group group, Family family) throws EranosException {
        section("3. By parts");
        System.out.print(
                """
                Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:
                2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.
                """);
        ExpenseId food =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("125.00")),
                                        family.day(),
                                        "Food",
                                        new Split.Equal(List.of(family.ana())),
                                        new ExpenseSplit.Whole(
                                                new Split.Parts(
                                                        List.of(
                                                                new MemberParts(
                                                                        family.ana(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        family.bob(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        family.mia(),
                                                                        Parts.parse("0.5"))))))
                                .build());
        report(
                group,
                food,
                Map.of(family.ana(), "1 part", family.bob(), "1 part", family.mia(), "0.5 parts"));
        return food;
    }

    private static void byAmounts(Group group, Family family) throws EranosException {
        section("4. By exact amounts");
        System.out.print(
                """
                Everyone pays a fixed amount, such as a ticket price; the amounts must add up to
                exactly the expense amount.
                """);
        ExpenseId tickets =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("35.00")),
                                        family.day(),
                                        "Museum",
                                        new Split.Equal(List.of(family.bob())),
                                        new ExpenseSplit.Whole(
                                                new Split.Amounts(
                                                        List.of(
                                                                new MemberAmount(
                                                                        eur("15.00"), family.ana()),
                                                                new MemberAmount(
                                                                        eur("15.00"), family.bob()),
                                                                new MemberAmount(
                                                                        eur("5.00"),
                                                                        family.mia())))))
                                .build());
        report(
                group,
                tickets,
                Map.of(
                        family.ana(),
                        "15.00 EUR",
                        family.bob(),
                        "15.00 EUR",
                        family.mia(),
                        "5.00 EUR"));
    }

    private static void severalPayers(Group group, Family family) throws EranosException {
        section("5. Several payers");
        System.out.print(
                """
                The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are
                divided with the same methods as the participants.
                """);
        ExpenseId dinner =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Local(eur("90.00")),
                                        family.day(),
                                        "Dinner",
                                        new Split.Amounts(
                                                List.of(
                                                        new MemberAmount(
                                                                eur("50.00"), family.ana()),
                                                        new MemberAmount(
                                                                eur("40.00"), family.bob()))),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(
                                                                family.ana(),
                                                                family.bob(),
                                                                family.mia()))))
                                .build());
        report(group, dinner, family.thirds());
    }

    private static void mistakes(Group group, Family family) throws EranosException {
        section("6. Mistakes are refused");
        System.out.println(
                "A percentage split that adds up to 90 % is refused when the expense is built:");
        Expense invalid =
                Expense.builder(
                                new Cost.Local(eur("10.00")),
                                family.day(),
                                "Invalid",
                                new Split.Equal(List.of(family.ana())),
                                new ExpenseSplit.Whole(
                                        new Split.Percentages(
                                                List.of(
                                                        new MemberPercentage(
                                                                family.ana(),
                                                                Percentage.parse("60")),
                                                        new MemberPercentage(
                                                                family.bob(),
                                                                Percentage.parse("30"))))))
                        .build();
        try {
            group.addExpense(invalid);
            throw new IllegalStateException("the split must be refused");
        } catch (EranosException refusal) {
            System.out.println("  error: " + refusal.getMessage());
        }
    }

    private static void printTotals(Group group) {
        section("7. The holiday in total");
        Summary summary = group.summary();
        System.out.printf("%s spent over five expenses:%n", summary.totalSpent());
        List<List<String>> rows = new ArrayList<>();
        for (Member member : group.members()) {
            Optional<MemberSummary> found = member.id().flatMap(id -> totals(id, summary));
            found.ifPresent(
                    totals ->
                            rows.add(
                                    List.of(
                                            member.firstName(),
                                            totals.paid().toString(),
                                            totals.share().toString(),
                                            totals.balance().toString())));
        }
        printTable(List.of("Member", "Paid", "Consumed", "Balance"), rows, 1);
    }

    /**
     * Prints the report of the family holiday.
     *
     * @param arguments none
     * @throws EranosException if the library refuses a value the example gives it
     */
    public static void main(String[] arguments) throws EranosException {
        try (Group group = Group.builder(Currency.of("EUR"), "Family holiday").build()) {
            MemberId ana = group.addMember(Member.builder("Ana").build());
            MemberId bob = group.addMember(Member.builder("Bob").build());
            CalendarDate day = CalendarDate.of(14, 7, 2026);
            MemberId mia = group.addMember(Member.builder("Mia").build());
            Family family = new Family(ana, bob, day, mia);

            System.out.println("Family holiday: every way to split an expense");
            System.out.println("═════════════════════════════════════════════");
            System.out.print(
                    """
                    Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a
                    different split method; whatever the method, the shares add up to exactly the
                    amount, to the cent.
                    """);
            equally(group, family);
            byPercentage(group, family);
            ExpenseId food = byParts(group, family);
            byAmounts(group, family);
            severalPayers(group, family);
            mistakes(group, family);
            printTotals(group);

            // Parts: 125.00 over 2.5 parts is 50.00 per part.
            Optional<Money> consumed =
                    group.expenseShares(food)
                            .flatMap(
                                    shares ->
                                            shares.members().stream()
                                                    .filter(share -> share.member().equals(mia))
                                                    .findFirst())
                            .map(MemberShare::share);
            check(Optional.of(eur("25.00")).equals(consumed));
        }
    }
}
