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
    try Money(parsing: text, in: CurrencyCode(code: "EUR"))
}

func ron(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: CurrencyCode(code: "RON"))
}

/// A date of the trip, in October 2026.
func october(_ date: Int32) throws(EranosError) -> CalendarDate {
    try CalendarDate(year: 2026, month: 10, day: date)
}

func name(_ group: ExpenseGroup, _ member: Member.ID) throws(EranosError) -> String {
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
func sum(_ currency: CurrencyCode, _ amounts: [Money]) throws(EranosError) -> Money {
    var total = Money.zero(in: currency)
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
    case .local:
        try ExchangeRate(parsing: "1")
    case let .converted(_, rate):
        rate
    }
}

func printBalances(_ group: ExpenseGroup) throws(EranosError) {
    let summary = group.summary
    printTable(
        group.members().map { member in
            let totals = summary.member(withID: member.id)
            return BalanceRow(
                name: member.firstName,
                paid: totals.paid,
                consumed: totals.share,
                sent: totals.sent,
                received: totals.received,
                balance: totals.balance
            )
        },
        numbersFrom: 1
    )
}

func printCategories(_ group: ExpenseGroup) throws(EranosError) {
    let expenses = group.expenses()
    printTable(
        try group.summary.categoryTotals.map { total throws(EranosError) in
            CategoryRow(
                name: try group.category(withID: total.category)?.name ?? "?",
                expenses: expenses.filter { expense in expense.category == total.category }.count,
                total: total.total
            )
        },
        numbersFrom: 1
    )
}

/// Every expense and transfer in a currency `shown` accepts, as entered and in the group currency.
func printEntries(_ group: ExpenseGroup, _ shown: (CurrencyCode) -> Bool) throws(EranosError) {
    let zero = Money.zero(in: group.currency)
    let expenses = try group.expenses().filter { expense in shown(expense.cost.amount.currency) }.map { expense throws(EranosError) in
        EntryRow(
            name: expense.name,
            payer: try names(group, expense.payers.members),
            original: try expense.total(),
            rate: try rate(of: expense.cost),
            converted: try group.shares(of: expense.id)?.total ?? zero
        )
    }
    let transfers = try group.transfers().filter { transfer in shown(transfer.cost.amount.currency) }.map { transfer throws(EranosError) in
        EntryRow(
            name: "Transfer to \(try name(group, transfer.receiver))",
            payer: try name(group, transfer.sender),
            original: transfer.cost.amount,
            rate: try rate(of: transfer.cost),
            converted: try group.amount(of: transfer.id) ?? zero
        )
    }
    printTable(expenses + transfers, numbersFrom: 2)
}
