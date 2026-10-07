// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.

import Eranos
import Report

struct ExpenseRow: TableRow {
    static let headers = ["Expense", "Paid by", "Shared by", "Amount"]

    let name: String
    let payer: String
    let participants: String
    let amount: Money

    var cells: [String] {
        [name, payer, participants, "\(amount)"]
    }
}

struct BalanceRow: TableRow {
    static let headers = ["Member", "Paid", "Consumed", "Balance"]

    let name: String
    let paid: Money
    let consumed: Money
    let balance: Balance

    var cells: [String] {
        [name, "\(paid)", "\(consumed)", "\(balance)"]
    }
}

struct PaymentRow: TableRow {
    static let headers = ["From", "To", "Amount"]

    let from: String
    let to: String
    let amount: Money

    var cells: [String] {
        [from, to, "\(amount)"]
    }
}

struct StrategyRow: TableRow {
    static let headers = ["Strategy", "Guarantee", "Cost", "Payments here"]

    let strategy: String
    let guarantee: String
    let cost: String
    let payments: Int

    var cells: [String] {
        [strategy, guarantee, cost, "\(payments)"]
    }
}

/// The number of payments of each strategy's plan.
struct Counts: Equatable {
    let adaptive: Int
    let minimum: Int
    let greedy: Int
    let direct: Int
}

let euro = try Currency(code: "EUR")

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

/// Records an expense of `amount` EUR paid by `payer` and split equally.
func spend(_ group: ExpenseGroup, _ name: String, _ amount: String, _ payer: Member.ID, _ participants: [Member.ID]) throws(EranosError) {
    _ = try group.add(
        Expense(
            name: name,
            cost: .local(try Money(parsing: amount, in: euro)),
            payers: .equal([payer]),
            split: .whole(.equal(participants)),
            date: try CalendarDate(year: 2026, month: 9, day: 20)
        )
    )
}

func printBalances(_ group: ExpenseGroup) throws(EranosError) {
    let summary = try group.summary()
    printTable(
        try group.members().compactMap { member in
            member.id.flatMap { id in totals(of: id, in: summary) }.map { totals in
                BalanceRow(name: member.firstName, paid: totals.paid, consumed: totals.share, balance: totals.balance)
            }
        },
        numbersFrom: 1
    )
}

func printPayments(_ group: ExpenseGroup, _ settlement: SettlementPlan) throws(EranosError) {
    printTable(
        try settlement.payments.map { payment throws(EranosError) in
            PaymentRow(from: try names(group, [payment.from]), to: try names(group, [payment.to]), amount: payment.amount)
        },
        numbersFrom: 2
    )
}

/// Prints each strategy's plan with what it promises; returns the number of payments of each.
func strategies(_ group: ExpenseGroup) throws(EranosError) -> Counts {
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

    section("6. Adaptive")
    let adaptive = try group.settlement(by: .adaptive)
    print("""
        The fewest payments while they are quick to find, greedy's beyond: up to 20 open
        balances the minimum solver settles them, past that greedy does, and the plan
        names the strategy whose plan it is. Six balances are well within the limit, so
        the plan is the one \(strategyName(adaptive.strategy)) gives, the same \(adaptive.payments.count) payments.
        """)

    section("7. The strategies side by side")
    printTable(
        [
            StrategyRow(strategy: "MinimumTransfers", guarantee: "the fewest payments", cost: "O(2ⁿ·n) for n open balances", payments: minimum.payments.count),
            StrategyRow(strategy: "Greedy", guarantee: "at most n − 1 payments", cost: "O(n log n)", payments: greedy.payments.count),
            StrategyRow(strategy: "Direct", guarantee: "one payment per pair in debt", cost: "O(pairs)", payments: direct.payments.count),
            StrategyRow(strategy: "Adaptive", guarantee: "the fewest up to 20 balances", cost: "as MinimumTransfers, then Greedy", payments: adaptive.payments.count)
        ],
        numbersFrom: 3
    )
    return Counts(
        adaptive: adaptive.payments.count,
        minimum: minimum.payments.count,
        greedy: greedy.payments.count,
        direct: direct.payments.count
    )
}

/// Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which
/// strategy's plan it took; returns the number of payments.
func festival() throws(EranosError) -> Int {
    section("8. A larger group")
    let group = try ExpenseGroup(name: "Festival", currency: euro)
    var friends: [Member.ID] = []
    for number in 1...21 {
        friends.append(try group.add(Member(firstName: "Friend \(number)")))
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
    // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
    for (number, payer) in zip(Int64(1)..., friends) {
        _ = try group.add(
            Expense(
                name: "Round",
                cost: .local(Money(minorUnits: 21 * number * number, currency: euro)),
                payers: .equal([payer]),
                split: .whole(.equal(friends)),
                date: try CalendarDate(year: 2026, month: 9, day: 18)
            )
        )
    }
    let open = try group.summary().members.count { member in 0 != member.net.minorUnits }
    let adaptive = try group.settlement(by: .adaptive)
    let greedy = try group.settlement(by: .greedy)
    print("""
        Twenty-one friends at a festival each bought one round for everyone, each round
        at another price, so all \(open) end with a balance and no two of them cancel out.
        That is more than the 20 the adaptive strategy settles exactly, so it takes the
        plan \(strategyName(adaptive.strategy)) gives and says so: the same \(adaptive.payments.count) payments.
        """)
    precondition(21 == open)
    precondition(SettlementStrategy.greedy == adaptive.strategy)
    precondition(adaptive.payments == greedy.payments)
    return adaptive.payments.count
}

func main() throws(EranosError) {
    let group = try ExpenseGroup(name: "Ski week", currency: euro)
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    let dan = try group.add(Member(firstName: "Dan"))
    let eva = try group.add(Member(firstName: "Eva"))
    let filip = try group.add(Member(firstName: "Filip"))

    title("Ski week: four ways to settle up")
    print("""
        Six friends spent a week in the mountains. The library can turn their balances
        into payments in four ways, and the app picks one.
        """)

    section("1. What was spent")
    try spend(group, "Cabin", "12.00", dan, [dan, eva, filip])
    try spend(group, "Lift passes", "9.00", bob, [bob, carol, filip])
    try spend(group, "Groceries", "6.00", carol, [bob, dan])
    try spend(group, "Coffee", "2.00", ana, [dan, eva])
    try spend(group, "Snacks", "1.00", filip, [ana])
    printTable(
        try group.expenses().map { expense throws(EranosError) in
            ExpenseRow(
                name: expense.name,
                payer: try names(group, expense.payers.members),
                participants: try names(group, participants(expense.split)),
                amount: try expense.total()
            )
        },
        numbersFrom: 3
    )

    section("2. Balances")
    print("Four members are owed money and two owe it:")
    try printBalances(group)

    let counts = try strategies(group)
    let festivalPayments = try festival()

    section("9. Settling up for real")
    let plan = try group.settlement(by: .minimumTransfers)
    for payment in plan.payments {
        _ = try group.add(Transfer(from: payment.from, to: payment.to, cost: .local(payment.amount), date: try CalendarDate(year: 2026, month: 9, day: 27)))
    }
    print("""
        Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards
        every balance is zero and a new settlement has nothing left to pay:
        """)
    try printBalances(group)
    print("Payments still needed: \(try group.settlement(by: .minimumTransfers).payments.count).")

    // {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that
    // one; greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
    precondition(Counts(adaptive: 4, minimum: 4, greedy: 5, direct: 7) == counts)
    // Greedy settles the festival's 21 balances in 19 payments.
    precondition(19 == festivalPayments)
    let summary = try group.summary()
    for member in [ana, bob, carol, dan, eva, filip] {
        precondition(Balance.settled == totals(of: member, in: summary)?.balance)
    }
}

try main()
