// Every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact
// amounts, and with several payers. The port of the Rust example `split_methods`, which prints the same report.
package eranos.examples.splitmethods

import eranos.CalendarDate
import eranos.Cost
import eranos.CurrencyCode
import eranos.EranosException
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseId
import eranos.ExpenseSplit
import eranos.Member
import eranos.MemberAmount
import eranos.MemberId
import eranos.MemberParts
import eranos.MemberPercentage
import eranos.Money
import eranos.Parts
import eranos.Percentage
import eranos.Split
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

private data class ShareRow(
    val name: String,
    val rule: String,
    val paid: Money,
    val consumed: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, rule, "$paid", "$consumed")

    companion object {
        val HEADERS = listOf("Member", "Rule", "Paid", "Consumed")
    }
}

private data class TotalRow(
    val name: String,
    val paid: Money,
    val consumed: Money,
    val balance: String,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$paid", "$consumed", balance)

    companion object {
        val HEADERS = listOf("Member", "Paid", "Consumed", "Balance")
    }
}

private fun eur(text: String): Money = Money.parse(text, CurrencyCode.of("EUR"))

/** Prints how one expense divides: every involved member's rule, what they paid, and what they consumed. */
private fun report(
    group: ExpenseGroup,
    id: ExpenseId,
    rules: Map<MemberId, String>,
) {
    val expense = group.expenseBlocking(id) ?: return
    val shares = group.expenseSharesBlocking(id) ?: return
    println("${expense.name}, ${shares.total}:")
    printTable(
        ShareRow.HEADERS,
        shares.members.map { share ->
            ShareRow(
                name = names(group, listOf(share.member)),
                rule = rules[share.member] ?: "(payer only)",
                paid = share.paid,
                consumed = share.share,
            )
        },
        numbersFrom = 2,
    )
}

/** The family and the day every expense happened. */
private class Family(
    val ana: MemberId,
    val bob: MemberId,
    val mia: MemberId,
    val day: CalendarDate,
) {
    /** The rule of an expense shared equally by the three. */
    val thirds: Map<MemberId, String>
        get() = mapOf(ana to "an equal third", bob to "an equal third", mia to "an equal third")
}

private fun equally(
    group: ExpenseGroup,
    family: Family,
) {
    section("1. Equally")
    println(
        """
        Everyone pays the same. 10.00 does not divide by three, so one member pays the
        leftover cent; which member it is rotates from expense to expense, so over a trip
        nobody always pays it.
        """.trimIndent(),
    )
    val taxi =
        group.addBlocking(
            Expense(
                "Taxi",
                Cost.Local(eur("10.00")),
                Split.Equal(listOf(family.ana)),
                ExpenseSplit.Whole(Split.Equal(listOf(family.ana, family.bob, family.mia))),
                family.day,
            ),
        )
    report(group, taxi, family.thirds)
}

private fun byPercentage(
    group: ExpenseGroup,
    family: Family,
) {
    section("2. By percentage")
    println(
        """
        Everyone pays a percentage of the amount; the percentages must add up to exactly
        100 %. Mia's room is smaller, so she takes 20 %.
        """.trimIndent(),
    )
    val rent =
        group.addBlocking(
            Expense(
                "Flat",
                Cost.Local(eur("500.00")),
                Split.Equal(listOf(family.bob)),
                ExpenseSplit.Whole(
                    Split.Percentages(
                        listOf(
                            MemberPercentage(family.ana, Percentage.parse("40")),
                            MemberPercentage(family.bob, Percentage.parse("40")),
                            MemberPercentage(family.mia, Percentage.parse("20")),
                        ),
                    ),
                ),
                family.day,
            ),
        )
    report(group, rent, mapOf(family.ana to "40%", family.bob to "40%", family.mia to "20%"))
}

private fun byParts(
    group: ExpenseGroup,
    family: Family,
): ExpenseId {
    section("3. By parts")
    println(
        """
        Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:
        2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.
        """.trimIndent(),
    )
    val food =
        group.addBlocking(
            Expense(
                "Food",
                Cost.Local(eur("125.00")),
                Split.Equal(listOf(family.ana)),
                ExpenseSplit.Whole(
                    Split.Parts(
                        listOf(
                            MemberParts(family.ana, Parts.parse("1")),
                            MemberParts(family.bob, Parts.parse("1")),
                            MemberParts(family.mia, Parts.parse("0.5")),
                        ),
                    ),
                ),
                family.day,
            ),
        )
    report(group, food, mapOf(family.ana to "1 part", family.bob to "1 part", family.mia to "0.5 parts"))
    return food
}

private fun byAmounts(
    group: ExpenseGroup,
    family: Family,
) {
    section("4. By exact amounts")
    println(
        """
        Everyone pays a fixed amount, such as a ticket price; the amounts must add up to
        exactly the expense amount.
        """.trimIndent(),
    )
    val tickets =
        group.addBlocking(
            Expense(
                "Museum",
                Cost.Local(eur("35.00")),
                Split.Equal(listOf(family.bob)),
                ExpenseSplit.Whole(
                    Split.Amounts(
                        listOf(
                            MemberAmount(family.ana, eur("15.00")),
                            MemberAmount(family.bob, eur("15.00")),
                            MemberAmount(family.mia, eur("5.00")),
                        ),
                    ),
                ),
                family.day,
            ),
        )
    report(group, tickets, mapOf(family.ana to "15.00 EUR", family.bob to "15.00 EUR", family.mia to "5.00 EUR"))
}

private fun severalPayers(
    group: ExpenseGroup,
    family: Family,
) {
    section("5. Several payers")
    println(
        """
        The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are
        divided with the same methods as the participants.
        """.trimIndent(),
    )
    val dinner =
        group.addBlocking(
            Expense(
                "Dinner",
                Cost.Local(eur("90.00")),
                Split.Amounts(listOf(MemberAmount(family.ana, eur("50.00")), MemberAmount(family.bob, eur("40.00")))),
                ExpenseSplit.Whole(Split.Equal(listOf(family.ana, family.bob, family.mia))),
                family.day,
            ),
        )
    report(group, dinner, family.thirds)
}

private fun mistakes(
    group: ExpenseGroup,
    family: Family,
) {
    section("6. Mistakes are refused")
    println("A percentage split that adds up to 90 % is refused when the expense is built:")
    val invalid =
        Expense(
            "Invalid",
            Cost.Local(eur("10.00")),
            Split.Equal(listOf(family.ana)),
            ExpenseSplit.Whole(
                Split.Percentages(
                    listOf(MemberPercentage(family.ana, Percentage.parse("60")), MemberPercentage(family.bob, Percentage.parse("30"))),
                ),
            ),
            family.day,
        )
    try {
        group.addBlocking(invalid)
        error("the split must be refused")
    } catch (refusal: EranosException) {
        println("  error: ${refusal.message}")
    }
}

private fun printTotals(group: ExpenseGroup) {
    section("7. The holiday in total")
    val summary = group.summaryBlocking()
    println("${summary.totalSpent} spent over five expenses:")
    printTable(
        TotalRow.HEADERS,
        group.membersBlocking().mapNotNull { member ->
            totals(member.id, summary)?.let { totals ->
                TotalRow(name = member.firstName, paid = totals.paid, consumed = totals.share, balance = "${totals.balance}")
            }
        },
        numbersFrom = 1,
    )
}

/** Prints the report of the family holiday. */
fun main() {
    ExpenseGroup("Family holiday", CurrencyCode.of("EUR")).use { group ->
        val family =
            Family(
                ana = group.addBlocking(Member("Ana")),
                bob = group.addBlocking(Member("Bob")),
                mia = group.addBlocking(Member("Mia")),
                day = CalendarDate.of(year = 2026, month = 7, day = 14),
            )

        println("Family holiday: every way to split an expense")
        println("═════════════════════════════════════════════")
        println(
            """
            Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a
            different split method; whatever the method, the shares add up to exactly the
            amount, to the cent.
            """.trimIndent(),
        )
        equally(group, family)
        byPercentage(group, family)
        val food = byParts(group, family)
        byAmounts(group, family)
        severalPayers(group, family)
        mistakes(group, family)
        printTotals(group)

        // Parts: 125.00 over 2.5 parts is 50.00 per part.
        val shares = group.expenseSharesBlocking(food)
        check(eur("25.00") == shares?.members?.firstOrNull { share -> share.member == family.mia }?.share)
    }
}
