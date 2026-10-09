// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.

import Eranos
import Report

struct ExpenseRow: TableRow {
    static let headers = ["Expense", "Paid by", "Shared by", "Amount"]

    let name: String
    let payers: String
    let participants: String
    let amount: Money

    var cells: [String] {
        [name, payers, participants, "\(amount)"]
    }
}

struct MemberRow: TableRow {
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

    let sender: String
    let receiver: String
    let amount: Money

    var cells: [String] {
        [sender, receiver, "\(amount)"]
    }
}

func main() throws(EranosError) {
    let euro = try CurrencyCode(code: "EUR")
    let group = try ExpenseGroup(name: "Weekend in Sinaia", currency: euro)
    title(group.name)
    print("""
        Three friends rent a cabin and buy groceries. The group keeps its books in \(group.currency), so
        every total, balance, and payment below is in \(group.currency).
        """)

    section("1. Members")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    print("\(group.members().count) people joined the group: \(try names(group, [ana, bob, carol])).")

    section("2. Expenses")
    print("""
        Each expense records who paid and who shares it. Here everyone shares everything
        equally.
        """)
    let friday = try CalendarDate(year: 2026, month: 9, day: 25)
    let everyone = Split.equal([ana, bob, carol])
    _ = try group.add(Expense(name: "Cabin", cost: .local(Money(parsing: "240.00", in: euro)), payers: .equal([ana]), split: .whole(everyone), date: friday))
    _ = try group.add(Expense(name: "Groceries", cost: .local(Money(parsing: "60.00", in: euro)), payers: .equal([bob]), split: .whole(everyone), date: friday))
    printTable(
        try group.expenses().map { expense throws(EranosError) in
            ExpenseRow(
                name: expense.name,
                payers: try names(group, expense.payers.members),
                participants: try names(group, try expense.split.participants()),
                amount: try expense.total()
            )
        },
        numbersFrom: 3
    )

    section("3. Who paid and who consumed")
    let summary = group.summary
    print("""
        The group spent \(summary.totalSpent) in total. A member's share of an expense is what they consumed,
        and their balance is what they paid minus what they consumed.
        """)
    printTable(
        group.members().map { member in
            let totals = summary.member(withID: member.id)
            return MemberRow(name: member.firstName, paid: totals.paid, consumed: totals.share, balance: totals.balance)
        },
        numbersFrom: 1
    )
    let anaTotals = summary.member(withID: ana)
    print("Ana paid \(anaTotals.paid) but consumed only \(anaTotals.share), so the others owe her the difference.")

    section("4. Settling up")
    let settlement = try group.settlement(by: .minimumTransfers)
    print("The fewest payments that bring every balance to zero (\(settlement.payments.count) in total):")
    printTable(
        try settlement.payments.map { payment throws(EranosError) in
            PaymentRow(sender: try names(group, [payment.sender]), receiver: try names(group, [payment.receiver]), amount: payment.amount)
        },
        numbersFrom: 2
    )
    print("Once these payments are made, nobody owes anything.")

    // Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
    let spent = try Money(parsing: "300.00", in: euro)
    let owed = try Money(parsing: "140.00", in: euro)
    precondition(spent == summary.totalSpent)
    precondition(Balance.getsBack(owed) == anaTotals.balance)
    precondition(2 == settlement.payments.count)
}

try main()
