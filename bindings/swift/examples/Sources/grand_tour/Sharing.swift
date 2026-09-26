// Section 14 of the tour: the group shared between tasks, a settlement computed with the asynchronous calls, and a
// change that waits for a report's read and is cancelled.

import Eranos
import Report

func sharing(_ group: Group, _ people: People) async throws(EranosError) {
    section("14. Sharing the group")
    // Another handle on the same group, as another task holds: a group is shared as it is.
    let other = group
    let everyone = [people.ana, people.bob, people.carol, people.dan, people.eva]
    let coffee = Expense(cost: .local(try ron("75.00")), date: try october(8), name: "Airport coffee", payers: .equal([people.bob]), split: .whole(.equal(everyone)))
    _ = try await other.add(coffee)
    let plan = try await group.settlement(by: .minimumTransfers)
    print("""
        An app shares the group between its tasks and threads through a SharedGroup,
        whose calls are futures any executor runs; here the library's block_on runs
        each one on this thread. Bob adds the airport coffee through another handle,
        and the plan that settles it comes through the first:
        """)
    let rows = try await group.withRead { reader async throws(EranosError) in
        try plan.payments.map { payment throws(EranosError) in
            [try reader.member(withID: payment.from)?.firstName ?? "?", try reader.member(withID: payment.to)?.firstName ?? "?", "\(payment.amount)"]
        }
    }
    printTable(["From", "To", "Amount"], rows, numbersFrom: 2)

    let before = try await group.summary()
    let euro = try Currency(code: "EUR")
    let rates = [try Currency(code: "RON"): try ExchangeRate(parsing: "0.2")]
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    let report = try await group.read()
    let change = Task.detached { try await group.changeCurrency(to: euro, rates: rates) }
    let reported = try report.summary().totalSpent
    // Cancelling the task cancels the change, whether it waits for the lock or has yet to start.
    change.cancel()
    report.release()
    let outcome = await change.result
    let after = try await group.summary()
    let currency = try await group.currency()
    print("""
        While a report reads the group, a change of its currency to EUR waits for the
        report. The app drops the waiting change, which cancels it: the group still
        keeps its books in \(currency), and the trip cost \(after.totalSpent), as the report read.
        """)
    print("With the other handle gone, the app takes the group back: \(try await group.expenses().count) expenses.")

    precondition(4 == plan.payments.count, "the airport coffee needs four payments")
    let share = try ron("15.00")
    for payment in plan.payments {
        precondition(people.bob == payment.to && share == payment.amount, "everyone pays Bob their share")
    }
    guard case let .failure(error) = outcome, case .cancelled? = error as? EranosError else {
        preconditionFailure("the waiting change is cancelled")
    }
    expect(before, after, "the cancelled change leaves the totals")
    expect(reported, after.totalSpent, "the report read the totals")
    expect(try Currency(code: "RON"), currency, "the group keeps its currency")
}
