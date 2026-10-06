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
        Cost.Local(eur("10.00")),
        october(1),
        "Groceries",
        Split.Equal(listOf(people.ana)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol))),
        category = categories.food,
    )

/** The apartment, divided by percentage. */
internal fun apartmentByPercentage(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        Cost.Local(eur("800.00")),
        october(1),
        "Apartment",
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
        category = categories.lodging,
    )

/** The seafood dinner, divided by parts: Eva counts as half. */
internal fun seafoodByParts(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        Cost.Local(eur("90.00")),
        october(2),
        "Seafood dinner",
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
        category = categories.food,
    )

/** The train to Porto, divided by exact amounts. */
internal fun trainByAmounts(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        Cost.Local(eur("88.50")),
        october(3),
        "Train to Porto",
        Split.Equal(listOf(people.bob)),
        ExpenseSplit.Whole(
            Split.Amounts(
                listOf(
                    MemberAmount(eur("24.50"), people.ana),
                    MemberAmount(eur("24.50"), people.bob),
                    MemberAmount(eur("24.50"), people.carol),
                    MemberAmount(eur("15.00"), people.eva),
                ),
            ),
        ),
        category = categories.transport,
    )

/** The museum, paid by Ana and Bob. */
internal fun museumWithTwoPayers(people: People): Expense =
    Expense(
        Cost.Local(eur("60.00")),
        october(3),
        "Museum",
        Split.Equal(listOf(people.ana, people.bob)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan))),
    )

/** The receipt of the fado night: every line with its own split. */
internal fun fadoItems(people: People): List<Item> =
    listOf(
        Item("Vinho verde", Split.Equal(listOf(people.ana, people.bob, people.dan)), eur("12.00"), quantity = 2),
        Item("Bacalhau", Split.Equal(listOf(people.carol)), eur("18.00")),
        Item(
            "Custard tarts",
            Split.Parts(
                listOf(
                    MemberParts(people.ana, Parts.parse("1")),
                    MemberParts(people.bob, Parts.parse("1")),
                    MemberParts(people.carol, Parts.parse("1")),
                    MemberParts(people.dan, Parts.parse("1")),
                    MemberParts(people.eva, Parts.parse("2")),
                ),
            ),
            eur("1.50"),
            quantity = 6,
        ),
        Item("Juice", Split.Equal(listOf(people.eva)), eur("3.00")),
    )

/** The fado night, item by item, paid by Carol and Dan, with a 10 % tip. */
internal fun fadoItemByItem(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        Cost.Local(eur("54.00")),
        october(4),
        "Fado night",
        Split.Amounts(listOf(MemberAmount(eur("39.40"), people.carol), MemberAmount(eur("20.00"), people.dan))),
        ExpenseSplit.Itemised(fadoItems(people)),
        category = categories.food,
        description = "Tasca do Chico",
        time = TimeOfDay.of(hour = 22, minute = 15),
        tip = Tip(TipValue.Percentage(Percentage.parse("10"))),
    )

/** The tuk-tuk tour, with a fixed tip shared equally. */
internal fun tourWithATip(
    people: People,
    categories: Categories,
): Expense =
    Expense(
        Cost.Local(eur("45.00")),
        october(5),
        "Tuk-tuk tour",
        Split.Equal(listOf(people.dan)),
        ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan, people.eva))),
        category = categories.transport,
        tip = Tip(TipValue.Fixed(eur("5.00")), TipDistribution.EQUAL),
    )
