// The tables several sections of the tour print: the balances, the categories, and the entries.
package eranos.examples.grandtour

import eranos.Balance
import eranos.CategoryId
import eranos.Currency
import eranos.Group
import eranos.Money
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.totals

/** The identifier of the built-in category, which the group lists first. */
internal fun builtInCategory(group: Group): CategoryId? = group.categoriesBlocking().firstOrNull()?.id

internal fun printBalances(group: Group) {
    val summary = group.summaryBlocking()
    val zero = "${Money(group.currencyBlocking(), 0)}"
    printTable(
        listOf("Member", "Paid", "Consumed", "Sent", "Received", "Balance"),
        group.membersBlocking().map { member ->
            // The summary lists the members with a total; every total of the others is zero.
            val totals = member.id?.let { id -> totals(id, summary) }
            if (null == totals) {
                listOf(member.firstName, zero, zero, zero, zero, "${Balance.Settled}")
            } else {
                listOf(member.firstName, "${totals.paid}", "${totals.share}", "${totals.sent}", "${totals.received}", "${totals.balance}")
            }
        },
        numbersFrom = 1,
    )
}

internal fun printCategories(group: Group) {
    val expenses = group.expensesBlocking()
    val builtIn = builtInCategory(group)
    printTable(
        listOf("Category", "Expenses", "Total"),
        group.summaryBlocking().categoryTotals.map { total ->
            listOf(
                group.categoryBlocking(total.category)?.name ?: "?",
                "${expenses.count { expense -> (expense.category ?: builtIn) == total.category }}",
                "${total.total}",
            )
        },
        numbersFrom = 1,
    )
}

/** Every expense and transfer in a currency [shown] accepts, as entered and in the group currency. */
internal fun printEntries(
    group: Group,
    shown: (Currency) -> Boolean,
) {
    val zero = Money(group.currencyBlocking(), 0)
    val expenses =
        group.expensesBlocking().filter { expense -> shown(expense.cost.amount.currency) }.map { expense ->
            listOf(
                expense.name,
                names(group, expense.payers.members),
                "${expense.total()}",
                "${rate(expense.cost)}",
                "${expense.id?.let { id -> group.expenseSharesBlocking(id) }?.total ?: zero}",
            )
        }
    val transfers =
        group.transfersBlocking().filter { transfer -> shown(transfer.cost.amount.currency) }.map { transfer ->
            listOf(
                "Transfer to ${name(group, transfer.to)}",
                name(group, transfer.from),
                "${transfer.cost.amount}",
                "${rate(transfer.cost)}",
                "${transfer.id?.let { id -> group.transferAmountBlocking(id) } ?: zero}",
            )
        }
    printTable(listOf("Entry", "Paid by", "Entered as", "Rate", "In the group currency"), expenses + transfers, numbersFrom = 2)
}
