// The entries. An entry that converted to the generated code and back with a field changed or
// dropped, or read back from the group another way than it was added, would store another expense,
// member, or transfer than the app built.

package eranos;

import static eranos.Flat.assertValue;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;

/** The expenses, members, categories, and transfers, and the splits, costs, and tips they hold. */
class EntriesTest {
    /**
     * A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally: with {@link
     * #dinner}, every kind of cost, split, and tip.
     */
    private static Expense taxi(Flat flat) throws EranosException {
        CurrencyCode lei = CurrencyCode.of("RON");
        return Expense.builder(
                        "Taxi",
                        new Cost.Converted(new Money(5_000, lei), ExchangeRate.parse("0.2")),
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(10_000)))),
                        new ExpenseSplit.Itemised(
                                List.of(
                                        new ExpenseItem(
                                                "Fare",
                                                new Money(2_500, lei),
                                                new Split.Parts(
                                                        List.of(
                                                                new MemberParts(
                                                                        flat.ana,
                                                                        Parts.ofHundredths(100)),
                                                                new MemberParts(
                                                                        flat.bob,
                                                                        Parts.ofHundredths(50)))),
                                                2))),
                        flat.day)
                .tip(new TipRule(new TipValue.Fixed(new Money(500, lei)), TipDistribution.EQUAL))
                .time(TimeOfDay.of(9, 5))
                .category(flat.food)
                .description("To the airport")
                .build();
    }

    /**
     * A dinner in euros, split equally, paid by amounts, with a percentage tip: with {@link #taxi},
     * every kind of cost, split, and tip.
     */
    private static Expense dinner(Flat flat) throws EranosException {
        return Expense.builder(
                        "Dinner",
                        new Cost.Local(flat.euros(3_000)),
                        new Split.Amounts(List.of(new MemberAmount(flat.ana, flat.euros(3_300)))),
                        new ExpenseSplit.Whole(new Split.Equal(List.of(flat.ana, flat.bob))),
                        flat.day)
                .tip(new TipRule(new TipValue.Percentage(Percentage.ofBasisPoints(1_000))))
                .build();
    }

    /**
     * An expense that converted with a field changed or dropped, or that the group stored another
     * way, would divide another expense than the app built.
     */
    @Test
    void everyKindOfExpenseConvertsAndStoresWhole() throws EranosException {
        Flat flat = new Flat();
        for (Expense expense : List.of(taxi(flat), dinner(flat))) {
            assertEquals(expense, Expense.converted(expense.generated()));
            ExpenseId id = flat.group.addExpense(expense);
            assertEquals(expense.id(), id);
            Expense stored = flat.group.expense(id).orElseThrow();
            assertEquals(id, stored.id());
            assertEquals(expense.name(), stored.name());
            assertEquals(expense.cost(), stored.cost());
            assertEquals(expense.payers(), stored.payers());
            assertEquals(expense.split(), stored.split());
            assertEquals(expense.date(), stored.date());
            assertEquals(expense.tip(), stored.tip());
            assertEquals(expense.time(), stored.time());
            assertEquals(expense.category(), stored.category());
            assertEquals(expense.description(), stored.description());
            assertValue(
                    stored, stored.toBuilder().build(), stored.toBuilder().name("Lunch").build());
        }
        Expense plain = flat.expense("Lunch", flat.ana, List.of(flat.bob));
        Expense edited =
                plain.toBuilder()
                        .cost(new Cost.Local(flat.euros(1)))
                        .payers(new Split.Equal(List.of(flat.bob)))
                        .split(new ExpenseSplit.Whole(new Split.Equal(List.of(flat.ana))))
                        .date(CalendarDate.of(2026, 9, 24))
                        .build();
        assertEquals(new Cost.Local(flat.euros(1)), edited.cost());
        assertEquals(CalendarDate.of(2026, 9, 24), edited.date());
    }

    /**
     * An expense built without a tip or a category that carried another tip or category to the
     * group, or read back with another value than the no-tip rule and the built-in category, would
     * divide another amount or let an app take the defaults for values it set; a builder that took
     * {@code null} for either would fail only when the expense reached the group.
     */
    @Test
    void anExpenseStartsWithoutATipInTheBuiltInCategoryAndReadsBackSo() throws EranosException {
        Flat flat = new Flat();
        Expense plain = flat.expense("Lunch", flat.ana, List.of(flat.bob));
        assertEquals(TipRule.NONE, plain.tip());
        assertEquals(CategoryId.UNCATEGORISED, plain.category());
        Expense stored = flat.group.expense(flat.group.addExpense(plain)).orElseThrow();
        assertEquals(TipRule.NONE, stored.tip());
        assertEquals(CategoryId.UNCATEGORISED, stored.category());
        assertEquals(flat.euros(0), stored.tipAmount());
        Expense.Builder builder = plain.toBuilder();
        assertThrows(NullPointerException.class, () -> builder.tip(null));
        assertThrows(NullPointerException.class, () -> builder.category(null));
        assertEquals(plain, builder.build());
    }

    /**
     * A tip of 0 % set on purpose that read back as another rule than the no-tip rule when it is
     * shared proportionally, or as the no-tip rule when it is shared equally, would show an app
     * another tip than the group holds.
     */
    @Test
    void aZeroTipReadsBackAsTheGroupHoldsIt() throws EranosException {
        Flat flat = new Flat();
        Percentage zero = Percentage.ofBasisPoints(0);
        Expense proportional =
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).toBuilder()
                        .tip(new TipRule(new TipValue.Percentage(zero)))
                        .build();
        assertEquals(
                TipRule.NONE,
                flat.group.expense(flat.group.addExpense(proportional)).orElseThrow().tip());
        TipRule equal = new TipRule(new TipValue.Percentage(zero), TipDistribution.EQUAL);
        Expense shared =
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).toBuilder().tip(equal).build();
        TipRule stored = flat.group.expense(flat.group.addExpense(shared)).orElseThrow().tip();
        assertEquals(equal, stored);
        assertNotEquals(TipRule.NONE, stored);
    }

    /**
     * The tip, total, and members an app shows for an expense would differ from those the group
     * records when it adds it.
     */
    @Test
    void anExpenseTellsItsTipTotalAndMembers() throws EranosException {
        Flat flat = new Flat();
        CurrencyCode lei = CurrencyCode.of("RON");
        Expense taxi = taxi(flat);
        assertEquals(new Money(500, lei), taxi.tipAmount());
        assertEquals(new Money(5_500, lei), taxi.total());
        assertEquals(List.of(flat.ana, flat.bob), taxi.members());
        assertEquals(flat.euros(300), dinner(flat).tipAmount());
        Expense unreadable =
                taxi.toBuilder().payers(new Split.Equal(List.of(new MemberId(" nobody")))).build();
        assertThrows(EranosException.Id.class, unreadable::members);
        assertThrows(
                EranosException.Expense.class,
                () ->
                        taxi.toBuilder()
                                .tip(new TipRule(new TipValue.Fixed(flat.euros(1))))
                                .build()
                                .tipAmount());
        assertThrows(
                EranosException.Expense.class,
                () ->
                        taxi.toBuilder()
                                .tip(new TipRule(new TipValue.Fixed(new Money(-1, lei))))
                                .build()
                                .total());
        assertEquals(
                List.of(TipDistribution.PROPORTIONAL, TipDistribution.EQUAL),
                List.of(TipDistribution.values()));
        for (TipDistribution distribution : TipDistribution.values()) {
            assertEquals(distribution, TipDistribution.converted(distribution.generated()));
        }
    }

    /**
     * A split, a cost, or a tip that converted with a field changed would divide another amount.
     */
    @Test
    void splitsCostsAndTipsConvertWhole() throws EranosException {
        Flat flat = new Flat();
        List<Split> splits =
                List.of(
                        new Split.Equal(List.of(flat.bob, flat.ana)),
                        new Split.Parts(List.of(new MemberParts(flat.bob, Parts.ofHundredths(1)))),
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(1)))),
                        new Split.Amounts(List.of(new MemberAmount(flat.bob, flat.euros(1)))));
        assertEquals(
                List.of(
                        List.of(flat.bob, flat.ana),
                        List.of(flat.bob),
                        List.of(flat.ana),
                        List.of(flat.bob)),
                splits.stream().map(Split::members).toList());
        for (Split split : splits) {
            assertEquals(split, Conversions.converted(Conversions.generated(split)));
        }
        List<Cost> costs =
                List.of(
                        new Cost.Local(flat.euros(2)),
                        new Cost.Converted(flat.euros(1), ExchangeRate.parse("0.2")));
        for (Cost cost : costs) {
            assertEquals(cost, Conversions.converted(Conversions.generated(cost)));
        }
        assertEquals(
                List.of(flat.euros(2), flat.euros(1)), costs.stream().map(Cost::amount).toList());
        for (TipValue value :
                List.<TipValue>of(
                        new TipValue.Percentage(Percentage.ofBasisPoints(1)),
                        new TipValue.Fixed(flat.euros(1)))) {
            assertEquals(value, Conversions.converted(Conversions.generated(value)));
        }
        assertEquals(
                1,
                new ExpenseItem("Bread", flat.euros(1), new Split.Equal(List.of(flat.bob)))
                        .quantity());
        assertEquals(
                TipDistribution.PROPORTIONAL,
                new TipRule(new TipValue.Fixed(flat.euros(1))).distribution());
    }

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group
     * stored another way, would pay or reach another person.
     */
    @Test
    void membersAndTransfersConvertAndStoreWhole() throws EranosException {
        Flat flat = new Flat();
        List<PaymentAccount> accounts =
                List.of(
                        new PaymentAccount(
                                "Main account",
                                new PaymentAccountDetails.Iban(
                                        Iban.parse("RO49AAAA1B31007593840000"))),
                        new PaymentAccount(
                                "Payment app",
                                new PaymentAccountDetails.Username(Username.parse("carol.pays"))));
        Member carol =
                Member.builder("Carol")
                        .lastName("Popa")
                        .email(Email.parse("carol@example.com"))
                        .phone(PhoneNumber.parse("+40712345678"))
                        .accounts(accounts)
                        .build();
        assertEquals(carol, Member.converted(carol.generated()));
        MemberId id = flat.group.addMember(carol);
        assertEquals(carol.id(), id);
        Member stored = flat.group.member(id).orElseThrow();
        assertEquals(id, stored.id());
        assertEquals(carol.lastName(), stored.lastName());
        assertEquals(carol.email(), stored.email());
        assertEquals(carol.phone(), stored.phone());
        assertEquals(carol.accounts(), stored.accounts());
        assertValue(
                stored,
                stored.toBuilder().build(),
                stored.toBuilder().firstName("Caroline").build());
        Member plain = stored.toBuilder().lastName(null).email(null).phone(null).build();
        assertEquals(Optional.empty(), plain.lastName());
        assertEquals(Optional.empty(), plain.email());
        assertEquals(Optional.empty(), plain.phone());
        assertEquals(List.of(), Member.builder("Dan").build().accounts());

        CurrencyCode lei = CurrencyCode.of("RON");
        Transfer cash =
                Transfer.builder(
                                flat.bob,
                                flat.ana,
                                new Cost.Converted(
                                        new Money(1_000, lei), ExchangeRate.parse("0.2")),
                                flat.day)
                        .time(TimeOfDay.of(10, 0))
                        .note("Cash")
                        .build();
        assertEquals(cash, Transfer.converted(cash.generated()));
        TransferId payment = flat.group.addTransfer(cash);
        assertEquals(cash.id(), payment);
        Transfer storedCash = flat.group.transfer(payment).orElseThrow();
        assertEquals(payment, storedCash.id());
        assertEquals(cash.from(), storedCash.from());
        assertEquals(cash.to(), storedCash.to());
        assertEquals(cash.cost(), storedCash.cost());
        assertEquals(cash.date(), storedCash.date());
        assertEquals(Optional.of(TimeOfDay.of(10, 0)), storedCash.time());
        assertEquals(Optional.of("Cash"), storedCash.note());
        assertEquals(Optional.of(flat.euros(200)), flat.group.transferAmount(payment));
        assertValue(
                storedCash,
                storedCash.toBuilder().build(),
                storedCash.toBuilder().note(null).build());
        Transfer back =
                storedCash.toBuilder()
                        .from(flat.ana)
                        .to(flat.bob)
                        .cost(new Cost.Local(flat.euros(5)))
                        .date(CalendarDate.of(2026, 9, 24))
                        .time(null)
                        .build();
        assertEquals(flat.ana, back.from());
        assertEquals(flat.bob, back.to());

        Category travel = new Category("Travel");
        assertEquals(travel, Category.converted(travel.generated()));
        CategoryId trip = flat.group.addCategory(travel);
        assertEquals(travel.id(), trip);
        Category storedTravel = flat.group.category(trip).orElseThrow();
        assertEquals(trip, storedTravel.id());
        assertValue(storedTravel, storedTravel.renamed("Travel"), storedTravel.renamed("Trips"));

        EranosException.ExpenseGroup again =
                assertThrows(
                        EranosException.ExpenseGroup.class, () -> flat.group.addMember(stored));
        assertEquals(new ExpenseGroupFailure.DuplicateMember(id), again.failure());
        for (PaymentAccount account : accounts) {
            assertEquals(
                    account.details(),
                    Conversions.converted(Conversions.generated(account.details())));
        }
    }

    /**
     * An expense, a member, or a transfer whose description, last name, or note is absent or blank
     * that read back with text, added or as a replacement, a replacement without it that kept the
     * stored text, or text that kept stray whitespace, would show an app text nobody gave.
     */
    @Test
    void absentOrBlankTextReadsBackAbsent() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        Expense lunch =
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).toBuilder()
                        .description(" A birthday\n")
                        .build();
        Member carol = Member.builder("Carol").lastName(" Popa ").build();
        Transfer cash =
                Transfer.builder(flat.bob, flat.ana, new Cost.Local(flat.euros(100)), flat.day)
                        .note(" Cash\n")
                        .build();
        group.addExpense(lunch);
        group.addMember(carol);
        group.addTransfer(cash);
        for (String absent : Arrays.asList(null, " ")) {
            group.replaceExpense(lunch);
            group.replaceMember(carol);
            group.replaceTransfer(cash);
            assertEquals(
                    Optional.of("A birthday"),
                    group.expense(lunch.id()).orElseThrow().description());
            assertEquals(Optional.of("Popa"), group.member(carol.id()).orElseThrow().lastName());
            assertEquals(Optional.of("Cash"), group.transfer(cash.id()).orElseThrow().note());

            group.replaceExpense(lunch.toBuilder().description(absent).build());
            group.replaceMember(carol.toBuilder().lastName(absent).build());
            group.replaceTransfer(cash.toBuilder().note(absent).build());
            assertEquals(Optional.empty(), group.expense(lunch.id()).orElseThrow().description());
            assertEquals(Optional.empty(), group.member(carol.id()).orElseThrow().lastName());
            assertEquals(Optional.empty(), group.transfer(cash.id()).orElseThrow().note());

            ExpenseId snack =
                    group.addExpense(
                            flat.expense("Snack", flat.bob, List.of(flat.ana)).toBuilder()
                                    .description(absent)
                                    .build());
            MemberId dan = group.addMember(Member.builder("Dan").lastName(absent).build());
            TransferId change =
                    group.addTransfer(
                            Transfer.builder(
                                            flat.ana,
                                            flat.bob,
                                            new Cost.Local(flat.euros(1)),
                                            flat.day)
                                    .note(absent)
                                    .build());
            assertEquals(Optional.empty(), group.expense(snack).orElseThrow().description());
            assertEquals(Optional.empty(), group.member(dan).orElseThrow().lastName());
            assertEquals(Optional.empty(), group.transfer(change).orElseThrow().note());
        }
    }

    /**
     * An entry made under another identifier than the one the app gave it, two entries made one
     * after the other under one generated identifier, a second entry added under an identifier the
     * group holds, a replacement of an entry the group does not hold, or an identifier the rules
     * refuse let through would leave the app's own store pointing at another entry.
     */
    @Test
    void anEntryKeepsTheIdentifierItIsMadeWith() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;

        MemberId carol = MemberId.parse("carol");
        Member member = Member.builder(carol, "Carol").build();
        assertEquals(carol, group.addMember(member));
        assertEquals(carol, group.member(carol).orElseThrow().toBuilder().build().id());
        EranosException.ExpenseGroup memberAgain =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.addMember(Member.builder(carol, "Caroline").build()));
        assertEquals(new ExpenseGroupFailure.DuplicateMember(carol), memberAgain.failure());
        Member generated = Member.builder("Carol").build();
        assertNotEquals(carol, generated.id());
        assertNotEquals(generated.id(), Member.builder("Carol").build().id());
        assertEquals(generated.id(), group.addMember(generated));
        MemberId dan = MemberId.parse("dan");
        Member unknownMember = member.toBuilder().id(dan).build();
        assertEquals(dan, unknownMember.id());
        EranosException.ExpenseGroup memberUnknown =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.replaceMember(unknownMember));
        assertEquals(new ExpenseGroupFailure.UnknownMember(dan), memberUnknown.failure());

        CategoryId travel = CategoryId.parse("travel");
        assertEquals(travel, group.addCategory(new Category(travel, "Travel")));
        assertEquals(travel, group.category(travel).orElseThrow().id());
        EranosException.ExpenseGroup categoryAgain =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.addCategory(new Category(travel, "Trips")));
        assertEquals(new ExpenseGroupFailure.DuplicateCategory(travel), categoryAgain.failure());
        assertNotEquals(new Category("Trips").id(), new Category("Trips").id());
        CategoryId trips = CategoryId.parse("trips");
        EranosException.ExpenseGroup categoryUnknown =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.replaceCategory(new Category(trips, "Trips")));
        assertEquals(new ExpenseGroupFailure.UnknownCategory(trips), categoryUnknown.failure());

        ExpenseId lunch = ExpenseId.parse("lunch");
        Expense expense =
                Expense.builder(
                                lunch,
                                "Lunch",
                                new Cost.Local(flat.euros(3_000)),
                                new Split.Equal(List.of(flat.ana)),
                                new ExpenseSplit.Whole(new Split.Equal(List.of(flat.bob))),
                                flat.day)
                        .build();
        assertEquals(lunch, group.addExpense(expense));
        assertEquals(lunch, group.expense(lunch).orElseThrow().id());
        EranosException.ExpenseGroup expenseAgain =
                assertThrows(EranosException.ExpenseGroup.class, () -> group.addExpense(expense));
        assertEquals(new ExpenseGroupFailure.DuplicateExpense(lunch), expenseAgain.failure());
        assertNotEquals(
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).id(),
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).id());
        ExpenseId brunch = ExpenseId.parse("brunch");
        Expense unknownExpense = expense.toBuilder().id(brunch).build();
        assertEquals(brunch, unknownExpense.id());
        EranosException.ExpenseGroup expenseUnknown =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.replaceExpense(unknownExpense));
        assertEquals(new ExpenseGroupFailure.UnknownExpense(brunch), expenseUnknown.failure());

        TransferId cash = TransferId.parse("cash");
        Cost hundred = new Cost.Local(flat.euros(100));
        Transfer transfer = Transfer.builder(cash, flat.bob, flat.ana, hundred, flat.day).build();
        assertEquals(cash, group.addTransfer(transfer));
        assertEquals(cash, group.transfer(cash).orElseThrow().id());
        EranosException.ExpenseGroup transferAgain =
                assertThrows(EranosException.ExpenseGroup.class, () -> group.addTransfer(transfer));
        assertEquals(new ExpenseGroupFailure.DuplicateTransfer(cash), transferAgain.failure());
        assertNotEquals(
                Transfer.builder(flat.bob, flat.ana, hundred, flat.day).build().id(),
                Transfer.builder(flat.bob, flat.ana, hundred, flat.day).build().id());
        TransferId card = TransferId.parse("card");
        Transfer unknownTransfer = transfer.toBuilder().id(card).build();
        assertEquals(card, unknownTransfer.id());
        EranosException.ExpenseGroup transferUnknown =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> group.replaceTransfer(unknownTransfer));
        assertEquals(new ExpenseGroupFailure.UnknownTransfer(card), transferUnknown.failure());

        String json = group.save();
        assertEquals(json, ExpenseGroup.load(json).save());
        EranosException.Id refused =
                assertThrows(
                        EranosException.Id.class,
                        () -> group.addMember(Member.builder(new MemberId(" x"), "Dan").build()));
        assertEquals(new IdFailure.SurroundingWhitespace(), refused.failure());
    }

    /**
     * A null identifier let through to the generated code would fail there, far from the call that
     * passed it, instead of where the program made the mistake.
     */
    @Test
    void aNullIdentifierIsRefusedWhereItIsGiven() throws EranosException {
        Flat flat = new Flat();
        Cost cost = new Cost.Local(flat.euros(100));
        Split payers = new Split.Equal(List.of(flat.ana));
        ExpenseSplit split = new ExpenseSplit.Whole(payers);
        for (Executable call :
                List.<Executable>of(
                        () -> Member.builder(null, "Ana"),
                        () -> Member.builder("Ana").id(null),
                        () -> new Category(null, "Food"),
                        () -> Expense.builder(null, "Lunch", cost, payers, split, flat.day),
                        () -> Expense.builder("Lunch", cost, payers, split, flat.day).id(null),
                        () -> Transfer.builder(null, flat.bob, flat.ana, cost, flat.day),
                        () -> Transfer.builder(flat.bob, flat.ana, cost, flat.day).id(null),
                        () -> ExpenseGroup.builder(null, "Flat", flat.euro),
                        () -> ExpenseGroup.builder("Flat", flat.euro).id(null))) {
            NullPointerException refused = assertThrows(NullPointerException.class, call);
            assertEquals("id", refused.getMessage());
        }
    }

    /**
     * An entry whose lists the app changed after building it would change behind the app's back.
     */
    @Test
    void anEntryKeepsTheListsItWasBuiltWith() throws EranosException {
        Flat flat = new Flat();
        List<MemberId> members = new ArrayList<>(List.of(flat.ana));
        Split split = new Split.Equal(members);
        List<PaymentAccount> accounts =
                new ArrayList<>(
                        List.of(
                                new PaymentAccount(
                                        "Payment app",
                                        new PaymentAccountDetails.Username(
                                                Username.parse("ana.pays")))));
        Member member = Member.builder("Ana").accounts(accounts).build();
        members.add(flat.bob);
        accounts.clear();
        assertEquals(List.of(flat.ana), split.members());
        assertEquals(1, member.accounts().size());
        assertTrue(member.toString().contains("Payment app"));
    }
}
