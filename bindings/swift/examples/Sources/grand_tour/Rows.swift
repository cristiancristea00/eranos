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

    let name: String
    let email: String
    let phone: String
    let accounts: String

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

    let expense: String
    let how: String
    let members: String
    let paid: String
    let consumed: String

    var cells: [String] {
        [expense, how, members, paid, consumed]
    }
}

struct ItemRow: TableRow {
    static let headers = ["Item", "Shared by", "Quantity", "Unit price", "Line total"]

    let name: String
    let sharedBy: String
    let quantity: Quantity
    let unitPrice: Money
    let lineTotal: Money

    var cells: [String] {
        [name, sharedBy, "\(quantity)", "\(unitPrice)", "\(lineTotal)"]
    }
}

struct EntryRow: TableRow {
    static let headers = ["Entry", "Paid by", "Entered as", "Rate", "In the group currency"]

    let name: String
    let payer: String
    let original: Money
    let rate: ExchangeRate
    let converted: Money

    var cells: [String] {
        [name, payer, "\(original)", "\(rate)", "\(converted)"]
    }
}

struct EditRow: TableRow {
    static let headers = ["Entry", "Edit"]

    let entry: String
    let edit: String

    var cells: [String] {
        [entry, edit]
    }
}

struct BalanceRow: TableRow {
    static let headers = ["Member", "Paid", "Consumed", "Sent", "Received", "Balance"]

    let name: String
    let paid: Money
    let consumed: Money
    let sent: Money
    let received: Money
    let balance: Balance

    var cells: [String] {
        [name, "\(paid)", "\(consumed)", "\(sent)", "\(received)", "\(balance)"]
    }
}

struct StatusRow: TableRow {
    static let headers = ["Member", "Status", "Balance"]

    let name: String
    let status: String
    let balance: Balance

    var cells: [String] {
        [name, status, "\(balance)"]
    }
}

struct CategoryRow: TableRow {
    static let headers = ["Category", "Expenses", "Total"]

    let name: String
    let expenses: Int
    let total: Money

    var cells: [String] {
        [name, "\(expenses)", "\(total)"]
    }
}

struct MemberCategoryRow: TableRow {
    static let headers = ["Member", "Food", "Lodging", "Getting around", "Uncategorised"]

    let name: String
    let food: Money
    let lodging: Money
    let transport: Money
    let uncategorised: Money

    var cells: [String] {
        [name, "\(food)", "\(lodging)", "\(transport)", "\(uncategorised)"]
    }
}

struct StrategyRow: TableRow {
    static let headers = ["Strategy", "Computed by", "Payments"]

    let strategy: String
    let computedBy: String
    let payments: Int

    var cells: [String] {
        [strategy, computedBy, "\(payments)"]
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
    let name: String
    let numericCode: Int
    let decimalPlaces: Int

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
