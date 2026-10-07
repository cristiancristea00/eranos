// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example
// `multi_currency`, which prints the same report.
package eranos.examples.multicurrency

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.ExchangeRate
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseSplit
import eranos.Member
import eranos.MemberId
import eranos.Money
import eranos.Split
import eranos.Transfer
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

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

/** The first name of [member] in [group], or `?` for one the group does not hold. */
private fun name(
    group: ExpenseGroup,
    member: MemberId,
): String = names(group, listOf(member))

/** Every expense and transfer, as entered and in the group currency. */
private fun printEntries(group: ExpenseGroup) {
    val zero = Money(0, group.currencyBlocking())
    val expenses =
        group.expensesBlocking().map { expense ->
            EntryRow(
                name = expense.name,
                payer = names(group, expense.payers.members),
                original = expense.total(),
                rate = rate(expense.cost),
                converted = expense.id?.let { id -> group.expenseSharesBlocking(id) }?.total ?: zero,
            )
        }
    val transfers =
        group.transfersBlocking().map { transfer ->
            EntryRow(
                name = "Transfer to ${name(group, transfer.to)}",
                payer = name(group, transfer.from),
                original = transfer.cost.amount,
                rate = rate(transfer.cost),
                converted = transfer.id?.let { id -> group.transferAmountBlocking(id) } ?: zero,
            )
        }
    printTable(EntryRow.HEADERS, expenses + transfers, numbersFrom = 2)
}

/** The rate [cost] was entered at: 1 for a cost in the group currency. */
private fun rate(cost: Cost): ExchangeRate =
    when (cost) {
        is Cost.Local -> ExchangeRate.parse("1")
        is Cost.Converted -> cost.rate
    }

private fun printBalances(group: ExpenseGroup) {
    val summary = group.summaryBlocking()
    printTable(
        BalanceRow.HEADERS,
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id -> totals(id, summary) }?.let { totals ->
                BalanceRow(
                    name = member.firstName,
                    paid = totals.paid,
                    consumed = totals.share,
                    sent = totals.sent,
                    received = totals.received,
                    balance = totals.balance,
                )
            }
        },
        numbersFrom = 1,
    )
}

private fun expenses(
    group: ExpenseGroup,
    ana: MemberId,
    bob: MemberId,
) {
    section("1. Expenses in three currencies")
    val both = ExpenseSplit.Whole(Split.Equal(listOf(ana, bob)))
    group.addBlocking(
        Expense(
            "Fuel",
            Cost.Converted(Money.parse("400.00", Currency.of("RON")), ExchangeRate.parse("0.2")),
            Split.Equal(listOf(ana)),
            both,
            CalendarDate.of(year = 2026, month = 8, day = 1),
        ),
    )
    group.addBlocking(
        Expense(
            "Dinner",
            Cost.Converted(Money.parse("24000.00", Currency.of("HUF")), ExchangeRate.parse("0.0025")),
            Split.Equal(listOf(bob)),
            both,
            CalendarDate.of(year = 2026, month = 8, day = 2),
        ),
    )
    group.addBlocking(
        Expense(
            "Ferry",
            Cost.Local(Money.parse("60.00", Currency.of("EUR"))),
            Split.Equal(listOf(bob)),
            both,
            CalendarDate.of(year = 2026, month = 8, day = 3),
        ),
    )
    printEntries(group)
    println(
        """
        The rate says how many ${group.currencyBlocking()} one unit of the entry's currency is worth: 400.00 RON at
        0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.
        """.trimIndent(),
    )
}

private fun repayment(
    group: ExpenseGroup,
    ana: MemberId,
    bob: MemberId,
) {
    section("3. A repayment in cash, in another currency")
    group.addBlocking(
        Transfer(
            ana,
            bob,
            Cost.Converted(Money.parse("100.00", Currency.of("RON")), ExchangeRate.parse("0.2")),
            CalendarDate.of(year = 2026, month = 8, day = 4),
            note = "Cash at the border",
        ),
    )
    println(
        """
        Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.
        Transfers move balances but are not spending, so the total stays at ${group.summaryBlocking().totalSpent}:
        """.trimIndent(),
    )
    printBalances(group)
}

private fun newCurrency(group: ExpenseGroup) {
    section("4. Moving the books to another currency")
    println(
        """
        Changing the group currency needs a rate into the new currency for every other
        currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies
        take the new rate; entries already in RON become local costs, at 1.
        """.trimIndent(),
    )
    group.changeCurrencyBlocking(
        Currency.of("RON"),
        mapOf(Currency.of("EUR") to ExchangeRate.parse("5"), Currency.of("HUF") to ExchangeRate.parse("0.0125")),
    )
    printEntries(group)
    println("In ${group.currencyBlocking()}, the trip cost ${group.summaryBlocking().totalSpent}, and everyone is still settled:")
    printBalances(group)
}

/** Prints the report of the road trip across currencies. */
fun main() {
    val euro = Currency.of("EUR")
    val leu = Currency.of("RON")
    ExpenseGroup("Danube road trip", euro).use { group ->
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))

        println("${group.nameBlocking()}: expenses in several currencies")
        println("════════════════════════════════════════════════")
        println(
            """
            Ana and Bob drive from Romania to Hungary. The group keeps its books in ${group.currencyBlocking()}; an
            expense in another currency carries the exchange rate used when it was entered,
            and is converted once, rounding half away from zero to the cent.
            """.trimIndent(),
        )
        expenses(group, ana, bob)

        section("2. Balances")
        println(
            """
            The group spent ${group.summaryBlocking().totalSpent}. Bob paid 120.00 of it but consumed half, so he gets back the
            difference:
            """.trimIndent(),
        )
        printBalances(group)
        check(Balance.GetsBack(Money.parse("20.00", euro)) == totals(bob, group.summaryBlocking())?.balance)

        repayment(group, ana, bob)
        check(Balance.Settled == totals(bob, group.summaryBlocking())?.balance)

        newCurrency(group)
        // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
        val summary = group.summaryBlocking()
        check(Money.parse("1000.00", leu) == summary.totalSpent)
        check(Balance.Settled == totals(bob, summary)?.balance)
    }
}
