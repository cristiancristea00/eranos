// The entries. An entry that converted to the generated code and back with a field changed or dropped, or read back from the
// group another way than it was added, would store another expense, member, or transfer than the app built.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The expenses, members, categories, and transfers, and the splits, costs, and tips they hold. */
class EntriesTest {
    /**
     * A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally, and a dinner in euros, split equally, paid by
     * amounts, with a percentage tip: between them, every kind of cost, split, and tip.
     */
    private fun expenses(flat: Flat): Expenses {
        val lei = Currency.of("RON")
        val taxi =
            Expense(
                cost = Cost.Converted(Money(lei, 5_000), ExchangeRate.parse("0.2")),
                date = flat.day,
                name = "Taxi",
                payers = Split.Percentages(listOf(MemberPercentage(flat.ana, Percentage.ofBasisPoints(10_000)))),
                split =
                    ExpenseSplit.Itemised(
                        listOf(
                            Item(
                                name = "Fare",
                                split =
                                    Split.Parts(
                                        listOf(MemberParts(flat.ana, Parts.ofHundredths(100)), MemberParts(flat.bob, Parts.ofHundredths(50))),
                                    ),
                                unitPrice = Money(lei, 2_500),
                                quantity = 2,
                            ),
                        ),
                    ),
                category = flat.food,
                description = "To the airport",
                time = TimeOfDay.of(9, 5),
                tip = Tip(TipValue.Fixed(Money(lei, 500)), TipDistribution.EQUAL),
            )
        val dinner =
            Expense(
                cost = Cost.Local(flat.euros(3_000)),
                date = flat.day,
                name = "Dinner",
                payers = Split.Amounts(listOf(MemberAmount(flat.euros(3_300), flat.ana))),
                split = ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))),
                tip = Tip(TipValue.Percentage(Percentage.ofBasisPoints(1_000))),
            )
        return Expenses(dinner, taxi)
    }

    /**
     * An expense that converted with a field changed or dropped, or that the group stored another way, would divide another
     * expense than the app built.
     */
    @Test
    fun everyKindOfExpenseConvertsAndStoresWhole() {
        val flat = Flat()
        val expenses = expenses(flat)
        for (expense in listOf(expenses.taxi, expenses.dinner)) {
            assertEquals(expense, expense.generated.converted())
            val id = flat.group.addBlocking(expense)
            val stored = present(flat.group.expenseBlocking(id))
            assertEquals(id, stored.id)
            assertEquals(expense, stored.withoutId())
            assertValue(stored, stored.copy(), stored.copy(name = "Lunch"))
        }
    }

    /**
     * The tip, total, and members an app shows for an expense would differ from those the group records when it adds it.
     */
    @Test
    fun anExpenseTellsItsTipTotalAndMembers() {
        val flat = Flat()
        val lei = Currency.of("RON")
        val expenses = expenses(flat)
        assertEquals(Money(lei, 500), expenses.taxi.tipAmount())
        assertEquals(Money(lei, 5_500), expenses.taxi.total())
        assertEquals(listOf(flat.ana, flat.bob), expenses.taxi.members())
        assertEquals(flat.euros(300), expenses.dinner.tipAmount())
        val unreadable = expenses.taxi.copy(payers = Split.Equal(listOf(MemberId("nobody"))))
        assertThrows<EranosException.Id> { unreadable.members() }
        assertThrows<EranosException.Expense> { expenses.taxi.copy(tip = Tip(TipValue.Fixed(flat.euros(1)))).tipAmount() }
        assertThrows<EranosException.Expense> { expenses.taxi.copy(tip = Tip(TipValue.Fixed(Money(lei, -1)))).total() }
        assertEquals(listOf(TipDistribution.EQUAL, TipDistribution.PROPORTIONAL), TipDistribution.entries)
        for (distribution in TipDistribution.entries) {
            assertEquals(distribution, distribution.generated.converted())
        }
    }

    /**
     * A split, a cost, or a tip that compared, hashed, or printed with a field left out would merge two an app holds apart.
     */
    @Test
    fun splitsCostsAndTipsAreValues() {
        val flat = Flat()
        val expenses = expenses(flat)
        val amounts = Split.Amounts(listOf(MemberAmount(flat.euros(1), flat.bob)))
        assertEquals(listOf(flat.bob), amounts.members)
        assertValue(amounts, Split.Amounts(amounts.shares), Split.Amounts(listOf(MemberAmount(flat.euros(2), flat.bob))))
        val equal = Split.Equal(listOf(flat.bob, flat.ana))
        assertEquals(listOf(flat.bob, flat.ana), equal.members)
        assertValue(equal, Split.Equal(equal.members), Split.Equal(listOf(flat.ana)))
        val parts = Split.Parts(listOf(MemberParts(flat.bob, Parts.ofHundredths(1))))
        assertEquals(listOf(flat.bob), parts.members)
        assertValue(parts, Split.Parts(parts.shares), Split.Parts(listOf(MemberParts(flat.ana, Parts.ofHundredths(1)))))
        val percentages = Split.Percentages(listOf(MemberPercentage(flat.ana, Percentage.ofBasisPoints(1))))
        assertEquals(listOf(flat.ana), percentages.members)
        assertValue(
            percentages,
            Split.Percentages(percentages.shares),
            Split.Percentages(listOf(MemberPercentage(flat.ana, Percentage.ofBasisPoints(2)))),
        )
        for (split in listOf<Split>(amounts, equal, parts, percentages)) {
            assertEquals(split, split.generated.converted())
        }
        assertValue(amounts.shares[0], MemberAmount(flat.euros(1), flat.bob), MemberAmount(flat.euros(1), flat.ana))
        assertValue(parts.shares[0], MemberParts(flat.bob, Parts.ofHundredths(1)), MemberParts(flat.bob, Parts.ofHundredths(2)))
        assertValue(
            percentages.shares[0],
            MemberPercentage(flat.ana, Percentage.ofBasisPoints(1)),
            MemberPercentage(flat.bob, Percentage.ofBasisPoints(1)),
        )

        assertEquals(Money(Currency.of("RON"), 5_000), expenses.taxi.cost.amount)
        assertValue(
            expenses.taxi.cost,
            Cost.Converted(expenses.taxi.cost.amount, ExchangeRate.parse("0.2")),
            Cost.Converted(expenses.taxi.cost.amount, ExchangeRate.parse("0.3")),
        )
        assertValue(expenses.dinner.cost, Cost.Local(flat.euros(3_000)), Cost.Local(flat.euros(1)))
        val itemised = expenses.taxi.split as ExpenseSplit.Itemised
        assertValue(itemised, ExpenseSplit.Itemised(itemised.items), ExpenseSplit.Itemised(emptyList()))
        assertValue(expenses.dinner.split, ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))), ExpenseSplit.Whole(equal))
        val item = itemised.items[0]
        assertValue(item, Item(item.name, item.split, item.unitPrice, 2), Item(item.name, item.split, item.unitPrice))
        assertEquals(1, Item("Bread", equal, flat.euros(1)).quantity)
        assertValue(
            present(expenses.taxi.tip),
            Tip(TipValue.Fixed(Money(Currency.of("RON"), 500)), TipDistribution.EQUAL),
            Tip(TipValue.Fixed(Money(Currency.of("RON"), 500))),
        )
        assertEquals(TipDistribution.PROPORTIONAL, present(expenses.dinner.tip).distribution)
        assertValue(TipValue.Fixed(flat.euros(1)), TipValue.Fixed(flat.euros(1)), TipValue.Fixed(flat.euros(2)))
        assertValue(
            TipValue.Percentage(Percentage.ofBasisPoints(1)),
            TipValue.Percentage(Percentage.ofBasisPoints(1)),
            TipValue.Percentage(Percentage.ofBasisPoints(2)),
        )
    }

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group stored another way, would
     * pay or reach another person.
     */
    @Test
    fun membersAndTransfersConvertAndStoreWhole() {
        val flat = Flat()
        val mainAccount = Account(AccountDetails.Iban(Iban.parse("RO49AAAA1B31007593840000")), "Main account")
        val paymentApp = Account(AccountDetails.Username(Username.parse("carol.pays")), "Payment app")
        val carol =
            Member(
                firstName = "Carol",
                accounts = listOf(mainAccount, paymentApp),
                email = Email.parse("carol@example.com"),
                lastName = "Popa",
                phone = PhoneNumber.parse("+40712345678"),
            )
        assertEquals(carol, carol.generated.converted())
        val id = flat.group.addBlocking(carol)
        val stored = present(flat.group.memberBlocking(id))
        assertEquals(id, stored.id)
        assertEquals(carol, Member(stored.firstName, stored.accounts, stored.email, stored.lastName, stored.phone))
        assertValue(stored, stored.copy(), stored.copy(firstName = "Caroline"))
        assertEquals(emptyList<Account>(), Member("Dan").accounts)
        assertValue(mainAccount, Account(mainAccount.details, "Main account"), Account(mainAccount.details, "Savings"))
        assertValue(
            mainAccount.details,
            AccountDetails.Iban(Iban.parse("RO49AAAA1B31007593840000")),
            AccountDetails.Iban(Iban.parse("GB82WEST12345698765432")),
        )
        assertValue(paymentApp.details, AccountDetails.Username(Username.parse("carol.pays")), AccountDetails.Username(Username.parse("dan.pays")))

        val lei = Currency.of("RON")
        val cash =
            Transfer(
                cost = Cost.Converted(Money(lei, 1_000), ExchangeRate.parse("0.2")),
                date = flat.day,
                from = flat.bob,
                to = flat.ana,
                note = "Cash",
                time = TimeOfDay.of(10, 0),
            )
        assertEquals(cash, cash.generated.converted())
        val payment = flat.group.addBlocking(cash)
        val storedCash = present(flat.group.transferBlocking(payment))
        assertEquals(payment, storedCash.id)
        assertEquals("Cash", storedCash.note)
        assertEquals("10:00", storedCash.time.toString())
        assertEquals(flat.euros(200), flat.group.transferAmountBlocking(payment))
        assertValue(storedCash, storedCash.copy(), storedCash.copy(note = null))

        val travel = Category("Travel")
        assertEquals(travel, travel.generated.converted())
        val trip = flat.group.addBlocking(travel)
        val storedTravel = present(flat.group.categoryBlocking(trip))
        assertValue(storedTravel, storedTravel.copy(), storedTravel.copy(name = "Trips"))

        val given = assertThrows<IllegalArgumentException> { flat.group.addBlocking(stored) }
        assertEquals("an entry to add must not carry an identifier", given.message)
        val missing = assertThrows<IllegalArgumentException> { flat.group.replaceBlocking(carol) }
        assertEquals("an entry to replace must carry its identifier", missing.message)
    }

    /** An entry whose lists the app changed after building it would change behind the app's back. */
    @Test
    fun anEntryKeepsTheListsItWasBuiltWith() {
        val flat = Flat()
        val members = mutableListOf(flat.ana)
        val split = Split.Equal(members)
        val accounts = mutableListOf(Account(AccountDetails.Username(Username.parse("ana.pays")), "Payment app"))
        val member = Member("Ana", accounts)
        members.add(flat.bob)
        accounts.clear()
        assertEquals(listOf(flat.ana), split.members)
        assertEquals(1, member.accounts.size)
    }
}

/** A taxi and a dinner: between them, every kind of cost, split, and tip. */
private data class Expenses(
    val dinner: Expense,
    val taxi: Expense,
)

/** This expense as it was before the group gave it an identifier. */
private fun Expense.withoutId(): Expense = Expense(null, cost, date, name, payers, split, category, description, time, tip)
