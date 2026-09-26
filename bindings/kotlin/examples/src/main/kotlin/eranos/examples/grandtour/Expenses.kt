// Section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption.
package eranos.examples.grandtour

import eranos.Cost
import eranos.Expense
import eranos.ExpenseId
import eranos.ExpenseSplit
import eranos.Group
import eranos.Item
import eranos.MemberAmount
import eranos.MemberId
import eranos.MemberParts
import eranos.MemberPercentage
import eranos.Money
import eranos.Parts
import eranos.Percentage
import eranos.Split
import eranos.TimeOfDay
import eranos.Tip
import eranos.TipDistribution
import eranos.TipValue
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section

/** How an expense divides, in a few words. */
internal fun how(expense: Expense): String {
    val split =
        when (val divided = expense.split) {
            is ExpenseSplit.Itemised -> {
                "${divided.items.size} items"
            }

            is ExpenseSplit.Whole -> {
                when (divided.split) {
                    is Split.Amounts -> "by amounts"
                    is Split.Equal -> "equally"
                    is Split.Parts -> "by parts"
                    is Split.Percentages -> "by percentage"
                }
            }
        }
    val count = expense.payers.members.size
    val payers = if (1 == count) "" else ", $count payers"
    val tip =
        if (0L == expense.tipAmount().minorUnits) {
            ""
        } else {
            when (val value = expense.tip?.value) {
                is TipValue.Fixed -> ", ${value.amount} tip"
                is TipValue.Percentage -> ", ${value.percentage} tip"
                null -> ""
            }
        }
    return split + payers + tip
}

/** Each expense of [expenses], with every involved member's payment and consumption. */
internal fun printShares(
    group: Group,
    expenses: List<ExpenseId>,
) {
    val rows =
        expenses.mapNotNull { id ->
            val expense = group.expenseBlocking(id)
            val shares = group.expenseSharesBlocking(id)
            if (null == expense || null == shares) {
                null
            } else {
                listOf(
                    expense.name,
                    how(expense),
                    cell(shares.members.map { share -> name(group, share.member) }),
                    cell(shares.members.map { share -> if (0L == share.paid.minorUnits) "" else "${share.paid}" }),
                    cell(shares.members.map { share -> "${share.share}" }),
                )
            }
        }
    printTable(listOf("Expense", "Split", "Member", "Paid", "Consumed"), rows, numbersFrom = 3)
}

internal fun printItems(
    group: Group,
    id: ExpenseId,
) {
    val split = group.expenseBlocking(id)?.split
    if (split is ExpenseSplit.Itemised) {
        printTable(
            listOf("Item", "Shared by", "Quantity", "Unit price", "Line total"),
            split.items.map { item ->
                val total = Money(item.unitPrice.currency, item.unitPrice.minorUnits * item.quantity)
                listOf(item.name, names(group, item.split.members), "${item.quantity}", "${item.unitPrice}", "$total")
            },
            numbersFrom = 2,
        )
    }
}

/** The receipt of the fado night: every line with its own split. */
internal fun fadoItems(people: People): List<Item> =
    listOf(
        Item("Vinho verde", Split.Equal(listOf(people.ana, people.bob, people.dan)), eur("12.00"), quantity = 2),
        Item("Bacalhau", Split.Equal(listOf(people.carol)), eur("18.00")),
        Item(
            "Custard tarts",
            Split.Parts(
                listOf(
                    MemberParts(people.ana, Parts.parse("1")),
                    MemberParts(people.bob, Parts.parse("1")),
                    MemberParts(people.carol, Parts.parse("1")),
                    MemberParts(people.dan, Parts.parse("1")),
                    MemberParts(people.eva, Parts.parse("2")),
                ),
            ),
            eur("1.50"),
            quantity = 6,
        ),
        Item("Juice", Split.Equal(listOf(people.eva)), eur("3.00")),
    )

/** The first three expenses: equally, by percentage, and by parts. */
internal fun dividedExpenses(
    people: People,
    categories: Categories,
): List<Expense> =
    listOf(
        Expense(
            Cost.Local(eur("10.00")),
            october(1),
            "Groceries",
            Split.Equal(listOf(people.ana)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol))),
            category = categories.food,
        ),
        Expense(
            Cost.Local(eur("800.00")),
            october(1),
            "Apartment",
            Split.Equal(listOf(people.ana)),
            ExpenseSplit.Whole(
                Split.Percentages(
                    listOf(
                        MemberPercentage(people.ana, Percentage.parse("25")),
                        MemberPercentage(people.bob, Percentage.parse("25")),
                        MemberPercentage(people.carol, Percentage.parse("35")),
                        MemberPercentage(people.dan, Percentage.parse("15")),
                    ),
                ),
            ),
            category = categories.lodging,
        ),
        Expense(
            Cost.Local(eur("90.00")),
            october(2),
            "Seafood dinner",
            Split.Equal(listOf(people.carol)),
            ExpenseSplit.Whole(
                Split.Parts(
                    listOf(
                        MemberParts(people.ana, Parts.parse("1")),
                        MemberParts(people.bob, Parts.parse("1")),
                        MemberParts(people.carol, Parts.parse("1")),
                        MemberParts(people.dan, Parts.parse("1")),
                        MemberParts(people.eva, Parts.parse("0.5")),
                    ),
                ),
            ),
            category = categories.food,
        ),
    )

/** The last four expenses: by exact amounts, with two payers, item by item, and with a tip. */
internal fun paidExpenses(
    people: People,
    categories: Categories,
): List<Expense> =
    listOf(
        Expense(
            Cost.Local(eur("88.50")),
            october(3),
            "Train to Porto",
            Split.Equal(listOf(people.bob)),
            ExpenseSplit.Whole(
                Split.Amounts(
                    listOf(
                        MemberAmount(eur("24.50"), people.ana),
                        MemberAmount(eur("24.50"), people.bob),
                        MemberAmount(eur("24.50"), people.carol),
                        MemberAmount(eur("15.00"), people.eva),
                    ),
                ),
            ),
            category = categories.transport,
        ),
        Expense(
            Cost.Local(eur("60.00")),
            october(3),
            "Museum",
            Split.Equal(listOf(people.ana, people.bob)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan))),
        ),
        Expense(
            Cost.Local(eur("54.00")),
            october(4),
            "Fado night",
            Split.Amounts(listOf(MemberAmount(eur("39.40"), people.carol), MemberAmount(eur("20.00"), people.dan))),
            ExpenseSplit.Itemised(fadoItems(people)),
            category = categories.food,
            description = "Tasca do Chico",
            time = TimeOfDay.of(hour = 22, minute = 15),
            tip = Tip(TipValue.Percentage(Percentage.parse("10"))),
        ),
        Expense(
            Cost.Local(eur("45.00")),
            october(5),
            "Tuk-tuk tour",
            Split.Equal(listOf(people.dan)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan, people.eva))),
            category = categories.transport,
            tip = Tip(TipValue.Fixed(eur("5.00")), TipDistribution.EQUAL),
        ),
    )

/** The consumption of [member] in the expense [id]. */
internal fun consumed(
    group: Group,
    id: ExpenseId,
    member: MemberId,
): Money? =
    group
        .expenseSharesBlocking(id)
        ?.members
        ?.firstOrNull { share -> share.member == member }
        ?.share

/** Prints how the fado night's tip was shared, and its receipt. */
internal fun printFado(
    group: Group,
    fado: Expense,
) {
    val rule =
        when (val value = fado.tip?.value) {
            is TipValue.Fixed -> "${value.amount}"
            is TipValue.Percentage -> "${value.percentage}"
            null -> "a tip"
        }
    val shared =
        when (fado.tip?.distribution) {
            TipDistribution.EQUAL -> "equally"
            TipDistribution.PROPORTIONAL -> "in proportion to what each member ordered"
            null -> "another way"
        }
    println(
        """
        The groceries' 10.00 EUR did not divide into thirds, so one member consumed the
        leftover cent; the expense's identifier picks which, so it moves from expense to
        expense. The fado night, at ${fado.time?.toString().orEmpty()} on ${fado.date} at ${fado.description.orEmpty()}, was itemised; its
        $rule tip of ${fado.tipAmount()} was shared $shared:
        """.trimIndent(),
    )
    fado.id?.let { id -> printItems(group, id) }
    println("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.")
}

internal fun expenses(
    group: Group,
    people: People,
    categories: Categories,
): ExpenseId {
    section("4. Expenses, one for every way of dividing")
    val ids = (dividedExpenses(people, categories) + paidExpenses(people, categories)).map { expense -> group.addBlocking(expense) }
    val groceries = ids[0]
    val apartment = ids[1]
    val seafood = ids[2]
    val train = ids[3]
    val museum = ids[4]
    val fado = ids[5]
    val tour = ids[6]
    println(
        """
        Seven expenses cover every way of dividing one: equally, by percentage, by parts
        (Eva counts as half), by exact amounts, with two payers, item by item, and with a
        tip. Every member's consumption adds up to the expense's total, to the cent:
        """.trimIndent(),
    )
    printShares(group, ids)
    val fadoExpense = checkNotNull(group.expenseBlocking(fado)) { "the fado night was added" }
    printFado(group, fadoExpense)

    val thirds = listOf(people.ana, people.bob, people.carol).mapNotNull { member -> consumed(group, groceries, member)?.minorUnits }
    check(listOf(333L, 333L, 334L) == thirds.sorted())
    check(eur("280.00") == consumed(group, apartment, people.carol))
    check(eur("10.00") == consumed(group, seafood, people.eva))
    check(eur("15.00") == consumed(group, train, people.eva))
    check(
        eur("30.00") ==
            group
                .expenseSharesBlocking(museum)
                ?.members
                ?.firstOrNull { share -> share.member == people.bob }
                ?.paid,
    )
    check(eur("15.00") == consumed(group, museum, people.dan))
    check(4 == group.expenseBlocking(museum)?.members()?.size)
    check(eur("59.40") == fadoExpense.total())
    check(eur("5.40") == fadoExpense.tipAmount())
    check(eur("21.45") == consumed(group, fado, people.carol))
    check(eur("6.60") == consumed(group, fado, people.eva))
    check(eur("10.00") == consumed(group, tour, people.eva))
    return seafood
}
