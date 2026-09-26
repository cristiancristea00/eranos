package eranos.examples.grandtour;

import static eranos.examples.Report.check;
import static eranos.examples.Report.printTable;
import static eranos.examples.Report.section;
import static eranos.examples.Report.totals;
import static eranos.examples.grandtour.Tables.printBalances;
import static eranos.examples.grandtour.Tables.printEntries;
import static eranos.examples.grandtour.Tour.eur;
import static eranos.examples.grandtour.Tour.groupFailure;
import static eranos.examples.grandtour.Tour.message;
import static eranos.examples.grandtour.Tour.name;
import static eranos.examples.grandtour.Tour.october;
import static eranos.examples.grandtour.Tour.refusal;
import static eranos.examples.grandtour.Tour.ron;

import eranos.Cost;
import eranos.Currency;
import eranos.EranosException;
import eranos.ExchangeRate;
import eranos.Expense;
import eranos.ExpenseFailure;
import eranos.ExpenseId;
import eranos.ExpenseShares;
import eranos.ExpenseSplit;
import eranos.Group;
import eranos.GroupFailure;
import eranos.Member;
import eranos.MemberPercentage;
import eranos.MemberSummary;
import eranos.Money;
import eranos.Percentage;
import eranos.Split;
import eranos.Summary;
import eranos.TimeOfDay;
import eranos.Transfer;
import eranos.TransferFailure;
import eranos.TransferId;
import eranos.examples.grandtour.Tour.Categories;
import eranos.examples.grandtour.Tour.People;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members
 * who come and go.
 */
final class Edits {
    private Edits() {}

    static void anotherCurrency(Group group, People people, Categories categories)
            throws EranosException {
        section("5. Another currency");
        Currency euro = Currency.of("EUR");
        ExchangeRate rate = ExchangeRate.parse("0.2");
        ExpenseId parking =
                group.addExpense(
                        Expense.builder(
                                        new Cost.Converted(ron("250.00"), rate),
                                        october(6),
                                        "Airport parking",
                                        new Split.Equal(List.of(people.ana())),
                                        new ExpenseSplit.Whole(
                                                new Split.Equal(
                                                        List.of(people.ana(), people.bob()))))
                                .category(categories.transport())
                                .build());
        TransferId cash =
                group.addTransfer(
                        Transfer.builder(
                                        new Cost.Converted(ron("100.00"), rate),
                                        october(6),
                                        people.bob(),
                                        people.ana())
                                .note("Cash for the parking")
                                .time(TimeOfDay.of(7, 40))
                                .build());
        System.out.print(
                """
                Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in
                cash. An entry in another currency carries the rate into the group currency, how
                many EUR one RON is worth, and is converted once, when it is entered:
                """);
        printEntries(group, currency -> !euro.equals(currency));
        Transfer transfer = group.transfer(cash).orElseThrow();
        Money converted = ExchangeRate.parse("0.2011").convert(ron("250.00"), euro);
        System.out.print(
                """
                Bob's transfer, at %s ("%s"), counts as %s.
                A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,
                which becomes %s.
                """
                        .formatted(
                                transfer.time().map(TimeOfDay::toString).orElse(""),
                                transfer.note().orElse(""),
                                group.transferAmount(cash).orElse(new Money(euro, 0)),
                                converted));

        check(eur("50.28").equals(converted));
        check(
                Optional.of(eur("50.00"))
                        .equals(group.expenseShares(parking).map(ExpenseShares::total)));
        check(Optional.of(eur("20.00")).equals(group.transferAmount(cash)));
        check(transfer.cost() instanceof Cost.Converted);
    }

    /** Replaces an expense and a transfer, and removes one of each; returns what changed. */
    static List<List<String>> replaceAndRemove(
            Group group, People people, Categories categories, ExpenseId seafood)
            throws EranosException {
        // The dinner's receipt said 99.00.
        Expense corrected =
                group.expense(seafood).orElseThrow().toBuilder()
                        .cost(new Cost.Local(eur("99.00")))
                        .description("Cervejaria Ramiro")
                        .build();
        Expense previous = group.replaceExpense(corrected);
        // Bob entered the coffee twice.
        Expense coffee =
                Expense.builder(
                                new Cost.Local(eur("6.00")),
                                october(5),
                                "Coffee",
                                new Split.Equal(List.of(people.bob())),
                                new ExpenseSplit.Whole(
                                        new Split.Equal(
                                                List.of(
                                                        people.ana(),
                                                        people.bob(),
                                                        people.carol()))))
                        .category(categories.food())
                        .build();
        group.addExpense(coffee);
        ExpenseId copy = group.addExpense(coffee);
        Expense removed = group.removeExpense(copy);
        // Dan's part of the apartment was 120.00, not 100.00.
        TransferId first =
                group.addTransfer(
                        Transfer.builder(
                                        new Cost.Local(eur("100.00")),
                                        october(5),
                                        people.dan(),
                                        people.ana())
                                .build());
        Transfer fixed =
                group.transfer(first).orElseThrow().toBuilder()
                        .cost(new Cost.Local(eur("120.00")))
                        .note("Apartment")
                        .build();
        Transfer replaced = group.replaceTransfer(fixed);
        // Carol sent 10.00 to the wrong member.
        TransferId mistake =
                group.addTransfer(
                        Transfer.builder(
                                        new Cost.Local(eur("10.00")),
                                        october(5),
                                        people.carol(),
                                        people.bob())
                                .build());
        Transfer undone = group.removeTransfer(mistake);

        check(eur("90.00").equals(previous.cost().amount()));
        check(group.expense(copy).isEmpty());
        check(Optional.of(eur("120.00")).equals(group.transferAmount(first)));
        check(group.transfer(mistake).isEmpty());
        Money zero = new Money(Currency.of("EUR"), 0);
        Optional<Expense> dinner = group.expense(seafood);
        Optional<Transfer> apartment = group.transfer(first);
        return List.of(
                List.of(
                        previous.name(),
                        "%s became %s, at %s"
                                .formatted(
                                        previous.cost().amount(),
                                        dinner.map(expense -> expense.cost().amount()).orElse(zero),
                                        dinner.flatMap(Expense::description).orElse(""))),
                List.of(
                        removed.name(),
                        "entered twice; the copy of %s was removed"
                                .formatted(removed.cost().amount())),
                List.of(
                        name(group, people.dan()) + " to " + name(group, people.ana()),
                        "%s became %s, noted %s"
                                .formatted(
                                        replaced.cost().amount(),
                                        apartment
                                                .map(transfer -> transfer.cost().amount())
                                                .orElse(zero),
                                        apartment.flatMap(Transfer::note).orElse(""))),
                List.of(
                        name(group, people.carol()) + " to " + name(group, people.bob()),
                        "%s sent to the wrong member; removed".formatted(undone.cost().amount())));
    }

    static void edits(Group group, People people, Categories categories, ExpenseId seafood)
            throws EranosException {
        section("6. Edits");
        List<List<String>> changes = replaceAndRemove(group, people, categories, seafood);
        System.out.print(
                """
                Entries are values: an edit starts from a copy made with to_builder, keeps the
                identifier, and replaces the entry; a removal takes it out with its totals.
                """);
        printTable(List.of("Entry", "Edit"), changes, 2);
        System.out.println("The balances after the edits:");
        printBalances(group);

        Summary before = group.summary();
        Expense taxi =
                Expense.builder(
                                new Cost.Local(eur("30.00")),
                                october(6),
                                "Taxi",
                                new Split.Equal(List.of(people.ana())),
                                new ExpenseSplit.Whole(
                                        new Split.Percentages(
                                                List.of(
                                                        new MemberPercentage(
                                                                people.ana(),
                                                                Percentage.parse("50")),
                                                        new MemberPercentage(
                                                                people.bob(),
                                                                Percentage.parse("49.99"))))))
                        .build();
        Optional<EranosException> unbalanced = refusal(() -> group.addExpense(taxi));
        Transfer toHerself =
                Transfer.builder(
                                new Cost.Local(eur("5.00")), october(6), people.ana(), people.ana())
                        .build();
        Optional<EranosException> herself = refusal(() -> group.addTransfer(toHerself));
        Expense souvenirs =
                Expense.builder(
                                new Cost.Local(ron("40.00")),
                                october(6),
                                "Souvenirs",
                                new Split.Equal(List.of(people.ana())),
                                new ExpenseSplit.Whole(new Split.Equal(List.of(people.ana()))))
                        .build();
        Optional<EranosException> unrated = refusal(() -> group.addExpense(souvenirs));
        System.out.println("A change that does not fit is refused, and the group stays as it was:");
        printTable(
                List.of("Change", "Refused with"),
                List.of(
                        List.of("A taxi split 50% and 49.99%", message(unbalanced)),
                        List.of("Ana paying herself back", message(herself)),
                        List.of("Souvenirs in RON, without a rate", message(unrated))),
                2);
        System.out.println("The balances again, unchanged:");
        printBalances(group);

        check(
                Optional.of(new ExpenseFailure.PercentagesDoNotAddUp(9_999))
                        .equals(
                                unbalanced
                                        .filter(EranosException.Expense.class::isInstance)
                                        .map(EranosException.Expense.class::cast)
                                        .map(EranosException.Expense::failure)));
        check(
                Optional.of(new TransferFailure.SameMember())
                        .equals(
                                herself.filter(EranosException.Transfer.class::isInstance)
                                        .map(EranosException.Transfer.class::cast)
                                        .map(EranosException.Transfer::failure)));
        check(
                Optional.of(new GroupFailure.MissingExchangeRate(Currency.of("RON")))
                        .equals(groupFailure(unrated)));
        check(before.equals(group.summary()));
    }

    static void comingsAndGoings(Group group, People people) throws EranosException {
        section("7. Members come and go");
        Member removed = group.removeMember(people.filip());
        group.archiveMember(people.dan());
        Expense dinner =
                Expense.builder(
                                new Cost.Local(eur("40.00")),
                                october(6),
                                "Farewell dinner",
                                new Split.Equal(List.of(people.ana())),
                                new ExpenseSplit.Whole(
                                        new Split.Equal(
                                                List.of(
                                                        people.ana(),
                                                        people.bob(),
                                                        people.carol(),
                                                        people.dan()))))
                        .build();
        Optional<EranosException> farewell = refusal(() -> group.addExpense(dinner));
        Optional<EranosException> inUse = refusal(() -> group.removeMember(people.dan()));
        System.out.print(
                """
                %s was never part of an entry, so removing him leaves no trace. Dan flew home
                early and was archived: his history and his balance stay, but a new expense
                cannot include him, and he cannot be removed while entries reference him.
                """
                        .formatted(removed.firstName()));
        Summary summary = group.summary();
        List<List<String>> rows = new ArrayList<>();
        for (Member member : group.members()) {
            if (member.id().isPresent()) {
                Optional<MemberSummary> found = totals(member.id().orElseThrow(), summary);
                if (found.isPresent()) {
                    rows.add(
                            List.of(
                                    member.firstName(),
                                    group.isArchived(member.id().orElseThrow())
                                            ? "archived"
                                            : "active",
                                    found.orElseThrow().balance().toString()));
                }
            }
        }
        printTable(List.of("Member", "Status", "Balance"), rows, 2);
        printTable(
                List.of("Change", "Refused with"),
                List.of(
                        List.of("A farewell dinner with Dan", message(farewell)),
                        List.of("Removing Dan", message(inUse))),
                2);
        group.unarchiveMember(people.dan());
        System.out.println("When Dan joins the next trip, unarchiving him makes him active again.");

        check("Filip".equals(removed.firstName()));
        check(group.member(people.filip()).isEmpty());
        check(
                Optional.of(new GroupFailure.ArchivedMember(people.dan()))
                        .equals(groupFailure(farewell)));
        check(groupFailure(inUse).filter(GroupFailure.MemberInUse.class::isInstance).isPresent());
        check(!group.isArchived(people.dan()));
    }
}
