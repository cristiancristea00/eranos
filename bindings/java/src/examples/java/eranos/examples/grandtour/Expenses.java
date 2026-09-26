package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.names;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.grandtour.Tour.cell;
import static eranos.examples.grandtour.Tour.eur;
import static eranos.examples.grandtour.Tour.name;
import static eranos.examples.grandtour.Tour.october;

import eranos.Cost;
import eranos.EranosException;
import eranos.Expense;
import eranos.ExpenseId;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.Item;
import eranos.MemberAmount;
import eranos.MemberId;
import eranos.MemberParts;
import eranos.MemberPercentage;
import eranos.MemberShare;
import eranos.Money;
import eranos.Parts;
import eranos.Percentage;
import eranos.Split;
import eranos.TimeOfDay;
import eranos.Tip;
import eranos.TipDistribution;
import eranos.TipValue;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/** Section 4 of the tour: an expense for every way of dividing one, with each member's shares. */
final class Expenses {
    private Expenses() {}

    /** How an expense divides, in a few words. */
    static String how(Expense expense) throws EranosException {
        String split =
                switch (expense.split()) {
                    case ExpenseSplit.Itemised itemised -> itemised.items().size() + " items";
                    case ExpenseSplit.Whole whole ->
                            switch (whole.split()) {
                                case Split.Amounts _ -> "by amounts";
                                case Split.Equal _ -> "equally";
                                case Split.Parts _ -> "by parts";
                                case Split.Percentages _ -> "by percentage";
                            };
                };
        int count = expense.payers().members().size();
        String payers = 1 == count ? "" : ", " + count + " payers";
        String tip = "";
        if (0 != expense.tipAmount().minorUnits() && expense.tip().isPresent()) {
            tip =
                    switch (expense.tip().orElseThrow().value()) {
                        case TipValue.Fixed fixed -> ", " + fixed.amount() + " tip";
                        case TipValue.Percentage percentage ->
                                ", " + percentage.percentage() + " tip";
                    };
        }
        return split + payers + tip;
    }

    /** Each expense of {@code expenses}, with every involved member's payment and consumption. */
    static void printShares(Group group, List<ExpenseId> expenses) throws EranosException {
        List<List<String>> rows = new ArrayList<>();
        for (ExpenseId id : expenses) {
            Optional<Expense> expense = group.expense(id);
            Optional<ExpenseShares> shares = group.expenseShares(id);
            if (expense.isPresent() && shares.isPresent()) {
                List<String> members = new ArrayList<>();
                List<String> paid = new ArrayList<>();
                List<String> consumed = new ArrayList<>();
                for (MemberShare share : shares.orElseThrow().members()) {
                    members.add(name(group, share.member()));
                    paid.add(0 == share.paid().minorUnits() ? "" : share.paid().toString());
                    consumed.add(share.share().toString());
                }
                rows.add(
                        List.of(
                                expense.orElseThrow().name(),
                                how(expense.orElseThrow()),
                                cell(members),
                                cell(paid),
                                cell(consumed)));
            }
        }
        printTable(List.of("Expense", "Split", "Member", "Paid", "Consumed"), rows, 3);
    }

    static void printItems(Group group, ExpenseId id) throws EranosException {
        Optional<ExpenseSplit> split = group.expense(id).map(Expense::split);
        if (split.isPresent() && split.orElseThrow() instanceof ExpenseSplit.Itemised itemised) {
            List<List<String>> rows = new ArrayList<>();
            for (Item item : itemised.items()) {
                Money total =
                        new Money(
                                item.unitPrice().currency(),
                                item.unitPrice().minorUnits() * item.quantity());
                rows.add(
                        List.of(
                                item.name(),
                                names(group, item.split().members()),
                                String.valueOf(item.quantity()),
                                item.unitPrice().toString(),
                                total.toString()));
            }
            printTable(
                    List.of("Item", "Shared by", "Quantity", "Unit price", "Line total"), rows, 2);
        }
    }

    /** The receipt of the fado night: every line with its own split. */
    static List<Item> fadoItems(People people) throws EranosException {
        return List.of(
                new Item(
                        "Vinho verde",
                        new Split.Equal(List.of(people.ana(), people.bob(), people.dan())),
                        eur("12.00"),
                        2),
                new Item("Bacalhau", new Split.Equal(List.of(people.carol())), eur("18.00")),
                new Item(
                        "Custard tarts",
                        new Split.Parts(
                                List.of(
                                        new MemberParts(people.ana(), Parts.parse("1")),
                                        new MemberParts(people.bob(), Parts.parse("1")),
                                        new MemberParts(people.carol(), Parts.parse("1")),
                                        new MemberParts(people.dan(), Parts.parse("1")),
                                        new MemberParts(people.eva(), Parts.parse("2")))),
                        eur("1.50"),
                        6),
                new Item("Juice", new Split.Equal(List.of(people.eva())), eur("3.00")));
    }

    /** The first three expenses: equally, by percentage, and by parts. */
    static List<Expense> dividedExpenses(People people, Categories categories)
            throws EranosException {
        return List.of(
                Expense.builder(
                                new Cost.Local(eur("10.00")),
                                october(1),
                                "Groceries",
                                new Split.Equal(List.of(people.ana())),
                                new ExpenseSplit.Whole(
                                        new Split.Equal(
                                                List.of(
                                                        people.ana(),
                                                        people.bob(),
                                                        people.carol()))))
                        .category(categories.food())
                        .build(),
                Expense.builder(
                                new Cost.Local(eur("800.00")),
                                october(1),
                                "Apartment",
                                new Split.Equal(List.of(people.ana())),
                                new ExpenseSplit.Whole(
                                        new Split.Percentages(
                                                List.of(
                                                        new MemberPercentage(
                                                                people.ana(),
                                                                Percentage.parse("25")),
                                                        new MemberPercentage(
                                                                people.bob(),
                                                                Percentage.parse("25")),
                                                        new MemberPercentage(
                                                                people.carol(),
                                                                Percentage.parse("35")),
                                                        new MemberPercentage(
                                                                people.dan(),
                                                                Percentage.parse("15"))))))
                        .category(categories.lodging())
                        .build(),
                Expense.builder(
                                new Cost.Local(eur("90.00")),
                                october(2),
                                "Seafood dinner",
                                new Split.Equal(List.of(people.carol())),
                                new ExpenseSplit.Whole(
                                        new Split.Parts(
                                                List.of(
                                                        new MemberParts(
                                                                people.ana(), Parts.parse("1")),
                                                        new MemberParts(
                                                                people.bob(), Parts.parse("1")),
                                                        new MemberParts(
                                                                people.carol(), Parts.parse("1")),
                                                        new MemberParts(
                                                                people.dan(), Parts.parse("1")),
                                                        new MemberParts(
                                                                people.eva(),
                                                                Parts.parse("0.5"))))))
                        .category(categories.food())
                        .build());
    }

    /** The last four expenses: by exact amounts, with two payers, item by item, and with a tip. */
    static List<Expense> paidExpenses(People people, Categories categories) throws EranosException {
        return List.of(
                Expense.builder(
                                new Cost.Local(eur("88.50")),
                                october(3),
                                "Train to Porto",
                                new Split.Equal(List.of(people.bob())),
                                new ExpenseSplit.Whole(
                                        new Split.Amounts(
                                                List.of(
                                                        new MemberAmount(
                                                                eur("24.50"), people.ana()),
                                                        new MemberAmount(
                                                                eur("24.50"), people.bob()),
                                                        new MemberAmount(
                                                                eur("24.50"), people.carol()),
                                                        new MemberAmount(
                                                                eur("15.00"), people.eva())))))
                        .category(categories.transport())
                        .build(),
                Expense.builder(
                                new Cost.Local(eur("60.00")),
                                october(3),
                                "Museum",
                                new Split.Equal(List.of(people.ana(), people.bob())),
                                new ExpenseSplit.Whole(
                                        new Split.Equal(
                                                List.of(
                                                        people.ana(),
                                                        people.bob(),
                                                        people.carol(),
                                                        people.dan()))))
                        .build(),
                Expense.builder(
                                new Cost.Local(eur("54.00")),
                                october(4),
                                "Fado night",
                                new Split.Amounts(
                                        List.of(
                                                new MemberAmount(eur("39.40"), people.carol()),
                                                new MemberAmount(eur("20.00"), people.dan()))),
                                new ExpenseSplit.Itemised(fadoItems(people)))
                        .category(categories.food())
                        .description("Tasca do Chico")
                        .time(TimeOfDay.of(22, 15))
                        .tip(new Tip(new TipValue.Percentage(Percentage.parse("10"))))
                        .build(),
                Expense.builder(
                                new Cost.Local(eur("45.00")),
                                october(5),
                                "Tuk-tuk tour",
                                new Split.Equal(List.of(people.dan())),
                                new ExpenseSplit.Whole(
                                        new Split.Equal(
                                                List.of(
                                                        people.ana(),
                                                        people.bob(),
                                                        people.carol(),
                                                        people.dan(),
                                                        people.eva()))))
                        .category(categories.transport())
                        .tip(new Tip(new TipValue.Fixed(eur("5.00")), TipDistribution.EQUAL))
                        .build());
    }

    /** The consumption of {@code member} in the expense {@code id}. */
    static Optional<Money> consumed(Group group, ExpenseId id, MemberId member)
            throws EranosException {
        return group.expenseShares(id)
                .flatMap(
                        shares ->
                                shares.members().stream()
                                        .filter(share -> share.member().equals(member))
                                        .findFirst())
                .map(MemberShare::share);
    }

    /** Prints how the fado night's tip was shared, and its receipt. */
    static void printFado(Group group, Expense fado) throws EranosException {
        String rule =
                fado.tip()
                        .map(
                                tip ->
                                        switch (tip.value()) {
                                            case TipValue.Fixed fixed -> fixed.amount().toString();
                                            case TipValue.Percentage percentage ->
                                                    percentage.percentage().toString();
                                        })
                        .orElse("a tip");
        String shared =
                fado.tip()
                        .map(
                                tip ->
                                        switch (tip.distribution()) {
                                            case EQUAL -> "equally";
                                            case PROPORTIONAL ->
                                                    "in proportion to what each member ordered";
                                        })
                        .orElse("another way");
        System.out.print(
                """
                The groceries' 10.00 EUR did not divide into thirds, so one member consumed the
                leftover cent; the expense's identifier picks which, so it moves from expense to
                expense. The fado night, at %s on %s at %s, was itemised; its
                %s tip of %s was shared %s:
                """
                        .formatted(
                                fado.time().map(TimeOfDay::toString).orElse(""),
                                fado.date(),
                                fado.description().orElse(""),
                                rule,
                                fado.tipAmount(),
                                shared));
        if (fado.id().isPresent()) {
            printItems(group, fado.id().orElseThrow());
        }
        System.out.println("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
    }

    static ExpenseId expenses(Group group, People people, Categories categories)
            throws EranosException {
        section("4. Expenses, one for every way of dividing");
        List<ExpenseId> ids = new ArrayList<>();
        for (Expense expense : dividedExpenses(people, categories)) {
            ids.add(group.addExpense(expense));
        }
        for (Expense expense : paidExpenses(people, categories)) {
            ids.add(group.addExpense(expense));
        }
        ExpenseId groceries = ids.get(0);
        ExpenseId apartment = ids.get(1);
        ExpenseId seafood = ids.get(2);
        ExpenseId train = ids.get(3);
        ExpenseId museum = ids.get(4);
        ExpenseId fado = ids.get(5);
        ExpenseId tour = ids.get(6);
        System.out.print(
                """
                Seven expenses cover every way of dividing one: equally, by percentage, by parts
                (Eva counts as half), by exact amounts, with two payers, item by item, and with a
                tip. Every member's consumption adds up to the expense's total, to the cent:
                """);
        printShares(group, ids);
        Expense fadoExpense = group.expense(fado).orElseThrow();
        printFado(group, fadoExpense);

        List<Long> thirds = new ArrayList<>();
        for (MemberId member : List.of(people.ana(), people.bob(), people.carol())) {
            consumed(group, groceries, member).map(Money::minorUnits).ifPresent(thirds::add);
        }
        check(List.of(333L, 333L, 334L).equals(thirds.stream().sorted().toList()));
        check(Optional.of(eur("280.00")).equals(consumed(group, apartment, people.carol())));
        check(Optional.of(eur("10.00")).equals(consumed(group, seafood, people.eva())));
        check(Optional.of(eur("15.00")).equals(consumed(group, train, people.eva())));
        check(
                Optional.of(eur("30.00"))
                        .equals(
                                group.expenseShares(museum)
                                        .flatMap(
                                                shares ->
                                                        shares.members().stream()
                                                                .filter(
                                                                        share ->
                                                                                share.member()
                                                                                        .equals(
                                                                                                people
                                                                                                        .bob()))
                                                                .findFirst())
                                        .map(MemberShare::paid)));
        check(Optional.of(eur("15.00")).equals(consumed(group, museum, people.dan())));
        check(4 == group.expense(museum).orElseThrow().members().size());
        check(eur("59.40").equals(fadoExpense.total()));
        check(eur("5.40").equals(fadoExpense.tipAmount()));
        check(Optional.of(eur("21.45")).equals(consumed(group, fado, people.carol())));
        check(Optional.of(eur("6.60")).equals(consumed(group, fado, people.eva())));
        check(Optional.of(eur("10.00")).equals(consumed(group, tour, people.eva())));
        return seafood;
    }
}
