// The entries. An entry that converted to the generated code and back with a field changed or dropped, or read back from the
// group another way than it was added, would store another expense, member, or transfer than the app built.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** The expenses, members, categories, and transfers, and the splits, costs, and tips they hold. */
class EntriesTest {
    /**
     * A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally, and a dinner in euros, split equally, paid by
     * amounts, with a percentage tip: between them, every kind of cost, split, and tip.
     */
    private fun expenses(flat: Flat): Expenses {
        val lei = CurrencyCode.of("RON")
        val taxi =
            Expense(
                name = "Taxi",
                cost = Cost.Converted(Money(5_000, lei), ExchangeRate.parse("0.2")),
                payers = Split.Percentages(listOf(MemberPercentage(flat.ana, Percentage.ofBasisPoints(10_000)))),
                split =
                    ExpenseSplit.Itemised(
                        listOf(
                            ExpenseItem(
                                name = "Fare",
                                unitPrice = Money(2_500, lei),
                                split =
                                    Split.Parts(
                                        listOf(MemberParts(flat.ana, Parts.ofHundredths(100)), MemberParts(flat.bob, Parts.ofHundredths(50))),
                                    ),
                                quantity = Quantity.ofWhole(2),
                            ),
                        ),
                    ),
                date = flat.day,
                tip = TipRule(TipValue.Fixed(Money(500, lei)), TipDistribution.EQUAL),
                time = TimeOfDay.of(9, 5),
                category = flat.food,
                note = "To the airport",
            )
        val dinner =
            Expense(
                name = "Dinner",
                cost = Cost.Local(flat.euros(3_000)),
                payers = Split.Amounts(listOf(MemberAmount(flat.ana, flat.euros(3_300)))),
                split = ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))),
                date = flat.day,
                tip = TipRule(TipValue.Percentage(Percentage.ofBasisPoints(1_000))),
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
            assertEquals(expense.id, id)
            val stored = present(flat.group.expenseBlocking(id))
            assertEquals(expense, stored)
            assertValue(stored, stored.copy(), stored.copy(name = "Lunch"))
        }
    }

    /**
     * An expense made without a tip or a category that carried, or read back with, another tip or category than the library's
     * defaults, or a 0 % tip set explicitly that read back as another rule than no tip, would show the app a tip or a category
     * nobody gave the expense.
     */
    @Test
    fun anExpenseWithoutATipOrACategoryHasNoTipAndTheBuiltInCategory() {
        val flat = Flat()
        val group = flat.group
        assertEquals(TipRule(TipValue.Percentage(Percentage.ofBasisPoints(0)), TipDistribution.PROPORTIONAL), TipRule.NONE)
        assertEquals(CategoryId.UNCATEGORISED, group.categoriesBlocking().first().id)
        val lunch = flat.expense("Lunch", flat.ana, listOf(flat.ana, flat.bob))
        val supper =
            Expense(
                ExpenseId.generate(),
                "Supper",
                Cost.Local(flat.euros(2_000)),
                Split.Equal(listOf(flat.bob)),
                ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))),
                flat.day,
            )
        val coffee =
            Expense(
                "Coffee",
                Cost.Local(flat.euros(400)),
                Split.Equal(listOf(flat.ana)),
                ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))),
                flat.day,
                tip = TipRule(TipValue.Percentage(Percentage.ofBasisPoints(0))),
            )
        for (expense in listOf(lunch, supper, coffee)) {
            assertEquals(TipRule.NONE, expense.tip)
            assertEquals(CategoryId.UNCATEGORISED, expense.category)
            assertEquals(eranos.internal.tipRuleNone(), expense.generated.tip)
            assertEquals(eranos.internal.categoryIdUncategorised(), expense.generated.category)
            assertEquals(flat.euros(0), expense.tipAmount())
            val stored = present(group.expenseBlocking(group.addBlocking(expense)))
            assertEquals(TipRule.NONE, stored.tip)
            assertEquals(CategoryId.UNCATEGORISED, stored.category)
        }
    }

    /**
     * The tip, total, and members an app shows for an expense would differ from those the group records when it adds it.
     */
    @Test
    fun anExpenseTellsItsTipTotalAndMembers() {
        val flat = Flat()
        val lei = CurrencyCode.of("RON")
        val expenses = expenses(flat)
        assertEquals(Money(500, lei), expenses.taxi.tipAmount())
        assertEquals(Money(5_500, lei), expenses.taxi.total())
        assertEquals(listOf(flat.ana, flat.bob), expenses.taxi.members())
        assertEquals(flat.euros(300), expenses.dinner.tipAmount())
        val unreadable = expenses.taxi.copy(payers = Split.Equal(listOf(MemberId(" nobody"))))
        assertThrows<EranosException.Id> { unreadable.members() }
        assertThrows<EranosException.Expense> { expenses.taxi.copy(tip = TipRule(TipValue.Fixed(flat.euros(1)))).tipAmount() }
        assertThrows<EranosException.Expense> { expenses.taxi.copy(tip = TipRule(TipValue.Fixed(Money(-1, lei)))).total() }
        assertEquals(listOf(TipDistribution.PROPORTIONAL, TipDistribution.EQUAL), TipDistribution.entries)
        for (distribution in TipDistribution.entries) {
            assertEquals(distribution, distribution.generated.converted())
        }
    }

    /**
     * A line total multiplied or rounded in Kotlin rather than by the library would show a receipt another amount than the
     * group divides, and an item whose fractional quantity crossed another way would bill another weight.
     */
    @Test
    fun anItemTellsTheLineTotalTheLibraryComputes() {
        val flat = Flat()
        val ana = Split.Equal(listOf(flat.ana))
        // 0.35 kg at 42.00 EUR a kilogram.
        val cheese = ExpenseItem("Cheese", flat.euros(4_200), ana, Quantity.parse("0.35"))
        assertEquals(flat.euros(1_470), cheese.lineTotal())
        // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
        assertEquals(flat.euros(34), ExpenseItem("Olives", flat.euros(100), ana, Quantity.parse("0.335")).lineTotal())
        assertEquals(flat.euros(1_350), ExpenseItem("Beer", flat.euros(450), ana, Quantity.ofWhole(3)).lineTotal())
        val bread = ExpenseItem("Bread", flat.euros(120), ana)
        assertEquals(Quantity.ONE, bread.quantity)
        assertEquals(flat.euros(120), bread.lineTotal())
        // A ten-thousandth of a cent rounds to a free line.
        assertEquals(flat.euros(0), ExpenseItem("Mint", flat.euros(1), ana, Quantity.ofTenThousandths(1)).lineTotal())
        val overflow =
            assertThrows<EranosException.Expense> {
                ExpenseItem("Gold", flat.euros(Long.MAX_VALUE), ana, Quantity.ofWhole(2)).lineTotal()
            }
        assertEquals(ExpenseFailure.Overflow, overflow.failure)

        val deli = Expense("Deli", Cost.Local(flat.euros(1_470)), ana, ExpenseSplit.Itemised(listOf(cheese)), flat.day)
        assertEquals(deli, deli.generated.converted())
        assertEquals(deli, present(flat.group.expenseBlocking(flat.group.addBlocking(deli))))
    }

    /**
     * A discount taken or rounded in Kotlin rather than by the library would show a receipt another amount than the group
     * divides, and a discount off each unit taken as one off the line would bill another total.
     */
    @Test
    fun anItemTellsTheDiscountTheLibraryTakes() {
        val flat = Flat()
        val ana = Split.Equal(listOf(flat.ana))
        val three = Quantity.ofWhole(3)
        // Three lemonades at 3.33, half off: 1.67 off each unit, 5.01 in all, or 5.00 off the line of 9.99.
        val half = DiscountValue.Percentage(Percentage.ofBasisPoints(5_000))
        val halfOffEachUnit = ExpenseItem("Lemonade", flat.euros(333), ana, three, Discount(half, DiscountScope.UNIT))
        assertEquals(flat.euros(999), halfOffEachUnit.lineTotalBeforeDiscount())
        assertEquals(flat.euros(501), halfOffEachUnit.discountAmount())
        assertEquals(flat.euros(498), halfOffEachUnit.lineTotal())
        val halfOffTheLine = ExpenseItem("Lemonade", flat.euros(333), ana, three, Discount(half))
        assertEquals(DiscountScope.LINE, present(halfOffTheLine.discount).scope)
        assertEquals(flat.euros(999), halfOffTheLine.lineTotalBeforeDiscount())
        assertEquals(flat.euros(500), halfOffTheLine.discountAmount())
        assertEquals(flat.euros(499), halfOffTheLine.lineTotal())
        // 1.00 off each of three lemonades at 3.00, and 1.00 off their line.
        val oneEuro = DiscountValue.Fixed(flat.euros(100))
        val oneOffEachUnit = ExpenseItem("Lemonade", flat.euros(300), ana, three, Discount(oneEuro, DiscountScope.UNIT))
        assertEquals(flat.euros(300), oneOffEachUnit.discountAmount())
        assertEquals(flat.euros(600), oneOffEachUnit.lineTotal())
        val oneOffTheLine = ExpenseItem("Lemonade", flat.euros(300), ana, three, Discount(oneEuro))
        assertEquals(flat.euros(100), oneOffTheLine.discountAmount())
        assertEquals(flat.euros(800), oneOffTheLine.lineTotal())
        // A dish on the house: the whole price off leaves a free line.
        val onTheHouse = ExpenseItem("Papanași", flat.euros(900), ana, discount = Discount(DiscountValue.Percentage(Percentage.parse("100"))))
        assertEquals(flat.euros(900), onTheHouse.lineTotalBeforeDiscount())
        assertEquals(flat.euros(900), onTheHouse.discountAmount())
        assertEquals(flat.euros(0), onTheHouse.lineTotal())
        // Without a discount nothing is taken off.
        val bread = ExpenseItem("Bread", flat.euros(120), ana)
        assertNull(bread.discount)
        assertEquals(flat.euros(0), bread.discountAmount())
        assertEquals(bread.lineTotal(), bread.lineTotalBeforeDiscount())
    }

    /**
     * A discount the library refuses that reached a receipt, or one refused with another failure than the library's, would
     * bill a line the group cannot divide, or tell the app another mistake than the one it made.
     */
    @Test
    fun anItemRefusesADiscountItsLineCannotTake() {
        val flat = Flat()
        val refusals =
            listOf(
                DiscountRefusal(discount = Discount(DiscountValue.Fixed(flat.euros(-1))), failure = ExpenseFailure.NegativeDiscount),
                DiscountRefusal(
                    discount = Discount(DiscountValue.Percentage(Percentage.ofBasisPoints(10_001)), DiscountScope.UNIT),
                    failure = ExpenseFailure.DiscountAboveHundredPercent,
                ),
                // 3.01 off a unit of 3.00, and 9.01 off a line of 9.00.
                DiscountRefusal(
                    discount = Discount(DiscountValue.Fixed(flat.euros(301)), DiscountScope.UNIT),
                    failure = ExpenseFailure.DiscountAbovePrice,
                ),
                DiscountRefusal(discount = Discount(DiscountValue.Fixed(flat.euros(901))), failure = ExpenseFailure.DiscountAbovePrice),
                DiscountRefusal(
                    discount = Discount(DiscountValue.Fixed(Money(100, CurrencyCode.of("RON")))),
                    failure = ExpenseFailure.CurrencyMismatch(flat.euro, CurrencyCode.of("RON")),
                ),
            )
        for (refusal in refusals) {
            val item = ExpenseItem("Lemonade", flat.euros(300), Split.Equal(listOf(flat.ana)), Quantity.ofWhole(3), refusal.discount)
            for (total in listOf(item::lineTotalBeforeDiscount, item::discountAmount, item::lineTotal)) {
                val refused = assertThrows<EranosException.Expense> { total() }
                assertEquals(refusal.failure, refused.failure)
            }
        }
    }

    /**
     * A discount that converted to the generated code and back with its value or scope changed, or that the group stored
     * another way, would take another amount off the line than the app built; a discount that compared, hashed, or printed
     * with a field left out would merge two an app holds apart.
     */
    @Test
    fun aDiscountConvertsAndStoresWhole() {
        val flat = Flat()
        val ana = Split.Equal(listOf(flat.ana))
        val half = DiscountValue.Percentage(Percentage.ofBasisPoints(5_000))
        val oneEuro = DiscountValue.Fixed(flat.euros(100))
        val discounts = listOf(Discount(half, DiscountScope.UNIT), Discount(oneEuro), Discount(DiscountValue.Percentage(Percentage.parse("100"))))
        for (discount in discounts) {
            assertEquals(discount, discount.generated.converted())
        }
        // Three lemonades at 3.33: 4.98 with half off each, 8.99 with 1.00 off the line, and free; beside bread at full price.
        val lines =
            discounts.map { discount -> ExpenseItem("Lemonade", flat.euros(333), ana, Quantity.ofWhole(3), discount) } +
                ExpenseItem("Bread", flat.euros(120), ana)
        val round = Expense("Lemonades", Cost.Local(flat.euros(1_517)), ana, ExpenseSplit.Itemised(lines), flat.day)
        assertEquals(round, round.generated.converted())
        assertEquals(round, present(flat.group.expenseBlocking(flat.group.addBlocking(round))))

        assertValue(Discount(half), Discount(half, DiscountScope.LINE), Discount(half, DiscountScope.UNIT))
        assertValue(Discount(half), Discount(DiscountValue.Percentage(Percentage.ofBasisPoints(5_000))), Discount(oneEuro))
        assertValue(half, DiscountValue.Percentage(Percentage.ofBasisPoints(5_000)), DiscountValue.Percentage(Percentage.ofBasisPoints(5_001)))
        assertValue(oneEuro, DiscountValue.Fixed(flat.euros(100)), DiscountValue.Fixed(flat.euros(101)))
        val discounted = lines.first()
        assertValue(
            discounted,
            ExpenseItem(discounted.name, discounted.unitPrice, discounted.split, discounted.quantity, Discount(half, DiscountScope.UNIT)),
            ExpenseItem(discounted.name, discounted.unitPrice, discounted.split, discounted.quantity),
        )
        assertEquals(listOf(DiscountScope.LINE, DiscountScope.UNIT), DiscountScope.entries)
        for (scope in DiscountScope.entries) {
            assertEquals(scope, scope.generated.converted())
        }
    }

    /**
     * A split, a cost, or a tip that compared, hashed, or printed with a field left out would merge two an app holds apart.
     */
    @Test
    fun splitsCostsAndTipsAreValues() {
        val flat = Flat()
        val expenses = expenses(flat)
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
        val amounts = Split.Amounts(listOf(MemberAmount(flat.bob, flat.euros(1))))
        assertEquals(listOf(flat.bob), amounts.members)
        assertValue(amounts, Split.Amounts(amounts.shares), Split.Amounts(listOf(MemberAmount(flat.bob, flat.euros(2)))))
        for (split in listOf<Split>(equal, parts, percentages, amounts)) {
            assertEquals(split, split.generated.converted())
        }
        assertValue(parts.shares[0], MemberParts(flat.bob, Parts.ofHundredths(1)), MemberParts(flat.bob, Parts.ofHundredths(2)))
        assertValue(
            percentages.shares[0],
            MemberPercentage(flat.ana, Percentage.ofBasisPoints(1)),
            MemberPercentage(flat.bob, Percentage.ofBasisPoints(1)),
        )
        assertValue(amounts.shares[0], MemberAmount(flat.bob, flat.euros(1)), MemberAmount(flat.ana, flat.euros(1)))

        assertValue(expenses.dinner.cost, Cost.Local(flat.euros(3_000)), Cost.Local(flat.euros(1)))
        assertEquals(Money(5_000, CurrencyCode.of("RON")), expenses.taxi.cost.amount)
        assertValue(
            expenses.taxi.cost,
            Cost.Converted(expenses.taxi.cost.amount, ExchangeRate.parse("0.2")),
            Cost.Converted(expenses.taxi.cost.amount, ExchangeRate.parse("0.3")),
        )
        assertValue(expenses.dinner.split, ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))), ExpenseSplit.Whole(equal))
        val itemised = expenses.taxi.split as ExpenseSplit.Itemised
        assertValue(itemised, ExpenseSplit.Itemised(itemised.items), ExpenseSplit.Itemised(emptyList()))
        val item = itemised.items[0]
        assertValue(item, ExpenseItem(item.name, item.unitPrice, item.split, Quantity.ofWhole(2)), ExpenseItem(item.name, item.unitPrice, item.split))
        assertEquals(Quantity.ONE, ExpenseItem("Bread", flat.euros(1), equal).quantity)
        assertValue(
            expenses.taxi.tip,
            TipRule(TipValue.Fixed(Money(500, CurrencyCode.of("RON"))), TipDistribution.EQUAL),
            TipRule(TipValue.Fixed(Money(500, CurrencyCode.of("RON")))),
        )
        assertEquals(TipDistribution.PROPORTIONAL, expenses.dinner.tip.distribution)
        assertValue(
            TipValue.Percentage(Percentage.ofBasisPoints(1)),
            TipValue.Percentage(Percentage.ofBasisPoints(1)),
            TipValue.Percentage(Percentage.ofBasisPoints(2)),
        )
        assertValue(TipValue.Fixed(flat.euros(1)), TipValue.Fixed(flat.euros(1)), TipValue.Fixed(flat.euros(2)))
    }

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group stored another way, would
     * pay or reach another person.
     */
    @Test
    fun membersAndTransfersConvertAndStoreWhole() {
        val flat = Flat()
        val mainAccount = PaymentAccount("Main account", PaymentAccountDetails.Iban(Iban.parse("RO49AAAA1B31007593840000")))
        val paymentApp = PaymentAccount("Payment app", PaymentAccountDetails.Username(Username.parse("carol.pays")))
        val carol =
            Member(
                firstName = "Carol",
                lastName = "Popa",
                email = Email.parse("carol@example.com"),
                phone = PhoneNumber.parse("+40712345678"),
                accounts = listOf(mainAccount, paymentApp),
            )
        assertEquals(carol, carol.generated.converted())
        val id = flat.group.addBlocking(carol)
        assertEquals(carol.id, id)
        val stored = present(flat.group.memberBlocking(id))
        assertEquals(carol, stored)
        assertValue(stored, stored.copy(), stored.copy(firstName = "Caroline"))
        assertEquals(emptyList<PaymentAccount>(), Member("Dan").accounts)
        assertValue(mainAccount, PaymentAccount("Main account", mainAccount.details), PaymentAccount("Savings", mainAccount.details))
        assertValue(
            mainAccount.details,
            PaymentAccountDetails.Iban(Iban.parse("RO49AAAA1B31007593840000")),
            PaymentAccountDetails.Iban(Iban.parse("GB82WEST12345698765432")),
        )
        assertValue(
            paymentApp.details,
            PaymentAccountDetails.Username(Username.parse("carol.pays")),
            PaymentAccountDetails.Username(Username.parse("dan.pays")),
        )

        val lei = CurrencyCode.of("RON")
        val cash =
            Transfer(
                sender = flat.bob,
                receiver = flat.ana,
                cost = Cost.Converted(Money(1_000, lei), ExchangeRate.parse("0.2")),
                date = flat.day,
                time = TimeOfDay.of(10, 0),
                note = "Cash",
            )
        assertEquals(cash, cash.generated.converted())
        val payment = flat.group.addBlocking(cash)
        assertEquals(cash.id, payment)
        val storedCash = present(flat.group.transferBlocking(payment))
        assertEquals(cash, storedCash)
        assertEquals("10:00", storedCash.time.toString())
        assertEquals("Cash", storedCash.note)
        assertEquals(flat.euros(200), flat.group.transferAmountBlocking(payment))
        assertValue(storedCash, storedCash.copy(), storedCash.copy(note = null))

        val travel = Category("Travel")
        assertEquals(travel, travel.generated.converted())
        val trip = flat.group.addBlocking(travel)
        assertEquals(travel.id, trip)
        val storedTravel = present(flat.group.categoryBlocking(trip))
        assertEquals(travel, storedTravel)
        assertValue(storedTravel, storedTravel.copy(), storedTravel.copy(name = "Trips"))
    }

    /**
     * An expense's note, a member's last name, or a transfer's note, absent or blank, that read back as text, that a
     * replacement without it left as it was stored, or that kept the whitespace around it would show an app text nobody gave
     * the entry.
     */
    @Test
    fun anAbsentOrBlankTextReadsBackAbsent() {
        val flat = Flat()
        val group = flat.group
        for (text in listOf(null, " ")) {
            val lunch = flat.expense("Lunch", flat.ana, listOf(flat.ana))
            assertNull(present(group.expenseBlocking(group.addBlocking(lunch.copy(note = text)))).note)
            group.replaceBlocking(lunch.copy(note = "Lunch out"))
            group.replaceBlocking(lunch.copy(note = text))
            assertNull(present(group.expenseBlocking(lunch.id)).note)

            val carol = Member("Carol")
            assertNull(present(group.memberBlocking(group.addBlocking(carol.copy(lastName = text)))).lastName)
            group.replaceBlocking(carol.copy(lastName = "Popa"))
            group.replaceBlocking(carol.copy(lastName = text))
            assertNull(present(group.memberBlocking(carol.id)).lastName)

            val cash = Transfer(flat.bob, flat.ana, Cost.Local(flat.euros(500)), flat.day)
            assertNull(present(group.transferBlocking(group.addBlocking(cash.copy(note = text)))).note)
            group.replaceBlocking(cash.copy(note = "Cash"))
            group.replaceBlocking(cash.copy(note = text))
            assertNull(present(group.transferBlocking(cash.id)).note)
        }
        val taxi = flat.expense("Taxi", flat.bob, listOf(flat.bob)).copy(note = " To the airport\n")
        assertEquals("To the airport", present(group.expenseBlocking(group.addBlocking(taxi))).note)
        val dan = Member("Dan", lastName = "\tPopescu ")
        assertEquals("Popescu", present(group.memberBlocking(group.addBlocking(dan))).lastName)
        val change = Transfer(flat.ana, flat.bob, Cost.Local(flat.euros(100)), flat.day, note = " Change\n")
        assertEquals("Change", present(group.transferBlocking(group.addBlocking(change))).note)
    }

    /**
     * An entry made without an identifier that generated none when it was made, or the same one as another entry, would
     * leave the app nothing to store the entry under, or two entries under one identifier.
     */
    @Test
    fun anEntryMadeWithoutAnIdentifierGeneratesANewOne() {
        val flat = Flat()
        assertNotEquals(Member("Dan").id, Member("Dan").id)
        assertNotEquals(Category("Travel").id, Category("Travel").id)
        assertNotEquals(flat.expense("Lunch", flat.ana, listOf(flat.ana)).id, flat.expense("Lunch", flat.ana, listOf(flat.ana)).id)
        val cost = Cost.Local(flat.euros(500))
        assertNotEquals(Transfer(flat.bob, flat.ana, cost, flat.day).id, Transfer(flat.bob, flat.ana, cost, flat.day).id)
    }

    /**
     * A replacement under an identifier the group does not hold that the group added instead, or refused with another
     * failure, would store an entry the app meant to change, or hide from it that the entry it changes is gone.
     */
    @Test
    fun aReplacementForAnEntryTheGroupDoesNotHoldIsRefused() {
        val flat = Flat()
        val group = flat.group
        val dan = Member("Dan")
        val travel = Category("Travel")
        val lunch = flat.expense("Lunch", flat.ana, listOf(flat.ana, flat.bob))
        val cash = Transfer(flat.bob, flat.ana, Cost.Local(flat.euros(500)), flat.day)
        val unknowns =
            listOf(
                Refusal(call = { group.replaceBlocking(dan) }, failure = ExpenseGroupFailure.UnknownMember(dan.id)),
                Refusal(call = { group.replaceBlocking(travel) }, failure = ExpenseGroupFailure.UnknownCategory(travel.id)),
                Refusal(call = { group.replaceBlocking(lunch) }, failure = ExpenseGroupFailure.UnknownExpense(lunch.id)),
                Refusal(call = { group.replaceBlocking(cash) }, failure = ExpenseGroupFailure.UnknownTransfer(cash.id)),
            )
        for (unknown in unknowns) {
            val refused = assertThrows<EranosException.ExpenseGroup> { unknown.call() }
            assertEquals(unknown.failure, refused.failure)
        }
        assertEquals(listOf(flat.ana, flat.bob), group.membersBlocking().map { member -> member.id })
    }

    /**
     * An entry to add that lost the identifier its app gives it, took one the rules refuse, or took one the group holds would
     * leave the app's own store pointing at another entry or at none; identifiers that compared as canonically equivalent
     * text would merge two members the library keeps apart.
     */
    @Test
    fun anEntryToAddKeepsTheIdentifierItsAppGivesIt() {
        val flat = Flat()
        val group = flat.group
        val carol = Member(MemberId.parse("carol"), "Carol", lastName = "Popa")
        assertEquals(MemberId("carol"), group.addBlocking(carol))
        assertEquals(carol, group.memberBlocking(MemberId("carol")))
        val travel = Category(CategoryId.parse("-travel"), "Travel")
        assertEquals(CategoryId("-travel"), group.addBlocking(travel))
        assertEquals(travel, group.categoryBlocking(CategoryId("-travel")))
        val lunch =
            Expense(
                ExpenseId.parse("Lisboa · 2026"),
                "Lunch",
                Cost.Local(flat.euros(1_000)),
                Split.Equal(listOf(flat.ana)),
                ExpenseSplit.Whole(Split.Equal(listOf(flat.ana, flat.bob))),
                flat.day,
                category = travel.id,
            )
        assertEquals(ExpenseId("Lisboa · 2026"), group.addBlocking(lunch))
        assertEquals(lunch, group.expenseBlocking(ExpenseId("Lisboa · 2026")))
        val cash = Transfer(TransferId.parse("zz"), flat.bob, flat.ana, Cost.Local(flat.euros(500)), flat.day, note = "Cash")
        assertEquals(TransferId("zz"), group.addBlocking(cash))
        assertEquals(cash, group.transferBlocking(TransferId("zz")))

        val duplicates =
            listOf(
                Refusal(
                    call = { group.addBlocking(Member(carol.id, "Caroline")) },
                    failure = ExpenseGroupFailure.DuplicateMember(MemberId("carol")),
                ),
                Refusal(
                    call = { group.addBlocking(Category(travel.id, "Trips")) },
                    failure = ExpenseGroupFailure.DuplicateCategory(CategoryId("-travel")),
                ),
                Refusal(
                    call = { group.addBlocking(Category(CategoryId.UNCATEGORISED, "Other")) },
                    failure = ExpenseGroupFailure.DuplicateCategory(CategoryId.UNCATEGORISED),
                ),
                Refusal(
                    call = { group.addBlocking(lunch.copy(name = "Dinner")) },
                    failure = ExpenseGroupFailure.DuplicateExpense(ExpenseId("Lisboa · 2026")),
                ),
                Refusal(
                    call = { group.addBlocking(cash.copy(note = null)) },
                    failure = ExpenseGroupFailure.DuplicateTransfer(TransferId("zz")),
                ),
            )
        for (duplicate in duplicates) {
            val refused = assertThrows<EranosException.ExpenseGroup> { duplicate.call() }
            assertEquals(duplicate.failure, refused.failure)
        }
        val padded = assertThrows<EranosException.Id> { group.addBlocking(Member(MemberId(" x"), "Dan")) }
        assertEquals(IdFailure.SurroundingWhitespace, padded.failure)

        val composed = group.addBlocking(Member(MemberId.parse("\u00e9"), "Eva"))
        val decomposed = group.addBlocking(Member(MemberId.parse("e\u0301"), "Eve"))
        assertNotEquals(composed, decomposed)
        assertEquals(listOf("Eva", "Eve"), listOf(composed, decomposed).map { id -> present(group.memberBlocking(id)).firstName })
    }

    /** An entry whose lists the app changed after building it would change behind the app's back. */
    @Test
    fun anEntryKeepsTheListsItWasBuiltWith() {
        val flat = Flat()
        val members = mutableListOf(flat.ana)
        val split = Split.Equal(members)
        val accounts = mutableListOf(PaymentAccount("Payment app", PaymentAccountDetails.Username(Username.parse("ana.pays"))))
        val member = Member("Ana", accounts = accounts)
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

/** A call the group refuses, such as an add under an identifier it holds, and the failure it refuses it with. */
private class Refusal(
    val call: () -> Any,
    val failure: ExpenseGroupFailure,
)

/** A discount an item's line cannot take, and the failure every total of the item refuses it with. */
private class DiscountRefusal(
    val discount: Discount,
    val failure: ExpenseFailure,
)
