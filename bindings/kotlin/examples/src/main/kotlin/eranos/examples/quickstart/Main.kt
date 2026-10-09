// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.
package eranos.examples.quickstart

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.CurrencyCode
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseSplit
import eranos.Member
import eranos.Money
import eranos.SettlementStrategy
import eranos.Split
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.title

private data class ExpenseRow(
    val name: String,
    val payers: String,
    val participants: String,
    val amount: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, payers, participants, "$amount")

    companion object {
        val HEADERS = listOf("Expense", "Paid by", "Shared by", "Amount")
    }
}

private data class MemberRow(
    val name: String,
    val paid: Money,
    val consumed: Money,
    val balance: Balance,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$paid", "$consumed", "$balance")

    companion object {
        val HEADERS = listOf("Member", "Paid", "Consumed", "Balance")
    }
}

private data class PaymentRow(
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

/** Prints the report of the weekend trip. */
fun main() {
    val euro = CurrencyCode.of("EUR")
    ExpenseGroup("Weekend in Sinaia", euro).use { group ->
        title(group.nameBlocking())
        println(
            """
            Three friends rent a cabin and buy groceries. The group keeps its books in ${group.currencyBlocking()}, so
            every total, balance, and payment below is in ${group.currencyBlocking()}.
            """.trimIndent(),
        )

        section("1. Members")
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))
        val carol = group.addBlocking(Member("Carol"))
        println("${group.membersBlocking().size} people joined the group: ${names(group, listOf(ana, bob, carol))}.")

        section("2. Expenses")
        println(
            """
            Each expense records who paid and who shares it. Here everyone shares everything
            equally.
            """.trimIndent(),
        )
        val friday = CalendarDate.of(year = 2026, month = 9, day = 25)
        val everyone = Split.Equal(listOf(ana, bob, carol))
        group.addBlocking(
            Expense("Cabin", Cost.Local(Money.parse("240.00", euro)), Split.Equal(listOf(ana)), ExpenseSplit.Whole(everyone), friday),
        )
        group.addBlocking(
            Expense("Groceries", Cost.Local(Money.parse("60.00", euro)), Split.Equal(listOf(bob)), ExpenseSplit.Whole(everyone), friday),
        )
        printTable(
            ExpenseRow.HEADERS,
            group.expensesBlocking().map { expense ->
                ExpenseRow(
                    name = expense.name,
                    payers = names(group, expense.payers.members),
                    participants = names(group, expense.split.participants()),
                    amount = expense.total(),
                )
            },
            numbersFrom = 3,
        )

        section("3. Who paid and who consumed")
        val summary = group.summaryBlocking()
        println(
            """
            The group spent ${summary.totalSpent} in total. A member's share of an expense is what they consumed,
            and their balance is what they paid minus what they consumed.
            """.trimIndent(),
        )
        printTable(
            MemberRow.HEADERS,
            group.membersBlocking().map { member ->
                val totals = summary.member(member.id)
                MemberRow(name = member.firstName, paid = totals.paid, consumed = totals.share, balance = totals.balance)
            },
            numbersFrom = 1,
        )
        val anaTotals = summary.member(ana)
        println("Ana paid ${anaTotals.paid} but consumed only ${anaTotals.share}, so the others owe her the difference.")

        section("4. Settling up")
        val settlement = group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS)
        println("The fewest payments that bring every balance to zero (${settlement.payments.size} in total):")
        printTable(
            PaymentRow.HEADERS,
            settlement.payments.map { payment ->
                PaymentRow(sender = names(group, listOf(payment.sender)), receiver = names(group, listOf(payment.receiver)), amount = payment.amount)
            },
            numbersFrom = 2,
        )
        println("Once these payments are made, nobody owes anything.")

        // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
        check(Money.parse("300.00", euro) == summary.totalSpent)
        check(Balance.GetsBack(Money.parse("140.00", euro)) == anaTotals.balance)
        check(2 == settlement.payments.size)
    }
}
