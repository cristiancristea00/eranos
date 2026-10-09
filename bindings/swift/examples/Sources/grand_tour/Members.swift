// Sections 1 to 3 of the tour: the group and its details, the members with their contact details and accounts, and
// the categories.

import Eranos
import Report

func theGroup() throws(EranosError) -> ExpenseGroup {
    section("1. The group")
    let group = try ExpenseGroup(name: "Lisbon", currency: CurrencyCode(code: "EUR"), note: "A week by the Atlantic", language: Language("pt-pt"))
    let blank = Result { () throws(EranosError) in try group.rename(to: "   ") }
    try group.rename(to: "Lisbon and Porto")
    group.changeNote(to: "A week by the Atlantic, with a day in Porto")
    printTable(
        [
            DetailRow(detail: "Name", value: group.name),
            DetailRow(detail: "Currency", value: "\(group.currency)"),
            DetailRow(detail: "Language", value: group.language?.description ?? ""),
            DetailRow(detail: "Description", value: group.note ?? "")
        ],
        numbersFrom: 2
    )
    print("""
        The group started as Lisbon and was renamed when a day in Porto joined the plan.
        Every total is kept in its currency, \(group.currency). The language is a BCP 47 tag the apps
        show the group in, stored in canonical form: pt-pt became \(group.language?.description ?? ""). A blank name is
        refused:
          error: \(message(blank))
        """)

    guard case .failure(.expenseGroup(.emptyName, _)) = blank else {
        preconditionFailure("a blank name is refused")
    }
    expect("Lisbon and Porto", group.name, "the group is renamed")
    expect("pt-PT", group.language?.description, "the tag is stored in canonical form")
    return group
}

/// An account with its label, its kind, and its details.
func describe(_ account: PaymentAccount) -> String {
    switch account.details {
    case let .iban(iban):
        "\(account.name), IBAN \(iban)"
    case let .username(username):
        "\(account.name), username \(username)"
    }
}

func printMembers(_ group: ExpenseGroup) throws(EranosError) {
    printTable(
        group.members().map { member in
            MemberRow(
                name: [member.firstName, member.lastName].compactMap(\.self).joined(separator: " "),
                email: member.email?.description ?? "",
                phone: member.phone?.description ?? "",
                accounts: cell(member.accounts.map(describe))
            )
        },
        numbersFrom: 4
    )
}

func validation(_ group: ExpenseGroup) {
    print("""
        IBANs are checked against the SWIFT registry and their mod-97 check digits,
        emails against RFC 5321 and 5322, and phone numbers against their country's
        numbering plan. Required text must not be blank:
        """)
    let iban = { (input: String) in
        CheckRow(input: "\"\(input)\"", kind: "IBAN", result: outcome({ () throws(EranosError) in try IBAN(input) }, { iban in "\(iban), country \(iban.countryCode)" }))
    }
    let email = { (input: String) in
        CheckRow(input: "\"\(input)\"", kind: "email", result: outcome({ () throws(EranosError) in try Email(input) }, \.description))
    }
    let phone = { (input: String) in
        CheckRow(input: "\"\(input)\"", kind: "phone", result: outcome({ () throws(EranosError) in try PhoneNumber(input) }, { phone in "\(phone), stored as \(phone.e164)" }))
    }
    printTable(
        [
            iban("ro49 aaaa 1b31 0075 9384 0000"),
            iban("RO49 AAAA 1B31 0075 9384 0001"),
            email("ana..popescu@example.com"),
            phone("+351 912 345 678"),
            phone("+40 612 345 678"),
            CheckRow(input: "\"   \"", kind: "username", result: outcome({ () throws(EranosError) in try Username("   ") }, \.description)),
            CheckRow(input: "\"  \"", kind: "first name", result: outcome({ () throws(EranosError) in try group.add(Member(firstName: "  ")) }, \.rawValue))
        ],
        numbersFrom: 3
    )
}

func members(_ group: ExpenseGroup) throws(EranosError) -> People {
    section("2. Members")
    let ana = try group.add(
        Member(
            firstName: "Ana",
            lastName: "Popescu",
            email: try Email("ana@example.com"),
            phone: try PhoneNumber("+40 712 345 678"),
            accounts: [
                PaymentAccount(name: "ING", details: .iban(try IBAN("RO49 AAAA 1B31 0075 9384 0000"))),
                PaymentAccount(name: "Revolut", details: .username(try Username("@ana")))
            ]
        )
    )
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    let dan = try group.add(Member(firstName: "Dan"))
    let eva = try group.add(Member(firstName: "Eva"))
    let filip = try group.add(Member(firstName: "Filip"))
    // An edit starts from a copy of the member, which keeps its identifier.
    guard var edited = try group.member(withID: bob) else {
        preconditionFailure("Bob is a member")
    }
    edited.lastName = "Ionescu"
    edited.email = try Email("bob@example.com")
    let before = try group.replace(edited)
    try printMembers(group)
    print("""
        Only a first name is required. Ana gave every detail; Bob added his last name and
        email later, through an edit that kept his identifier:
          \(bob)
        Eva is Carol's daughter, and Filip was added by mistake.
        """)
    validation(group)

    precondition(nil == before.lastName)
    expect("Ionescu", try group.member(withID: bob)?.lastName, "Bob's edit is kept")
    expect(2, try group.member(withID: ana)?.accounts.count, "Ana has two accounts")
    return People(ana: ana, bob: bob, carol: carol, dan: dan, eva: eva, filip: filip)
}

func addCategories(_ group: ExpenseGroup) throws(EranosError) -> Categories {
    section("3. Categories")
    let food = try group.add(Category(name: "Food"))
    // An app that keeps its categories elsewhere gives them its own keys.
    let lodging = try group.add(Category(id: Category.ID("lodging"), name: "Lodging"))
    let transport = try group.add(Category(name: "Transport"))
    let homonym = Result { () throws(EranosError) in try group.add(Category(name: "FOOD")) }
    guard var renamed = try group.category(withID: transport) else {
        preconditionFailure("Transport was added")
    }
    renamed.name = "Getting around"
    _ = try group.replace(renamed)
    guard var other = try group.category(withID: .uncategorised) else {
        preconditionFailure("every group has the built-in category")
    }
    other.name = "Other"
    let builtIn = Result { () throws(EranosError) in try group.replace(other) }
    print("""
        Every group has the built-in Uncategorised; this one adds Food, Lodging, under the
        identifier lodging that the app gives it, and Transport, then renames Transport, which
        keeps its identifier. In identifier order, where lodging follows the generated ones:
          \(group.categories().map(\.name).joined(separator: ", "))
        Names are compared caselessly, and the built-in category keeps its name:
        """)
    printTable(
        [
            RefusalRow(change: "Add a category named FOOD", error: message(homonym)),
            RefusalRow(change: "Rename Uncategorised to Other", error: message(builtIn))
        ],
        numbersFrom: 2
    )

    guard case .failure(.expenseGroup(.duplicateCategoryName(food), _)) = homonym else {
        preconditionFailure("the name is Food's")
    }
    guard case .failure(.expenseGroup(.builtInCategory, _)) = builtIn else {
        preconditionFailure("the built-in category keeps its name")
    }
    expect("Getting around", try group.category(withID: transport)?.name, "Transport is renamed")
    expect("lodging", lodging.rawValue, "Lodging keeps the app's identifier")
    expect([.uncategorised, food, transport, lodging], group.categories().map(\.id), "lodging follows the generated identifiers")
    return Categories(food: food, lodging: lodging, transport: transport)
}
