// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

import Eranos
import Report

struct ComparisonRow: TableRow {
    static let headers = ["Member", "Balance before saving", "Balance after loading"]

    let after: String
    let before: String
    let name: String

    var cells: [String] {
        [name, before, after]
    }
}

func main() throws(EranosError) {
    let euro = try Currency(code: "EUR")
    let day = try CalendarDate(day: 23, month: 9, year: 2026)
    let group = try Group(currency: euro, name: "Road trip")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    _ = try group.add(Expense(cost: .local(Money(parsing: "70.00", in: euro)), date: day, name: "Fuel", payers: .equal([ana]), split: .whole(.equal([ana, bob]))))
    _ = try group.add(
        Expense(
            cost: .local(Money(parsing: "30.00", in: euro)),
            date: day,
            name: "Lunch",
            payers: .equal([bob]),
            split: .whole(.equal([ana, bob])),
            tip: Tip(value: .percentage(try Percentage(parsing: "10")))
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
    let saved = try group.save()
    print(saved)
    print("""
        Amounts are minor units with their currency, dates are YYYY-MM-DD, and the
        `format` number lets a future release refuse a file it cannot read instead of
        dropping fields.
        """)

    section("2. Loading it back")
    let loaded = try Group.load(saved)
    print("""
        Loading replays every entry through the same checks as adding it, then rebuilds
        the totals. The loaded group equals the saved one, and so do its balances:
        """)
    let before = try group.summary()
    let after = try loaded.summary()
    printTable(
        try [ana, bob].map { member throws(EranosError) in
            ComparisonRow(
                after: totals(of: member, in: after)?.balance.description ?? "?",
                before: totals(of: member, in: before)?.balance.description ?? "?",
                name: try names(group, [member])
            )
        },
        numbersFrom: 1
    )
    let plan = try loaded.settlement(by: .greedy)
    print("Payments needed to settle the loaded group: \(plan.payments.count).")

    section("3. A corrupted file is refused")
    print("""
        Here the fuel's payer was edited to an identifier no member has. Reading the file
        as a SavedGroup and loading it with into_group replays every entry through the
        same checks as adding it, and refuses the file instead of producing balances that
        do not add up:
        """)
    // The fuel is the first entry that names Ana: the expenses come before the members in the saved form.
    let corrupted = saved.replacing(ana.rawValue, with: "0199a000-0000-7000-8000-00000000ffff", maxReplacements: 1)
    do {
        _ = try Group.load(corrupted)
        preconditionFailure("the file must be refused")
    } catch {
        print("  error: \(error.detail)")
    }

    let resaved = try loaded.save()
    precondition(saved == resaved)
    precondition(before == after)
    precondition(1 == plan.payments.count)
}

try main()
