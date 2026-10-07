// Every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact
// amounts, and with several payers. The port of the Rust example `split_methods`, which prints the same report.

import Eranos
import Report

struct ShareRow: TableRow {
    static let headers = ["Member", "Rule", "Paid", "Consumed"]

    let name: String
    let rule: String
    let paid: Money
    let consumed: Money

    var cells: [String] {
        [name, rule, "\(paid)", "\(consumed)"]
    }
}

struct TotalRow: TableRow {
    static let headers = ["Member", "Paid", "Consumed", "Balance"]

    let name: String
    let paid: Money
    let consumed: Money
    let balance: String

    var cells: [String] {
        [name, "\(paid)", "\(consumed)", balance]
    }
}

let euro = try CurrencyCode(code: "EUR")

func eur(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: euro)
}

/// Prints how one expense divides: every involved member's rule, what they paid, and what they consumed.
func report(_ group: ExpenseGroup, _ id: Expense.ID, _ rules: [Member.ID: String]) throws(EranosError) {
    guard let expense = try group.expense(withID: id), let shares = try group.shares(of: id) else {
        return
    }
    print("\(expense.name), \(shares.total):")
    printTable(
        try shares.members.map { share throws(EranosError) in
            ShareRow(name: try names(group, [share.member]), rule: rules[share.member] ?? "(payer only)", paid: share.paid, consumed: share.share)
        },
        numbersFrom: 2
    )
}

/// The family and the day every expense happened.
struct Family {
    let ana: Member.ID
    let bob: Member.ID
    let mia: Member.ID
    let day: CalendarDate

    /// The rule of an expense shared equally by the three.
    var thirds: [Member.ID: String] {
        [ana: "an equal third", bob: "an equal third", mia: "an equal third"]
    }
}

func equally(_ group: ExpenseGroup, _ family: Family) throws(EranosError) {
    section("1. Equally")
    print("""
        Everyone pays the same. 10.00 does not divide by three, so one member pays the
        leftover cent; which member it is rotates from expense to expense, so over a trip
        nobody always pays it.
        """)
    let taxi = try group.add(
        Expense(
            name: "Taxi",
            cost: .local(try eur("10.00")),
            payers: .equal([family.ana]),
            split: .whole(.equal([family.ana, family.bob, family.mia])),
            date: family.day
        )
    )
    try report(group, taxi, family.thirds)
}

func byPercentage(_ group: ExpenseGroup, _ family: Family) throws(EranosError) {
    section("2. By percentage")
    print("""
        Everyone pays a percentage of the amount; the percentages must add up to exactly
        100 %. Mia's room is smaller, so she takes 20 %.
        """)
    let rent = try group.add(
        Expense(
            name: "Flat",
            cost: .local(try eur("500.00")),
            payers: .equal([family.bob]),
            split: .whole(
                .percentages([
                    MemberPercentage(member: family.ana, percentage: try Percentage(parsing: "40")),
                    MemberPercentage(member: family.bob, percentage: try Percentage(parsing: "40")),
                    MemberPercentage(member: family.mia, percentage: try Percentage(parsing: "20"))
                ])
            ),
            date: family.day
        )
    )
    try report(group, rent, [family.ana: "40%", family.bob: "40%", family.mia: "20%"])
}

func byParts(_ group: ExpenseGroup, _ family: Family) throws(EranosError) -> Expense.ID {
    section("3. By parts")
    print("""
        Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:
        2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.
        """)
    let food = try group.add(
        Expense(
            name: "Food",
            cost: .local(try eur("125.00")),
            payers: .equal([family.ana]),
            split: .whole(
                .parts([
                    MemberParts(member: family.ana, parts: try Parts(parsing: "1")),
                    MemberParts(member: family.bob, parts: try Parts(parsing: "1")),
                    MemberParts(member: family.mia, parts: try Parts(parsing: "0.5"))
                ])
            ),
            date: family.day
        )
    )
    try report(group, food, [family.ana: "1 part", family.bob: "1 part", family.mia: "0.5 parts"])
    return food
}

func byAmounts(_ group: ExpenseGroup, _ family: Family) throws(EranosError) {
    section("4. By exact amounts")
    print("""
        Everyone pays a fixed amount, such as a ticket price; the amounts must add up to
        exactly the expense amount.
        """)
    let tickets = try group.add(
        Expense(
            name: "Museum",
            cost: .local(try eur("35.00")),
            payers: .equal([family.bob]),
            split: .whole(
                .amounts([
                    MemberAmount(member: family.ana, amount: try eur("15.00")),
                    MemberAmount(member: family.bob, amount: try eur("15.00")),
                    MemberAmount(member: family.mia, amount: try eur("5.00"))
                ])
            ),
            date: family.day
        )
    )
    try report(group, tickets, [family.ana: "15.00 EUR", family.bob: "15.00 EUR", family.mia: "5.00 EUR"])
}

func severalPayers(_ group: ExpenseGroup, _ family: Family) throws(EranosError) {
    section("5. Several payers")
    print("""
        The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are
        divided with the same methods as the participants.
        """)
    let dinner = try group.add(
        Expense(
            name: "Dinner",
            cost: .local(try eur("90.00")),
            payers: .amounts([
                MemberAmount(member: family.ana, amount: try eur("50.00")),
                MemberAmount(member: family.bob, amount: try eur("40.00"))
            ]),
            split: .whole(.equal([family.ana, family.bob, family.mia])),
            date: family.day
        )
    )
    try report(group, dinner, family.thirds)
}

func mistakes(_ group: ExpenseGroup, _ family: Family) throws(EranosError) {
    section("6. Mistakes are refused")
    print("A percentage split that adds up to 90 % is refused when the expense is built:")
    let invalid = Expense(
        name: "Invalid",
        cost: .local(try eur("10.00")),
        payers: .equal([family.ana]),
        split: .whole(
            .percentages([
                MemberPercentage(member: family.ana, percentage: try Percentage(parsing: "60")),
                MemberPercentage(member: family.bob, percentage: try Percentage(parsing: "30"))
            ])
        ),
        date: family.day
    )
    do {
        _ = try group.add(invalid)
        preconditionFailure("the split must be refused")
    } catch {
        print("  error: \(error.detail)")
    }
}

func printTotals(_ group: ExpenseGroup) throws(EranosError) {
    section("7. The holiday in total")
    let summary = try group.summary()
    print("\(summary.totalSpent) spent over five expenses:")
    printTable(
        try group.members().compactMap { member in
            member.id.flatMap { id in totals(of: id, in: summary) }.map { totals in
                TotalRow(name: member.firstName, paid: totals.paid, consumed: totals.share, balance: "\(totals.balance)")
            }
        },
        numbersFrom: 1
    )
}

func main() throws(EranosError) {
    let group = try ExpenseGroup(name: "Family holiday", currency: euro)
    let family = Family(
        ana: try group.add(Member(firstName: "Ana")),
        bob: try group.add(Member(firstName: "Bob")),
        mia: try group.add(Member(firstName: "Mia")),
        day: try CalendarDate(year: 2026, month: 7, day: 14)
    )

    print("Family holiday: every way to split an expense")
    print("═════════════════════════════════════════════")
    print("""
        Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a
        different split method; whatever the method, the shares add up to exactly the
        amount, to the cent.
        """)
    try equally(group, family)
    try byPercentage(group, family)
    let food = try byParts(group, family)
    try byAmounts(group, family)
    try severalPayers(group, family)
    try mistakes(group, family)
    try printTotals(group)

    // Parts: 125.00 over 2.5 parts is 50.00 per part.
    let half = try eur("25.00")
    let shares = try group.shares(of: food)
    precondition(half == shares?.members.first { share in share.member == family.mia }?.share)
}

try main()
