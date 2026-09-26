// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.
package eranos.examples.quickstart

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.Expense
import eranos.ExpenseSplit
import eranos.Group
import eranos.Member
import eranos.Money
import eranos.SettlementStrategy
import eranos.Split
import eranos.examples.names
import eranos.examples.participants
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.title
import eranos.examples.totals

/** Prints the report of the weekend trip. */
fun main() {
    val euro = Currency.of("EUR")
    Group(euro, "Weekend in Sinaia").use { group ->
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
        val friday = CalendarDate.of(day = 25, month = 9, year = 2026)
        val everyone = Split.Equal(listOf(ana, bob, carol))
        group.addBlocking(
            Expense(Cost.Local(Money.parse("240.00", euro)), friday, "Cabin", Split.Equal(listOf(ana)), ExpenseSplit.Whole(everyone)),
        )
        group.addBlocking(
            Expense(Cost.Local(Money.parse("60.00", euro)), friday, "Groceries", Split.Equal(listOf(bob)), ExpenseSplit.Whole(everyone)),
        )
        printTable(
            listOf("Expense", "Paid by", "Shared by", "Amount"),
            group.expensesBlocking().map { expense ->
                listOf(expense.name, names(group, expense.payers.members), names(group, participants(expense.split)), "${expense.total()}")
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
            listOf("Member", "Paid", "Consumed", "Balance"),
            group.membersBlocking().mapNotNull { member ->
                member.id?.let { id -> totals(id, summary) }?.let { totals ->
                    listOf(member.firstName, "${totals.paid}", "${totals.share}", "${totals.balance}")
                }
            },
            numbersFrom = 1,
        )
        val anaTotals = totals(ana, summary)
        println("Ana paid ${anaTotals?.paid ?: "?"} but consumed only ${anaTotals?.share ?: "?"}, so the others owe her the difference.")

        section("4. Settling up")
        val settlement = group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS)
        println("The fewest payments that bring every balance to zero (${settlement.payments.size} in total):")
        printTable(
            listOf("From", "To", "Amount"),
            settlement.payments.map { payment ->
                listOf(names(group, listOf(payment.from)), names(group, listOf(payment.to)), "${payment.amount}")
            },
            numbersFrom = 2,
        )
        println("Once these payments are made, nobody owes anything.")

        // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
        check(Money.parse("300.00", euro) == summary.totalSpent)
        check(Balance.GetsBack(Money.parse("140.00", euro)) == anaTotals?.balance)
        check(2 == settlement.payments.size)
    }
}
