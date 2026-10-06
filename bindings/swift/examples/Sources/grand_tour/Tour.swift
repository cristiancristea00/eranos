// What the sections of the tour share: the members and categories they name, the amounts and dates of the trip, and
// the tables several sections print.

import Eranos
import Report

/// The members of the trip.
struct People {
    let ana: Member.ID
    let bob: Member.ID
    let carol: Member.ID
    let dan: Member.ID
    let eva: Member.ID
    let filip: Member.ID
}

/// The categories the group adds to the built-in one.
struct Categories {
    let food: Category.ID
    let lodging: Category.ID
    let transport: Category.ID
}

func eur(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: Currency(code: "EUR"))
}

func ron(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: Currency(code: "RON"))
}

/// A date of the trip, in October 2026.
func october(_ date: Int32) throws(EranosError) -> CalendarDate {
    try CalendarDate(day: date, month: 10, year: 2026)
}

func name(_ group: Group, _ member: Member.ID) throws(EranosError) -> String {
    try names(group, [member])
}

/// `lines`, one below the other in a table cell.
func cell(_ lines: [String]) -> String {
    lines.joined(separator: "\n")
}

/// "valid: …" with `shown` of the value `make` returns, or "refused: …" with the message.
func outcome<Value>(_ make: () throws(EranosError) -> Value, _ shown: (Value) -> String) -> String {
    do {
        return "valid: \(shown(try make()))"
    } catch {
        return "refused: \(error.detail)"
    }
}

/// The message of the error `result` holds, or "accepted".
func message<Value>(_ result: Result<Value, EranosError>) -> String {
    switch result {
    case let .failure(error):
        error.detail
    case .success:
        "accepted"
    }
}

/// The sum of `amounts`, all in `currency`.
func sum(_ currency: Currency, _ amounts: [Money]) throws(EranosError) -> Money {
    var total = Money(currency: currency, minorUnits: 0)
    for amount in amounts {
        total = try total.adding(amount)
    }
    return total
}

/// Stops the tour, naming `what`, unless `found` is `expected`.
func expect<Value: Equatable>(_ expected: Value, _ found: Value?, _ what: String) {
    precondition(expected == found, what)
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

/// The identifier of the built-in category, which the group lists first.
func builtInCategory(_ group: Group) throws(EranosError) -> Category.ID? {
    try group.categories().first?.id
}

func printBalances(_ group: Group) throws(EranosError) {
    let summary = try group.summary()
    let zero = Money(currency: try group.currency(), minorUnits: 0)
    printTable(
        try group.members().map { member in
            // The summary lists the members with a total; every total of the others is zero.
            guard let totals = member.id.flatMap({ id in totals(of: id, in: summary) }) else {
                return BalanceRow(balance: .settled, consumed: zero, name: member.firstName, paid: zero, received: zero, sent: zero)
            }
            return BalanceRow(
                balance: totals.balance,
                consumed: totals.share,
                name: member.firstName,
                paid: totals.paid,
                received: totals.received,
                sent: totals.sent
            )
        },
        numbersFrom: 1
    )
}

func printCategories(_ group: Group) throws(EranosError) {
    let expenses = try group.expenses()
    let builtIn = try builtInCategory(group)
    printTable(
        try group.summary().categoryTotals.map { total throws(EranosError) in
            CategoryRow(
                expenses: expenses.filter { expense in (expense.category ?? builtIn) == total.category }.count,
                name: try group.category(withID: total.category)?.name ?? "?",
                total: total.total
            )
        },
        numbersFrom: 1
    )
}

/// Every expense and transfer in a currency `shown` accepts, as entered and in the group currency.
func printEntries(_ group: Group, _ shown: (Currency) -> Bool) throws(EranosError) {
    let zero = Money(currency: try group.currency(), minorUnits: 0)
    let expenses = try group.expenses().filter { expense in shown(expense.cost.amount.currency) }.map { expense throws(EranosError) in
        EntryRow(
            converted: try expense.id.flatMap { id throws(EranosError) in try group.shares(of: id) }?.total ?? zero,
            name: expense.name,
            original: try expense.total(),
            payer: try names(group, expense.payers.members),
            rate: try rate(of: expense.cost)
        )
    }
    let transfers = try group.transfers().filter { transfer in shown(transfer.cost.amount.currency) }.map { transfer throws(EranosError) in
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
