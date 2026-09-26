// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.
package eranos.examples.settlingup

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.Expense
import eranos.ExpenseSplit
import eranos.Group
import eranos.Member
import eranos.MemberId
import eranos.Money
import eranos.Settlement
import eranos.SettlementStrategy
import eranos.Split
import eranos.Transfer
import eranos.examples.names
import eranos.examples.participants
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

/** Records an expense of [amount] EUR paid by [payer] and split equally. */
private fun spend(
    group: Group,
    name: String,
    amount: String,
    payer: MemberId,
    participants: List<MemberId>,
) {
    group.addBlocking(
        Expense(
            Cost.Local(Money.parse(amount, Currency.of("EUR"))),
            CalendarDate.of(day = 20, month = 9, year = 2026),
            name,
            Split.Equal(listOf(payer)),
            ExpenseSplit.Whole(Split.Equal(participants)),
        ),
    )
}

private fun printBalances(group: Group) {
    val summary = group.summaryBlocking()
    printTable(
        listOf("Member", "Paid", "Consumed", "Balance"),
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id -> totals(id, summary) }?.let { totals ->
                listOf(member.firstName, "${totals.paid}", "${totals.share}", "${totals.balance}")
            }
        },
        numbersFrom = 1,
    )
}

private fun printPayments(
    group: Group,
    settlement: Settlement,
) {
    printTable(
        listOf("From", "To", "Amount"),
        settlement.payments.map { payment ->
            listOf(names(group, listOf(payment.from)), names(group, listOf(payment.to)), "${payment.amount}")
        },
        numbersFrom = 2,
    )
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each, in the order minimum, greedy, direct. */
private fun strategies(group: Group): List<Int> {
    section("3. Minimum transfers")
    val minimum = group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS)
    println(
        """
        The fewest payments possible. Finding them is NP-hard; the solver does it exactly by
        splitting the balances into as many groups that sum to zero as it can: here
        {Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than
        its size. ${minimum.payments.size} payments:
        """.trimIndent(),
    )
    printPayments(group, minimum)

    section("4. Greedy")
    val greedy = group.settlementBlocking(SettlementStrategy.GREEDY)
    println(
        """
        The largest debtor pays the largest creditor, again and again: fast, at most one
        payment fewer than the members with a balance, but not always the minimum. ${greedy.payments.size}
        payments here:
        """.trimIndent(),
    )
    printPayments(group, greedy)

    section("5. Direct")
    val direct = group.settlementBlocking(SettlementStrategy.DIRECT)
    println(
        """
        No simplification: everyone pays back the people who paid for them, with the debts
        between each pair of members netted. The most payments (${direct.payments.size}), but each one traces
        back to shared expenses:
        """.trimIndent(),
    )
    printPayments(group, direct)

    section("6. The strategies side by side")
    printTable(
        listOf("Strategy", "Guarantee", "Cost", "Payments here"),
        listOf(
            listOf("MinimumTransfers", "the fewest payments", "O(2ⁿ·n) for n open balances", "${minimum.payments.size}"),
            listOf("Greedy", "at most n − 1 payments", "O(n log n)", "${greedy.payments.size}"),
            listOf("Direct", "one payment per pair in debt", "O(pairs)", "${direct.payments.size}"),
        ),
        numbersFrom = 3,
    )
    return listOf(minimum.payments.size, greedy.payments.size, direct.payments.size)
}

/** Prints the report of the ski week. */
fun main() {
    Group(Currency.of("EUR"), "Ski week").use { group ->
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))
        val carol = group.addBlocking(Member("Carol"))
        val dan = group.addBlocking(Member("Dan"))
        val eva = group.addBlocking(Member("Eva"))
        val filip = group.addBlocking(Member("Filip"))

        println("Ski week: three ways to settle up")
        println("═════════════════════════════════")
        println(
            """
            Six friends spent a week in the mountains. The library can turn their balances
            into payments in three ways; the app picks one, and they differ here.
            """.trimIndent(),
        )

        section("1. What was spent")
        spend(group, "Cabin", "12.00", dan, listOf(dan, eva, filip))
        spend(group, "Lift passes", "9.00", bob, listOf(bob, carol, filip))
        spend(group, "Groceries", "6.00", carol, listOf(bob, dan))
        spend(group, "Coffee", "2.00", ana, listOf(dan, eva))
        spend(group, "Snacks", "1.00", filip, listOf(ana))
        printTable(
            listOf("Expense", "Paid by", "Shared by", "Amount"),
            group.expensesBlocking().map { expense ->
                listOf(expense.name, names(group, expense.payers.members), names(group, participants(expense.split)), "${expense.total()}")
            },
            numbersFrom = 3,
        )

        section("2. Balances")
        println("Four members are owed money and two owe it:")
        printBalances(group)

        val payments = strategies(group)

        section("7. Settling up for real")
        val plan = group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS)
        for (payment in plan.payments) {
            group.addBlocking(Transfer(Cost.Local(payment.amount), CalendarDate.of(day = 27, month = 9, year = 2026), payment.from, payment.to))
        }
        println(
            """
            Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards
            every balance is zero and a new settlement has nothing left to pay:
            """.trimIndent(),
        )
        printBalances(group)
        println("Payments still needed: ${group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS).payments.size}.")

        // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest
        // amounts first and needs five; the unsimplified debts between pairs need seven.
        check(listOf(4, 5, 7) == payments)
        val summary = group.summaryBlocking()
        for (member in listOf(ana, bob, carol, dan, eva, filip)) {
            check(Balance.Settled == totals(member, summary)?.balance)
        }
    }
}
