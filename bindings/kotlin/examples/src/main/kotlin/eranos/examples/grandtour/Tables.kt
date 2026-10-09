// The tables several sections of the tour print, and the rows they share: the balances, the categories, the entries, the
// payments, and the refused changes.
package eranos.examples.grandtour

import eranos.Balance
import eranos.CurrencyCode
import eranos.ExchangeRate
import eranos.ExpenseGroup
import eranos.Money
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable

private data class BalanceRow(
    val name: String,
    val paid: Money,
    val consumed: Money,
    val sent: Money,
    val received: Money,
    val balance: Balance,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$paid", "$consumed", "$sent", "$received", "$balance")

    companion object {
        val HEADERS = listOf("Member", "Paid", "Consumed", "Sent", "Received", "Balance")
    }
}

private data class CategoryRow(
    val name: String,
    val expenses: Int,
    val total: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$expenses", "$total")

    companion object {
        val HEADERS = listOf("Category", "Expenses", "Total")
    }
}

private data class EntryRow(
    val name: String,
    val payer: String,
    val original: Money,
    val rate: ExchangeRate,
    val converted: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, payer, "$original", "$rate", "$converted")

    companion object {
        val HEADERS = listOf("Entry", "Paid by", "Entered as", "Rate", "In the group currency")
    }
}

internal data class PaymentRow(
    val sender: String,
    val receiver: String,
    val amount: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(sender, receiver, "$amount")

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

internal fun printBalances(group: ExpenseGroup) {
    val summary = group.summaryBlocking()
    printTable(
        BalanceRow.HEADERS,
        group.membersBlocking().map { member ->
            val totals = summary.member(member.id)
            BalanceRow(
                name = member.firstName,
                paid = totals.paid,
                consumed = totals.share,
                sent = totals.sent,
                received = totals.received,
                balance = totals.balance,
            )
        },
        numbersFrom = 1,
    )
}

internal fun printCategories(group: ExpenseGroup) {
    val expenses = group.expensesBlocking()
    printTable(
        CategoryRow.HEADERS,
        group.summaryBlocking().categoryTotals.map { total ->
            CategoryRow(
                name = group.categoryBlocking(total.category)?.name ?: "?",
                expenses = expenses.count { expense -> expense.category == total.category },
                total = total.total,
            )
        },
        numbersFrom = 1,
    )
}

/** Every expense and transfer in a currency [shown] accepts, as entered and in the group currency. */
internal fun printEntries(
    group: ExpenseGroup,
    shown: (CurrencyCode) -> Boolean,
) {
    val zero = Money.zero(group.currencyBlocking())
    val expenses =
        group.expensesBlocking().filter { expense -> shown(expense.cost.amount.currency) }.map { expense ->
            EntryRow(
                name = expense.name,
                payer = names(group, expense.payers.members),
                original = expense.total(),
                rate = rate(expense.cost),
                converted = group.expenseSharesBlocking(expense.id)?.total ?: zero,
            )
        }
    val transfers =
        group.transfersBlocking().filter { transfer -> shown(transfer.cost.amount.currency) }.map { transfer ->
            EntryRow(
                name = "Transfer to ${name(group, transfer.receiver)}",
                payer = name(group, transfer.sender),
                original = transfer.cost.amount,
                rate = rate(transfer.cost),
                converted = group.transferAmountBlocking(transfer.id) ?: zero,
            )
        }
    printTable(EntryRow.HEADERS, expenses + transfers, numbersFrom = 2)
}
