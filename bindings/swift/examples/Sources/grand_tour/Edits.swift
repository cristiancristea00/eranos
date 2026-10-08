// Sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go.

import Eranos
import Report

func anotherCurrency(_ group: ExpenseGroup, _ people: People, _ categories: Categories) throws(EranosError) {
    section("5. Another currency")
    let euro = try CurrencyCode(code: "EUR")
    let rate = try ExchangeRate(parsing: "0.2")
    let parking = try group.add(
        Expense(
            name: "Airport parking",
            cost: .converted(try ron("250.00"), rate: rate),
            payers: .equal([people.ana]),
            split: .whole(.equal([people.ana, people.bob])),
            date: try october(6),
            category: categories.transport
        )
    )
    let cash = try group.add(
        Transfer(
            sender: people.bob,
            receiver: people.ana,
            cost: .converted(try ron("100.00"), rate: rate),
            date: try october(6),
            time: try TimeOfDay(hour: 7, minute: 40),
            note: "Cash for the parking"
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
    let counted = try group.amount(of: cash) ?? Money(minorUnits: 0, currency: euro)
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
func replaceAndRemove(_ group: ExpenseGroup, _ people: People, _ categories: Categories, _ seafood: Expense.ID) throws(EranosError) -> [EditRow] {
    // The dinner's receipt said 99.00.
    guard var corrected = try group.expense(withID: seafood) else {
        preconditionFailure("the dinner was added")
    }
    corrected.cost = .local(try eur("99.00"))
    corrected.description = "Cervejaria Ramiro"
    let previous = try group.replace(corrected)
    // Bob entered the coffee twice. Each entry is an expense with an identifier of its own: the same
    // value added twice would be refused as a duplicate.
    let coffee = { () throws(EranosError) in
        Expense(
            name: "Coffee",
            cost: .local(try eur("6.00")),
            payers: .equal([people.bob]),
            split: .whole(.equal([people.ana, people.bob, people.carol])),
            date: try october(5),
            category: categories.food
        )
    }
    _ = try group.add(coffee())
    let copy = try group.add(coffee())
    let removed = try group.remove(copy)
    // Dan's part of the apartment was 120.00, not 100.00.
    let first = try group.add(Transfer(sender: people.dan, receiver: people.ana, cost: .local(try eur("100.00")), date: try october(5)))
    guard var fixed = try group.transfer(withID: first) else {
        preconditionFailure("the transfer was added")
    }
    fixed.cost = .local(try eur("120.00"))
    fixed.note = "Apartment"
    let replaced = try group.replace(fixed)
    // Carol sent 10.00 to the wrong member.
    let mistake = try group.add(Transfer(sender: people.carol, receiver: people.bob, cost: .local(try eur("10.00")), date: try october(5)))
    let undone = try group.remove(mistake)

    expect(try eur("90.00"), previous.cost.amount, "the dinner's first amount")
    let copied = try group.expense(withID: copy)
    precondition(nil == copied, "the copy is removed")
    expect(try eur("120.00"), try group.amount(of: first), "Dan's corrected transfer")
    let mistaken = try group.transfer(withID: mistake)
    precondition(nil == mistaken, "the mistake is removed")
    let zero = Money(minorUnits: 0, currency: try CurrencyCode(code: "EUR"))
    let dinner = try group.expense(withID: seafood)
    let apartment = try group.transfer(withID: first)
    return [
        EditRow(entry: previous.name, edit: "\(previous.cost.amount) became \(dinner?.cost.amount ?? zero), at \(dinner?.description ?? "")"),
        EditRow(entry: removed.name, edit: "entered twice; the copy of \(removed.cost.amount) was removed"),
        EditRow(
            entry: "\(try name(group, people.dan)) to \(try name(group, people.ana))",
            edit: "\(replaced.cost.amount) became \(apartment?.cost.amount ?? zero), noted \(apartment?.note ?? "")"
        ),
        EditRow(entry: "\(try name(group, people.carol)) to \(try name(group, people.bob))", edit: "\(undone.cost.amount) sent to the wrong member; removed")
    ]
}

func edits(_ group: ExpenseGroup, _ people: People, _ categories: Categories, _ seafood: Expense.ID) throws(EranosError) {
    section("6. Edits")
    let changes = try replaceAndRemove(group, people, categories, seafood)
    print("""
        Entries are values: an edit starts from a copy made with to_builder, keeps the
        identifier, and replaces the entry; a removal takes it out with its totals.
        """)
    printTable(changes, numbersFrom: 2)
    print("The balances after the edits:")
    try printBalances(group)

    let before = try group.summary()
    let taxi = Expense(
        name: "Taxi",
        cost: .local(try eur("30.00")),
        payers: .equal([people.ana]),
        split: .whole(
            .percentages([
                MemberPercentage(member: people.ana, percentage: try Percentage(parsing: "50")),
                MemberPercentage(member: people.bob, percentage: try Percentage(parsing: "49.99"))
            ])
        ),
        date: try october(6)
    )
    let unbalanced = Result { () throws(EranosError) in try group.add(taxi) }
    let toHerself = Result { () throws(EranosError) in
        try group.add(Transfer(sender: people.ana, receiver: people.ana, cost: .local(try eur("5.00")), date: try october(6)))
    }
    let souvenirs = Expense(name: "Souvenirs", cost: .local(try ron("40.00")), payers: .equal([people.ana]), split: .whole(.equal([people.ana])), date: try october(6))
    let unrated = Result { () throws(EranosError) in try group.add(souvenirs) }
    print("A change that does not fit is refused, and the group stays as it was:")
    printTable(
        [
            RefusalRow(change: "A taxi split 50% and 49.99%", error: message(unbalanced)),
            RefusalRow(change: "Ana paying herself back", error: message(toHerself)),
            RefusalRow(change: "Souvenirs in RON, without a rate", error: message(unrated))
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
    let leu = try CurrencyCode(code: "RON")
    guard case .failure(.expenseGroup(.missingExchangeRate(leu), _)) = unrated else {
        preconditionFailure("an amount in RON needs a rate")
    }
    expect(before, try group.summary(), "a refused change leaves the totals")
}

func comingsAndGoings(_ group: ExpenseGroup, _ people: People) throws(EranosError) {
    section("7. Members come and go")
    let removed = try group.remove(people.filip)
    try group.archive(people.dan)
    let dinner = Expense(
        name: "Farewell dinner",
        cost: .local(try eur("40.00")),
        payers: .equal([people.ana]),
        split: .whole(.equal([people.ana, people.bob, people.carol, people.dan])),
        date: try october(6)
    )
    let farewell = Result { () throws(EranosError) in try group.add(dinner) }
    let inUse = Result { () throws(EranosError) in try group.remove(people.dan) }
    print("""
        \(removed.firstName) was never part of an entry, so removing him leaves no trace. Dan flew home
        early and was archived: his history and his balance stay, but a new expense
        cannot include him, and he cannot be removed while entries reference him.
        """)
    let summary = try group.summary()
    var rows: [StatusRow] = []
    for member in try group.members() {
        guard let totals = totals(of: member.id, in: summary) else {
            continue
        }
        let status = switch try group.isArchived(member.id) {
        case true?:
            "archived"
        case false?:
            "active"
        case nil:
            "not in the group"
        }
        rows.append(StatusRow(name: member.firstName, status: status, balance: totals.balance))
    }
    printTable(rows, numbersFrom: 2)
    printTable(
        [
            RefusalRow(change: "A farewell dinner with Dan", error: message(farewell)),
            RefusalRow(change: "Removing Dan", error: message(inUse))
        ],
        numbersFrom: 2
    )
    try group.unarchive(people.dan)
    print("When Dan joins the next trip, unarchiving him makes him active again.")

    expect("Filip", removed.firstName, "Filip is removed")
    let gone = try group.member(withID: people.filip)
    precondition(nil == gone, "Filip is gone")
    guard case .failure(.expenseGroup(.archivedMember(people.dan), _)) = farewell else {
        preconditionFailure("Dan is archived")
    }
    guard case .failure(.expenseGroup(.memberInUse, _)) = inUse else {
        preconditionFailure("entries reference Dan")
    }
    let archived = try group.isArchived(people.dan)
    precondition(false == archived, "Dan is active again")
}
