internal import EranosGenerated

/// Something bought for the group: what it cost, who paid, and how it divides.
///
/// A new expense has no identifier; `Group.add(_:)` gives it one. To change an expense,
/// read it from the group, change its properties, and pass it to `Group.replace(_:)`.
public struct Expense: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of an expense.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: absent for an expense to add, present for one the group holds.
    public private(set) var id: ID?

    /// What was bought.
    public var name: String

    /// What it cost.
    public var cost: Cost

    /// Who paid the total, amount and tip, and how much each.
    public var payers: Split

    /// How the amount is shared.
    public var split: ExpenseSplit

    /// The day it was bought.
    public var date: CalendarDate

    /// The tip on top of the amount.
    public var tip: Tip?

    /// The time of day it was bought.
    public var time: TimeOfDay?

    /// The category; absent for the built-in one.
    public var category: Category.ID?

    /// A longer description.
    public var description: String?

    var generated: EranosGenerated.Expense {
        .init(
            id: id?.rawValue,
            name: name,
            cost: cost.generated,
            payers: payers.generated,
            split: split.generated,
            date: date.generated,
            tip: tip?.generated,
            time: time?.generated,
            category: category?.rawValue,
            description: description
        )
    }

    /// An expense to add, of `cost`, that `payers` paid and `split` shares, on `date`.
    public init(
        name: String,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: Tip? = nil,
        time: TimeOfDay? = nil,
        category: Category.ID? = nil,
        description: String? = nil
    ) {
        self.name = name
        self.cost = cost
        self.payers = payers
        self.split = split
        self.date = date
        self.tip = tip
        self.time = time
        self.category = category
        self.description = description
    }

    init(_ expense: EranosGenerated.Expense) {
        self.init(
            name: expense.name,
            cost: Cost(expense.cost),
            payers: Split(expense.payers),
            split: ExpenseSplit(expense.split),
            date: CalendarDate(expense.date),
            tip: expense.tip.map(Tip.init),
            time: expense.time.map(TimeOfDay.init),
            category: expense.category.map { Category.ID(rawValue: $0) },
            description: expense.description
        )
        id = expense.id.map { ID(rawValue: $0) }
    }

    /// The total, amount and tip, in the expense's own currency: what its payers paid.
    ///
    /// - Throws: As ``tipAmount()`` does, and ``EranosError/money(_:detail:)`` with
    ///   ``MoneyFailure/overflow`` for a total out of range.
    public func total() throws(EranosError) -> Money {
        let total = try converting { try EranosGenerated.expenseTotal(expense: generated) }
        return Money(total)
    }

    /// The tip, in the expense's own currency.
    ///
    /// - Throws: ``EranosError/expense(_:detail:)`` with ``ExpenseFailure/currencyMismatch(expected:found:)``
    ///   for a fixed tip in another currency, and ``ExpenseFailure/negativeTip`` for a fixed tip
    ///   below zero.
    public func tipAmount() throws(EranosError) -> Money {
        let amount = try converting { try EranosGenerated.expenseTipAmount(expense: generated) }
        return Money(amount)
    }

    /// Every member the expense involves, as a payer or a participant, in identifier order.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for a member whose identifier is not one, and the
    ///   failures of an item that does not build.
    public func members() throws(EranosError) -> [Member.ID] {
        let members = try converting { try EranosGenerated.expenseMembers(expense: generated) }
        return members.map { Member.ID(rawValue: $0) }
    }
}

/// What an entry cost: an amount in the group currency, or one in another currency with the rate
/// that converts it into the group currency.
public enum Cost: Hashable, Sendable {
    /// An amount in the group currency.
    case local(Money)

    /// An amount in another currency than the group's, converted at `rate`.
    case converted(Money, rate: ExchangeRate)

    /// The amount, in its own currency.
    public var amount: Money {
        switch self {
        case let .local(amount):
            amount
        case let .converted(amount, _):
            amount
        }
    }

    var generated: EranosGenerated.Cost {
        switch self {
        case let .local(amount):
            .local(amount: amount.generated)
        case let .converted(amount, rate):
            .converted(amount: amount.generated, rate: rate.description)
        }
    }

    init(_ cost: EranosGenerated.Cost) {
        switch cost {
        case let .local(amount):
            self = .local(Money(amount))
        case let .converted(amount, rate):
            self = .converted(Money(amount), rate: ExchangeRate(known: rate))
        }
    }
}

/// How an expense's cost is shared: as a whole, or line by line of an itemised bill.
public enum ExpenseSplit: Hashable, Sendable {
    /// The whole amount is divided by one split.
    case whole(Split)

    /// Every line is divided by its own split; the lines add up to the amount.
    case itemised([Item])

    var generated: EranosGenerated.ExpenseSplit {
        switch self {
        case let .whole(split):
            .whole(split: split.generated)
        case let .itemised(items):
            .itemised(items: items.map(\.generated))
        }
    }

    init(_ split: EranosGenerated.ExpenseSplit) {
        switch split {
        case let .whole(split):
            self = .whole(Split(split))
        case let .itemised(items):
            self = .itemised(items.map(Item.init))
        }
    }
}

/// A line of an itemised bill.
public struct Item: Hashable, Sendable {
    /// What was bought.
    public var name: String

    /// The price of one unit, in the expense's currency.
    public var unitPrice: Money

    /// How the line's total divides.
    public var split: Split

    /// How many units, from 1.
    public var quantity: Int32

    var generated: EranosGenerated.Item {
        .init(name: name, unitPrice: unitPrice.generated, split: split.generated, quantity: quantity)
    }

    /// A line of `quantity` units of `name` at `unitPrice`, which `split` divides.
    public init(name: String, unitPrice: Money, split: Split, quantity: Int32 = 1) {
        self.name = name
        self.unitPrice = unitPrice
        self.split = split
        self.quantity = quantity
    }

    init(_ item: EranosGenerated.Item) {
        self.init(name: item.name, unitPrice: Money(item.unitPrice), split: Split(item.split), quantity: item.quantity)
    }
}

/// A tip on top of an expense's amount, which its payers cover.
public struct Tip: Hashable, Sendable {
    /// How the tip is expressed.
    public var value: TipValue

    /// How the tip is shared among the participants.
    public var distribution: TipDistribution

    var generated: EranosGenerated.Tip {
        .init(value: value.generated, distribution: distribution.generated)
    }

    /// A tip of `value`, shared by `distribution`, which is in proportion to each share unless
    /// given.
    public init(value: TipValue, distribution: TipDistribution = .proportional) {
        self.value = value
        self.distribution = distribution
    }

    init(_ tip: EranosGenerated.Tip) {
        self.init(value: TipValue(tip.value), distribution: TipDistribution(tip.distribution))
    }
}

/// How a tip is expressed.
public enum TipValue: Hashable, Sendable {
    /// A percentage of the expense's amount.
    case percentage(Percentage)

    /// A fixed amount in the expense's currency.
    case fixed(Money)

    var generated: EranosGenerated.TipValue {
        switch self {
        case let .percentage(percentage):
            .percentage(basisPoints: percentage.basisPoints)
        case let .fixed(amount):
            .fixed(amount: amount.generated)
        }
    }

    init(_ value: EranosGenerated.TipValue) {
        switch value {
        case let .percentage(basisPoints):
            self = .percentage(Percentage(known: basisPoints))
        case let .fixed(amount):
            self = .fixed(Money(amount))
        }
    }
}

/// How a tip is shared among an expense's participants.
public enum TipDistribution: Hashable, Sendable, CaseIterable {
    /// In proportion to each participant's share of the amount.
    case proportional

    /// The same for every participant.
    case equal

    var generated: EranosGenerated.TipDistribution {
        switch self {
        case .proportional:
            .proportional
        case .equal:
            .equal
        }
    }

    init(_ distribution: EranosGenerated.TipDistribution) {
        switch distribution {
        case .proportional:
            self = .proportional
        case .equal:
            self = .equal
        }
    }
}
