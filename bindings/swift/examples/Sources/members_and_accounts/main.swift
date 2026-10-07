// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

import Eranos
import Report

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

struct BalanceRow: TableRow {
    static let headers = ["Member", "Status", "Consumed", "Balance"]

    let name: String
    let status: String
    let consumed: Money
    let balance: Balance

    var cells: [String] {
        [name, status, "\(consumed)", "\(balance)"]
    }
}

/// The two flatmates: Ana, who gives every detail, and Bob, who moves out.
struct Flatmates {
    let ana: Member.ID
    let bob: Member.ID
}

let euro = try CurrencyCode(code: "EUR")

/// An account with its label and its details.
func describe(_ account: PaymentAccount) -> String {
    let details = switch account.details {
    case let .iban(iban):
        iban.description
    case let .username(username):
        username.description
    }
    return "\(account.name): \(details)"
}

/// `valid: ` and what `shown` writes of the value `make` returns, or `refused: ` and the library's message.
func outcome<Value>(_ make: () throws(EranosError) -> Value, _ shown: (Value) -> String) -> String {
    do {
        return "valid: \(shown(try make()))"
    } catch {
        return "refused: \(error.detail)"
    }
}

func printMembers(_ group: ExpenseGroup) throws(EranosError) {
    printTable(
        try group.members().map { member in
            MemberRow(
                name: [member.firstName, member.lastName].compactMap(\.self).joined(separator: " "),
                email: member.email?.description ?? "",
                phone: member.phone?.description ?? "",
                accounts: member.accounts.map(describe).joined(separator: "\n")
            )
        },
        numbersFrom: 4
    )
}

func printBalances(_ group: ExpenseGroup) throws(EranosError) {
    let summary = try group.summary()
    var rows: [BalanceRow] = []
    for member in try group.members() {
        guard let totals = totals(of: member.id, in: summary) else {
            continue
        }
        rows.append(BalanceRow(name: member.firstName, status: try group.isArchived(member.id) ? "archived" : "active", consumed: totals.share, balance: totals.balance))
    }
    printTable(rows, numbersFrom: 2)
}

func members(_ group: ExpenseGroup) throws(EranosError) -> Flatmates {
    section("1. Members")
    let ana = try group.add(
        Member(
            firstName: "Ana",
            lastName: "Popescu",
            email: try Email("ana@example.com"),
            phone: try PhoneNumber("+40 712 345 678"),
            accounts: [
                PaymentAccount(name: "ING", details: .iban(try IBAN("ro49 aaaa 1b31 0075 9384 0000"))),
                PaymentAccount(name: "Revolut", details: .username(try Username("@ana")))
            ]
        )
    )
    let bob = try group.add(Member(firstName: "Bob"))
    try printMembers(group)
    print("""
        The phone number was stored in E.164 form, +40712345678, and the table shows it
        grouped for reading; the IBAN was stored in its electronic form, uppercase and
        without spaces, the way banking apps accept it.
        """)
    return Flatmates(ana: ana, bob: bob)
}

func validation() {
    section("2. What validation accepts")
    print("""
        IBANs are checked against the SWIFT registry: the country, its length, which
        positions hold digits and which letters, and the mod-97 check digits, which catch
        any single mistyped character. Emails are checked against the rules of RFC 5321
        and 5322, which refuse the doubled dots and stray commas of hurried typing, and
        phone numbers against their country's numbering plan, stored in E.164 form.
        """)
    let iban = { (input: String) in CheckRow(input: input, kind: "IBAN", result: outcome({ () throws(EranosError) in try IBAN(input) }, \.description)) }
    let email = { (input: String) in CheckRow(input: input, kind: "email", result: outcome({ () throws(EranosError) in try Email(input) }, \.description)) }
    let phone = { (input: String) in CheckRow(input: input, kind: "phone", result: outcome({ () throws(EranosError) in try PhoneNumber(input) }, \.e164)) }
    printTable(
        [
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            iban("RO49 AAAA 1B31 0075"),
            iban("DE46 37O4 0044 0532 0130 00"),
            email("ana@example.com"),
            email("ana.example.com"),
            email("ana..popescu@example.com"),
            phone("+40 (712) 345-678"),
            phone("0712 345 678"),
            phone("+40 612 345 678"),
            phone("+44 (0)20 7946 0958")
        ],
        numbersFrom: 3
    )
}

func movingOut(_ group: ExpenseGroup, _ flatmates: Flatmates) throws(EranosError) {
    section("3. A flatmate moves out")
    let ana = flatmates.ana
    let bob = flatmates.bob
    let rent = try group.add(
        Expense(
            name: "Rent",
            cost: .local(try Money(parsing: "900.00", in: euro)),
            payers: .equal([ana]),
            split: .whole(.equal([ana, bob])),
            date: try CalendarDate(year: 2026, month: 9, day: 1)
        )
    )
    try group.archive(bob)
    print("""
        Bob shared September's rent, then moved out. He is archived, not removed: his
        history and his balance stay, and he can still pay Ana back.
        """)
    try printBalances(group)

    let utilities = Expense(
        name: "Utilities",
        cost: .local(try Money(parsing: "60.00", in: euro)),
        payers: .equal([ana]),
        split: .whole(.equal([ana, bob])),
        date: try CalendarDate(year: 2026, month: 10, day: 1)
    )
    let refused: EranosError
    do {
        _ = try group.add(utilities)
        preconditionFailure("the archived member must be refused")
    } catch {
        refused = error
    }
    print("October's utilities cannot include him:\n  error: \(refused.detail)")
    let removal: EranosError
    do {
        _ = try group.remove(bob)
        preconditionFailure("the referenced member must stay")
    } catch {
        removal = error
    }
    print("Nor can he be removed while the rent references him:\n  error: \(removal.detail)")

    precondition(EranosError.expenseGroup(.archivedMember(bob), detail: refused.detail) == refused)
    guard case .expenseGroup(.memberInUse, _) = removal else {
        preconditionFailure("the rent references Bob")
    }
    let owed = try Money(parsing: "450.00", in: euro)
    let summary = try group.summary()
    precondition(Balance.owes(owed) == totals(of: bob, in: summary)?.balance)
    let stored = try group.expense(withID: rent)
    precondition(nil != stored)
}

func main() throws(EranosError) {
    let group = try ExpenseGroup(name: "Flat 4B", currency: euro)
    print("Flat 4B: members and their accounts")
    print("═══════════════════════════════════")
    print("""
        Ana and Bob share a flat. A member needs only a first name; the last name,
        email, phone, and the accounts they can be paid to are optional.
        """)
    let flatmates = try members(group)
    validation()
    try movingOut(group, flatmates)
}

try main()
