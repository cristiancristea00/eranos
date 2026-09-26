// Sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and settling up.
package eranos.examples.grandtour

import eranos.Balance
import eranos.CategoryId
import eranos.Cost
import eranos.Currency
import eranos.EranosException
import eranos.ExchangeRate
import eranos.Group
import eranos.GroupFailure
import eranos.MemberId
import eranos.Money
import eranos.SettlementStrategy
import eranos.Transfer
import eranos.examples.printTable
import eranos.examples.section
import eranos.examples.totals

internal fun summary(
    group: Group,
    people: People,
    categories: Categories,
) {
    section("8. The summary")
    val summary = group.summaryBlocking()
    println(
        """
        The group spent ${summary.totalSpent} in all. For each member, what they paid, what they
        consumed, the transfers they sent and received, and what is left:
        """.trimIndent(),
    )
    printBalances(group)
    println("The spending per category:")
    printCategories(group)
    println("And each member's consumption per category:")
    val builtIn = builtInCategory(group)
    val zero = Money(group.currencyBlocking(), 0)
    printTable(
        listOf("Member", "Food", "Lodging", "Getting around", "Uncategorised"),
        group.membersBlocking().mapNotNull { member ->
            member.id?.let { id ->
                val totals =
                    summary.memberCategoryTotals
                        .firstOrNull { entry -> entry.member == id }
                        ?.totals
                        .orEmpty()
                val total = { category: CategoryId? -> "${totals.firstOrNull { total -> total.category == category }?.total ?: zero}" }
                listOf(member.firstName, total(categories.food), total(categories.lodging), total(categories.transport), total(builtIn))
            }
        },
        numbersFrom = 1,
    )

    val member = { id: MemberId -> totals(id, summary) }
    val category = { id: CategoryId? -> summary.categoryTotals.firstOrNull { total -> total.category == id }?.total }
    check(eur("1222.90") == summary.totalSpent)
    check(eur("890.00") == member(people.ana)?.paid)
    check(eur("140.00") == member(people.ana)?.received)
    check(eur("20.00") == member(people.bob)?.sent)
    check(Balance.GetsBack(eur("12.55")) == member(people.dan)?.balance)
    check(Balance.Owes(eur("42.60")) == member(people.eva)?.balance)
    check(eur("174.40") == category(categories.food))
    check(eur("800.00") == category(categories.lodging))
    check(eur("188.50") == category(categories.transport))
    check(eur("60.00") == category(builtIn))
    for (totals in summary.members) {
        val byCategory =
            summary.memberCategoryTotals
                .firstOrNull { entry -> entry.member == totals.member }
                ?.totals
                .orEmpty()
        check(sum(zero.currency, byCategory.map { total -> total.total }) == totals.share)
    }
    check(0L == sum(zero.currency, summary.members.map { totals -> totals.net }).minorUnits)
}

internal fun removingACategory(
    group: Group,
    categories: Categories,
) {
    section("9. Removing a category")
    val removed = group.removeBlocking(categories.transport)
    val uncategorised = checkNotNull(builtInCategory(group)) { "every group has the built-in category" }
    val builtIn = refusal { group.removeBlocking(uncategorised) }
    println(
        """
        Removing ${removed.name} moves its expenses to Uncategorised and the category
        totals with them; no balance changes. The built-in category cannot be removed:
          error: ${message(builtIn)}
        """.trimIndent(),
    )
    printCategories(group)

    check(GroupFailure.BuiltInCategory == (builtIn as? EranosException.Group)?.failure)
    check(null == group.categoryBlocking(categories.transport))
    check(
        eur("248.50") ==
            group
                .summaryBlocking()
                .categoryTotals
                .firstOrNull { total -> total.category == uncategorised }
                ?.total,
    )
    check(group.expensesBlocking().none { expense -> categories.transport == expense.category })
}

internal fun newCurrency(
    group: Group,
    people: People,
) {
    section("10. A new currency")
    val euro = Currency.of("EUR")
    val leu = Currency.of("RON")
    val missing = refusal { group.changeCurrencyBlocking(leu, emptyMap()) }
    group.changeCurrencyBlocking(leu, mapOf(euro to ExchangeRate.parse("5")))
    println(
        """
        Home again, the group moves its books to RON. The change needs a rate into RON
        for every other currency in use, here only EUR, or it is refused:
          error: ${message(missing)}
        Entries in EUR take the new rate, and the RON parking and cash become local:
        """.trimIndent(),
    )
    printEntries(group) { true }
    println(
        """
        Each entry keeps its currency and converts at the new rate. At 5 every share
        converts exactly, so each balance is five times what it was. The trip cost
        ${group.summaryBlocking().totalSpent}:
        """.trimIndent(),
    )
    printBalances(group)

    val summary = group.summaryBlocking()
    check(GroupFailure.MissingExchangeRate(euro) == (missing as? EranosException.Group)?.failure)
    check(leu == group.currencyBlocking())
    check(ron("6114.50") == summary.totalSpent)
    check(Balance.GetsBack(ron("62.75")) == totals(people.dan, summary)?.balance)
    check(Balance.Owes(ron("213.00")) == totals(people.eva, summary)?.balance)
}

/** The name of [strategy] as the apps show it. */
internal fun strategyName(strategy: SettlementStrategy): String =
    when (strategy) {
        SettlementStrategy.DIRECT -> "Direct"
        SettlementStrategy.GREEDY -> "Greedy"
        SettlementStrategy.MINIMUM_TRANSFERS -> "MinimumTransfers"
    }

internal fun settlingUp(group: Group) {
    section("11. Settling up")
    val counts =
        SettlementStrategy.entries.map { strategy ->
            val settlement = group.settlementBlocking(strategy)
            check(strategy == settlement.strategy)
            listOf(strategyName(settlement.strategy), "${settlement.payments.size}")
        }
    println(
        """
        Each strategy turns the balances into payments: the direct one pays back every
        pair's own debts, greedy pays the largest amounts first, and the minimum one
        finds the fewest payments.
        """.trimIndent(),
    )
    printTable(listOf("Strategy", "Payments"), counts, numbersFrom = 1)
    val plan = group.settlementBlocking(SettlementStrategy.MINIMUM_TRANSFERS)
    println("The group follows the minimum plan:")
    printTable(
        listOf("From", "To", "Amount"),
        plan.payments.map { payment -> listOf(name(group, payment.from), name(group, payment.to), "${payment.amount}") },
        numbersFrom = 2,
    )
    for (payment in plan.payments) {
        group.addBlocking(Transfer(Cost.Local(payment.amount), october(7), payment.from, payment.to))
    }
    println("Each payment, once made, is recorded as a transfer, and every balance is settled:")
    printBalances(group)

    check(group.settlementBlocking(SettlementStrategy.GREEDY).payments.isEmpty())
    check(4 == plan.payments.size)
    check(group.summaryBlocking().members.all { totals -> Balance.Settled == totals.balance })
}
