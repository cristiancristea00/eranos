// The entries. An entry that converted to the generated code and back with a field changed or
// dropped, or read back from the group another way than it was added, would store another
// expense, member, or transfer than the app built.

@testable import Eranos
import Testing

/// A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally, and a dinner in
/// euros, split equally, paid by amounts, with a percentage tip: between them, every kind of
/// cost, split, and tip.
private struct Expenses {
    let dinner: Expense
    let taxi: Expense

    init(of flat: Flat) throws {
        let lei = try Currency(code: "RON")
        taxi = try Expense(
            name: "Taxi",
            cost: .converted(Money(minorUnits: 5_000, currency: lei), rate: ExchangeRate(parsing: "0.2")),
            payers: .percentages([MemberPercentage(member: flat.ana, percentage: Percentage(basisPoints: 10_000))]),
            split: .itemised([
                Item(
                    name: "Fare",
                    unitPrice: Money(minorUnits: 2_500, currency: lei),
                    split: .parts([
                        MemberParts(member: flat.ana, parts: Parts(hundredths: 100)),
                        MemberParts(member: flat.bob, parts: Parts(hundredths: 50))
                    ]),
                    quantity: 2
                )
            ]),
            date: flat.day,
            tip: Tip(value: .fixed(Money(minorUnits: 500, currency: lei)), distribution: .equal),
            time: TimeOfDay(hour: 9, minute: 5),
            category: flat.food,
            description: "To the airport"
        )
        dinner = try Expense(
            name: "Dinner",
            cost: .local(flat.euros(3_000)),
            payers: .amounts([MemberAmount(member: flat.ana, amount: flat.euros(3_300))]),
            split: .whole(.equal([flat.ana, flat.bob])),
            date: flat.day,
            tip: Tip(value: .percentage(Percentage(basisPoints: 1_000)))
        )
    }
}

/// An expense that converted with a field changed or dropped, or that the group stored another
/// way, would divide another expense than the app built.
@Test
func everyKindOfExpenseConvertsAndStoresWhole() throws {
    let flat = try Flat()
    let expenses = try Expenses(of: flat)
    for expense in [expenses.taxi, expenses.dinner] {
        #expect(Expense(expense.generated) == expense)
        let id = try flat.group.add(expense)
        let stored = try #require(try flat.group.expense(withID: id))
        #expect(stored.id == id)
        #expect(stored.cost == expense.cost)
        #expect(stored.payers == expense.payers)
        #expect(stored.split == expense.split)
        #expect(stored.tip == expense.tip)
        #expect(stored.time == expense.time)
        #expect(stored.category == expense.category)
        #expect(stored.description == expense.description)
    }
}

/// The tip, total, and members an app shows for an expense would differ from those the group
/// records when it adds it.
@Test
func anExpenseTellsItsTipTotalAndMembers() throws {
    let flat = try Flat()
    let lei = try Currency(code: "RON")
    let taxi = try Expenses(of: flat).taxi
    #expect(try taxi.total() == Money(minorUnits: 5_500, currency: lei))
    #expect(try taxi.tipAmount() == Money(minorUnits: 500, currency: lei))
    #expect(try taxi.members() == [flat.ana, flat.bob])
    #expect(taxi.cost.amount == Money(minorUnits: 5_000, currency: lei))
    #expect(Cost.local(flat.euros(1)).amount == flat.euros(1))
    #expect(Split.equal([flat.bob, flat.ana]).members == [flat.bob, flat.ana])
    #expect(Split.parts([MemberParts(member: flat.bob, parts: try Parts(hundredths: 1))]).members == [flat.bob])
    #expect(Split.percentages([MemberPercentage(member: flat.ana, percentage: try Percentage(basisPoints: 1))]).members == [flat.ana])
    #expect(Split.amounts([MemberAmount(member: flat.bob, amount: flat.euros(1))]).members == [flat.bob])
    var unreadable = taxi
    unreadable.payers = .equal([Member.ID(rawValue: "nobody")])
    #expect(throws: EranosError.self) {
        try unreadable.members()
    }
    #expect(TipDistribution.allCases == [.proportional, .equal])
}

/// A member or a transfer that converted with a detail changed or dropped, or that the group
/// stored another way, would pay or reach another person than the app named.
@Test
func membersAndTransfersConvertAndStoreWhole() throws {
    let flat = try Flat()
    let carol = try Member(
        firstName: "Carol",
        lastName: "Popa",
        email: Email("carol@example.com"),
        phone: PhoneNumber("+40712345678"),
        accounts: [
            PaymentAccount(name: "Main account", details: .iban(IBAN("RO49AAAA1B31007593840000"))),
            PaymentAccount(name: "Payment app", details: .username(Username("carol.pays")))
        ]
    )
    #expect(Member(carol.generated) == carol)
    let id = try flat.group.add(carol)
    let stored = try #require(try flat.group.member(withID: id))
    #expect(stored.id == id)
    #expect("Popa" == stored.lastName)
    #expect("carol@example.com" == stored.email?.description)
    #expect("+40712345678" == stored.phone?.e164)
    #expect(stored.accounts == carol.accounts)

    let lei = try Currency(code: "RON")
    let cash = try Transfer(
        from: flat.bob,
        to: flat.ana,
        cost: .converted(Money(minorUnits: 1_000, currency: lei), rate: ExchangeRate(parsing: "0.2")),
        date: flat.day,
        time: TimeOfDay(hour: 10, minute: 0),
        note: "Cash"
    )
    #expect(Transfer(cash.generated) == cash)
    let payment = try flat.group.add(cash)
    let storedCash = try #require(try flat.group.transfer(withID: payment))
    #expect(storedCash.id == payment)
    #expect("10:00" == storedCash.time?.description)
    #expect("Cash" == storedCash.note)
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
