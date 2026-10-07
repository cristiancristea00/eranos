// The totals. A balance on the wrong side, a total of another member or category, or a plan with
// its members swapped would be what an app shows.

@testable import Eranos
import Testing

/// The summary and a settlement would reach an app with another member's or another category's
/// numbers, a balance that prints another way than the library writes it, or a strategy that
/// reached the library, or came back from it, as another.
@Test
func theSummaryAndTheSettlementKeepEveryNumber() throws {
    let flat = try Flat()
    let summary = try flat.group.summary()
    #expect(summary.totalSpent == flat.euros(3_000))
    let ana = try #require(summary.members.first { $0.member == flat.ana })
    #expect(ana.paid == flat.euros(3_000))
    #expect(ana.share == flat.euros(1_500))
    #expect(ana.sent == flat.euros(0))
    #expect(ana.received == flat.euros(1_000))
    #expect(ana.net == flat.euros(500))
    #expect(ana.balance == .getsBack(flat.euros(500)))
    #expect("gets back 5.00 EUR" == ana.balance.description)
    let bob = try #require(summary.members.first { $0.member == flat.bob })
    #expect(bob.balance == .owes(flat.euros(500)))
    #expect("owes 5.00 EUR" == bob.balance.description)
    let food = try #require(summary.categoryTotals.first { $0.category == flat.food })
    #expect(food.total == flat.euros(3_000))
    let anaInFood = try #require(summary.memberCategoryTotals.first { $0.member == flat.ana })
    #expect(anaInFood.totals.map(\.total) == [flat.euros(1_500)])

    #expect(SettlementStrategy.allCases == [.adaptive, .minimumTransfers, .greedy, .direct])
    for strategy in SettlementStrategy.allCases {
        #expect(SettlementStrategy(strategy.generated) == strategy)
    }
    let settlement = try flat.group.settlement(by: .minimumTransfers)
    let payment = try #require(settlement.payments.first)
    #expect(payment.from == flat.bob)
    #expect(payment.to == flat.ana)
    #expect(payment.amount == flat.euros(500))
    try flat.group.add(Transfer(from: payment.from, to: payment.to, cost: .local(payment.amount), date: flat.day))
    let settled = try flat.group.summary()
    #expect(settled.members.map(\.balance) == [.settled, .settled])
    #expect("settled" == Balance.settled.description)
}
