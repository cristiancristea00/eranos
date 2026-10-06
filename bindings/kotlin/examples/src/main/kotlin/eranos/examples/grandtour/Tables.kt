// The tables several sections of the tour print, and the rows they share: the balances, the categories, the entries, the
// payments, and the refused changes.
package eranos.examples.grandtour

import eranos.Balance
import eranos.CategoryId
import eranos.Currency
import eranos.ExchangeRate
import eranos.Group
import eranos.Money
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.totals

private data class BalanceRow(
    val balance: Balance,
    val consumed: Money,
    val name: String,
    val paid: Money,
    val received: Money,
    val sent: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$paid", "$consumed", "$sent", "$received", "$balance")

    companion object {
        val HEADERS = listOf("Member", "Paid", "Consumed", "Sent", "Received", "Balance")
    }
}

private data class CategoryRow(
    val expenses: Int,
    val name: String,
    val total: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$expenses", "$total")

    companion object {
        val HEADERS = listOf("Category", "Expenses", "Total")
    }
}

private data class EntryRow(
    val converted: Money,
    val name: String,
    val original: Money,
    val payer: String,
    val rate: ExchangeRate,
) : Row {
    override val cells: List<String>
        get() = listOf(name, payer, "$original", "$rate", "$converted")

    companion object {
        val HEADERS = listOf("Entry", "Paid by", "Entered as", "Rate", "In the group currency")
    }
}

internal data class PaymentRow(
    val amount: Money,
    val from: String,
    val to: String,
) : Row {
    override val cells: List<String>
        get() = listOf(from, to, "$amount")

    companion object {
        val HEADERS = listOf("From", "To", "Amount")
    }
}

internal data class RefusalRow(
    val change: String,
    val error: String,
) : Row {
    override val cells: List<String>
        get() = listOf(change, error)

    companion object {
        val HEADERS = listOf("Change", "Refused with")
    }
}

/** The identifier of the built-in category, which the group lists first. */
internal fun builtInCategory(group: Group): CategoryId? = group.categoriesBlocking().firstOrNull()?.id

internal fun printBalances(group: Group) {
    val summary = group.summaryBlocking()
    val zero = Money(group.currencyBlocking(), 0)
    printTable(
        BalanceRow.HEADERS,
        group.membersBlocking().map { member ->
            // The summary lists the members with a total; every total of the others is zero.
            val totals = member.id?.let { id -> totals(id, summary) }
            if (null == totals) {
                BalanceRow(balance = Balance.Settled, consumed = zero, name = member.firstName, paid = zero, received = zero, sent = zero)
            } else {
                BalanceRow(
                    balance = totals.balance,
                    consumed = totals.share,
                    name = member.firstName,
                    paid = totals.paid,
                    received = totals.received,
                    sent = totals.sent,
                )
            }
        },
        numbersFrom = 1,
    )
}

internal fun printCategories(group: Group) {
    val expenses = group.expensesBlocking()
    val builtIn = builtInCategory(group)
    printTable(
        CategoryRow.HEADERS,
        group.summaryBlocking().categoryTotals.map { total ->
            CategoryRow(
                expenses = expenses.count { expense -> (expense.category ?: builtIn) == total.category },
                name = group.categoryBlocking(total.category)?.name ?: "?",
                total = total.total,
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
            EntryRow(
                converted = expense.id?.let { id -> group.expenseSharesBlocking(id) }?.total ?: zero,
                name = expense.name,
                original = expense.total(),
                payer = names(group, expense.payers.members),
                rate = rate(expense.cost),
            )
        }
    val transfers =
        group.transfersBlocking().filter { transfer -> shown(transfer.cost.amount.currency) }.map { transfer ->
            EntryRow(
                converted = transfer.id?.let { id -> group.transferAmountBlocking(id) } ?: zero,
                name = "Transfer to ${name(group, transfer.to)}",
                original = transfer.cost.amount,
                payer = name(group, transfer.from),
                rate = rate(transfer.cost),
            )
        }
    printTable(EntryRow.HEADERS, expenses + transfers, numbersFrom = 2)
}
