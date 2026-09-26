// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.
package eranos.examples.restaurantbill

import eranos.CalendarDate
import eranos.Cost
import eranos.Currency
import eranos.EranosException
import eranos.Expense
import eranos.ExpenseShares
import eranos.ExpenseSplit
import eranos.Group
import eranos.Item
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
import eranos.examples.names
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.title

/** The amount written [text], in euros. */
private fun eur(text: String): Money = Money.parse(text, Currency.of("EUR"))

/** The receipt: every line with the members who had it. */
private fun receipt(
    ana: MemberId,
    bob: MemberId,
    carol: MemberId,
): List<Item> =
    listOf(
        Item("Ciorbă", Split.Equal(listOf(ana)), eur("8.00")),
        Item("Sarmale", Split.Equal(listOf(bob, carol)), eur("14.00"), quantity = 2),
        Item(
            "Papanași",
            Split.Parts(listOf(MemberParts(ana, Parts.ofHundredths(hundredths = 100)), MemberParts(carol, Parts.ofHundredths(hundredths = 200)))),
            eur("9.00"),
        ),
        Item("Lemonade", Split.Equal(listOf(ana, bob, carol)), eur("3.00"), quantity = 3),
    )

/**
 * How [expense] would divide if it were added to [group], computed on a copy loaded from the group's saved form, so the
 * group itself is unchanged.
 */
private fun preview(
    group: Group,
    expense: Expense,
): ExpenseShares? = Group.loadBlocking(group.saveBlocking()).use { copy -> copy.expenseSharesBlocking(copy.addBlocking(expense)) }

/** What [member] consumed of the expense that [shares] divides. */
private fun consumed(
    shares: ExpenseShares?,
    member: MemberId,
): Money? = shares?.members?.firstOrNull { share -> share.member == member }?.share

/** Prints the report of the dinner. */
fun main() {
    Group(Currency.of("EUR"), "Dinner at Caru' cu Bere").use { group ->
        val ana = group.addBlocking(Member("Ana"))
        val bob = group.addBlocking(Member("Bob"))
        val carol = group.addBlocking(Member("Carol"))
        val day = CalendarDate.of(day = 23, month = 9, year = 2026)

        title("${group.nameBlocking()}: an itemised bill")
        println(
            """
            Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
            every line of the receipt is split among the people who had it.
            """.trimIndent(),
        )

        section("1. The receipt")
        printTable(
            listOf("Item", "Shared by", "Quantity", "Unit price", "Line total"),
            receipt(ana, bob, carol).map { item ->
                val total = Money(item.unitPrice.currency, item.unitPrice.minorUnits * item.quantity)
                listOf(item.name, names(group, item.split.members), "${item.quantity}", "${item.unitPrice}", "$total")
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
                Cost.Local(eur("54.00")),
                day,
                "Dinner",
                Split.Equal(listOf(ana)),
                ExpenseSplit.Itemised(receipt(ana, bob, carol)),
                tip = Tip(TipValue.Percentage(Percentage.parse("10")), distribution),
            )
        }
        val untipped = Expense(Cost.Local(eur("54.00")), day, "Dinner", Split.Equal(listOf(ana)), ExpenseSplit.Itemised(receipt(ana, bob, carol)))
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
            listOf(ana to "Ana", bob to "Bob", carol to "Carol").map { (member, firstName) ->
                val subtotal = checkNotNull(consumed(ordered, member)) { "every member ordered" }
                val proportionalTotal = checkNotNull(consumed(byConsumption, member)) { "every member ordered" }
                val equalTotal = checkNotNull(consumed(even, member)) { "every member ordered" }
                listOf(
                    firstName,
                    "$subtotal",
                    "${proportionalTotal - subtotal}",
                    "$proportionalTotal",
                    "${equalTotal - subtotal}",
                    "$equalTotal",
                )
            }
        printTable(listOf("Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"), rows, numbersFrom = 1)
        println(
            """
            By consumption, Carol, who ordered the most, adds the largest part of the tip;
            shared equally, everyone adds the same 1.80.
            """.trimIndent(),
        )

        section("3. A total that does not match is refused")
        try {
            group.addBlocking(
                Expense(Cost.Local(eur("55.00")), day, "Dinner", Split.Equal(listOf(ana)), ExpenseSplit.Itemised(receipt(ana, bob, carol))),
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
