// The entries. An entry that converted to the generated code and back with a field changed or
// dropped, or read back from the group another way than it was added, would store another
// expense, member, or transfer than the app built.

@testable import Eranos
import Testing

/// A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally, and a dinner in
/// euros, split equally, paid by amounts, with a percentage tip: between them, every kind of
/// cost, split, and tip.
private func expenses(of flat: Flat) throws -> [Expense] {
    let lei = try Currency(code: "RON")
    let taxi = try Expense(
        cost: .converted(Money(currency: lei, minorUnits: 5_000), rate: ExchangeRate(parsing: "0.2")),
        date: flat.day,
        name: "Taxi",
        payers: .percentages([MemberPercentage(member: flat.ana, percentage: Percentage(basisPoints: 10_000))]),
        split: .itemised([
            Item(
                name: "Fare",
                split: .parts([
                    MemberParts(member: flat.ana, parts: Parts(hundredths: 100)),
                    MemberParts(member: flat.bob, parts: Parts(hundredths: 50))
                ]),
                unitPrice: Money(currency: lei, minorUnits: 2_500),
                quantity: 2
            )
        ]),
        category: flat.food,
        description: "To the airport",
        time: TimeOfDay(hour: 9, minute: 5),
        tip: Tip(value: .fixed(Money(currency: lei, minorUnits: 500)), distribution: .equal)
    )
    let dinner = try Expense(
        cost: .local(flat.euros(3_000)),
        date: flat.day,
        name: "Dinner",
        payers: .amounts([MemberAmount(amount: flat.euros(3_300), member: flat.ana)]),
        split: .whole(.equal([flat.ana, flat.bob])),
        tip: Tip(value: .percentage(Percentage(basisPoints: 1_000)))
    )
    return [taxi, dinner]
}

/// An expense that converted with a field changed or dropped, or that the group stored another
/// way, would divide another expense than the app built.
@Test
func everyKindOfExpenseConvertsAndStoresWhole() throws {
    let flat = try Flat()
    for expense in try expenses(of: flat) {
        #expect(Expense(expense.generated) == expense)
        let id = try flat.group.add(expense)
        let stored = try #require(try flat.group.expense(withID: id))
        #expect(stored.id == id)
        #expect(stored.cost == expense.cost)
        #expect(stored.payers == expense.payers)
        #expect(stored.split == expense.split)
        #expect([stored.category, stored.time, stored.tip] as [AnyHashable?] == [expense.category, expense.time, expense.tip])
        #expect(stored.description == expense.description)
    }
}

/// The tip, total, and members an app shows for an expense would differ from those the group
/// records when it adds it.
@Test
func anExpenseTellsItsTipTotalAndMembers() throws {
    let flat = try Flat()
    let lei = try Currency(code: "RON")
    let taxi = try expenses(of: flat)[0]
    #expect(try taxi.tipAmount() == Money(currency: lei, minorUnits: 500))
    #expect(try taxi.total() == Money(currency: lei, minorUnits: 5_500))
    #expect(try taxi.members() == [flat.ana, flat.bob])
    #expect(taxi.cost.amount == Money(currency: lei, minorUnits: 5_000))
    #expect(Cost.local(flat.euros(1)).amount == flat.euros(1))
    #expect(Split.amounts([MemberAmount(amount: flat.euros(1), member: flat.bob)]).members == [flat.bob])
    #expect(Split.equal([flat.bob, flat.ana]).members == [flat.bob, flat.ana])
    #expect(Split.parts([MemberParts(member: flat.bob, parts: try Parts(hundredths: 1))]).members == [flat.bob])
    #expect(Split.percentages([MemberPercentage(member: flat.ana, percentage: try Percentage(basisPoints: 1))]).members == [flat.ana])
    var unreadable = taxi
    unreadable.payers = .equal([Member.ID(rawValue: "nobody")])
    #expect(throws: EranosError.self) {
        try unreadable.members()
    }
    #expect(TipDistribution.allCases == [.equal, .proportional])
}

/// A member or a transfer that converted with a detail changed or dropped, or that the group
/// stored another way, would pay or reach another person than the app named.
@Test
func membersAndTransfersConvertAndStoreWhole() throws {
    let flat = try Flat()
    let carol = try Member(
        firstName: "Carol",
        accounts: [
            Account(details: .iban(IBAN("RO49AAAA1B31007593840000")), name: "Main account"),
            Account(details: .username(Username("carol.pays")), name: "Payment app")
        ],
        email: Email("carol@example.com"),
        lastName: "Popa",
        phone: PhoneNumber("+40712345678")
    )
    #expect(Member(carol.generated) == carol)
    let id = try flat.group.add(carol)
    let stored = try #require(try flat.group.member(withID: id))
    #expect(stored.id == id)
    #expect(stored.accounts == carol.accounts)
    #expect([stored.email?.description, stored.lastName, stored.phone?.e164] == ["carol@example.com", "Popa", "+40712345678"])

    let lei = try Currency(code: "RON")
    let cash = try Transfer(
        cost: .converted(Money(currency: lei, minorUnits: 1_000), rate: ExchangeRate(parsing: "0.2")),
        date: flat.day,
        from: flat.bob,
        to: flat.ana,
        note: "Cash",
        time: TimeOfDay(hour: 10, minute: 0)
    )
    #expect(Transfer(cash.generated) == cash)
    let payment = try flat.group.add(cash)
    let storedCash = try #require(try flat.group.transfer(withID: payment))
    #expect(storedCash.id == payment)
    #expect([storedCash.note, storedCash.time?.description] == ["Cash", "10:00"])
    #expect(try flat.group.amount(of: payment) == flat.euros(200))

    let category = Category(name: "Travel")
    #expect(Category(category.generated) == category)
    #expect(throws: EranosError.identifierGiven(detail: "an entry to add must not carry an identifier")) {
        try flat.group.add(stored)
    }
    #expect(throws: EranosError.identifierMissing(detail: "an entry to replace must carry its identifier")) {
        try flat.group.replace(carol)
    }
}
