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

    /// What it cost.
    public var cost: Cost

    /// The day it was bought.
    public var date: CalendarDate

    /// What was bought.
    public var name: String

    /// Who paid the total, amount and tip, and how much each.
    public var payers: Split

    /// How the amount is shared.
    public var split: ExpenseSplit

    /// The category; absent for the built-in one.
    public var category: Category.ID?

    /// A longer description.
    public var description: String?

    /// The time of day it was bought.
    public var time: TimeOfDay?

    /// The tip on top of the amount.
    public var tip: Tip?

    var generated: EranosGenerated.Expense {
        .init(
            id: id?.rawValue,
            cost: cost.generated,
            date: date.generated,
            name: name,
            payers: payers.generated,
            split: split.generated,
            category: category?.rawValue,
            description: description,
            time: time?.generated,
            tip: tip?.generated
        )
    }

    /// An expense to add, of `cost` on `date`, that `payers` paid and `split` shares.
    public init(
        cost: Cost,
        date: CalendarDate,
        name: String,
        payers: Split,
        split: ExpenseSplit,
        category: Category.ID? = nil,
        description: String? = nil,
        time: TimeOfDay? = nil,
        tip: Tip? = nil
    ) {
        self.cost = cost
        self.date = date
        self.name = name
        self.payers = payers
        self.split = split
        self.category = category
        self.description = description
        self.time = time
        self.tip = tip
    }

    init(_ expense: EranosGenerated.Expense) {
        self.init(
            cost: Cost(expense.cost),
            date: CalendarDate(expense.date),
            name: expense.name,
            payers: Split(expense.payers),
            split: ExpenseSplit(expense.split),
            category: expense.category.map { Category.ID(rawValue: $0) },
            description: expense.description,
            time: expense.time.map(TimeOfDay.init),
            tip: expense.tip.map(Tip.init)
        )
        id = expense.id.map { ID(rawValue: $0) }
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

    /// The total, amount and tip, in the expense's own currency: what its payers paid.
    ///
    /// - Throws: As ``tipAmount()`` does, and ``EranosError/money(_:detail:)`` with
    ///   ``MoneyFailure/overflow`` for a total out of range.
    public func total() throws(EranosError) -> Money {
        let total = try converting { try EranosGenerated.expenseTotal(expense: generated) }
        return Money(total)
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
    /// An amount in another currency than the group's, converted at `rate`.
    case converted(Money, rate: ExchangeRate)

    /// An amount in the group currency.
    case local(Money)

    /// The amount, in its own currency.
    public var amount: Money {
        switch self {
        case let .converted(amount, _):
            amount
        case let .local(amount):
            amount
        }
    }

    var generated: EranosGenerated.Cost {
        switch self {
        case let .converted(amount, rate):
            .converted(amount: amount.generated, rate: rate.description)
        case let .local(amount):
            .local(amount: amount.generated)
        }
    }

    init(_ cost: EranosGenerated.Cost) {
        switch cost {
        case let .converted(amount, rate):
            self = .converted(Money(amount), rate: ExchangeRate(known: rate))
        case let .local(amount):
            self = .local(Money(amount))
        }
    }
}

/// How an expense's cost is shared: as a whole, or line by line of an itemised bill.
public enum ExpenseSplit: Hashable, Sendable {
    /// Every line is divided by its own split; the lines add up to the amount.
    case itemised([Item])

    /// The whole amount is divided by one split.
    case whole(Split)

    var generated: EranosGenerated.ExpenseSplit {
        switch self {
        case let .itemised(items):
            .itemised(items: items.map(\.generated))
        case let .whole(split):
            .whole(split: split.generated)
        }
    }

    init(_ split: EranosGenerated.ExpenseSplit) {
        switch split {
        case let .itemised(items):
            self = .itemised(items.map(Item.init))
        case let .whole(split):
            self = .whole(Split(split))
        }
    }
}

/// A line of an itemised bill.
public struct Item: Hashable, Sendable {
    /// What was bought.
    public var name: String

    /// How the line's total divides.
    public var split: Split

    /// The price of one unit, in the expense's currency.
    public var unitPrice: Money

    /// How many units, from 1.
    public var quantity: Int32

    var generated: EranosGenerated.Item {
        .init(name: name, split: split.generated, unitPrice: unitPrice.generated, quantity: quantity)
    }

    /// A line of `quantity` units of `name` at `unitPrice`, which `split` divides.
    public init(name: String, split: Split, unitPrice: Money, quantity: Int32 = 1) {
        self.name = name
        self.split = split
        self.unitPrice = unitPrice
        self.quantity = quantity
    }

    init(_ item: EranosGenerated.Item) {
        self.init(name: item.name, split: Split(item.split), unitPrice: Money(item.unitPrice), quantity: item.quantity)
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
    /// A fixed amount in the expense's currency.
    case fixed(Money)

    /// A percentage of the expense's amount.
    case percentage(Percentage)

    var generated: EranosGenerated.TipValue {
        switch self {
        case let .fixed(amount):
            .fixed(amount: amount.generated)
        case let .percentage(percentage):
            .percentage(basisPoints: percentage.basisPoints)
        }
    }

    init(_ value: EranosGenerated.TipValue) {
        switch value {
        case let .fixed(amount):
            self = .fixed(Money(amount))
        case let .percentage(basisPoints):
            self = .percentage(Percentage(known: basisPoints))
        }
    }
}

/// How a tip is shared among an expense's participants.
public enum TipDistribution: Hashable, Sendable, CaseIterable {
    /// The same for every participant.
    case equal

    /// In proportion to each participant's share of the amount.
    case proportional

    var generated: EranosGenerated.TipDistribution {
        switch self {
        case .equal:
            .equal
        case .proportional:
            .proportional
        }
    }

    init(_ distribution: EranosGenerated.TipDistribution) {
        switch distribution {
        case .equal:
            self = .equal
        case .proportional:
            self = .proportional
        }
    }
}
