// Sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go.

import Eranos
import Report

func anotherCurrency(_ group: Group, _ people: People, _ categories: Categories) throws(EranosError) {
    section("5. Another currency")
    let euro = try Currency(code: "EUR")
    let rate = try ExchangeRate(parsing: "0.2")
    let parking = try group.add(
        Expense(
            cost: .converted(try ron("250.00"), rate: rate),
            date: try october(6),
            name: "Airport parking",
            payers: .equal([people.ana]),
            split: .whole(.equal([people.ana, people.bob])),
            category: categories.transport
        )
    )
    let cash = try group.add(
        Transfer(
            cost: .converted(try ron("100.00"), rate: rate),
            date: try october(6),
            from: people.bob,
            to: people.ana,
            note: "Cash for the parking",
            time: try TimeOfDay(hour: 7, minute: 40)
        )
    )
    print("""
        Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in
        cash. An entry in another currency carries the rate into the group currency, how
        many EUR one RON is worth, and is converted once, when it is entered:
        """)
    try printEntries(group) { currency in euro != currency }
    guard let transfer = try group.transfer(withID: cash) else {
        preconditionFailure("the cash was recorded")
    }
    let converted = try ExchangeRate(parsing: "0.2011").convert(try ron("250.00"), into: euro)
    let counted = try group.amount(of: cash) ?? Money(currency: euro, minorUnits: 0)
    print("""
        Bob's transfer, at \(transfer.time?.description ?? "") ("\(transfer.note ?? "")"), counts as \(counted).
        A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,
        which becomes \(converted).
        """)

    expect(try eur("50.28"), converted, "the conversion rounds half away from zero")
    expect(try eur("50.00"), try group.shares(of: parking)?.total, "the parking in the group currency")
    expect(try eur("20.00"), try group.amount(of: cash), "the cash in the group currency")
    guard case .converted = transfer.cost else {
        preconditionFailure("the cash was paid in RON")
    }
}

/// Replaces an expense and a transfer, and removes one of each; returns what changed.
func replaceAndRemove(_ group: Group, _ people: People, _ categories: Categories, _ seafood: Expense.ID) throws(EranosError) -> [[String]] {
    // The dinner's receipt said 99.00.
    guard var corrected = try group.expense(withID: seafood) else {
        preconditionFailure("the dinner was added")
    }
    corrected.cost = .local(try eur("99.00"))
    corrected.description = "Cervejaria Ramiro"
    let previous = try group.replace(corrected)
    // Bob entered the coffee twice.
    let coffee = Expense(
        cost: .local(try eur("6.00")),
        date: try october(5),
        name: "Coffee",
        payers: .equal([people.bob]),
        split: .whole(.equal([people.ana, people.bob, people.carol])),
        category: categories.food
    )
    _ = try group.add(coffee)
    let copy = try group.add(coffee)
    let removed = try group.remove(copy)
    // Dan's part of the apartment was 120.00, not 100.00.
    let first = try group.add(Transfer(cost: .local(try eur("100.00")), date: try october(5), from: people.dan, to: people.ana))
    guard var fixed = try group.transfer(withID: first) else {
        preconditionFailure("the transfer was added")
    }
    fixed.cost = .local(try eur("120.00"))
    fixed.note = "Apartment"
    let replaced = try group.replace(fixed)
    // Carol sent 10.00 to the wrong member.
    let mistake = try group.add(Transfer(cost: .local(try eur("10.00")), date: try october(5), from: people.carol, to: people.bob))
    let undone = try group.remove(mistake)

    expect(try eur("90.00"), previous.cost.amount, "the dinner's first amount")
    let copied = try group.expense(withID: copy)
    precondition(nil == copied, "the copy is removed")
    expect(try eur("120.00"), try group.amount(of: first), "Dan's corrected transfer")
    let mistaken = try group.transfer(withID: mistake)
    precondition(nil == mistaken, "the mistake is removed")
    let zero = Money(currency: try Currency(code: "EUR"), minorUnits: 0)
    let dinner = try group.expense(withID: seafood)
    let apartment = try group.transfer(withID: first)
    return [
        [previous.name, "\(previous.cost.amount) became \(dinner?.cost.amount ?? zero), at \(dinner?.description ?? "")"],
        [removed.name, "entered twice; the copy of \(removed.cost.amount) was removed"],
        ["\(try name(group, people.dan)) to \(try name(group, people.ana))", "\(replaced.cost.amount) became \(apartment?.cost.amount ?? zero), noted \(apartment?.note ?? "")"],
        ["\(try name(group, people.carol)) to \(try name(group, people.bob))", "\(undone.cost.amount) sent to the wrong member; removed"]
    ]
}

func edits(_ group: Group, _ people: People, _ categories: Categories, _ seafood: Expense.ID) throws(EranosError) {
    section("6. Edits")
    let changes = try replaceAndRemove(group, people, categories, seafood)
    print("""
        Entries are values: an edit starts from a copy made with to_builder, keeps the
        identifier, and replaces the entry; a removal takes it out with its totals.
        """)
    printTable(["Entry", "Edit"], changes, numbersFrom: 2)
    print("The balances after the edits:")
    try printBalances(group)

    let before = try group.summary()
    let taxi = Expense(
        cost: .local(try eur("30.00")),
        date: try october(6),
        name: "Taxi",
        payers: .equal([people.ana]),
        split: .whole(
            .percentages([
                MemberPercentage(member: people.ana, percentage: try Percentage(parsing: "50")),
                MemberPercentage(member: people.bob, percentage: try Percentage(parsing: "49.99"))
            ])
        )
    )
    let unbalanced = Result { () throws(EranosError) in try group.add(taxi) }
    let toHerself = Result { () throws(EranosError) in
        try group.add(Transfer(cost: .local(try eur("5.00")), date: try october(6), from: people.ana, to: people.ana))
    }
    let souvenirs = Expense(cost: .local(try ron("40.00")), date: try october(6), name: "Souvenirs", payers: .equal([people.ana]), split: .whole(.equal([people.ana])))
    let unrated = Result { () throws(EranosError) in try group.add(souvenirs) }
    print("A change that does not fit is refused, and the group stays as it was:")
    printTable(
        ["Change", "Refused with"],
        [
            ["A taxi split 50% and 49.99%", message(unbalanced)],
            ["Ana paying herself back", message(toHerself)],
            ["Souvenirs in RON, without a rate", message(unrated)]
        ],
        numbersFrom: 2
    )
    print("The balances again, unchanged:")
    try printBalances(group)

    guard case .failure(.expense(.percentagesDoNotAddUp(totalBasisPoints: 9_999), _)) = unbalanced else {
        preconditionFailure("the percentages add up to 99.99%")
    }
    guard case .failure(.transfer(.sameMember, _)) = toHerself else {
        preconditionFailure("a transfer is between two members")
    }
    let leu = try Currency(code: "RON")
    guard case .failure(.group(.missingExchangeRate(leu), _)) = unrated else {
        preconditionFailure("an amount in RON needs a rate")
    }
    expect(before, try group.summary(), "a refused change leaves the totals")
}

func comingsAndGoings(_ group: Group, _ people: People) throws(EranosError) {
    section("7. Members come and go")
    let removed = try group.remove(people.filip)
    try group.archive(people.dan)
    let dinner = Expense(
        cost: .local(try eur("40.00")),
        date: try october(6),
        name: "Farewell dinner",
        payers: .equal([people.ana]),
        split: .whole(.equal([people.ana, people.bob, people.carol, people.dan]))
    )
    let farewell = Result { () throws(EranosError) in try group.add(dinner) }
    let inUse = Result { () throws(EranosError) in try group.remove(people.dan) }
    print("""
        \(removed.firstName) was never part of an entry, so removing him leaves no trace. Dan flew home
        early and was archived: his history and his balance stay, but a new expense
        cannot include him, and he cannot be removed while entries reference him.
        """)
    let summary = try group.summary()
    var rows: [[String]] = []
    for member in try group.members() {
        guard let id = member.id, let totals = totals(of: id, in: summary) else {
            continue
        }
        rows.append([member.firstName, try group.isArchived(id) ? "archived" : "active", "\(totals.balance)"])
    }
    printTable(["Member", "Status", "Balance"], rows, numbersFrom: 2)
    printTable(
        ["Change", "Refused with"],
        [
            ["A farewell dinner with Dan", message(farewell)],
            ["Removing Dan", message(inUse)]
        ],
        numbersFrom: 2
    )
    try group.unarchive(people.dan)
    print("When Dan joins the next trip, unarchiving him makes him active again.")

    expect("Filip", removed.firstName, "Filip is removed")
    let gone = try group.member(withID: people.filip)
    precondition(nil == gone, "Filip is gone")
    guard case .failure(.group(.archivedMember(people.dan), _)) = farewell else {
        preconditionFailure("Dan is archived")
    }
    guard case .failure(.group(.memberInUse, _)) = inUse else {
        preconditionFailure("entries reference Dan")
    }
    let archived = try group.isArchived(people.dan)
    precondition(!archived, "Dan is active again")
}
