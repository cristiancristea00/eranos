// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.

import Eranos
import Report

let euro = try Currency(code: "EUR")

/// Records an expense of `amount` EUR paid by `payer` and split equally.
func spend(_ group: Group, _ name: String, _ amount: String, _ payer: Member.ID, _ participants: [Member.ID]) throws(EranosError) {
    _ = try group.add(
        Expense(
            cost: .local(try Money(parsing: amount, in: euro)),
            date: try CalendarDate(day: 20, month: 9, year: 2026),
            name: name,
            payers: .equal([payer]),
            split: .whole(.equal(participants))
        )
    )
}

func printBalances(_ group: Group) throws(EranosError) {
    let summary = try group.summary()
    printTable(
        ["Member", "Paid", "Consumed", "Balance"],
        try group.members().compactMap { member in
            member.id.flatMap { id in totals(of: id, in: summary) }.map { totals in
                [member.firstName, "\(totals.paid)", "\(totals.share)", "\(totals.balance)"]
            }
        },
        numbersFrom: 1
    )
}

func printPayments(_ group: Group, _ settlement: Settlement) throws(EranosError) {
    printTable(
        ["From", "To", "Amount"],
        try settlement.payments.map { payment throws(EranosError) in
            [try names(group, [payment.from]), try names(group, [payment.to]), "\(payment.amount)"]
        },
        numbersFrom: 2
    )
}

/// Prints each strategy's plan with what it promises; returns the number of payments of each, in the order minimum,
/// greedy, direct.
func strategies(_ group: Group) throws(EranosError) -> [Int] {
    section("3. Minimum transfers")
    let minimum = try group.settlement(by: .minimumTransfers)
    print("""
        The fewest payments possible. Finding them is NP-hard; the solver does it exactly by
        splitting the balances into as many groups that sum to zero as it can: here
        {Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than
        its size. \(minimum.payments.count) payments:
        """)
    try printPayments(group, minimum)

    section("4. Greedy")
    let greedy = try group.settlement(by: .greedy)
    print("""
        The largest debtor pays the largest creditor, again and again: fast, at most one
        payment fewer than the members with a balance, but not always the minimum. \(greedy.payments.count)
        payments here:
        """)
    try printPayments(group, greedy)

    section("5. Direct")
    let direct = try group.settlement(by: .direct)
    print("""
        No simplification: everyone pays back the people who paid for them, with the debts
        between each pair of members netted. The most payments (\(direct.payments.count)), but each one traces
        back to shared expenses:
        """)
    try printPayments(group, direct)

    section("6. The strategies side by side")
    printTable(
        ["Strategy", "Guarantee", "Cost", "Payments here"],
        [
            ["MinimumTransfers", "the fewest payments", "O(2ⁿ·n) for n open balances", "\(minimum.payments.count)"],
            ["Greedy", "at most n − 1 payments", "O(n log n)", "\(greedy.payments.count)"],
            ["Direct", "one payment per pair in debt", "O(pairs)", "\(direct.payments.count)"]
        ],
        numbersFrom: 3
    )
    return [minimum.payments.count, greedy.payments.count, direct.payments.count]
}

func main() throws(EranosError) {
    let group = try Group(currency: euro, name: "Ski week")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    let dan = try group.add(Member(firstName: "Dan"))
    let eva = try group.add(Member(firstName: "Eva"))
    let filip = try group.add(Member(firstName: "Filip"))

    print("Ski week: three ways to settle up")
    print("═════════════════════════════════")
    print("""
        Six friends spent a week in the mountains. The library can turn their balances
        into payments in three ways; the app picks one, and they differ here.
        """)

    section("1. What was spent")
    try spend(group, "Cabin", "12.00", dan, [dan, eva, filip])
    try spend(group, "Lift passes", "9.00", bob, [bob, carol, filip])
    try spend(group, "Groceries", "6.00", carol, [bob, dan])
    try spend(group, "Coffee", "2.00", ana, [dan, eva])
    try spend(group, "Snacks", "1.00", filip, [ana])
    printTable(
        ["Expense", "Paid by", "Shared by", "Amount"],
        try group.expenses().map { expense throws(EranosError) in
            [expense.name, try names(group, expense.payers.members), try names(group, participants(expense.split)), "\(try expense.total())"]
        },
        numbersFrom: 3
    )

    section("2. Balances")
    print("Four members are owed money and two owe it:")
    try printBalances(group)

    let payments = try strategies(group)

    section("7. Settling up for real")
    let plan = try group.settlement(by: .minimumTransfers)
    for payment in plan.payments {
        _ = try group.add(Transfer(cost: .local(payment.amount), date: try CalendarDate(day: 27, month: 9, year: 2026), from: payment.from, to: payment.to))
    }
    print("""
        Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards
        every balance is zero and a new settlement has nothing left to pay:
        """)
    try printBalances(group)
    print("Payments still needed: \(try group.settlement(by: .minimumTransfers).payments.count).")

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest amounts
    // first and needs five; the unsimplified debts between pairs need seven.
    precondition([4, 5, 7] == payments)
    let summary = try group.summary()
    for member in [ana, bob, carol, dan, eva, filip] {
        precondition(Balance.settled == totals(of: member, in: summary)?.balance)
    }
}

try main()
