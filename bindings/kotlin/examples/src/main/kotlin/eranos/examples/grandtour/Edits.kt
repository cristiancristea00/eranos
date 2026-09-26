// Sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go.
package eranos.examples.grandtour

import eranos.Cost
import eranos.Currency
import eranos.EranosException
import eranos.ExchangeRate
import eranos.Expense
import eranos.ExpenseFailure
import eranos.ExpenseId
import eranos.ExpenseSplit
import eranos.Group
import eranos.GroupFailure
import eranos.MemberPercentage
import eranos.Money
import eranos.Percentage
import eranos.Split
import eranos.TimeOfDay
import eranos.Transfer
import eranos.TransferFailure
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

internal fun anotherCurrency(
    group: Group,
    people: People,
    categories: Categories,
) {
    section("5. Another currency")
    val euro = Currency.of("EUR")
    val rate = ExchangeRate.parse("0.2")
    val parking =
        group.addBlocking(
            Expense(
                Cost.Converted(ron("250.00"), rate),
                october(6),
                "Airport parking",
                Split.Equal(listOf(people.ana)),
                ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob))),
                category = categories.transport,
            ),
        )
    val cash =
        group.addBlocking(
            Transfer(
                Cost.Converted(ron("100.00"), rate),
                october(6),
                people.bob,
                people.ana,
                note = "Cash for the parking",
                time = TimeOfDay.of(hour = 7, minute = 40),
            ),
        )
    println(
        """
        Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in
        cash. An entry in another currency carries the rate into the group currency, how
        many EUR one RON is worth, and is converted once, when it is entered:
        """.trimIndent(),
    )
    printEntries(group) { currency -> euro != currency }
    val transfer = checkNotNull(group.transferBlocking(cash)) { "the cash was recorded" }
    val converted = ExchangeRate.parse("0.2011").convert(ron("250.00"), euro)
    val counted = group.transferAmountBlocking(cash) ?: Money(euro, 0)
    println(
        """
        Bob's transfer, at ${transfer.time?.toString().orEmpty()} ("${transfer.note.orEmpty()}"), counts as $counted.
        A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,
        which becomes $converted.
        """.trimIndent(),
    )

    check(eur("50.28") == converted)
    check(eur("50.00") == group.expenseSharesBlocking(parking)?.total)
    check(eur("20.00") == group.transferAmountBlocking(cash))
    check(transfer.cost is Cost.Converted)
}

/** Replaces an expense and a transfer, and removes one of each; returns what changed. */
internal fun replaceAndRemove(
    group: Group,
    people: People,
    categories: Categories,
    seafood: ExpenseId,
): List<List<String>> {
    // The dinner's receipt said 99.00.
    val corrected =
        checkNotNull(group.expenseBlocking(seafood)) { "the dinner was added" }
            .copy(cost = Cost.Local(eur("99.00")), description = "Cervejaria Ramiro")
    val previous = group.replaceBlocking(corrected)
    // Bob entered the coffee twice.
    val coffee =
        Expense(
            Cost.Local(eur("6.00")),
            october(5),
            "Coffee",
            Split.Equal(listOf(people.bob)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol))),
            category = categories.food,
        )
    group.addBlocking(coffee)
    val copy = group.addBlocking(coffee)
    val removed = group.removeBlocking(copy)
    // Dan's part of the apartment was 120.00, not 100.00.
    val first = group.addBlocking(Transfer(Cost.Local(eur("100.00")), october(5), people.dan, people.ana))
    val fixed = checkNotNull(group.transferBlocking(first)) { "the transfer was added" }.copy(cost = Cost.Local(eur("120.00")), note = "Apartment")
    val replaced = group.replaceBlocking(fixed)
    // Carol sent 10.00 to the wrong member.
    val mistake = group.addBlocking(Transfer(Cost.Local(eur("10.00")), october(5), people.carol, people.bob))
    val undone = group.removeBlocking(mistake)

    check(eur("90.00") == previous.cost.amount)
    check(null == group.expenseBlocking(copy))
    check(eur("120.00") == group.transferAmountBlocking(first))
    check(null == group.transferBlocking(mistake))
    val zero = Money(Currency.of("EUR"), 0)
    val dinner = group.expenseBlocking(seafood)
    val apartment = group.transferBlocking(first)
    return listOf(
        listOf(previous.name, "${previous.cost.amount} became ${dinner?.cost?.amount ?: zero}, at ${dinner?.description.orEmpty()}"),
        listOf(removed.name, "entered twice; the copy of ${removed.cost.amount} was removed"),
        listOf(
            "${name(group, people.dan)} to ${name(group, people.ana)}",
            "${replaced.cost.amount} became ${apartment?.cost?.amount ?: zero}, noted ${apartment?.note.orEmpty()}",
        ),
        listOf("${name(group, people.carol)} to ${name(group, people.bob)}", "${undone.cost.amount} sent to the wrong member; removed"),
    )
}

internal fun edits(
    group: Group,
    people: People,
    categories: Categories,
    seafood: ExpenseId,
) {
    section("6. Edits")
    val changes = replaceAndRemove(group, people, categories, seafood)
    println(
        """
        Entries are values: an edit starts from a copy made with to_builder, keeps the
        identifier, and replaces the entry; a removal takes it out with its totals.
        """.trimIndent(),
    )
    printTable(listOf("Entry", "Edit"), changes, numbersFrom = 2)
    println("The balances after the edits:")
    printBalances(group)

    val before = group.summaryBlocking()
    val taxi =
        Expense(
            Cost.Local(eur("30.00")),
            october(6),
            "Taxi",
            Split.Equal(listOf(people.ana)),
            ExpenseSplit.Whole(
                Split.Percentages(
                    listOf(MemberPercentage(people.ana, Percentage.parse("50")), MemberPercentage(people.bob, Percentage.parse("49.99"))),
                ),
            ),
        )
    val unbalanced = refusal { group.addBlocking(taxi) }
    val toHerself = refusal { group.addBlocking(Transfer(Cost.Local(eur("5.00")), october(6), people.ana, people.ana)) }
    val souvenirs =
        Expense(
            Cost.Local(ron("40.00")),
            october(6),
            "Souvenirs",
            Split.Equal(listOf(people.ana)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana))),
        )
    val unrated = refusal { group.addBlocking(souvenirs) }
    println("A change that does not fit is refused, and the group stays as it was:")
    printTable(
        listOf("Change", "Refused with"),
        listOf(
            listOf("A taxi split 50% and 49.99%", message(unbalanced)),
            listOf("Ana paying herself back", message(toHerself)),
            listOf("Souvenirs in RON, without a rate", message(unrated)),
        ),
        numbersFrom = 2,
    )
    println("The balances again, unchanged:")
    printBalances(group)

    check(ExpenseFailure.PercentagesDoNotAddUp(9_999) == (unbalanced as? EranosException.Expense)?.failure)
    check(TransferFailure.SameMember == (toHerself as? EranosException.Transfer)?.failure)
    check(GroupFailure.MissingExchangeRate(Currency.of("RON")) == (unrated as? EranosException.Group)?.failure)
    check(before == group.summaryBlocking())
}

internal fun comingsAndGoings(
    group: Group,
    people: People,
) {
    section("7. Members come and go")
    val removed = group.removeBlocking(people.filip)
    group.archiveBlocking(people.dan)
    val dinner =
        Expense(
            Cost.Local(eur("40.00")),
            october(6),
            "Farewell dinner",
            Split.Equal(listOf(people.ana)),
            ExpenseSplit.Whole(Split.Equal(listOf(people.ana, people.bob, people.carol, people.dan))),
        )
    val farewell = refusal { group.addBlocking(dinner) }
    val inUse = refusal { group.removeBlocking(people.dan) }
    println(
        """
        ${removed.firstName} was never part of an entry, so removing him leaves no trace. Dan flew home
        early and was archived: his history and his balance stay, but a new expense
        cannot include him, and he cannot be removed while entries reference him.
        """.trimIndent(),
    )
    val summary = group.summaryBlocking()
    val rows =
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id ->
                totals(id, summary)?.let { totals ->
                    listOf(member.firstName, if (group.isArchivedBlocking(id)) "archived" else "active", "${totals.balance}")
                }
            }
        }
    printTable(listOf("Member", "Status", "Balance"), rows, numbersFrom = 2)
    printTable(
        listOf("Change", "Refused with"),
        listOf(
            listOf("A farewell dinner with Dan", message(farewell)),
            listOf("Removing Dan", message(inUse)),
        ),
        numbersFrom = 2,
    )
    group.unarchiveBlocking(people.dan)
    println("When Dan joins the next trip, unarchiving him makes him active again.")

    check("Filip" == removed.firstName)
    check(null == group.memberBlocking(people.filip))
    check(GroupFailure.ArchivedMember(people.dan) == (farewell as? EranosException.Group)?.failure)
    check((inUse as? EranosException.Group)?.failure is GroupFailure.MemberInUse)
    check(!group.isArchivedBlocking(people.dan))
}
