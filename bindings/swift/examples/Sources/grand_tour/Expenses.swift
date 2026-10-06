// Section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption.

import Eranos
import Report

/// The seven expenses, one for every way of dividing one.
struct Expenses {
    let apartment: Expense.ID
    let fado: Expense.ID
    let groceries: Expense.ID
    let museum: Expense.ID
    let seafood: Expense.ID
    let tour: Expense.ID
    let train: Expense.ID
}

/// How an expense divides, in a few words.
func how(_ expense: Expense) throws(EranosError) -> String {
    let split = switch expense.split {
    case let .itemised(items):
        "\(items.count) items"
    case .whole(.amounts):
        "by amounts"
    case .whole(.equal):
        "equally"
    case .whole(.parts):
        "by parts"
    case .whole(.percentages):
        "by percentage"
    }
    let count = expense.payers.members.count
    let payers = 1 == count ? "" : ", \(count) payers"
    var tip = ""
    if try 0 != expense.tipAmount().minorUnits, let value = expense.tip?.value {
        tip = switch value {
        case let .fixed(amount):
            ", \(amount) tip"
        case let .percentage(rate):
            ", \(rate) tip"
        }
    }
    return split + payers + tip
}

/// Each expense of `expenses`, with every involved member's payment and consumption.
func printShares(_ group: Group, _ expenses: [Expense.ID]) throws(EranosError) {
    var rows: [ShareRow] = []
    for id in expenses {
        guard let expense = try group.expense(withID: id), let shares = try group.shares(of: id) else {
            continue
        }
        rows.append(
            ShareRow(
                consumed: cell(shares.members.map { share in "\(share.share)" }),
                expense: expense.name,
                how: try how(expense),
                members: cell(try shares.members.map { share throws(EranosError) in try name(group, share.member) }),
                paid: cell(shares.members.map { share in 0 == share.paid.minorUnits ? "" : "\(share.paid)" })
            )
        )
    }
    printTable(rows, numbersFrom: 3)
}

func printItems(_ group: Group, _ id: Expense.ID) throws(EranosError) {
    guard case let .itemised(items) = try group.expense(withID: id)?.split else {
        return
    }
    printTable(
        try items.map { item throws(EranosError) in
            ItemRow(
                lineTotal: Money(currency: item.unitPrice.currency, minorUnits: item.unitPrice.minorUnits * Int64(item.quantity)),
                name: item.name,
                quantity: item.quantity,
                sharedBy: try names(group, item.split.members),
                unitPrice: item.unitPrice
            )
        },
        numbersFrom: 2
    )
}

/// The receipt of the fado night: every line with its own split.
func fadoItems(_ people: People) throws(EranosError) -> [Item] {
    [
        Item(name: "Vinho verde", split: .equal([people.ana, people.bob, people.dan]), unitPrice: try eur("12.00"), quantity: 2),
        Item(name: "Bacalhau", split: .equal([people.carol]), unitPrice: try eur("18.00")),
        Item(
            name: "Custard tarts",
            split: .parts([
                MemberParts(member: people.ana, parts: try Parts(parsing: "1")),
                MemberParts(member: people.bob, parts: try Parts(parsing: "1")),
                MemberParts(member: people.carol, parts: try Parts(parsing: "1")),
                MemberParts(member: people.dan, parts: try Parts(parsing: "1")),
                MemberParts(member: people.eva, parts: try Parts(parsing: "2"))
            ]),
            unitPrice: try eur("1.50"),
            quantity: 6
        ),
        Item(name: "Juice", split: .equal([people.eva]), unitPrice: try eur("3.00"))
    ]
}

/// The fado night, item by item, paid by Carol and Dan, with a 10 % tip.
func fadoNight(_ people: People, _ categories: Categories) throws(EranosError) -> Expense {
    Expense(
        cost: .local(try eur("54.00")),
        date: try october(4),
        name: "Fado night",
        payers: .amounts([MemberAmount(amount: try eur("39.40"), member: people.carol), MemberAmount(amount: try eur("20.00"), member: people.dan)]),
        split: .itemised(try fadoItems(people)),
        category: categories.food,
        description: "Tasca do Chico",
        time: try TimeOfDay(hour: 22, minute: 15),
        tip: Tip(value: .percentage(try Percentage(parsing: "10")))
    )
}

/// Adds one expense for every way of dividing one.
func addExpenses(_ group: Group, _ people: People, _ categories: Categories) throws(EranosError) -> Expenses {
    let groceries = try group.add(
        Expense(
            cost: .local(try eur("10.00")),
            date: try october(1),
            name: "Groceries",
            payers: .equal([people.ana]),
            split: .whole(.equal([people.ana, people.bob, people.carol])),
            category: categories.food
        )
    )
    let apartment = try group.add(
        Expense(
            cost: .local(try eur("800.00")),
            date: try october(1),
            name: "Apartment",
            payers: .equal([people.ana]),
            split: .whole(
                .percentages([
                    MemberPercentage(member: people.ana, percentage: try Percentage(parsing: "25")),
                    MemberPercentage(member: people.bob, percentage: try Percentage(parsing: "25")),
                    MemberPercentage(member: people.carol, percentage: try Percentage(parsing: "35")),
                    MemberPercentage(member: people.dan, percentage: try Percentage(parsing: "15"))
                ])
            ),
            category: categories.lodging
        )
    )
    let seafood = try group.add(
        Expense(
            cost: .local(try eur("90.00")),
            date: try october(2),
            name: "Seafood dinner",
            payers: .equal([people.carol]),
            split: .whole(
                .parts([
                    MemberParts(member: people.ana, parts: try Parts(parsing: "1")),
                    MemberParts(member: people.bob, parts: try Parts(parsing: "1")),
                    MemberParts(member: people.carol, parts: try Parts(parsing: "1")),
                    MemberParts(member: people.dan, parts: try Parts(parsing: "1")),
                    MemberParts(member: people.eva, parts: try Parts(parsing: "0.5"))
                ])
            ),
            category: categories.food
        )
    )
    let train = try group.add(
        Expense(
            cost: .local(try eur("88.50")),
            date: try october(3),
            name: "Train to Porto",
            payers: .equal([people.bob]),
            split: .whole(
                .amounts([
                    MemberAmount(amount: try eur("24.50"), member: people.ana),
                    MemberAmount(amount: try eur("24.50"), member: people.bob),
                    MemberAmount(amount: try eur("24.50"), member: people.carol),
                    MemberAmount(amount: try eur("15.00"), member: people.eva)
                ])
            ),
            category: categories.transport
        )
    )
    let museum = try group.add(
        Expense(
            cost: .local(try eur("60.00")),
            date: try october(3),
            name: "Museum",
            payers: .equal([people.ana, people.bob]),
            split: .whole(.equal([people.ana, people.bob, people.carol, people.dan]))
        )
    )
    let fado = try group.add(try fadoNight(people, categories))
    let tour = try group.add(
        Expense(
            cost: .local(try eur("45.00")),
            date: try october(5),
            name: "Tuk-tuk tour",
            payers: .equal([people.dan]),
            split: .whole(.equal([people.ana, people.bob, people.carol, people.dan, people.eva])),
            category: categories.transport,
            tip: Tip(value: .fixed(try eur("5.00")), distribution: .equal)
        )
    )
    return Expenses(apartment: apartment, fado: fado, groceries: groceries, museum: museum, seafood: seafood, tour: tour, train: train)
}

/// The consumption of `member` in the expense `id`.
func consumed(_ group: Group, _ id: Expense.ID, _ member: Member.ID) throws(EranosError) -> Money? {
    try group.shares(of: id)?.members.first { share in share.member == member }?.share
}

/// Prints how the fado night's tip was shared, and its receipt.
func printFado(_ group: Group, _ fado: Expense) throws(EranosError) {
    let rule = switch fado.tip?.value {
    case let .fixed(amount)?:
        "\(amount)"
    case let .percentage(rate)?:
        "\(rate)"
    case nil:
        "a tip"
    }
    let shared = switch fado.tip?.distribution {
    case .equal?:
        "equally"
    case .proportional?:
        "in proportion to what each member ordered"
    case nil:
        "another way"
    }
    print("""
        The groceries' 10.00 EUR did not divide into thirds, so one member consumed the
        leftover cent; the expense's identifier picks which, so it moves from expense to
        expense. The fado night, at \(fado.time?.description ?? "") on \(fado.date) at \(fado.description ?? ""), was itemised; its
        \(rule) tip of \(try fado.tipAmount()) was shared \(shared):
        """)
    if let id = fado.id {
        try printItems(group, id)
    }
    print("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.")
}

func expenses(_ group: Group, _ people: People, _ categories: Categories) throws(EranosError) -> Expense.ID {
    section("4. Expenses, one for every way of dividing")
    let added = try addExpenses(group, people, categories)
    print("""
        Seven expenses cover every way of dividing one: equally, by percentage, by parts
        (Eva counts as half), by exact amounts, with two payers, item by item, and with a
        tip. Every member's consumption adds up to the expense's total, to the cent:
        """)
    try printShares(group, [added.groceries, added.apartment, added.seafood, added.train, added.museum, added.fado, added.tour])
    guard let fadoExpense = try group.expense(withID: added.fado) else {
        preconditionFailure("the fado night was added")
    }
    try printFado(group, fadoExpense)

    let thirds = try [people.ana, people.bob, people.carol].map { member throws(EranosError) in try consumed(group, added.groceries, member)?.minorUnits }
    precondition([333, 333, 334] == thirds.compactMap(\.self).sorted())
    expect(try eur("280.00"), try consumed(group, added.apartment, people.carol), "Carol's part of the apartment")
    expect(try eur("10.00"), try consumed(group, added.seafood, people.eva), "Eva's half part of the dinner")
    expect(try eur("15.00"), try consumed(group, added.train, people.eva), "Eva's ticket")
    expect(try eur("30.00"), try group.shares(of: added.museum)?.members.first { share in share.member == people.bob }?.paid, "Bob's half of the museum")
    expect(try eur("15.00"), try consumed(group, added.museum, people.dan), "Dan's museum ticket")
    expect(4, try group.expense(withID: added.museum)?.members().count, "the museum's members")
    expect(try eur("59.40"), try fadoExpense.total(), "the fado night with its tip")
    expect(try eur("5.40"), try fadoExpense.tipAmount(), "the fado night's tip")
    expect(try eur("21.45"), try consumed(group, added.fado, people.carol), "Carol's fado night")
    expect(try eur("6.60"), try consumed(group, added.fado, people.eva), "Eva's fado night")
    expect(try eur("10.00"), try consumed(group, added.tour, people.eva), "Eva's tour")
    return added.seafood
}
