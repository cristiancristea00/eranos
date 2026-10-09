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
    let summary = flat.group.summary
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
    #expect(summary.member(withID: flat.ana) == ana)
    let stranger = Member.ID(rawValue: "nobody")
    let nobody = summary.member(withID: stranger)
    #expect(nobody.member == stranger)
    #expect([nobody.paid, nobody.share, nobody.sent, nobody.received, nobody.net] == Array(repeating: flat.euros(0), count: 5))
    #expect(nobody.balance == .settled)
    #expect(summary.categoryTotal(of: flat.food) == flat.euros(3_000))
    #expect(summary.categoryTotal(of: .uncategorised) == flat.euros(0))
    #expect(summary.memberCategoryTotal(of: flat.ana, in: flat.food) == flat.euros(1_500))
    #expect(summary.memberCategoryTotal(of: flat.ana, in: .uncategorised) == flat.euros(0))
    #expect(summary.memberCategoryTotal(of: stranger, in: flat.food) == flat.euros(0))

    #expect(SettlementStrategy.allCases == [.adaptive, .minimumTransfers, .greedy, .direct])
    for strategy in SettlementStrategy.allCases {
        #expect(SettlementStrategy(strategy.generated) == strategy)
    }
    let settlement = try flat.group.settlement(by: .minimumTransfers)
    let payment = try #require(settlement.payments.first)
    #expect(payment.sender == flat.bob)
    #expect(payment.receiver == flat.ana)
    #expect(payment.amount == flat.euros(500))
    let repayment = Transfer(payment: payment, date: flat.day, note: "Settled")
    #expect(repayment.sender == flat.bob)
    #expect(repayment.receiver == flat.ana)
    #expect(repayment.cost == .local(flat.euros(500)))
    #expect(repayment.date == flat.day)
    #expect(repayment.note == "Settled")
    let named = Transfer(id: repayment.id, payment: payment, date: flat.day, note: "Settled")
    #expect(named == repayment)
    try flat.group.add(repayment)
    let settled = flat.group.summary
    #expect(settled.members.map(\.balance) == [.settled, .settled])
    #expect("settled" == Balance.settled.description)
}
