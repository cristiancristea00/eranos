// Section 14 of the tour: the group shared between coroutines, a settlement computed with the suspending calls, and a
// change that waits for a report's read and is cancelled.
package eranos.examples.grandtour

import eranos.Cost
import eranos.CurrencyCode
import eranos.ExchangeRate
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseSplit
import eranos.SettlementStrategy
import eranos.Split
import eranos.examples.printTable
import eranos.examples.section
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch

internal suspend fun sharing(
    group: ExpenseGroup,
    people: People,
): Unit =
    coroutineScope {
        section("14. Sharing the group")
        // Another handle on the same group, as another coroutine holds: a group is shared as it is.
        val other = group
        val everyone = listOf(people.ana, people.bob, people.carol, people.dan, people.eva)
        val coffee =
            Expense(
                "Airport coffee",
                Cost.Local(ron("75.00")),
                Split.Equal(listOf(people.bob)),
                ExpenseSplit.Whole(Split.Equal(everyone)),
                october(8),
            )
        other.add(coffee)
        val plan = group.settlement(SettlementStrategy.MINIMUM_TRANSFERS)
        println(
            """
            An app shares the group between its tasks and threads through a
            SharedExpenseGroup, whose calls are futures any executor runs; here the library's
            block_on runs each one on this thread. Bob adds the airport coffee through another
            handle, and the plan that settles it comes through the first:
            """.trimIndent(),
        )
        val rows =
            group.read().use { reader ->
                plan.payments.map { payment ->
                    PaymentRow(
                        sender = reader.member(payment.sender)?.firstName ?: "?",
                        receiver = reader.member(payment.receiver)?.firstName ?: "?",
                        amount = payment.amount,
                    )
                }
            }
        printTable(PaymentRow.HEADERS, rows, numbersFrom = 2)

        val before = group.summary()
        val rates = mapOf(CurrencyCode.of("RON") to ExchangeRate.parse("0.2"))
        // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
        // behind the change, so the report reads through its own handle.
        val report = group.read()
        // Started at once, the change runs until it waits for the report's lock.
        val change = launch(start = CoroutineStart.UNDISPATCHED) { group.changeCurrency(CurrencyCode.of("EUR"), rates) }
        val reported = report.summary().totalSpent
        // Cancelling the coroutine cancels the waiting change, which leaves the group as it was.
        change.cancel()
        report.close()
        change.join()
        val after = group.summary()
        val currency = group.currency()
        println(
            """
            While a report reads the group, a change of its currency to EUR waits for the
            report. The app drops the waiting change, which cancels it: the group still
            keeps its books in $currency, and the trip cost ${after.totalSpent}, as the report read.
            """.trimIndent(),
        )
        println("With the other handle gone, the app takes the group back: ${group.expenses().size} expenses.")

        check(4 == plan.payments.size)
        check(plan.payments.all { payment -> people.bob == payment.receiver && ron("15.00") == payment.amount })
        check(change.isCancelled)
        check(before == after)
        check(reported == after.totalSpent)
        check(CurrencyCode.of("RON") == currency)
    }
