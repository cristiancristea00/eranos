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
import eranos.ExpenseGroup;
import eranos.ExpenseId;
import eranos.ExpenseItem;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
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
import eranos.examples.grandtour.Tables.ItemRow;
import eranos.examples.grandtour.Tables.ShareRow;
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
                    case ExpenseSplit.Whole whole ->
                            switch (whole.split()) {
                                case Split.Equal _ -> "equally";
                                case Split.Parts _ -> "by parts";
                                case Split.Percentages _ -> "by percentage";
                                case Split.Amounts _ -> "by amounts";
                            };
                    case ExpenseSplit.Itemised itemised -> itemised.items().size() + " items";
                };
        int count = expense.payers().members().size();
        String payers = 1 == count ? "" : ", " + count + " payers";
        String tip = "";
        if (0 != expense.tipAmount().minorUnits() && expense.tip().isPresent()) {
            tip =
                    switch (expense.tip().orElseThrow().value()) {
                        case TipValue.Percentage percentage ->
                                ", " + percentage.percentage() + " tip";
                        case TipValue.Fixed fixed -> ", " + fixed.amount() + " tip";
                    };
        }
        return split + payers + tip;
    }

    /** Each expense of {@code expenses}, with every involved member's payment and consumption. */
    static void printShares(ExpenseGroup group, List<ExpenseId> expenses) throws EranosException {
        List<ShareRow> rows = new ArrayList<>();
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
                        new ShareRow(
                                expense.orElseThrow().name(),
                                how(expense.orElseThrow()),
                                cell(members),
                                cell(paid),
                                cell(consumed)));
            }
        }
        printTable(ShareRow.HEADERS, rows, 3);
    }

    static void printItems(ExpenseGroup group, ExpenseId id) throws EranosException {
        Optional<ExpenseSplit> split = group.expense(id).map(Expense::split);
        if (split.isPresent() && split.orElseThrow() instanceof ExpenseSplit.Itemised itemised) {
            List<ItemRow> rows = new ArrayList<>();
            for (ExpenseItem item : itemised.items()) {
                Money total =
                        new Money(
                                item.unitPrice().minorUnits() * item.quantity(),
                                item.unitPrice().currency());
                rows.add(
                        new ItemRow(
                                item.name(),
                                names(group, item.split().members()),
                                item.quantity(),
                                item.unitPrice(),
                                total));
            }
            printTable(ItemRow.HEADERS, rows, 2);
        }
    }

    /** The receipt of the fado night: every line with its own split. */
    static List<ExpenseItem> fadoItems(People people) throws EranosException {
        return List.of(
                new ExpenseItem(
                        "Vinho verde",
                        eur("12.00"),
                        new Split.Equal(List.of(people.ana(), people.bob(), people.dan())),
                        2),
                new ExpenseItem("Bacalhau", eur("18.00"), new Split.Equal(List.of(people.carol()))),
                new ExpenseItem(
                        "Custard tarts",
                        eur("1.50"),
                        new Split.Parts(
                                List.of(
                                        new MemberParts(people.ana(), Parts.parse("1")),
                                        new MemberParts(people.bob(), Parts.parse("1")),
                                        new MemberParts(people.carol(), Parts.parse("1")),
                                        new MemberParts(people.dan(), Parts.parse("1")),
                                        new MemberParts(people.eva(), Parts.parse("2")))),
                        6),
                new ExpenseItem("Juice", eur("3.00"), new Split.Equal(List.of(people.eva()))));
    }

    /** The fado night, item by item, paid by Carol and Dan, with a 10 % tip. */
    static Expense fadoNight(People people, Categories categories) throws EranosException {
        return Expense.builder(
                        "Fado night",
                        new Cost.Local(eur("54.00")),
                        new Split.Amounts(
                                List.of(
                                        new MemberAmount(people.carol(), eur("39.40")),
                                        new MemberAmount(people.dan(), eur("20.00")))),
                        new ExpenseSplit.Itemised(fadoItems(people)),
                        october(4))
                .tip(new Tip(new TipValue.Percentage(Percentage.parse("10"))))
                .time(TimeOfDay.of(22, 15))
                .category(categories.food())
                .description("Tasca do Chico")
                .build();
    }

    /** Adds one expense for every way of dividing one. */
    static Tour.Expenses addExpenses(ExpenseGroup group, People people, Categories categories)
            throws EranosException {
        ExpenseId groceries =
                group.addExpense(
                        Expense.builder(
                                        "Groceries",
                                        new Cost.Local(eur("10.00")),
                                        new Split.Equal(List.of(people.ana())),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(
                                                                people.ana(),
                                                                people.bob(),
                                                                people.carol()))),
                                        october(1))
                                .category(categories.food())
                                .build());
        ExpenseId apartment =
                group.addExpense(
                        Expense.builder(
                                        "Apartment",
                                        new Cost.Local(eur("800.00")),
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
                                                                        Percentage.parse("15"))))),
                                        october(1))
                                .category(categories.lodging())
                                .build());
        ExpenseId seafood =
                group.addExpense(
                        Expense.builder(
                                        "Seafood dinner",
                                        new Cost.Local(eur("90.00")),
                                        new Split.Equal(List.of(people.carol())),
                                        new ExpenseSplit.Whole(
                                                new Split.Parts(
                                                        List.of(
                                                                new MemberParts(
                                                                        people.ana(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        people.bob(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        people.carol(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        people.dan(),
                                                                        Parts.parse("1")),
                                                                new MemberParts(
                                                                        people.eva(),
                                                                        Parts.parse("0.5"))))),
                                        october(2))
                                .category(categories.food())
                                .build());
        ExpenseId train =
                group.addExpense(
                        Expense.builder(
                                        "Train to Porto",
                                        new Cost.Local(eur("88.50")),
                                        new Split.Equal(List.of(people.bob())),
                                        new ExpenseSplit.Whole(
                                                new Split.Amounts(
                                                        List.of(
                                                                new MemberAmount(
                                                                        people.ana(), eur("24.50")),
                                                                new MemberAmount(
                                                                        people.bob(), eur("24.50")),
                                                                new MemberAmount(
                                                                        people.carol(),
                                                                        eur("24.50")),
                                                                new MemberAmount(
                                                                        people.eva(),
                                                                        eur("15.00"))))),
                                        october(3))
                                .category(categories.transport())
                                .build());
        ExpenseId museum =
                group.addExpense(
                        Expense.builder(
                                        "Museum",
                                        new Cost.Local(eur("60.00")),
                                        new Split.Equal(List.of(people.ana(), people.bob())),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(
                                                                people.ana(),
                                                                people.bob(),
                                                                people.carol(),
                                                                people.dan()))),
                                        october(3))
                                .build());
        ExpenseId fado = group.addExpense(fadoNight(people, categories));
        ExpenseId tour =
                group.addExpense(
                        Expense.builder(
                                        "Tuk-tuk tour",
                                        new Cost.Local(eur("45.00")),
                                        new Split.Equal(List.of(people.dan())),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(
                                                                people.ana(),
                                                                people.bob(),
                                                                people.carol(),
                                                                people.dan(),
                                                                people.eva()))),
                                        october(5))
                                .tip(
                                        new Tip(
                                                new TipValue.Fixed(eur("5.00")),
                                                TipDistribution.EQUAL))
                                .category(categories.transport())
                                .build());
        return new Tour.Expenses(apartment, fado, groceries, museum, seafood, tour, train);
    }

    /** The consumption of {@code member} in the expense {@code id}. */
    static Optional<Money> consumed(ExpenseGroup group, ExpenseId id, MemberId member)
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
    static void printFado(ExpenseGroup group, Expense fado) throws EranosException {
        String rule =
                fado.tip()
                        .map(
                                tip ->
                                        switch (tip.value()) {
                                            case TipValue.Percentage percentage ->
                                                    percentage.percentage().toString();
                                            case TipValue.Fixed fixed -> fixed.amount().toString();
                                        })
                        .orElse("a tip");
        String shared =
                fado.tip()
                        .map(
                                tip ->
                                        switch (tip.distribution()) {
                                            case PROPORTIONAL ->
                                                    "in proportion to what each member ordered";
                                            case EQUAL -> "equally";
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

    static ExpenseId expenses(ExpenseGroup group, People people, Categories categories)
            throws EranosException {
        section("4. Expenses, one for every way of dividing");
        Tour.Expenses expenses = addExpenses(group, people, categories);
        ExpenseId apartment = expenses.apartment();
        ExpenseId fado = expenses.fado();
        ExpenseId groceries = expenses.groceries();
        ExpenseId museum = expenses.museum();
        ExpenseId seafood = expenses.seafood();
        ExpenseId tour = expenses.tour();
        ExpenseId train = expenses.train();
        System.out.print(
                """
                Seven expenses cover every way of dividing one: equally, by percentage, by parts
                (Eva counts as half), by exact amounts, with two payers, item by item, and with a
                tip. Every member's consumption adds up to the expense's total, to the cent:
                """);
        printShares(group, List.of(groceries, apartment, seafood, train, museum, fado, tour));
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
