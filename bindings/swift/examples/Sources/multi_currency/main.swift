// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example `multi_currency`,
// which prints the same report.

import Eranos
import Report

struct EntryRow: TableRow {
    static let headers = ["Entry", "Paid by", "Entered as", "Rate", "In the group currency"]

    let converted: Money
    let name: String
    let original: Money
    let payer: String
    let rate: ExchangeRate

    var cells: [String] {
        [name, payer, "\(original)", "\(rate)", "\(converted)"]
    }
}

struct BalanceRow: TableRow {
    static let headers = ["Member", "Paid", "Consumed", "Sent", "Received", "Balance"]

    let balance: Balance
    let consumed: Money
    let name: String
    let paid: Money
    let received: Money
    let sent: Money

    var cells: [String] {
        [name, "\(paid)", "\(consumed)", "\(sent)", "\(received)", "\(balance)"]
    }
}

let euro = try Currency(code: "EUR")
let forint = try Currency(code: "HUF")
let leu = try Currency(code: "RON")

/// The first name of `member` in `group`, or `?` for one the group does not hold.
func name(_ group: Group, _ member: Member.ID) throws(EranosError) -> String {
    try names(group, [member])
}

/// Every expense and transfer, as entered and in the group currency.
func printEntries(_ group: Group) throws(EranosError) {
    let zero = Money(currency: try group.currency(), minorUnits: 0)
    let expenses = try group.expenses().map { expense throws(EranosError) in
        EntryRow(
            converted: try expense.id.flatMap { id throws(EranosError) in try group.shares(of: id) }?.total ?? zero,
            name: expense.name,
            original: try expense.total(),
            payer: try names(group, expense.payers.members),
            rate: try rate(of: expense.cost)
        )
    }
    let transfers = try group.transfers().map { transfer throws(EranosError) in
        EntryRow(
            converted: try transfer.id.flatMap { id throws(EranosError) in try group.amount(of: id) } ?? zero,
            name: "Transfer to \(try name(group, transfer.to))",
            original: transfer.cost.amount,
            payer: try name(group, transfer.from),
            rate: try rate(of: transfer.cost)
        )
    }
    printTable(expenses + transfers, numbersFrom: 2)
}

/// The rate `cost` was entered at: 1 for a cost in the group currency.
func rate(of cost: Cost) throws(EranosError) -> ExchangeRate {
    switch cost {
    case let .converted(_, rate):
        rate
    case .local:
        try ExchangeRate(parsing: "1")
    }
}

func printBalances(_ group: Group) throws(EranosError) {
    let summary = try group.summary()
    printTable(
        try group.members().compactMap { member in
            member.id.flatMap { id in totals(of: id, in: summary) }.map { totals in
                BalanceRow(
                    balance: totals.balance,
                    consumed: totals.share,
                    name: member.firstName,
                    paid: totals.paid,
                    received: totals.received,
                    sent: totals.sent
                )
            }
        },
        numbersFrom: 1
    )
}

func expenses(_ group: Group, _ ana: Member.ID, _ bob: Member.ID) throws(EranosError) {
    section("1. Expenses in three currencies")
    let both = Split.equal([ana, bob])
    _ = try group.add(
        Expense(
            cost: .converted(try Money(parsing: "400.00", in: leu), rate: try ExchangeRate(parsing: "0.2")),
            date: try CalendarDate(day: 1, month: 8, year: 2026),
            name: "Fuel",
            payers: .equal([ana]),
            split: .whole(both)
        )
    )
    _ = try group.add(
        Expense(
            cost: .converted(try Money(parsing: "24000.00", in: forint), rate: try ExchangeRate(parsing: "0.0025")),
            date: try CalendarDate(day: 2, month: 8, year: 2026),
            name: "Dinner",
            payers: .equal([bob]),
            split: .whole(both)
        )
    )
    _ = try group.add(
        Expense(
            cost: .local(try Money(parsing: "60.00", in: euro)),
            date: try CalendarDate(day: 3, month: 8, year: 2026),
            name: "Ferry",
            payers: .equal([bob]),
            split: .whole(both)
        )
    )
    try printEntries(group)
    print("""
        The rate says how many \(try group.currency()) one unit of the entry's currency is worth: 400.00 RON at
        0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.
        """)
}

func repayment(_ group: Group, _ ana: Member.ID, _ bob: Member.ID) throws(EranosError) {
    section("3. A repayment in cash, in another currency")
    _ = try group.add(
        Transfer(
            cost: .converted(try Money(parsing: "100.00", in: leu), rate: try ExchangeRate(parsing: "0.2")),
            date: try CalendarDate(day: 4, month: 8, year: 2026),
            from: ana,
            to: bob,
            note: "Cash at the border"
        )
    )
    print("""
        Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.
        Transfers move balances but are not spending, so the total stays at \(try group.summary().totalSpent):
        """)
    try printBalances(group)
}

func newCurrency(_ group: Group) throws(EranosError) {
    section("4. Moving the books to another currency")
    print("""
        Changing the group currency needs a rate into the new currency for every other
        currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies
        take the new rate; entries already in RON become local costs, at 1.
        """)
    try group.changeCurrency(to: leu, rates: [euro: try ExchangeRate(parsing: "5"), forint: try ExchangeRate(parsing: "0.0125")])
    try printEntries(group)
    print("In \(try group.currency()), the trip cost \(try group.summary().totalSpent), and everyone is still settled:")
    try printBalances(group)
}

func main() throws(EranosError) {
    let group = try Group(currency: euro, name: "Danube road trip")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))

    print("\(try group.name()): expenses in several currencies")
    print("════════════════════════════════════════════════")
    print("""
        Ana and Bob drive from Romania to Hungary. The group keeps its books in \(try group.currency()); an
        expense in another currency carries the exchange rate used when it was entered,
        and is converted once, rounding half away from zero to the cent.
        """)
    try expenses(group, ana, bob)

    section("2. Balances")
    print("""
        The group spent \(try group.summary().totalSpent). Bob paid 120.00 of it but consumed half, so he gets back the
        difference:
        """)
    try printBalances(group)
    let owed = try Money(parsing: "20.00", in: euro)
    let afterExpenses = try group.summary()
    precondition(Balance.getsBack(owed) == totals(of: bob, in: afterExpenses)?.balance)

    try repayment(group, ana, bob)
    let afterRepayment = try group.summary()
    precondition(Balance.settled == totals(of: bob, in: afterRepayment)?.balance)

    try newCurrency(group)
    // 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
    let spent = try Money(parsing: "1000.00", in: leu)
    let afterChange = try group.summary()
    precondition(spent == afterChange.totalSpent)
    precondition(Balance.settled == totals(of: bob, in: afterChange)?.balance)
}

try main()
