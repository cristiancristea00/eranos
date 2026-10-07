// The expenses of section 4 of the tour, one for every way of dividing one, as an app builds them before the group adds
// them.
package eranos.examples.grandtour

import eranos.Cost
import eranos.Expense
import eranos.ExpenseSplit
import eranos.Item
import eranos.MemberAmount
import eranos.MemberParts
import eranos.MemberPercentage
import eranos.Parts
import eranos.Percentage
import eranos.Split
import eranos.TimeOfDay
import eranos.Tip
import eranos.TipDistribution
import eranos.TipValue

/** The groceries, shared equally by Ana, Bob, and Carol. */
internal fun groceriesEqually(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Groceries",
        Cost.Local(eur("10.00")),
        Split.Equal(listOf(people.ana)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol))),
        october(1),
        category = categories.food,
    )

/** The apartment, divided by percentage. */
internal fun apartmentByPercentage(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Apartment",
        Cost.Local(eur("800.00")),
        Split.Equal(listOf(people.ana)),
        ExpenseSplit.Whole(
            Split.Percentages(
                listOf(
                    MemberPercentage(people.ana, Percentage.parse("25")),
                    MemberPercentage(people.bob, Percentage.parse("25")),
                    MemberPercentage(people.carol, Percentage.parse("35")),
                    MemberPercentage(people.dan, Percentage.parse("15")),
                ),
            ),
        ),
        october(1),
        category = categories.lodging,
    )

/** The seafood dinner, divided by parts: Eva counts as half. */
internal fun seafoodByParts(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Seafood dinner",
        Cost.Local(eur("90.00")),
        Split.Equal(listOf(people.carol)),
        ExpenseSplit.Whole(
            Split.Parts(
                listOf(
                    MemberParts(people.ana, Parts.parse("1")),
                    MemberParts(people.bob, Parts.parse("1")),
                    MemberParts(people.carol, Parts.parse("1")),
                    MemberParts(people.dan, Parts.parse("1")),
                    MemberParts(people.eva, Parts.parse("0.5")),
                ),
            ),
        ),
        october(2),
        category = categories.food,
    )

/** The train to Porto, divided by exact amounts. */
internal fun trainByAmounts(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Train to Porto",
        Cost.Local(eur("88.50")),
        Split.Equal(listOf(people.bob)),
        ExpenseSplit.Whole(
            Split.Amounts(
                listOf(
                    MemberAmount(people.ana, eur("24.50")),
                    MemberAmount(people.bob, eur("24.50")),
                    MemberAmount(people.carol, eur("24.50")),
                    MemberAmount(people.eva, eur("15.00")),
                ),
            ),
        ),
        october(3),
        category = categories.transport,
    )

/** The museum, paid by Ana and Bob. */
internal fun museumWithTwoPayers(people: People): Expense =
    Expense(
        "Museum",
        Cost.Local(eur("60.00")),
        Split.Equal(listOf(people.ana, people.bob)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan))),
        october(3),
    )

/** The receipt of the fado night: every line with its own split. */
internal fun fadoItems(people: People): List<Item> =
    listOf(
        Item("Vinho verde", eur("12.00"), Split.Equal(listOf(people.ana, people.bob, people.dan)), quantity = 2),
        Item("Bacalhau", eur("18.00"), Split.Equal(listOf(people.carol))),
        Item(
            "Custard tarts",
            eur("1.50"),
            Split.Parts(
                listOf(
                    MemberParts(people.ana, Parts.parse("1")),
                    MemberParts(people.bob, Parts.parse("1")),
                    MemberParts(people.carol, Parts.parse("1")),
                    MemberParts(people.dan, Parts.parse("1")),
                    MemberParts(people.eva, Parts.parse("2")),
                ),
            ),
            quantity = 6,
        ),
        Item("Juice", eur("3.00"), Split.Equal(listOf(people.eva))),
    )

/** The fado night, item by item, paid by Carol and Dan, with a 10 % tip. */
internal fun fadoItemByItem(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Fado night",
        Cost.Local(eur("54.00")),
        Split.Amounts(listOf(MemberAmount(people.carol, eur("39.40")), MemberAmount(people.dan, eur("20.00")))),
        ExpenseSplit.Itemised(fadoItems(people)),
        october(4),
        tip = Tip(TipValue.Percentage(Percentage.parse("10"))),
        time = TimeOfDay.of(hour = 22, minute = 15),
        category = categories.food,
        description = "Tasca do Chico",
    )

/** The tuk-tuk tour, with a fixed tip shared equally. */
internal fun tourWithATip(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        "Tuk-tuk tour",
        Cost.Local(eur("45.00")),
        Split.Equal(listOf(people.dan)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan, people.eva))),
        october(5),
        tip = Tip(TipValue.Fixed(eur("5.00")), TipDistribution.EQUAL),
        category = categories.transport,
    )
