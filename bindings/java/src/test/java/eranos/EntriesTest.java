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

/**
 * The expenses, members, categories, and transfers, and the splits, costs, items, discounts, and
 * tips they hold.
 */
class EntriesTest {
    /** A line's totals: before its discount, the discount's amount, and the line total. */
    private record LineTotals(Money beforeDiscount, Money discount, Money lineTotal) {}

    /** A discount and the failure the library refuses it with. */
    private record Refusal(Discount discount, ExpenseFailure failure) {}

    /** The totals of {@code item}, as the library computes them. */
    private static LineTotals lineTotals(ExpenseItem item) throws EranosException {
        return new LineTotals(
                item.lineTotalBeforeDiscount(), item.discountAmount(), item.lineTotal());
    }

    /**
     * A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally: with {@link
     * #dinner}, every kind of cost, split, and tip.
     */
    private static Expense taxi(Flat flat) throws EranosException {
        CurrencyCode lei = CurrencyCode.of("RON");
        ExpenseItem fare =
                ExpenseItem.builder(
                                "Fare",
                                new Money(2_500, lei),
                                new Split.Parts(
                                        List.of(
                                                new MemberParts(flat.ana, Parts.ofHundredths(100)),
                                                new MemberParts(flat.bob, Parts.ofHundredths(50)))))
                        .quantity(Quantity.ofWhole(2))
                        .build();
        return Expense.builder(
                        "Taxi",
                        new Cost.Converted(new Money(5_000, lei), ExchangeRate.parse("0.2")),
                        new Split.Percentages(
                                List.of(
                                        new MemberPercentage(
                                                flat.ana, Percentage.ofBasisPoints(10_000)))),
                        new ExpenseSplit.Itemised(List.of(fare)),
                        flat.day)
                .tip(new TipRule(new TipValue.Fixed(new Money(500, lei)), TipDistribution.EQUAL))
                .time(TimeOfDay.of(9, 5))
                .category(flat.food)
                .note("To the airport")
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
            assertEquals(expense.note(), stored.note());
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
        assertEquals(
                List.of(flat.ana, flat.bob),
                new ExpenseSplit.Whole(new Split.Equal(List.of(flat.bob, flat.ana, flat.bob)))
                        .participants());
        ExpenseSplit unknown =
                new ExpenseSplit.Whole(new Split.Equal(List.of(new MemberId(" nobody"))));
        assertThrows(EranosException.Id.class, unknown::participants);
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
     * The line total an app shows for an item would differ from the one the library adds up against
     * the expense's amount: rounded another way, multiplied as a double, or refused for a free
     * line; an item the library refuses would show a line total; an item that converted or stored
     * with another quantity would divide another line; a builder that changed another field than
     * the one it was given would build another line; and an item that took {@code null} for a field
     * would fail only when its expense reached the group.
     */
    @Test
    void anItemTellsItsLineTotalAsTheLibraryComputesIt() throws EranosException {
        Flat flat = new Flat();
        Split ana = new Split.Equal(List.of(flat.ana));
        // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
        ExpenseItem olives =
                ExpenseItem.builder("Olives", flat.euros(100), ana)
                        .quantity(Quantity.parse("0.335"))
                        .build();
        assertEquals(flat.euros(34), olives.lineTotal());
        assertEquals(
                flat.euros(1_470),
                ExpenseItem.builder("Cheese", flat.euros(4_200), ana)
                        .quantity(Quantity.parse("0.35"))
                        .build()
                        .lineTotal());
        assertEquals(
                flat.euros(200),
                ExpenseItem.builder("Bread", flat.euros(100), ana)
                        .quantity(Quantity.ofWhole(2))
                        .build()
                        .lineTotal());
        // A ten-thousandth of a cent rounds to zero: a free line.
        ExpenseItem free =
                ExpenseItem.builder("Olives", flat.euros(100), ana)
                        .quantity(Quantity.ofTenThousandths(1))
                        .build();
        assertEquals(flat.euros(0), free.lineTotal());
        for (ExpenseItem item : List.of(olives, free)) {
            assertEquals(item, ExpenseItem.converted(item.generated()));
        }
        Expense weighed =
                Expense.builder(
                                "Olives",
                                new Cost.Local(flat.euros(34)),
                                ana,
                                new ExpenseSplit.Itemised(List.of(olives, free)),
                                flat.day)
                        .build();
        assertEquals(weighed, flat.group.expense(flat.group.addExpense(weighed)).orElseThrow());

        EranosException.Expense unpaid =
                assertThrows(
                        EranosException.Expense.class,
                        () ->
                                ExpenseItem.builder(
                                                "Olives",
                                                flat.euros(100),
                                                new Split.Amounts(
                                                        List.of(
                                                                new MemberAmount(
                                                                        flat.ana, flat.euros(1)))))
                                        .quantity(Quantity.ofTenThousandths(1))
                                        .build()
                                        .lineTotal());
        assertEquals(
                new ExpenseFailure.AmountsDoNotAddUp(flat.euros(0), flat.euros(1)),
                unpaid.failure());
        EranosException.Expense overflow =
                assertThrows(
                        EranosException.Expense.class,
                        () ->
                                ExpenseItem.builder("Olives", flat.euros(Long.MAX_VALUE), ana)
                                        .quantity(Quantity.ofWhole(2))
                                        .build()
                                        .lineTotal());
        assertEquals(new ExpenseFailure.Overflow(), overflow.failure());
        EranosException.Expense unnamed =
                assertThrows(
                        EranosException.Expense.class,
                        () -> ExpenseItem.builder(" ", flat.euros(100), ana).build().lineTotal());
        assertEquals(new ExpenseFailure.EmptyItemName(), unnamed.failure());
        ExpenseItem.Builder builder = olives.toBuilder();
        assertThrows(NullPointerException.class, () -> builder.name(null));
        assertThrows(NullPointerException.class, () -> builder.unitPrice(null));
        assertThrows(NullPointerException.class, () -> builder.split(null));
        assertThrows(NullPointerException.class, () -> builder.quantity(null));
        assertEquals(olives, builder.build());
        Split bob = new Split.Equal(List.of(flat.bob));
        ExpenseItem cheese = builder.name("Cheese").unitPrice(flat.euros(4_200)).split(bob).build();
        assertEquals("Cheese", cheese.name());
        assertEquals(flat.euros(4_200), cheese.unitPrice());
        assertEquals(bob, cheese.split());
        assertEquals(olives.quantity(), cheese.quantity());
    }

    /**
     * A discount an app showed another way than the library takes it would show a line total that
     * the library does not add up against the expense's amount: off each unit, 50 % of 3.33 rounds
     * to 1.67 before it is multiplied, so three units cost 4.98, and off the line, 50 % of 9.99
     * rounds once, to 5.00, so they cost 4.99; a fixed discount taken once instead of off each
     * unit, a line without a discount that showed one, or a discount of the whole price refused
     * instead of making a free line would charge another amount.
     */
    @Test
    void aDiscountComesOffTheLineOrEachUnitAsTheLibraryTakesIt() throws EranosException {
        Flat flat = new Flat();
        Split ana = new Split.Equal(List.of(flat.ana));
        ExpenseItem.Builder soups =
                ExpenseItem.builder("Soup", flat.euros(333), ana).quantity(Quantity.ofWhole(3));
        DiscountValue half = new DiscountValue.Percentage(Percentage.ofBasisPoints(5_000));
        assertEquals(
                new LineTotals(flat.euros(999), flat.euros(501), flat.euros(498)),
                lineTotals(soups.discount(new Discount(half, DiscountScope.UNIT)).build()));
        assertEquals(
                new LineTotals(flat.euros(999), flat.euros(500), flat.euros(499)),
                lineTotals(soups.discount(new Discount(half)).build()));
        ExpenseItem.Builder lemonades =
                ExpenseItem.builder("Lemonade", flat.euros(300), ana).quantity(Quantity.ofWhole(3));
        DiscountValue oneEuro = new DiscountValue.Fixed(flat.euros(100));
        assertEquals(
                new LineTotals(flat.euros(900), flat.euros(300), flat.euros(600)),
                lineTotals(lemonades.discount(new Discount(oneEuro, DiscountScope.UNIT)).build()));
        assertEquals(
                new LineTotals(flat.euros(900), flat.euros(100), flat.euros(800)),
                lineTotals(lemonades.discount(new Discount(oneEuro)).build()));
        assertEquals(
                new LineTotals(flat.euros(900), flat.euros(0), flat.euros(900)),
                lineTotals(lemonades.discount(null).build()));
        // The whole price, as a percentage or as a fixed amount, makes a free line.
        ExpenseItem.Builder dessert = ExpenseItem.builder("Papanași", flat.euros(900), ana);
        for (DiscountValue whole :
                List.of(
                        new DiscountValue.Percentage(Percentage.ofBasisPoints(10_000)),
                        new DiscountValue.Fixed(flat.euros(900)))) {
            assertEquals(
                    new LineTotals(flat.euros(900), flat.euros(900), flat.euros(0)),
                    lineTotals(dessert.discount(new Discount(whole)).build()));
        }
    }

    /**
     * A discount the library refuses that an item read totals for, or refused as another failure,
     * would let an app show a line that the group will not take: a negative amount, a percentage
     * above 100 %, an amount above the price it reduces, off the line or off each unit, and an
     * amount in another currency.
     */
    @Test
    void aDiscountTheLineCannotTakeIsRefused() throws EranosException {
        Flat flat = new Flat();
        CurrencyCode lei = CurrencyCode.of("RON");
        ExpenseItem.Builder lemonades =
                ExpenseItem.builder("Lemonade", flat.euros(300), new Split.Equal(List.of(flat.ana)))
                        .quantity(Quantity.ofWhole(3));
        List<Refusal> refusals =
                List.of(
                        new Refusal(
                                new Discount(new DiscountValue.Fixed(flat.euros(-100))),
                                new ExpenseFailure.NegativeDiscount()),
                        new Refusal(
                                new Discount(
                                        new DiscountValue.Percentage(
                                                Percentage.ofBasisPoints(10_001))),
                                new ExpenseFailure.DiscountAboveHundredPercent()),
                        new Refusal(
                                new Discount(new DiscountValue.Fixed(flat.euros(901))),
                                new ExpenseFailure.DiscountAbovePrice()),
                        new Refusal(
                                new Discount(
                                        new DiscountValue.Fixed(flat.euros(301)),
                                        DiscountScope.UNIT),
                                new ExpenseFailure.DiscountAbovePrice()),
                        new Refusal(
                                new Discount(new DiscountValue.Fixed(new Money(100, lei))),
                                new ExpenseFailure.CurrencyMismatch(flat.euro, lei)));
        for (Refusal refusal : refusals) {
            ExpenseItem item = lemonades.discount(refusal.discount()).build();
            for (Executable total :
                    List.<Executable>of(
                            item::lineTotalBeforeDiscount, item::discountAmount, item::lineTotal)) {
                EranosException.Expense refused =
                        assertThrows(EranosException.Expense.class, total);
                assertEquals(refusal.failure(), refused.failure());
            }
        }
    }

    /**
     * A discount that converted with its value or scope changed, a builder that dropped it or could
     * not clear it, or a group that stored it another way would take another amount off the line
     * than the app set; a discount that took {@code null} would fail only when its expense reached
     * the group; and a free line that divided to anything but zeros would charge its members for
     * what they did not pay.
     */
    @Test
    void aDiscountConvertsAndStoresWhole() throws EranosException {
        Flat flat = new Flat();
        assertEquals(
                List.of(DiscountScope.LINE, DiscountScope.UNIT), List.of(DiscountScope.values()));
        for (DiscountScope scope : DiscountScope.values()) {
            assertEquals(scope, DiscountScope.converted(scope.generated()));
        }
        DiscountValue tenPerCent = new DiscountValue.Percentage(Percentage.ofBasisPoints(1_000));
        DiscountValue oneEuro = new DiscountValue.Fixed(flat.euros(100));
        for (DiscountValue value : List.of(tenPerCent, oneEuro)) {
            assertEquals(value, Conversions.converted(Conversions.generated(value)));
            assertEquals(new Discount(value, DiscountScope.LINE), new Discount(value));
            for (DiscountScope scope : DiscountScope.values()) {
                Discount discount = new Discount(value, scope);
                assertEquals(discount, Discount.converted(discount.generated()));
            }
        }
        assertThrows(NullPointerException.class, () -> new Discount(null));
        assertThrows(NullPointerException.class, () -> new Discount(oneEuro, null));
        assertThrows(NullPointerException.class, () -> new DiscountValue.Percentage(null));
        assertThrows(NullPointerException.class, () -> new DiscountValue.Fixed(null));

        Discount offEach = new Discount(oneEuro, DiscountScope.UNIT);
        ExpenseItem lemonades =
                ExpenseItem.builder(
                                "Lemonade",
                                flat.euros(300),
                                new Split.Equal(List.of(flat.ana, flat.bob)))
                        .quantity(Quantity.ofWhole(3))
                        .discount(offEach)
                        .build();
        assertEquals(Optional.of(offEach), lemonades.discount());
        ExpenseItem undiscounted = lemonades.toBuilder().discount(null).build();
        assertEquals(Optional.empty(), undiscounted.discount());
        assertValue(lemonades, lemonades.toBuilder().build(), undiscounted);
        assertEquals(lemonades, ExpenseItem.converted(lemonades.generated()));
        ExpenseItem wine =
                ExpenseItem.builder("Wine", flat.euros(1_000), new Split.Equal(List.of(flat.ana)))
                        .discount(new Discount(tenPerCent))
                        .build();
        ExpenseItem dessert =
                ExpenseItem.builder("Papanași", flat.euros(900), new Split.Equal(List.of(flat.bob)))
                        .discount(
                                new Discount(
                                        new DiscountValue.Percentage(
                                                Percentage.ofBasisPoints(10_000))))
                        .build();
        // 6.00 for the lemonades, 9.00 for the wine, and nothing for the dessert.
        Expense drinks =
                Expense.builder(
                                "Drinks",
                                new Cost.Local(flat.euros(1_500)),
                                new Split.Equal(List.of(flat.ana)),
                                new ExpenseSplit.Itemised(List.of(lemonades, wine, dessert)),
                                flat.day)
                        .build();
        ExpenseId id = flat.group.addExpense(drinks);
        assertEquals(drinks, flat.group.expense(id).orElseThrow());
        assertEquals(
                List.of(
                        new MemberShare(flat.ana, flat.euros(1_500), flat.euros(1_200)),
                        new MemberShare(flat.bob, flat.euros(0), flat.euros(300))),
                flat.group.expenseShares(id).orElseThrow().members());
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
        ExpenseItem bread =
                ExpenseItem.builder("Bread", flat.euros(1), new Split.Equal(List.of(flat.bob)))
                        .build();
        assertEquals(Quantity.ONE, bread.quantity());
        assertEquals(Optional.empty(), bread.discount());
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
        assertEquals(cash.sender(), storedCash.sender());
        assertEquals(cash.receiver(), storedCash.receiver());
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
                        .sender(flat.ana)
                        .receiver(flat.bob)
                        .cost(new Cost.Local(flat.euros(5)))
                        .date(CalendarDate.of(2026, 9, 24))
                        .time(null)
                        .build();
        assertEquals(flat.ana, back.sender());
        assertEquals(flat.bob, back.receiver());

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
     * An expense, a member, or a transfer whose note, last name, or note is absent or blank that
     * read back with text, added or as a replacement, a replacement without it that kept the stored
     * text, or text that kept stray whitespace, would show an app text nobody gave.
     */
    @Test
    void absentOrBlankTextReadsBackAbsent() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        Expense lunch =
                flat.expense("Lunch", flat.ana, List.of(flat.bob)).toBuilder()
                        .note(" A birthday\n")
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
            assertEquals(Optional.of("A birthday"), group.expense(lunch.id()).orElseThrow().note());
            assertEquals(Optional.of("Popa"), group.member(carol.id()).orElseThrow().lastName());
            assertEquals(Optional.of("Cash"), group.transfer(cash.id()).orElseThrow().note());

            group.replaceExpense(lunch.toBuilder().note(absent).build());
            group.replaceMember(carol.toBuilder().lastName(absent).build());
            group.replaceTransfer(cash.toBuilder().note(absent).build());
            assertEquals(Optional.empty(), group.expense(lunch.id()).orElseThrow().note());
            assertEquals(Optional.empty(), group.member(carol.id()).orElseThrow().lastName());
            assertEquals(Optional.empty(), group.transfer(cash.id()).orElseThrow().note());

            ExpenseId snack =
                    group.addExpense(
                            flat.expense("Snack", flat.bob, List.of(flat.ana)).toBuilder()
                                    .note(absent)
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
            assertEquals(Optional.empty(), group.expense(snack).orElseThrow().note());
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
