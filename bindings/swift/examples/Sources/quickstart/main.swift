// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.

import Eranos
import Report

struct ExpenseRow: TableRow {
    static let headers = ["Expense", "Paid by", "Shared by", "Amount"]

    let amount: Money
    let name: String
    let participants: String
    let payers: String

    var cells: [String] {
        [name, payers, participants, "\(amount)"]
    }
}

struct MemberRow: TableRow {
    static let headers = ["Member", "Paid", "Consumed", "Balance"]

    let balance: Balance
    let consumed: Money
    let name: String
    let paid: Money

    var cells: [String] {
        [name, "\(paid)", "\(consumed)", "\(balance)"]
    }
}

struct PaymentRow: TableRow {
    static let headers = ["From", "To", "Amount"]

    let amount: Money
    let from: String
    let to: String

    var cells: [String] {
        [from, to, "\(amount)"]
    }
}

func main() throws(EranosError) {
    let euro = try Currency(code: "EUR")
    let group = try Group(currency: euro, name: "Weekend in Sinaia")
    title(try group.name())
    print("""
        Three friends rent a cabin and buy groceries. The group keeps its books in \(try group.currency()), so
        every total, balance, and payment below is in \(try group.currency()).
        """)

    section("1. Members")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    print("\(try group.members().count) people joined the group: \(try names(group, [ana, bob, carol])).")

    section("2. Expenses")
    print("""
        Each expense records who paid and who shares it. Here everyone shares everything
        equally.
        """)
    let friday = try CalendarDate(day: 25, month: 9, year: 2026)
    let everyone = Split.equal([ana, bob, carol])
    _ = try group.add(Expense(cost: .local(Money(parsing: "240.00", in: euro)), date: friday, name: "Cabin", payers: .equal([ana]), split: .whole(everyone)))
    _ = try group.add(Expense(cost: .local(Money(parsing: "60.00", in: euro)), date: friday, name: "Groceries", payers: .equal([bob]), split: .whole(everyone)))
    printTable(
        try group.expenses().map { expense throws(EranosError) in
            ExpenseRow(
                amount: try expense.total(),
                name: expense.name,
                participants: try names(group, participants(expense.split)),
                payers: try names(group, expense.payers.members)
            )
        },
        numbersFrom: 3
    )

    section("3. Who paid and who consumed")
    let summary = try group.summary()
    print("""
        The group spent \(summary.totalSpent) in total. A member's share of an expense is what they consumed,
        and their balance is what they paid minus what they consumed.
        """)
    printTable(
        try group.members().compactMap { member in
            member.id.flatMap { id in totals(of: id, in: summary) }.map { totals in
                MemberRow(balance: totals.balance, consumed: totals.share, name: member.firstName, paid: totals.paid)
            }
        },
        numbersFrom: 1
    )
    let anaTotals = totals(of: ana, in: summary)
    print("Ana paid \(anaTotals?.paid.description ?? "?") but consumed only \(anaTotals?.share.description ?? "?"), so the others owe her the difference.")

    section("4. Settling up")
    let settlement = try group.settlement(by: .minimumTransfers)
    print("The fewest payments that bring every balance to zero (\(settlement.payments.count) in total):")
    printTable(
        try settlement.payments.map { payment throws(EranosError) in
            PaymentRow(amount: payment.amount, from: try names(group, [payment.from]), to: try names(group, [payment.to]))
        },
        numbersFrom: 2
    )
    print("Once these payments are made, nobody owes anything.")

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    let spent = try Money(parsing: "300.00", in: euro)
    let owed = try Money(parsing: "140.00", in: euro)
    precondition(spent == summary.totalSpent)
    precondition(Balance.getsBack(owed) == anaTotals?.balance)
    precondition(2 == settlement.payments.count)
}

try main()
