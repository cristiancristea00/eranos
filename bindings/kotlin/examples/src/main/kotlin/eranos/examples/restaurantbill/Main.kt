// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.
package eranos.examples.restaurantbill

import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.EranosException
import eranos.Expense
import eranos.ExpenseGroup
import eranos.ExpenseItem
import eranos.ExpenseShares
import eranos.ExpenseSplit
import eranos.Member
import eranos.MemberId
import eranos.MemberParts
import eranos.Money
import eranos.Parts
import eranos.Percentage
import eranos.Split
import eranos.Tip
import eranos.TipDistribution
import eranos.TipValue
import eranos.examples.Row
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.title

private data class LineRow(
    val name: String,
    val participants: String,
    val quantity: Int,
    val unitPrice: Money,
    val lineTotal: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, participants, "$quantity", "$unitPrice", "$lineTotal")

    companion object {
        val HEADERS = listOf("Item", "Shared by", "Quantity", "Unit price", "Line total")
    }
}

private data class TipRow(
    val name: String,
    val subtotal: Money,
    val proportionalTip: Money,
    val proportionalTotal: Money,
    val equalTip: Money,
    val equalTotal: Money,
) : Row {
    override val cells: List<String>
        get() = listOf(name, "$subtotal", "$proportionalTip", "$proportionalTotal", "$equalTip", "$equalTotal")

    companion object {
        val HEADERS = listOf("Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays")
    }
}

/** The amount written [text], in euros. */
private fun eur(text: String): Money = Money.parse(text, Currency.of("EUR"))

/** The receipt: every line with the members who had it. */
private fun receipt(
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
): List<ExpenseItem> =
    listOf(
        ExpenseItem("Ciorbă", eur("8.00"), Split.Equal(listOf(ana))),
        ExpenseItem("Sarmale", eur("14.00"), Split.Equal(listOf(bob, carol)), quantity = 2),
        ExpenseItem(
            "Papanași",
            eur("9.00"),
            Split.Parts(listOf(MemberParts(ana, Parts.ofHundredths(hundredths = 100)), MemberParts(carol, Parts.ofHundredths(hundredths = 200)))),
        ),
        ExpenseItem("Lemonade", eur("3.00"), Split.Equal(listOf(ana, bob, carol)), quantity = 3),
    )

/**
 * How [expense] would divide if it were added to [group], computed on a copy loaded from the group's saved form, so the
 * group itself is unchanged.
 */
private fun preview(
    group: ExpenseGroup,
    expense: Expense,
): ExpenseShares? = ExpenseGroup.loadBlocking(group.saveBlocking()).use { copy -> copy.expenseSharesBlocking(copy.addBlocking(expense)) }

/** What [member] consumed of the expense that [shares] divides. */
private fun consumed(
    shares: ExpenseShares?,
    member: MemberId,
): Money? = shares?.members?.firstOrNull { share -> share.member == member }?.share

/** Prints the report of the dinner. */
fun main() {
    ExpenseGroup("Dinner at Caru' cu Bere", Currency.of("EUR")).use { group ->
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))
        val carol = group.addBlocking(Member("Carol"))
        val day = CalendarDate.of(year = 2026, month = 9, day = 23)

        title("${group.nameBlocking()}: an itemised bill")
        println(
            """
            Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
            every line of the receipt is split among the people who had it.
            """.trimIndent(),
        )

        section("1. The receipt")
        printTable(
            LineRow.HEADERS,
            receipt(ana, bob, carol).map { item ->
                LineRow(
                    name = item.name,
                    participants = names(group, item.split.members),
                    quantity = item.quantity,
                    unitPrice = item.unitPrice,
                    lineTotal = Money(item.unitPrice.minorUnits * item.quantity, item.unitPrice.currency),
                )
            },
            numbersFrom = 2,
        )
        println(
            """
            The papanași are split by parts: Carol ate two of the three. The line totals add up
            to 54.00 EUR, and the expense's amount must match them exactly.
            """.trimIndent(),
        )

        section("2. Who ordered what, and the tip")
        // The dinner, its tip of 10 % shared by `distribution`.
        val bill = { distribution: TipDistribution ->
            Expense(
                "Dinner",
                Cost.Local(eur("54.00")),
                Split.Equal(listOf(ana)),
                ExpenseSplit.Itemised(receipt(ana, bob, carol)),
                day,
                tip = Tip(TipValue.Percentage(Percentage.parse("10")), distribution),
            )
        }
        val untipped = Expense("Dinner", Cost.Local(eur("54.00")), Split.Equal(listOf(ana)), ExpenseSplit.Itemised(receipt(ana, bob, carol)), day)
        val ordered = preview(group, untipped)
        val proportional = group.addBlocking(bill(TipDistribution.PROPORTIONAL))
        val byConsumption = group.expenseSharesBlocking(proportional)
        val even = preview(group, bill(TipDistribution.EQUAL))
        println(
            """
            Ana paid the bill plus a 10 % tip (${byConsumption?.tip ?: "?"}). A tip can be shared in proportion to what
            each person ordered, or equally; both are shown:
            """.trimIndent(),
        )
        val rows =
            listOf(ana, bob, carol).map { member ->
                val subtotal = checkNotNull(consumed(ordered, member)) { "every member ordered" }
                val proportionalTotal = checkNotNull(consumed(byConsumption, member)) { "every member ordered" }
                val equalTotal = checkNotNull(consumed(even, member)) { "every member ordered" }
                TipRow(
                    name = names(group, listOf(member)),
                    subtotal = subtotal,
                    proportionalTip = proportionalTotal - subtotal,
                    proportionalTotal = proportionalTotal,
                    equalTip = equalTotal - subtotal,
                    equalTotal = equalTotal,
                )
            }
        printTable(TipRow.HEADERS, rows, numbersFrom = 1)
        println(
            """
            By consumption, Carol, who ordered the most, adds the largest part of the tip;
            shared equally, everyone adds the same 1.80.
            """.trimIndent(),
        )

        section("3. A total that does not match is refused")
        try {
            group.addBlocking(
                Expense("Dinner", Cost.Local(eur("55.00")), Split.Equal(listOf(ana)), ExpenseSplit.Itemised(receipt(ana, bob, carol)), day),
            )
            error("the mismatch must be refused")
        } catch (refusal: EranosException) {
            println(
                """
                An amount of 55.00 EUR with these lines is refused when the expense is built:
                  error: ${refusal.message}
                """.trimIndent(),
            )
        }

        // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to
        // each; an equal one 1.80 to everyone.
        check(eur("15.40") == consumed(byConsumption, ana))
        check(eur("18.70") == consumed(byConsumption, bob))
        check(eur("25.30") == consumed(byConsumption, carol))
        check(eur("24.80") == consumed(even, carol))
        check(eur("59.40") == even?.total)
    }
}
