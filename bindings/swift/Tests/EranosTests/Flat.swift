// A group the tests share the shape of: two members, a category, an expense, and a transfer.

@testable import Eranos

/// A group of two with a category, an expense, and a transfer, and the identifiers the group
/// gave them.
struct Flat {
    let ana: Member.ID
    let bob: Member.ID
    let day: CalendarDate
    let dinner: Expense.ID
    let euro: Currency
    let food: Category.ID
    let group: Group
    let repayment: Transfer.ID

    init() throws {
        euro = try Currency(code: "EUR")
        day = try CalendarDate(day: 23, month: 9, year: 2026)
        group = try Group(currency: euro, name: "Flat", description: "Rent", language: Language("pt-pt"))
        ana = try group.add(Member(firstName: "Ana"))
        bob = try group.add(Member(firstName: "Bob"))
        food = try group.add(Category(name: "Food"))
        dinner = try group.add(
            Expense(
                cost: .local(Money(currency: euro, minorUnits: 3_000)),
                date: day,
                name: "Dinner",
                payers: .equal([ana]),
                split: .whole(.equal([ana, bob])),
                category: food
            )
        )
        repayment = try group.add(Transfer(cost: .local(Money(currency: euro, minorUnits: 1_000)), date: day, from: bob, to: ana))
    }

    /// `minorUnits` cents.
    func euros(_ minorUnits: Int64) -> Money {
        Money(currency: euro, minorUnits: minorUnits)
    }

    /// An expense of 30.00 EUR that `payer` paid and `members` share equally.
    func expense(_ name: String, paidBy payer: Member.ID, sharedBy members: [Member.ID]) -> Expense {
        Expense(cost: .local(euros(3_000)), date: day, name: name, payers: .equal([payer]), split: .whole(.equal(members)))
    }
}
