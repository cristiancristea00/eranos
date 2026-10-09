// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

import Eranos
import Report

struct ComparisonRow: TableRow {
    static let headers = ["Member", "Balance before saving", "Balance after loading"]

    let name: String
    let before: String
    let after: String

    var cells: [String] {
        [name, before, after]
    }
}

func main() throws(EranosError) {
    let euro = try CurrencyCode(code: "EUR")
    let day = try CalendarDate(year: 2026, month: 9, day: 23)
    let group = try ExpenseGroup(name: "Road trip", currency: euro)
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    _ = try group.add(Expense(name: "Fuel", cost: .local(Money(parsing: "70.00", in: euro)), payers: .equal([ana]), split: .whole(.equal([ana, bob])), date: day))
    _ = try group.add(
        Expense(
            name: "Lunch",
            cost: .local(Money(parsing: "30.00", in: euro)),
            payers: .equal([bob]),
            split: .whole(.equal([ana, bob])),
            date: day,
            tip: TipRule(value: .percentage(try Percentage(parsing: "10")))
        )
    )

    title("Road trip: saving and loading a group")
    print("""
        With the `serde` feature a group serialises, and so does every type it stores.
        It saves only what was entered: its details, members, categories, expenses,
        and transfers. The totals, balances, and settlements are recomputed when it is
        loaded.
        """)

    section("1. The saved form")
    // Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an encrypted
    // database).
    let saved = group.save()
    print(saved)
    print("""
        Amounts are minor units with their currency, dates are YYYY-MM-DD, and the
        `format` number lets a future release refuse a file it cannot read instead of
        dropping fields.
        """)

    section("2. Loading it back")
    let loaded = try ExpenseGroup.load(saved)
    print("""
        Loading replays every entry through the same checks as adding it, then rebuilds
        the totals. The loaded group equals the saved one, and so do its balances:
        """)
    let before = group.summary
    let after = loaded.summary
    printTable(
        try [ana, bob].map { member throws(EranosError) in
            ComparisonRow(
                name: try names(group, [member]),
                before: before.member(withID: member).balance.description,
                after: after.member(withID: member).balance.description
            )
        },
        numbersFrom: 1
    )
    let plan = try loaded.settlement(by: .greedy)
    print("Payments needed to settle the loaded group: \(plan.payments.count).")

    section("3. A corrupted file is refused")
    print("""
        Here Ana's identifier was edited in her member entry, so the fuel's payer is an
        identifier no member has. Reading the file as a SavedExpenseGroup and loading it
        with into_expense_group replays every entry through the same checks as adding it,
        and refuses the file instead of producing balances that do not add up:
        """)
    // Ana's own entry is the first to name her: the members come before the expenses in the saved form, so the fuel she
    // paid is left naming an identifier no member has.
    let corrupted = saved.replacing(ana.rawValue, with: "0199a000-0000-7000-8000-00000000ffff", maxReplacements: 1)
    do {
        _ = try ExpenseGroup.load(corrupted)
        preconditionFailure("the file must be refused")
    } catch {
        print("  error: \(error.detail)")
    }

    let resaved = loaded.save()
    precondition(saved == resaved)
    precondition(before == after)
    precondition(1 == plan.payments.count)
}

try main()
