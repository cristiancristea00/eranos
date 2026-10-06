// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.
package eranos.examples.persistence

import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.EranosException
import eranos.Expense
import eranos.ExpenseSplit
import eranos.Group
import eranos.Member
import eranos.Money
import eranos.Percentage
import eranos.SettlementStrategy
import eranos.Split
import eranos.Tip
import eranos.TipValue
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.title
import eranos.examples.totals

private data class ComparisonRow(
    val after: String,
    val before: String,
    val name: String,
) : Row {
    override val cells: List<String>
        get() = listOf(name, before, after)

    companion object {
        val HEADERS = listOf("Member", "Balance before saving", "Balance after loading")
    }
}

/** Prints the report of the road trip saved and loaded. */
fun main() {
    val euro = Currency.of("EUR")
    val day = CalendarDate.of(day = 23, month = 9, year = 2026)
    Group(euro, "Road trip").use { group ->
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))
        val both = ExpenseSplit.Whole(Split.Equal(listOf(ana, bob)))
        group.addBlocking(Expense(Cost.Local(Money.parse("70.00", euro)), day, "Fuel", Split.Equal(listOf(ana)), both))
        group.addBlocking(
            Expense(
                Cost.Local(Money.parse("30.00", euro)),
                day,
                "Lunch",
                Split.Equal(listOf(bob)),
                both,
                tip = Tip(TipValue.Percentage(Percentage.parse("10"))),
            ),
        )

        title("Road trip: saving and loading a group")
        println(
            """
            With the `serde` feature a group serialises, and so does every type it stores.
            It saves only what was entered: its details, members, categories, expenses,
            and transfers. The totals, balances, and settlements are recomputed when it is
            loaded.
            """.trimIndent(),
        )

        section("1. The saved form")
        // Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an
        // encrypted database).
        val saved = group.saveBlocking()
        println(saved)
        println(
            """
            Amounts are minor units with their currency, dates are YYYY-MM-DD, and the
            `format` number lets a future release refuse a file it cannot read instead of
            dropping fields.
            """.trimIndent(),
        )

        section("2. Loading it back")
        Group.loadBlocking(saved).use { loaded ->
            println(
                """
                Loading replays every entry through the same checks as adding it, then rebuilds
                the totals. The loaded group equals the saved one, and so do its balances:
                """.trimIndent(),
            )
            val before = group.summaryBlocking()
            val after = loaded.summaryBlocking()
            printTable(
                ComparisonRow.HEADERS,
                listOf(ana, bob).map { member ->
                    ComparisonRow(
                        after = totals(member, after)?.balance?.toString() ?: "?",
                        before = totals(member, before)?.balance?.toString() ?: "?",
                        name = names(group, listOf(member)),
                    )
                },
                numbersFrom = 1,
            )
            val plan = loaded.settlementBlocking(SettlementStrategy.GREEDY)
            println("Payments needed to settle the loaded group: ${plan.payments.size}.")

            section("3. A corrupted file is refused")
            println(
                """
                Here the fuel's payer was edited to an identifier no member has. Reading the file
                as a SavedGroup and loading it with into_group replays every entry through the
                same checks as adding it, and refuses the file instead of producing balances that
                do not add up:
                """.trimIndent(),
            )
            // The fuel is the first entry that names Ana: the expenses come before the members in the saved form.
            val corrupted = saved.replaceFirst(ana.value, "0199a000-0000-7000-8000-00000000ffff")
            try {
                Group.loadBlocking(corrupted).close()
                error("the file must be refused")
            } catch (refusal: EranosException) {
                println("  error: ${refusal.message}")
            }

            check(saved == loaded.saveBlocking())
            check(before == after)
            check(1 == plan.payments.size)
        }
    }
}
