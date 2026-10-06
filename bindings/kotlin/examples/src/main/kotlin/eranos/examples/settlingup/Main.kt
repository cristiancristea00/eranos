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
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.participants
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

private data class ExpenseRow(
    val amount: Money,
    val name: String,
    val participants: String,
    val payer: String,
) : Row {
    override val cells: List<String>
        get() = listOf(name, payer, participants, "$amount")

    companion object {
        val HEADERS = listOf("Expense", "Paid by", "Shared by", "Amount")
    }
}

private data class BalanceRow(
    val balance: Balance,
    val consumed: Money,
    val name: String,
    val paid: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$paid", "$consumed", "$balance")

    companion object {
        val HEADERS = listOf("Member", "Paid", "Consumed", "Balance")
    }
}

private data class PaymentRow(
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

private data class StrategyRow(
    val cost: String,
    val guarantee: String,
    val payments: Int,
    val strategy: String,
) : Row {
    override val cells: List<String>
        get() = listOf(strategy, guarantee, cost, "$payments")

    companion object {
        val HEADERS = listOf("Strategy", "Guarantee", "Cost", "Payments here")
    }
}

/** The number of payments of each strategy's plan. */
private data class Counts(
    val adaptive: Int,
    val direct: Int,
    val greedy: Int,
    val minimum: Int,
)

/** The name of [strategy] as the apps show it. */
private fun strategyName(strategy: SettlementStrategy): String =
    when (strategy) {
        SettlementStrategy.ADAPTIVE -> "Adaptive"
        SettlementStrategy.DIRECT -> "Direct"
        SettlementStrategy.GREEDY -> "Greedy"
        SettlementStrategy.MINIMUM_TRANSFERS -> "MinimumTransfers"
    }

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
        BalanceRow.HEADERS,
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id -> totals(id, summary) }?.let { totals ->
                BalanceRow(balance = totals.balance, consumed = totals.share, name = member.firstName, paid = totals.paid)
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
        PaymentRow.HEADERS,
        settlement.payments.map { payment ->
            PaymentRow(amount = payment.amount, from = names(group, listOf(payment.from)), to = names(group, listOf(payment.to)))
        },
        numbersFrom = 2,
    )
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each. */
private fun strategies(group: Group): Counts {
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

    section("6. Adaptive")
    val adaptive = group.settlementBlocking(SettlementStrategy.ADAPTIVE)
    println(
        """
        The fewest payments while they are quick to find, greedy's beyond: up to 20 open
        balances the minimum solver settles them, past that greedy does, and the plan
        names the strategy whose plan it is. Six balances are well within the limit, so
        the plan is the one ${strategyName(adaptive.strategy)} gives, the same ${adaptive.payments.size} payments.
        """.trimIndent(),
    )

    section("7. The strategies side by side")
    printTable(
        StrategyRow.HEADERS,
        listOf(
            StrategyRow(
                cost = "O(2ⁿ·n) for n open balances",
                guarantee = "the fewest payments",
                payments = minimum.payments.size,
                strategy = "MinimumTransfers",
            ),
            StrategyRow(cost = "O(n log n)", guarantee = "at most n − 1 payments", payments = greedy.payments.size, strategy = "Greedy"),
            StrategyRow(cost = "O(pairs)", guarantee = "one payment per pair in debt", payments = direct.payments.size, strategy = "Direct"),
            StrategyRow(
                cost = "as MinimumTransfers, then Greedy",
                guarantee = "the fewest up to 20 balances",
                payments = adaptive.payments.size,
                strategy = "Adaptive",
            ),
        ),
        numbersFrom = 3,
    )
    return Counts(
        adaptive = adaptive.payments.size,
        direct = direct.payments.size,
        greedy = greedy.payments.size,
        minimum = minimum.payments.size,
    )
}

/**
 * Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which strategy's
 * plan it took; returns the number of payments.
 */
private fun festival(): Int {
    section("8. A larger group")
    return Group(Currency.of("EUR"), "Festival").use { group ->
        val friends = (1..21).map { number -> group.addBlocking(Member("Friend $number")) }
        // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
        // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
        for ((index, payer) in friends.withIndex()) {
            val number = index + 1L
            group.addBlocking(
                Expense(
                    Cost.Local(Money(Currency.of("EUR"), 21 * number * number)),
                    CalendarDate.of(day = 18, month = 9, year = 2026),
                    "Round",
                    Split.Equal(listOf(payer)),
                    ExpenseSplit.Whole(Split.Equal(friends)),
                ),
            )
        }
        val open = group.summaryBlocking().members.count { totals -> 0L != totals.net.minorUnits }
        val adaptive = group.settlementBlocking(SettlementStrategy.ADAPTIVE)
        val greedy = group.settlementBlocking(SettlementStrategy.GREEDY)
        println(
            """
            Twenty-one friends at a festival each bought one round for everyone, each round
            at another price, so all $open end with a balance and no two of them cancel out.
            That is more than the 20 the adaptive strategy settles exactly, so it takes the
            plan ${strategyName(adaptive.strategy)} gives and says so: the same ${adaptive.payments.size} payments.
            """.trimIndent(),
        )
        check(21 == open)
        check(SettlementStrategy.GREEDY == adaptive.strategy)
        check(adaptive.payments == greedy.payments)
        adaptive.payments.size
    }
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

        println("Ski week: four ways to settle up")
        println("════════════════════════════════")
        println(
            """
            Six friends spent a week in the mountains. The library can turn their balances
            into payments in four ways, and the app picks one.
            """.trimIndent(),
        )

        section("1. What was spent")
        spend(group, "Cabin", "12.00", dan, listOf(dan, eva, filip))
        spend(group, "Lift passes", "9.00", bob, listOf(bob, carol, filip))
        spend(group, "Groceries", "6.00", carol, listOf(bob, dan))
        spend(group, "Coffee", "2.00", ana, listOf(dan, eva))
        spend(group, "Snacks", "1.00", filip, listOf(ana))
        printTable(
            ExpenseRow.HEADERS,
            group.expensesBlocking().map { expense ->
                ExpenseRow(
                    amount = expense.total(),
                    name = expense.name,
                    participants = names(group, participants(expense.split)),
                    payer = names(group, expense.payers.members),
                )
            },
            numbersFrom = 3,
        )

        section("2. Balances")
        println("Four members are owed money and two owe it:")
        printBalances(group)

        val counts = strategies(group)
        val festivalPayments = festival()

        section("9. Settling up for real")
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

        // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that
        // one; greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
        check(Counts(adaptive = 4, direct = 7, greedy = 5, minimum = 4) == counts)
        // Greedy settles the festival's 21 balances in 19 payments.
        check(19 == festivalPayments)
        val summary = group.summaryBlocking()
        for (member in listOf(ana, bob, carol, dan, eva, filip)) {
            check(Balance.Settled == totals(member, summary)?.balance)
        }
    }
}
