// Sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and settling up.

import Eranos
import Report

func summary(_ group: ExpenseGroup, _ people: People, _ categories: Categories) throws(EranosError) {
    section("8. The summary")
    let summary = try group.summary()
    print("""
        The group spent \(summary.totalSpent) in all. For each member, what they paid, what they
        consumed, the transfers they sent and received, and what is left:
        """)
    try printBalances(group)
    print("The spending per category:")
    try printCategories(group)
    print("And each member's consumption per category:")
    let zero = Money(minorUnits: 0, currency: try group.currency())
    printTable(
        try group.members().map { member in
            let totals = summary.memberCategoryTotals.first { totals in totals.member == member.id }?.totals ?? []
            let total = { (category: Category.ID) in totals.first { total in total.category == category }?.total ?? zero }
            return MemberCategoryRow(
                name: member.firstName,
                food: total(categories.food),
                lodging: total(categories.lodging),
                transport: total(categories.transport),
                uncategorised: total(.uncategorised)
            )
        },
        numbersFrom: 1
    )

    let member = { (id: Member.ID) in totals(of: id, in: summary) }
    let category = { (id: Category.ID) in summary.categoryTotals.first { total in total.category == id }?.total }
    expect(try eur("1222.90"), summary.totalSpent, "the total spent")
    expect(try eur("890.00"), member(people.ana)?.paid, "what Ana paid")
    expect(try eur("140.00"), member(people.ana)?.received, "what Ana received")
    expect(try eur("20.00"), member(people.bob)?.sent, "what Bob sent")
    expect(Balance.getsBack(try eur("12.55")), member(people.dan)?.balance, "Dan's balance")
    expect(Balance.owes(try eur("42.60")), member(people.eva)?.balance, "Eva's balance")
    expect(try eur("174.40"), category(categories.food), "the food")
    expect(try eur("800.00"), category(categories.lodging), "the lodging")
    expect(try eur("188.50"), category(categories.transport), "the transport")
    expect(try eur("60.00"), category(.uncategorised), "the uncategorised spending")
    for totals in summary.members {
        let byCategory = summary.memberCategoryTotals.first { entry in entry.member == totals.member }?.totals ?? []
        expect(try sum(zero.currency, byCategory.map(\.total)), totals.share, "a member's consumption by category")
    }
    let net = try sum(zero.currency, summary.members.map(\.net))
    precondition(0 == net.minorUnits, "the balances add up to zero")
}

func removingACategory(_ group: ExpenseGroup, _ categories: Categories) throws(EranosError) {
    section("9. Removing a category")
    let removed = try group.remove(categories.transport)
    let builtIn = Result { () throws(EranosError) in try group.remove(Category.ID.uncategorised) }
    print("""
        Removing \(removed.name) moves its expenses to Uncategorised and the category
        totals with them; no balance changes. The built-in category cannot be removed:
          error: \(message(builtIn))
        """)
    try printCategories(group)

    guard case .failure(.expenseGroup(.builtInCategory, _)) = builtIn else {
        preconditionFailure("the built-in category stays")
    }
    let transport = try group.category(withID: categories.transport)
    precondition(nil == transport, "Transport is removed")
    expect(try eur("248.50"), try group.summary().categoryTotals.first { total in Category.ID.uncategorised == total.category }?.total, "the uncategorised spending")
    let moved = try group.expenses().allSatisfy { expense in expense.category != categories.transport }
    precondition(moved, "no expense is in Transport")
}

func newCurrency(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    section("10. A new currency")
    let euro = try CurrencyCode(code: "EUR")
    let leu = try CurrencyCode(code: "RON")
    let missing = Result { () throws(EranosError) in try group.changeCurrency(to: leu, rates: [:]) }
    try group.changeCurrency(to: leu, rates: [euro: try ExchangeRate(parsing: "5")])
    print("""
        Home again, the group moves its books to RON. The change needs a rate into RON
        for every other currency in use, here only EUR, or it is refused:
          error: \(message(missing))
        Entries in EUR take the new rate, and the RON parking and cash become local:
        """)
    try printEntries(group) { _ in true }
    print("""
        Each entry keeps its currency and converts at the new rate. At 5 every share
        converts exactly, so each balance is five times what it was. The trip cost
        \(try group.summary().totalSpent):
        """)
    try printBalances(group)

    guard case .failure(.expenseGroup(.missingExchangeRate(euro), _)) = missing else {
        preconditionFailure("the amounts in EUR need a rate")
    }
    let summary = try group.summary()
    expect(leu, try group.currency(), "the new currency")
    expect(try ron("6114.50"), summary.totalSpent, "the total spent in RON")
    expect(Balance.getsBack(try ron("62.75")), totals(of: people.dan, in: summary)?.balance, "Dan's balance in RON")
    expect(Balance.owes(try ron("213.00")), totals(of: people.eva, in: summary)?.balance, "Eva's balance in RON")
}

/// The name of `strategy` as the apps show it.
func strategyName(_ strategy: SettlementStrategy) -> String {
    switch strategy {
    case .adaptive:
        "Adaptive"
    case .minimumTransfers:
        "MinimumTransfers"
    case .greedy:
        "Greedy"
    case .direct:
        "Direct"
    }
}

func settlingUp(_ group: ExpenseGroup) throws(EranosError) {
    section("11. Settling up")
    var counts: [StrategyRow] = []
    for strategy in SettlementStrategy.allCases {
        let settlement = try group.settlement(by: strategy)
        // The group's open balances are well within the adaptive strategy's limit, so it settles
        // them exactly.
        let computedBy: SettlementStrategy = .adaptive == strategy ? .minimumTransfers : strategy
        precondition(computedBy == settlement.strategy, "a settlement names the strategy whose plan it is")
        counts.append(
            StrategyRow(
                strategy: strategyName(strategy),
                computedBy: strategyName(settlement.strategy),
                payments: settlement.payments.count
            )
        )
    }
    print("""
        Each strategy turns the balances into payments: the direct one pays back every
        pair's own debts, greedy pays the largest amounts first, the minimum one finds
        the fewest payments, and the adaptive one finds the fewest while that is quick
        and greedy's beyond, naming the strategy whose plan it chose.
        """)
    printTable(counts, numbersFrom: 2)
    let plan = try group.settlement(by: .minimumTransfers)
    print("The group follows the minimum plan:")
    printTable(
        try plan.payments.map { payment throws(EranosError) in PaymentRow(sender: try name(group, payment.sender), receiver: try name(group, payment.receiver), amount: payment.amount) },
        numbersFrom: 2
    )
    for payment in plan.payments {
        _ = try group.add(Transfer(sender: payment.sender, receiver: payment.receiver, cost: .local(payment.amount), date: try october(7)))
    }
    print("Each payment, once made, is recorded as a transfer, and every balance is settled:")
    try printBalances(group)

    let greedy = try group.settlement(by: .greedy)
    precondition(greedy.payments.isEmpty, "nothing is left to pay")
    precondition(4 == plan.payments.count, "the minimum plan has four payments")
    for totals in try group.summary().members {
        precondition(Balance.settled == totals.balance, "every balance is settled")
    }
}
