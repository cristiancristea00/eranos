// Section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption.
package eranos.examples.grandtour

import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseId
import eranos.ExpenseSplit
import eranos.MemberId
import eranos.Money
import eranos.Split
import eranos.TipDistribution
import eranos.TipValue
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section

private data class ShareRow(
    val expense: String,
    val how: String,
    val members: String,
    val paid: String,
    val consumed: String,
) : Row {
    override val cells: List<String>
        get() = listOf(expense, how, members, paid, consumed)

    companion object {
        val HEADERS = listOf("Expense", "Split", "Member", "Paid", "Consumed")
    }
}

private data class ItemRow(
    val name: String,
    val sharedBy: String,
    val quantity: Int,
    val unitPrice: Money,
    val lineTotal: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, sharedBy, "$quantity", "$unitPrice", "$lineTotal")

    companion object {
        val HEADERS = listOf("Item", "Shared by", "Quantity", "Unit price", "Line total")
    }
}

/** The seven expenses, one for every way of dividing one. */
private data class Expenses(
    val apartment: ExpenseId,
    val fado: ExpenseId,
    val groceries: ExpenseId,
    val museum: ExpenseId,
    val seafood: ExpenseId,
    val tour: ExpenseId,
    val train: ExpenseId,
)

/** How an expense divides, in a few words. */
internal fun how(expense: Expense): String {
    val split =
        when (val divided = expense.split) {
            is ExpenseSplit.Whole -> {
                when (divided.split) {
                    is Split.Equal -> "equally"
                    is Split.Parts -> "by parts"
                    is Split.Percentages -> "by percentage"
                    is Split.Amounts -> "by amounts"
                }
            }

            is ExpenseSplit.Itemised -> {
                "${divided.items.size} items"
            }
        }
    val count = expense.payers.members.size
    val payers = if (1 == count) "" else ", $count payers"
    val tip =
        if (0L == expense.tipAmount().minorUnits) {
            ""
        } else {
            when (val value = expense.tip?.value) {
                is TipValue.Percentage -> ", ${value.percentage} tip"
                is TipValue.Fixed -> ", ${value.amount} tip"
                null -> ""
            }
        }
    return split + payers + tip
}

/** Each expense of [expenses], with every involved member's payment and consumption. */
internal fun printShares(
    group: ExpenseGroup,
    expenses: List<ExpenseId>,
) {
    val rows =
        expenses.mapNotNull { id ->
            val expense = group.expenseBlocking(id)
            val shares = group.expenseSharesBlocking(id)
            if (null == expense || null == shares) {
                null
            } else {
                ShareRow(
                    expense = expense.name,
                    how = how(expense),
                    members = cell(shares.members.map { share -> name(group, share.member) }),
                    paid = cell(shares.members.map { share -> if (0L == share.paid.minorUnits) "" else "${share.paid}" }),
                    consumed = cell(shares.members.map { share -> "${share.share}" }),
                )
            }
        }
    printTable(ShareRow.HEADERS, rows, numbersFrom = 3)
}

internal fun printItems(
    group: ExpenseGroup,
    id: ExpenseId,
) {
    val split = group.expenseBlocking(id)?.split
    if (split is ExpenseSplit.Itemised) {
        printTable(
            ItemRow.HEADERS,
            split.items.map { item ->
                ItemRow(
                    name = item.name,
                    sharedBy = names(group, item.split.members),
                    quantity = item.quantity,
                    unitPrice = item.unitPrice,
                    lineTotal = Money(item.unitPrice.minorUnits * item.quantity, item.unitPrice.currency),
                )
            },
            numbersFrom = 2,
        )
    }
}

/** The consumption of [member] in the expense [id]. */
internal fun consumed(
    group: ExpenseGroup,
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
    group: ExpenseGroup,
    fado: Expense,
) {
    val rule =
        when (val value = fado.tip?.value) {
            is TipValue.Percentage -> "${value.percentage}"
            is TipValue.Fixed -> "${value.amount}"
            null -> "a tip"
        }
    val shared =
        when (fado.tip?.distribution) {
            TipDistribution.PROPORTIONAL -> "in proportion to what each member ordered"
            TipDistribution.EQUAL -> "equally"
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
    printItems(group, fado.id)
    println("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.")
}

/** Adds one expense for every way of dividing one. */
private fun addExpenses(
    group: ExpenseGroup,
    people: People,
    categories: Categories,
): Expenses {
    val groceries = group.addBlocking(groceriesEqually(people, categories))
    val apartment = group.addBlocking(apartmentByPercentage(people, categories))
    val seafood = group.addBlocking(seafoodByParts(people, categories))
    val train = group.addBlocking(trainByAmounts(people, categories))
    val museum = group.addBlocking(museumWithTwoPayers(people))
    val fado = group.addBlocking(fadoItemByItem(people, categories))
    val tour = group.addBlocking(tourWithATip(people, categories))
    return Expenses(apartment, fado, groceries, museum, seafood, tour, train)
}

internal fun expenses(
    group: ExpenseGroup,
    people: People,
    categories: Categories,
): ExpenseId {
    section("4. Expenses, one for every way of dividing")
    val expenses = addExpenses(group, people, categories)
    println(
        """
        Seven expenses cover every way of dividing one: equally, by percentage, by parts
        (Eva counts as half), by exact amounts, with two payers, item by item, and with a
        tip. Every member's consumption adds up to the expense's total, to the cent:
        """.trimIndent(),
    )
    printShares(
        group,
        listOf(expenses.groceries, expenses.apartment, expenses.seafood, expenses.train, expenses.museum, expenses.fado, expenses.tour),
    )
    val fadoExpense = checkNotNull(group.expenseBlocking(expenses.fado)) { "the fado night was added" }
    printFado(group, fadoExpense)

    val thirds = listOf(people.ana, people.bob, people.carol).mapNotNull { member -> consumed(group, expenses.groceries, member)?.minorUnits }
    check(listOf(333L, 333L, 334L) == thirds.sorted())
    check(eur("280.00") == consumed(group, expenses.apartment, people.carol))
    check(eur("10.00") == consumed(group, expenses.seafood, people.eva))
    check(eur("15.00") == consumed(group, expenses.train, people.eva))
    check(
        eur("30.00") ==
            group
                .expenseSharesBlocking(expenses.museum)
                ?.members
                ?.firstOrNull { share -> share.member == people.bob }
                ?.paid,
    )
    check(eur("15.00") == consumed(group, expenses.museum, people.dan))
    check(4 == group.expenseBlocking(expenses.museum)?.members()?.size)
    check(eur("59.40") == fadoExpense.total())
    check(eur("5.40") == fadoExpense.tipAmount())
    check(eur("21.45") == consumed(group, expenses.fado, people.carol))
    check(eur("6.60") == consumed(group, expenses.fado, people.eva))
    check(eur("10.00") == consumed(group, expenses.tour, people.eva))
    return expenses.seafood
}
