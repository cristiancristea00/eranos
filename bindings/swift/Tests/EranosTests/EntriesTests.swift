// The entries. An entry that converted to the generated code and back with a field changed or
// dropped, read back from the group another way than it was added, or stored under another
// identifier than the app gave it would store another expense, member, or transfer than the app
// built.

@testable import Eranos
import Testing

/// A taxi in lei, itemised, paid by percentages, with a fixed tip shared equally, and a dinner in
/// euros, split equally, paid by amounts, with a percentage tip: between them, every kind of
/// cost, split, and tip.
private struct Expenses {
    let dinner: Expense
    let taxi: Expense

    init(of flat: Flat) throws {
        let lei = try CurrencyCode(code: "RON")
        taxi = try Expense(
            name: "Taxi",
            cost: .converted(Money(minorUnits: 5_000, currency: lei), rate: ExchangeRate(parsing: "0.2")),
            payers: .percentages([MemberPercentage(member: flat.ana, percentage: Percentage(basisPoints: 10_000))]),
            split: .itemised([
                ExpenseItem(
                    name: "Fare",
                    unitPrice: Money(minorUnits: 2_500, currency: lei),
                    split: .parts([
                        MemberParts(member: flat.ana, parts: Parts(hundredths: 100)),
                        MemberParts(member: flat.bob, parts: Parts(hundredths: 50))
                    ]),
                    quantity: Quantity(whole: 2)
                )
            ]),
            date: flat.day,
            tip: TipRule(value: .fixed(Money(minorUnits: 500, currency: lei)), distribution: .equal),
            time: TimeOfDay(hour: 9, minute: 5),
            category: flat.food,
            note: "To the airport"
        )
        dinner = try Expense(
            name: "Dinner",
            cost: .local(flat.euros(3_000)),
            payers: .amounts([MemberAmount(member: flat.ana, amount: flat.euros(3_300))]),
            split: .whole(.equal([flat.ana, flat.bob])),
            date: flat.day,
            tip: TipRule(value: .percentage(Percentage(basisPoints: 1_000)))
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
        #expect(id == expense.id)
        let stored = try #require(try flat.group.expense(withID: id))
        #expect(stored.id == id)
        #expect(stored.cost == expense.cost)
        #expect(stored.payers == expense.payers)
        #expect(stored.split == expense.split)
        #expect(stored.tip == expense.tip)
        #expect(stored.time == expense.time)
        #expect(stored.category == expense.category)
        #expect(stored.note == expense.note)
    }
}

/// The tip, total, and members an app shows for an expense would differ from those the group
/// records when it adds it.
@Test
func anExpenseTellsItsTipTotalAndMembers() throws {
    let flat = try Flat()
    let lei = try CurrencyCode(code: "RON")
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
    unreadable.payers = .equal([Member.ID(rawValue: " ana")])
    #expect(throws: EranosError.id(.surroundingWhitespace, detail: "an identifier cannot start or end with whitespace")) {
        try unreadable.members()
    }
    #expect(TipDistribution.allCases == [.proportional, .equal])
}

/// A line total an app computed on its own could differ from the library's, which rounds a
/// fractional quantity's product half away from zero to the minor unit, and an item built without
/// a quantity would count another number of units than one.
@Test
func anItemsLineTotalIsTheLibrarysAndItsQuantityStartsAtOne() throws {
    let flat = try Flat()
    let flour = ExpenseItem(name: "Flour", unitPrice: flat.euros(4_200), split: .equal([flat.ana]), quantity: try Quantity(parsing: "0.35"))
    #expect(try flat.euros(1_470) == flour.lineTotal())
    // 33.5 cents, which round away from zero.
    let olives = ExpenseItem(name: "Olives", unitPrice: flat.euros(100), split: .equal([flat.ana]), quantity: try Quantity(parsing: "0.335"))
    #expect(try flat.euros(34) == olives.lineTotal())
    // A ten-thousandth of a cent, which rounds to a free line.
    let crumb = ExpenseItem(name: "Crumb", unitPrice: flat.euros(1), split: .equal([flat.ana]), quantity: try Quantity(tenThousandths: 1))
    #expect(try flat.euros(0) == crumb.lineTotal())

    let bread = ExpenseItem(name: "Bread", unitPrice: flat.euros(250), split: .equal([flat.ana, flat.bob]))
    #expect(Quantity.one == bread.quantity)
    #expect(try flat.euros(250) == bread.lineTotal())
    #expect(ExpenseItem(bread.generated) == bread)
    var loaves = bread
    loaves.quantity = try Quantity(whole: 3)
    #expect(try flat.euros(750) == loaves.lineTotal())
    #expect(ExpenseItem(loaves.generated) == loaves)
    #expect(ExpenseItem(flour.generated) == flour)

    var unnamed = bread
    unnamed.name = " "
    #expect(throws: EranosError.expense(.emptyItemName, detail: "the item name must not be empty")) {
        try unnamed.lineTotal()
    }
    var dearest = bread
    dearest.unitPrice = flat.euros(.max)
    dearest.quantity = try Quantity(whole: 2)
    #expect(throws: EranosError.expense(.overflow, detail: "the amounts are out of range")) {
        try dearest.lineTotal()
    }
}

/// A discount an app took off on its own could differ from the library's, which rounds a percentage
/// off each unit before it multiplies the reduced price and one off the line after it multiplies;
/// an item built without a discount would take one off; and a discount that converted or stored
/// with its value or scope changed would take another amount off the line than the app gave.
@Test
func aDiscountComesOffTheLineOrEachUnitAsTheLibraryTakesIt() throws {
    let flat = try Flat()
    let half = try Percentage(basisPoints: 5_000)
    let three = try Quantity(whole: 3)
    // Half of 3.33 is 1.665, which rounds to 1.67 off each unit and leaves 3 × 1.66; half of the
    // line, 9.99, is 4.995, which rounds to 5.00 off the line.
    let offEachUnit = Discount(value: .percentage(half), scope: .unit)
    let cozonac = ExpenseItem(name: "Cozonac", unitPrice: flat.euros(333), split: .equal([flat.ana]), quantity: three, discount: offEachUnit)
    #expect(try flat.euros(999) == cozonac.lineTotalBeforeDiscount())
    #expect(try flat.euros(501) == cozonac.discountAmount())
    #expect(try flat.euros(498) == cozonac.lineTotal())
    var offTheLine = cozonac
    offTheLine.discount = Discount(value: .percentage(half))
    #expect(DiscountScope.line == offTheLine.discount?.scope)
    #expect(try flat.euros(999) == offTheLine.lineTotalBeforeDiscount())
    #expect(try flat.euros(500) == offTheLine.discountAmount())
    #expect(try flat.euros(499) == offTheLine.lineTotal())
    #expect([.line, .unit] == DiscountScope.allCases)

    let lemonade = ExpenseItem(
        name: "Lemonade",
        unitPrice: flat.euros(300),
        split: .equal([flat.ana, flat.bob]),
        quantity: three,
        discount: Discount(value: .fixed(flat.euros(100)), scope: .unit)
    )
    #expect(try flat.euros(300) == lemonade.discountAmount())
    #expect(try flat.euros(600) == lemonade.lineTotal())
    var lemonadeOffTheLine = lemonade
    lemonadeOffTheLine.discount = Discount(value: .fixed(flat.euros(100)))
    #expect(try flat.euros(100) == lemonadeOffTheLine.discountAmount())
    #expect(try flat.euros(800) == lemonadeOffTheLine.lineTotal())

    // A discount of the whole price, 100 % or the price itself, leaves a free line.
    let onTheHouse = ExpenseItem(name: "Papanași", unitPrice: flat.euros(900), split: .equal([flat.bob]), discount: Discount(value: .percentage(try Percentage(basisPoints: 10_000))))
    #expect(try flat.euros(900) == onTheHouse.discountAmount())
    #expect(try flat.euros(0) == onTheHouse.lineTotal())
    var givenAway = onTheHouse
    givenAway.discount = Discount(value: .fixed(flat.euros(900)))
    #expect(try flat.euros(0) == givenAway.lineTotal())

    // The weighed line of the restaurant bill: a scale's 0.35 at 42.00 a unit, 10 % off the line.
    let telemea = ExpenseItem(
        name: "Telemea",
        unitPrice: flat.euros(4_200),
        split: .equal([flat.ana, flat.bob]),
        quantity: try Quantity(0.35),
        discount: Discount(value: .percentage(try Percentage(parsing: "10")))
    )
    #expect(try flat.euros(1_470) == telemea.lineTotalBeforeDiscount())
    #expect(try flat.euros(147) == telemea.discountAmount())
    #expect(try flat.euros(1_323) == telemea.lineTotal())

    let bread = ExpenseItem(name: "Bread", unitPrice: flat.euros(250), split: .equal([flat.ana]))
    #expect(nil == bread.discount)
    #expect(try flat.euros(250) == bread.lineTotalBeforeDiscount())
    #expect(try flat.euros(0) == bread.discountAmount())

    for item in [cozonac, offTheLine, lemonade, lemonadeOffTheLine, onTheHouse, bread] {
        #expect(ExpenseItem(item.generated) == item)
    }
    let round = Expense(
        name: "Second round",
        cost: .local(flat.euros(1_098)),
        payers: .equal([flat.ana]),
        split: .itemised([cozonac, lemonade, onTheHouse]),
        date: flat.day
    )
    let stored = try #require(try flat.group.expense(withID: flat.group.add(round)))
    #expect(stored.split == round.split)
}

/// A discount the line cannot take, let through, would leave a line below zero, or in another
/// currency than the bill, for its members to share.
@Test
func onlyADiscountTheLineCanTakeIsTakenOff() throws {
    let flat = try Flat()
    let lei = try CurrencyCode(code: "RON")
    let papanasi = ExpenseItem(name: "Papanași", unitPrice: flat.euros(900), split: .equal([flat.bob]))
    var refused = papanasi
    refused.discount = Discount(value: .fixed(flat.euros(-100)))
    #expect(throws: EranosError.expense(.negativeDiscount, detail: "a discount cannot be negative")) {
        try refused.lineTotal()
    }
    refused.discount = Discount(value: .percentage(try Percentage(basisPoints: 10_001)))
    #expect(throws: EranosError.expense(.discountAboveHundredPercent, detail: "a discount cannot be more than 100%")) {
        try refused.lineTotal()
    }
    refused.discount = Discount(value: .fixed(flat.euros(1_000)))
    #expect(throws: EranosError.expense(.discountAbovePrice, detail: "the discount is larger than the price it reduces")) {
        try refused.lineTotal()
    }
    refused.discount = Discount(value: .fixed(flat.euros(901)), scope: .unit)
    #expect(throws: EranosError.expense(.discountAbovePrice, detail: "the discount is larger than the price it reduces")) {
        try refused.discountAmount()
    }
    refused.discount = Discount(value: .fixed(Money(minorUnits: 100, currency: lei)))
    #expect(
        throws: EranosError.expense(.currencyMismatch(expected: flat.euro, found: lei), detail: "expected an amount in EUR, found one in RON")
    ) {
        try refused.lineTotalBeforeDiscount()
    }
}

/// An expense built without a tip or a category would start at, or read back as, another tip or
/// category than the library's defaults, and an app comparing with them would miss the expenses
/// that have no tip or stay in Uncategorised.
@Test
func anExpenseStartsWithoutATipInUncategorisedAndReadsBackSo() throws {
    let flat = try Flat()
    #expect("00000000-0000-0000-0000-000000000000" == Category.ID.uncategorised.rawValue)
    #expect(try Category.ID.uncategorised == flat.group.categories().first?.id)
    #expect(try TipRule.noTip == TipRule(value: .percentage(Percentage(basisPoints: 0)), distribution: .proportional))
    let bread = flat.expense("Bread", paidBy: flat.ana, sharedBy: [flat.ana, flat.bob])
    #expect(TipRule.noTip == bread.tip)
    #expect(Category.ID.uncategorised == bread.category)
    let stored = try #require(try flat.group.expense(withID: try flat.group.add(bread)))
    #expect(TipRule.noTip == stored.tip)
    #expect(Category.ID.uncategorised == stored.category)
    var water = flat.expense("Water", paidBy: flat.bob, sharedBy: [flat.ana, flat.bob])
    water.tip = TipRule(value: .percentage(try Percentage(basisPoints: 0)))
    #expect(try TipRule.noTip == flat.group.expense(withID: flat.group.add(water))?.tip)
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
    #expect(id == carol.id)
    let stored = try #require(try flat.group.member(withID: id))
    #expect(stored.id == id)
    #expect("Popa" == stored.lastName)
    #expect("carol@example.com" == stored.email?.description)
    #expect("+40712345678" == stored.phone?.e164)
    #expect("+40 712 345 678" == stored.phone?.international)
    #expect(stored.accounts == carol.accounts)

    let lei = try CurrencyCode(code: "RON")
    let cash = try Transfer(
        sender: flat.bob,
        receiver: flat.ana,
        cost: .converted(Money(minorUnits: 1_000, currency: lei), rate: ExchangeRate(parsing: "0.2")),
        date: flat.day,
        time: TimeOfDay(hour: 10, minute: 0),
        note: "Cash"
    )
    #expect(Transfer(cash.generated) == cash)
    let payment = try flat.group.add(cash)
    #expect(payment == cash.id)
    let storedCash = try #require(try flat.group.transfer(withID: payment))
    #expect(storedCash.id == payment)
    #expect("10:00" == storedCash.time?.description)
    #expect("Cash" == storedCash.note)
    #expect(try flat.group.amount(of: payment) == flat.euros(200))

    let category = Category(name: "Travel")
    #expect(Category(category.generated) == category)
    #expect(throws: EranosError.expenseGroup(.duplicateMember(id), detail: "a member with this identifier already exists")) {
        try flat.group.add(stored)
    }
}

/// A last name or a note that kept stray whitespace, read back as empty text when
/// it was added absent or blank, or stayed after a replacement without it would show an app a
/// detail the user never gave or removed.
@Test
func absentOrBlankTextReadsBackAbsent() throws {
    let flat = try Flat()
    let group = flat.group
    for text in [nil, " "] as [String?] {
        var carol = Member(firstName: "Carol", lastName: " Popa\n")
        var lunch = flat.expense("Lunch", paidBy: flat.ana, sharedBy: [flat.ana, flat.bob])
        lunch.note = " For the team\n"
        var cash = Transfer(sender: flat.bob, receiver: flat.ana, cost: .local(flat.euros(100)), date: flat.day, note: " Cash\n")
        #expect(try "Popa" == group.member(withID: group.add(carol))?.lastName)
        #expect(try "For the team" == group.expense(withID: group.add(lunch))?.note)
        #expect(try "Cash" == group.transfer(withID: group.add(cash))?.note)

        carol.lastName = text
        lunch.note = text
        cash.note = text
        try group.replace(carol)
        try group.replace(lunch)
        try group.replace(cash)
        #expect(try nil == group.member(withID: carol.id)?.lastName)
        #expect(try nil == group.expense(withID: lunch.id)?.note)
        #expect(try nil == group.transfer(withID: cash.id)?.note)

        #expect(try nil == group.member(withID: group.add(Member(firstName: "Dan", lastName: text)))?.lastName)
        var dinner = flat.expense("Dinner", paidBy: flat.bob, sharedBy: [flat.ana, flat.bob])
        dinner.note = text
        #expect(try nil == group.expense(withID: group.add(dinner))?.note)
        let refund = Transfer(sender: flat.ana, receiver: flat.bob, cost: .local(flat.euros(100)), date: flat.day, note: text)
        #expect(try nil == group.transfer(withID: group.add(refund))?.note)
    }
}

/// A replace that added an entry the group does not hold, instead of refusing it, would bring back
/// an entry the app removed while its user edited it.
@Test
func onlyAnEntryTheGroupHoldsIsReplaced() throws {
    let flat = try Flat()
    let group = flat.group
    let member = Member(firstName: "Carol")
    #expect(throws: EranosError.expenseGroup(.unknownMember(member.id), detail: "no member has this identifier")) {
        try group.replace(member)
    }
    let category = Category(name: "Travel")
    #expect(throws: EranosError.expenseGroup(.unknownCategory(category.id), detail: "no category has this identifier")) {
        try group.replace(category)
    }
    let expense = flat.expense("Lunch", paidBy: flat.ana, sharedBy: [flat.ana, flat.bob])
    #expect(throws: EranosError.expenseGroup(.unknownExpense(expense.id), detail: "no expense has this identifier")) {
        try group.replace(expense)
    }
    let transfer = Transfer(sender: flat.ana, receiver: flat.bob, cost: .local(flat.euros(100)), date: flat.day)
    #expect(throws: EranosError.expenseGroup(.unknownTransfer(transfer.id), detail: "no transfer has this identifier")) {
        try group.replace(transfer)
    }
}

/// An entry the group stored under another identifier than the one the app gave it, or a second
/// entry it let in under an identifier it holds, would leave the app's own store naming another
/// entry than the group's.
@Test
func everyEntryKeepsTheIdentifierTheAppGivesIt() throws {
    let flat = try Flat()
    let group = flat.group

    let carol = try Member.ID("carol")
    let member = Member(id: carol, firstName: "Carol")
    #expect(try group.add(member) == carol)
    #expect(try group.member(withID: carol) == member)
    #expect(throws: EranosError.expenseGroup(.duplicateMember(carol), detail: "a member with this identifier already exists")) {
        try group.add(Member(id: carol, firstName: "Caroline"))
    }

    let travel = try Category.ID("-travel")
    let category = Category(id: travel, name: "Travel")
    #expect(try group.add(category) == travel)
    #expect(try group.category(withID: travel) == category)
    #expect(throws: EranosError.expenseGroup(.duplicateCategory(travel), detail: "a category with this identifier already exists")) {
        try group.add(Category(id: travel, name: "Trips"))
    }
    let builtIn = try #require(try group.categories().first?.id)
    #expect(throws: EranosError.expenseGroup(.duplicateCategory(builtIn), detail: "a category with this identifier already exists")) {
        try group.add(Category(id: builtIn, name: "Other"))
    }

    let taxi = try Expense.ID("!taxi")
    let expense = Expense(
        id: taxi,
        name: "Taxi",
        cost: .local(flat.euros(1_200)),
        payers: .equal([carol]),
        split: .whole(.equal([flat.ana, carol])),
        date: flat.day,
        category: travel
    )
    #expect(try group.add(expense) == taxi)
    #expect(try group.expense(withID: taxi) == expense)
    #expect(throws: EranosError.expenseGroup(.duplicateExpense(taxi), detail: "an expense with this identifier already exists")) {
        try group.add(expense)
    }

    let repaid = try Transfer.ID("zz")
    let transfer = Transfer(id: repaid, sender: flat.ana, receiver: carol, cost: .local(flat.euros(600)), date: flat.day)
    #expect(try group.add(transfer) == repaid)
    #expect(try group.transfer(withID: repaid) == transfer)
    #expect(throws: EranosError.expenseGroup(.duplicateTransfer(repaid), detail: "a transfer with this identifier already exists")) {
        try group.add(transfer)
    }
}
