// The totals. A balance on the wrong side, a total of another member or category, or a plan with
// its members swapped would be what an app shows.

@testable import Eranos
import Testing

/// The summary and a settlement would reach an app with another member's or another category's
/// numbers, or a balance that prints another way than the library writes it.
@Test
func theSummaryAndTheSettlementKeepEveryNumber() throws {
    let flat = try Flat()
    let summary = try flat.group.summary()
    #expect(summary.totalSpent == flat.euros(3_000))
    let ana = try #require(summary.members.first { $0.member == flat.ana })
    #expect(ana.balance == .getsBack(flat.euros(500)))
    #expect(ana.balance.description == "gets back 5.00 EUR")
    #expect([ana.paid, ana.share, ana.received, ana.sent, ana.net] == [3_000, 1_500, 1_000, 0, 500].map(flat.euros))
    let bob = try #require(summary.members.first { $0.member == flat.bob })
    #expect(bob.balance == .owes(flat.euros(500)))
    #expect(bob.balance.description == "owes 5.00 EUR")
    let food = try #require(summary.categoryTotals.first { $0.category == flat.food })
    #expect(food.total == flat.euros(3_000))
    let anaInFood = try #require(summary.memberCategoryTotals.first { $0.member == flat.ana })
    #expect(anaInFood.totals.map(\.total) == [flat.euros(1_500)])

    let settlement = try flat.group.settlement(by: .minimumTransfers)
    let payment = try #require(settlement.payments.first)
    #expect([payment.from, payment.to] == [flat.bob, flat.ana])
    #expect(payment.amount == flat.euros(500))
    try flat.group.add(Transfer(cost: .local(payment.amount), date: flat.day, from: payment.from, to: payment.to))
    let settled = try flat.group.summary()
    #expect(settled.members.map(\.balance) == [.settled, .settled])
    #expect(Balance.settled.description == "settled")
}
