// The rows of the tour's tables: a type for each table of the Rust example, with its headings and its column order.

import Eranos
import Report

struct DetailRow: TableRow {
    static let headers = ["Detail", "Value"]

    let detail: String
    let value: String

    var cells: [String] {
        [detail, value]
    }
}

struct MemberRow: TableRow {
    static let headers = ["Member", "Email", "Phone", "Accounts"]

    let accounts: String
    let email: String
    let name: String
    let phone: String

    var cells: [String] {
        [name, email, phone, accounts]
    }
}

struct CheckRow: TableRow {
    static let headers = ["Input", "Checked as", "Result"]

    let input: String
    let kind: String
    let result: String

    var cells: [String] {
        [input, kind, result]
    }
}

struct RefusalRow: TableRow {
    static let headers = ["Change", "Refused with"]

    let change: String
    let error: String

    var cells: [String] {
        [change, error]
    }
}

struct ShareRow: TableRow {
    static let headers = ["Expense", "Split", "Member", "Paid", "Consumed"]

    let consumed: String
    let expense: String
    let how: String
    let members: String
    let paid: String

    var cells: [String] {
        [expense, how, members, paid, consumed]
    }
}

struct ItemRow: TableRow {
    static let headers = ["Item", "Shared by", "Quantity", "Unit price", "Line total"]

    let lineTotal: Money
    let name: String
    let quantity: Int32
    let sharedBy: String
    let unitPrice: Money

    var cells: [String] {
        [name, sharedBy, "\(quantity)", "\(unitPrice)", "\(lineTotal)"]
    }
}

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

struct EditRow: TableRow {
    static let headers = ["Entry", "Edit"]

    let edit: String
    let entry: String

    var cells: [String] {
        [entry, edit]
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

struct StatusRow: TableRow {
    static let headers = ["Member", "Status", "Balance"]

    let balance: Balance
    let name: String
    let status: String

    var cells: [String] {
        [name, status, "\(balance)"]
    }
}

struct CategoryRow: TableRow {
    static let headers = ["Category", "Expenses", "Total"]

    let expenses: Int
    let name: String
    let total: Money

    var cells: [String] {
        [name, "\(expenses)", "\(total)"]
    }
}

struct MemberCategoryRow: TableRow {
    static let headers = ["Member", "Food", "Lodging", "Getting around", "Uncategorised"]

    let food: Money
    let lodging: Money
    let name: String
    let transport: Money
    let uncategorised: Money

    var cells: [String] {
        [name, "\(food)", "\(lodging)", "\(transport)", "\(uncategorised)"]
    }
}

struct StrategyRow: TableRow {
    static let headers = ["Strategy", "Computed by", "Payments"]

    let computedBy: String
    let payments: Int
    let strategy: String

    var cells: [String] {
        [strategy, computedBy, "\(payments)"]
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

struct LoadRow: TableRow {
    static let headers = ["Change to the file", "Message"]

    let change: String
    let message: String

    var cells: [String] {
        [change, message]
    }
}

struct CurrencyRow: TableRow {
    static let headers = ["Code", "Name", "Numeric code", "Decimal places"]

    let code: String
    let decimalPlaces: Int
    let name: String
    let numericCode: Int

    var cells: [String] {
        [code, name, "\(numericCode)", "\(decimalPlaces)"]
    }
}

struct ValueRow: TableRow {
    static let headers = ["Input", "Result"]

    let input: String
    let result: String

    var cells: [String] {
        [input, result]
    }
}
