// The entries. An entry that converted to the generated code and back with a field changed or
// dropped, or read back from the group another way than it was added, would store another expense,
// member, or transfer than the app built.

package eranos;

import static eranos.Flat.assertValue;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/** The expenses, members, categories, and transfers, and the splits, costs, and tips they hold. */
class EntriesTest {
    /**
     * A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally: with {@link
     * #dinner}, every kind of cost, split, and tip.
     */
    private static Expense taxi(Flat flat) throws EranosException {
        Currency lei = Currency.of("RON");
        return Expense.builder(
                        new Cost.Converted(new Money(lei, 5_000), ExchangeRate.parse("0.2")),
                        flat.day,
                        "Taxi",
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(10_000)))),
                        new ExpenseSplit.Itemised(
                                List.of(
                                        new Item(
                                                "Fare",
                                                new Split.Parts(
                                                        List.of(
                                                                new MemberParts(
                                                                        flat.ana,
                                                                        Parts.ofHundredths(100)),
                                                                new MemberParts(
                                                                        flat.bob,
                                                                        Parts.ofHundredths(50)))),
                                                new Money(lei, 2_500),
                                                2))))
                .category(flat.food)
                .description("To the airport")
                .time(TimeOfDay.of(9, 5))
                .tip(new Tip(new TipValue.Fixed(new Money(lei, 500)), TipDistribution.EQUAL))
                .build();
    }

    /**
     * A dinner in euros, split equally, paid by amounts, with a percentage tip: with {@link #taxi},
     * every kind of cost, split, and tip.
     */
    private static Expense dinner(Flat flat) throws EranosException {
        return Expense.builder(
                        new Cost.Local(flat.euros(3_000)),
                        flat.day,
                        "Dinner",
                        new Split.Amounts(List.of(new MemberAmount(flat.euros(3_300), flat.ana))),
                        new ExpenseSplit.Whole(new Split.Equal(List.of(flat.ana, flat.bob))))
                .tip(new Tip(new TipValue.Percentage(Percentage.ofBasisPoints(1_000))))
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
            Expense stored = flat.group.expense(id).orElseThrow();
            assertEquals(Optional.of(id), stored.id());
            assertEquals(expense.cost(), stored.cost());
            assertEquals(expense.date(), stored.date());
            assertEquals(expense.name(), stored.name());
            assertEquals(expense.payers(), stored.payers());
            assertEquals(expense.split(), stored.split());
            assertEquals(expense.category(), stored.category());
            assertEquals(expense.description(), stored.description());
            assertEquals(expense.time(), stored.time());
            assertEquals(expense.tip(), stored.tip());
            assertValue(
                    stored, stored.toBuilder().build(), stored.toBuilder().name("Lunch").build());
        }
        Expense plain = flat.expense("Lunch", flat.ana, List.of(flat.bob));
        Expense edited =
                plain.toBuilder()
                        .cost(new Cost.Local(flat.euros(1)))
                        .date(CalendarDate.of(24, 9, 2026))
                        .payers(new Split.Equal(List.of(flat.bob)))
                        .split(new ExpenseSplit.Whole(new Split.Equal(List.of(flat.ana))))
                        .build();
        assertEquals(new Cost.Local(flat.euros(1)), edited.cost());
        assertEquals(CalendarDate.of(24, 9, 2026), edited.date());
        assertEquals(Optional.empty(), plain.toBuilder().category(null).build().category());
    }

    /**
     * The tip, total, and members an app shows for an expense would differ from those the group
     * records when it adds it.
     */
    @Test
    void anExpenseTellsItsTipTotalAndMembers() throws EranosException {
        Flat flat = new Flat();
        Currency lei = Currency.of("RON");
        Expense taxi = taxi(flat);
        assertEquals(new Money(lei, 500), taxi.tipAmount());
        assertEquals(new Money(lei, 5_500), taxi.total());
        assertEquals(List.of(flat.ana, flat.bob), taxi.members());
        assertEquals(flat.euros(300), dinner(flat).tipAmount());
        Expense unreadable =
                taxi.toBuilder().payers(new Split.Equal(List.of(new MemberId("nobody")))).build();
        assertThrows(EranosException.Id.class, unreadable::members);
        assertThrows(
                EranosException.Expense.class,
                () ->
                        taxi.toBuilder()
                                .tip(new Tip(new TipValue.Fixed(flat.euros(1))))
                                .build()
                                .tipAmount());
        assertThrows(
                EranosException.Expense.class,
                () ->
                        taxi.toBuilder()
                                .tip(new Tip(new TipValue.Fixed(new Money(lei, -1))))
                                .build()
                                .total());
        assertEquals(
                List.of(TipDistribution.EQUAL, TipDistribution.PROPORTIONAL),
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
                        new Split.Amounts(List.of(new MemberAmount(flat.euros(1), flat.bob))),
                        new Split.Equal(List.of(flat.bob, flat.ana)),
                        new Split.Parts(List.of(new MemberParts(flat.bob, Parts.ofHundredths(1)))),
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(1)))));
        assertEquals(
                List.of(
                        List.of(flat.bob),
                        List.of(flat.bob, flat.ana),
                        List.of(flat.bob),
                        List.of(flat.ana)),
                splits.stream().map(Split::members).toList());
        for (Split split : splits) {
            assertEquals(split, Conversions.converted(Conversions.generated(split)));
        }
        List<Cost> costs =
                List.of(
                        new Cost.Converted(flat.euros(1), ExchangeRate.parse("0.2")),
                        new Cost.Local(flat.euros(2)));
        for (Cost cost : costs) {
            assertEquals(cost, Conversions.converted(Conversions.generated(cost)));
        }
        assertEquals(
                List.of(flat.euros(1), flat.euros(2)), costs.stream().map(Cost::amount).toList());
        for (TipValue value :
                List.<TipValue>of(
                        new TipValue.Fixed(flat.euros(1)),
                        new TipValue.Percentage(Percentage.ofBasisPoints(1)))) {
            assertEquals(value, Conversions.converted(Conversions.generated(value)));
        }
        assertEquals(
                1, new Item("Bread", new Split.Equal(List.of(flat.bob)), flat.euros(1)).quantity());
        assertEquals(
                TipDistribution.PROPORTIONAL,
                new Tip(new TipValue.Fixed(flat.euros(1))).distribution());
    }

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group
     * stored another way, would pay or reach another person.
     */
    @Test
    void membersAndTransfersConvertAndStoreWhole() throws EranosException {
        Flat flat = new Flat();
        List<Account> accounts =
                List.of(
                        new Account(
                                new AccountDetails.Iban(Iban.parse("RO49AAAA1B31007593840000")),
                                "Main account"),
                        new Account(
                                new AccountDetails.Username(Username.parse("carol.pays")),
                                "Payment app"));
        Member carol =
                Member.builder("Carol")
                        .accounts(accounts)
                        .email(Email.parse("carol@example.com"))
                        .lastName("Popa")
                        .phone(PhoneNumber.parse("+40712345678"))
                        .build();
        assertEquals(carol, Member.converted(carol.generated()));
        MemberId id = flat.group.addMember(carol);
        Member stored = flat.group.member(id).orElseThrow();
        assertEquals(Optional.of(id), stored.id());
        assertEquals(carol.accounts(), stored.accounts());
        assertEquals(carol.email(), stored.email());
        assertEquals(carol.lastName(), stored.lastName());
        assertEquals(carol.phone(), stored.phone());
        assertValue(
                stored,
                stored.toBuilder().build(),
                stored.toBuilder().firstName("Caroline").build());
        Member plain = stored.toBuilder().email(null).lastName(null).phone(null).build();
        assertEquals(Optional.empty(), plain.email());
        assertEquals(Optional.empty(), plain.lastName());
        assertEquals(Optional.empty(), plain.phone());
        assertEquals(List.of(), Member.builder("Dan").build().accounts());

        Currency lei = Currency.of("RON");
        Transfer cash =
                Transfer.builder(
                                new Cost.Converted(
                                        new Money(lei, 1_000), ExchangeRate.parse("0.2")),
                                flat.day,
                                flat.bob,
                                flat.ana)
                        .note("Cash")
                        .time(TimeOfDay.of(10, 0))
                        .build();
        assertEquals(cash, Transfer.converted(cash.generated()));
        TransferId payment = flat.group.addTransfer(cash);
        Transfer storedCash = flat.group.transfer(payment).orElseThrow();
        assertEquals(Optional.of(payment), storedCash.id());
        assertEquals(cash.cost(), storedCash.cost());
        assertEquals(cash.date(), storedCash.date());
        assertEquals(cash.from(), storedCash.from());
        assertEquals(cash.to(), storedCash.to());
        assertEquals(Optional.of("Cash"), storedCash.note());
        assertEquals(Optional.of(TimeOfDay.of(10, 0)), storedCash.time());
        assertEquals(Optional.of(flat.euros(200)), flat.group.transferAmount(payment));
        assertValue(
                storedCash,
                storedCash.toBuilder().build(),
                storedCash.toBuilder().note(null).build());
        Transfer back =
                storedCash.toBuilder()
                        .cost(new Cost.Local(flat.euros(5)))
                        .date(CalendarDate.of(24, 9, 2026))
                        .from(flat.ana)
                        .to(flat.bob)
                        .time(null)
                        .build();
        assertEquals(flat.ana, back.from());
        assertEquals(flat.bob, back.to());

        Category travel = new Category("Travel");
        assertEquals(travel, Category.converted(travel.generated()));
        CategoryId trip = flat.group.addCategory(travel);
        Category storedTravel = flat.group.category(trip).orElseThrow();
        assertEquals(Optional.of(trip), storedTravel.id());
        assertValue(storedTravel, storedTravel.renamed("Travel"), storedTravel.renamed("Trips"));

        IllegalArgumentException given =
                assertThrows(IllegalArgumentException.class, () -> flat.group.addMember(stored));
        assertEquals("an entry to add must not carry an identifier", given.getMessage());
        IllegalArgumentException missing =
                assertThrows(IllegalArgumentException.class, () -> flat.group.replaceMember(carol));
        assertEquals("an entry to replace must carry its identifier", missing.getMessage());
        for (Account account : accounts) {
            assertEquals(
                    account.details(),
                    Conversions.converted(Conversions.generated(account.details())));
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
        List<Account> accounts =
                new ArrayList<>(
                        List.of(
                                new Account(
                                        new AccountDetails.Username(Username.parse("ana.pays")),
                                        "Payment app")));
        Member member = Member.builder("Ana").accounts(accounts).build();
        members.add(flat.bob);
        accounts.clear();
        assertEquals(List.of(flat.ana), split.members());
        assertEquals(1, member.accounts().size());
        assertTrue(member.toString().contains("Payment app"));
    }
}
