// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.
package eranos.examples.settlingup

import eranos.Balance
import eranos.CalendarDate
import eranos.Cost
import eranos.CurrencyCode
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseSplit
import eranos.Member
import eranos.MemberId
import eranos.Money
import eranos.SettlementPlan
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
    val name: String,
    val payer: String,
    val participants: String,
    val amount: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, payer, participants, "$amount")

    companion object {
        val HEADERS = listOf("Expense", "Paid by", "Shared by", "Amount")
    }
}

private data class BalanceRow(
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
    val from: String,
    val to: String,
    val amount: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(from, to, "$amount")

    companion object {
        val HEADERS = listOf("From", "To", "Amount")
    }
}

private data class StrategyRow(
    val strategy: String,
    val guarantee: String,
    val cost: String,
    val payments: Int,
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
    val minimum: Int,
    val greedy: Int,
    val direct: Int,
)

/** The name of [strategy] as the apps show it. */
private fun strategyName(strategy: SettlementStrategy): String =
    when (strategy) {
        SettlementStrategy.ADAPTIVE -> "Adaptive"
        SettlementStrategy.MINIMUM_TRANSFERS -> "MinimumTransfers"
        SettlementStrategy.GREEDY -> "Greedy"
        SettlementStrategy.DIRECT -> "Direct"
    }

/** Records an expense of [amount] EUR paid by [payer] and split equally. */
private fun spend(
    group: ExpenseGroup,
    name: String,
    amount: String,
    payer: MemberId,
    participants: List<MemberId>,
) {
    group.addBlocking(
        Expense(
            name,
            Cost.Local(Money.parse(amount, CurrencyCode.of("EUR"))),
            Split.Equal(listOf(payer)),
            ExpenseSplit.Whole(Split.Equal(participants)),
            CalendarDate.of(year = 2026, month = 9, day = 20),
        ),
    )
}

private fun printBalances(group: ExpenseGroup) {
    val summary = group.summaryBlocking()
    printTable(
        BalanceRow.HEADERS,
        group.membersBlocking().mapNotNull { member ->
            totals(member.id, summary)?.let { totals ->
                BalanceRow(name = member.firstName, paid = totals.paid, consumed = totals.share, balance = totals.balance)
            }
        },
        numbersFrom = 1,
    )
}

private fun printPayments(
    group: ExpenseGroup,
    settlement: SettlementPlan,
) {
    printTable(
        PaymentRow.HEADERS,
        settlement.payments.map { payment ->
            PaymentRow(from = names(group, listOf(payment.from)), to = names(group, listOf(payment.to)), amount = payment.amount)
        },
        numbersFrom = 2,
    )
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each. */
private fun strategies(group: ExpenseGroup): Counts {
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
                strategy = "MinimumTransfers",
                guarantee = "the fewest payments",
                cost = "O(2ⁿ·n) for n open balances",
                payments = minimum.payments.size,
            ),
            StrategyRow(strategy = "Greedy", guarantee = "at most n − 1 payments", cost = "O(n log n)", payments = greedy.payments.size),
            StrategyRow(strategy = "Direct", guarantee = "one payment per pair in debt", cost = "O(pairs)", payments = direct.payments.size),
            StrategyRow(
                strategy = "Adaptive",
                guarantee = "the fewest up to 20 balances",
                cost = "as MinimumTransfers, then Greedy",
                payments = adaptive.payments.size,
            ),
        ),
        numbersFrom = 3,
    )
    return Counts(
        adaptive = adaptive.payments.size,
        minimum = minimum.payments.size,
        greedy = greedy.payments.size,
        direct = direct.payments.size,
    )
}

/**
 * Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which strategy's
 * plan it took; returns the number of payments.
 */
private fun festival(): Int {
    section("8. A larger group")
    return ExpenseGroup("Festival", CurrencyCode.of("EUR")).use { group ->
        val friends = (1..21).map { number -> group.addBlocking(Member("Friend $number")) }
        // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
        // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
        for ((index, payer) in friends.withIndex()) {
            val number = index + 1L
            group.addBlocking(
                Expense(
                    "Round",
                    Cost.Local(Money(21 * number * number, CurrencyCode.of("EUR"))),
                    Split.Equal(listOf(payer)),
                    ExpenseSplit.Whole(Split.Equal(friends)),
                    CalendarDate.of(year = 2026, month = 9, day = 18),
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
    ExpenseGroup("Ski week", CurrencyCode.of("EUR")).use { group ->
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
                    name = expense.name,
                    payer = names(group, expense.payers.members),
                    participants = names(group, participants(expense.split)),
                    amount = expense.total(),
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
            group.addBlocking(Transfer(payment.from, payment.to, Cost.Local(payment.amount), CalendarDate.of(year = 2026, month = 9, day = 27)))
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
        check(Counts(adaptive = 4, minimum = 4, greedy = 5, direct = 7) == counts)
        // Greedy settles the festival's 21 balances in 19 payments.
        check(19 == festivalPayments)
        val summary = group.summaryBlocking()
        for (member in listOf(ana, bob, carol, dan, eva, filip)) {
            check(Balance.Settled == totals(member, summary)?.balance)
        }
    }
}
