// A group the tests share the shape of: two members, a category, an expense, and a transfer.

@testable import Eranos

/// A group of two with a category, an expense, and a transfer, and the identifiers the library
/// generated for them.
struct Flat {
    let ana: Member.ID
    let bob: Member.ID
    let day: CalendarDate
    let dinner: Expense.ID
    let euro: CurrencyCode
    let food: Category.ID
    let group: ExpenseGroup
    let repayment: Transfer.ID

    init() throws {
        euro = try CurrencyCode(code: "EUR")
        day = try CalendarDate(year: 2026, month: 9, day: 23)
        group = try ExpenseGroup(name: "Flat", currency: euro, description: "Rent", language: Language("pt-pt"))
        ana = try group.add(Member(firstName: "Ana"))
        bob = try group.add(Member(firstName: "Bob"))
        food = try group.add(Category(name: "Food"))
        dinner = try group.add(
            Expense(
                name: "Dinner",
                cost: .local(Money(minorUnits: 3_000, currency: euro)),
                payers: .equal([ana]),
                split: .whole(.equal([ana, bob])),
                date: day,
                category: food
            )
        )
        repayment = try group.add(Transfer(sender: bob, receiver: ana, cost: .local(Money(minorUnits: 1_000, currency: euro)), date: day))
    }

    /// `minorUnits` cents.
    func euros(_ minorUnits: Int64) -> Money {
        Money(minorUnits: minorUnits, currency: euro)
    }

    /// An expense of 30.00 EUR that `payer` paid and `members` share equally.
    func expense(_ name: String, paidBy payer: Member.ID, sharedBy members: [Member.ID]) -> Expense {
        Expense(name: name, cost: .local(euros(3_000)), payers: .equal([payer]), split: .whole(.equal(members)), date: day)
    }
}
