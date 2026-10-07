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
                        "Taxi",
                        new Cost.Converted(new Money(5_000, lei), ExchangeRate.parse("0.2")),
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(10_000)))),
                        new ExpenseSplit.Itemised(
                                List.of(
                                        new Item(
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
                .tip(new Tip(new TipValue.Fixed(new Money(500, lei)), TipDistribution.EQUAL))
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
        assertEquals(new Money(500, lei), taxi.tipAmount());
        assertEquals(new Money(5_500, lei), taxi.total());
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
                                .tip(new Tip(new TipValue.Fixed(new Money(-1, lei))))
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
                1, new Item("Bread", flat.euros(1), new Split.Equal(List.of(flat.bob))).quantity());
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
        Member stored = flat.group.member(id).orElseThrow();
        assertEquals(Optional.of(id), stored.id());
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

        Currency lei = Currency.of("RON");
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
        Transfer storedCash = flat.group.transfer(payment).orElseThrow();
        assertEquals(Optional.of(payment), storedCash.id());
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
        Category storedTravel = flat.group.category(trip).orElseThrow();
        assertEquals(Optional.of(trip), storedTravel.id());
        assertValue(storedTravel, storedTravel.renamed("Travel"), storedTravel.renamed("Trips"));

        IllegalArgumentException given =
                assertThrows(IllegalArgumentException.class, () -> flat.group.addMember(stored));
        assertEquals("an entry to add must not carry an identifier", given.getMessage());
        IllegalArgumentException missing =
                assertThrows(IllegalArgumentException.class, () -> flat.group.replaceMember(carol));
        assertEquals("an entry to replace must carry its identifier", missing.getMessage());
        for (PaymentAccount account : accounts) {
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
