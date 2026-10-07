internal import EranosGenerated

/// Something bought for the group: what it cost, who paid, and how it divides.
///
/// An expense has an identifier from the moment it is made: the app's own, which
/// ``init(id:name:cost:payers:split:date:tip:time:category:description:)`` takes, or one that
/// ``init(name:cost:payers:split:date:tip:time:category:description:)`` generates. To change an
/// expense, read it from the group, change its properties, and pass it to
/// `ExpenseGroup.replace(_:)`.
public struct Expense: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of an expense.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier, which the app gives: its own, or one ``Identifier/generate()`` makes.
    public var id: ID

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

    /// The tip on top of the amount: ``TipRule/noTip`` for an expense without one.
    public var tip: TipRule

    /// The time of day it was bought.
    public var time: TimeOfDay?

    /// The category: the built-in one, ``Identifier/uncategorised``, until it is given another.
    public var category: Category.ID

    /// A longer description.
    public var description: String?

    var generated: EranosGenerated.Expense {
        .init(
            id: id.rawValue,
            name: name,
            cost: cost.generated,
            payers: payers.generated,
            split: split.generated,
            date: date.generated,
            tip: tip.generated,
            time: time?.generated,
            category: category.rawValue,
            description: description
        )
    }

    /// An expense of `cost`, that `payers` paid and `split` shares, on `date`, under `id`, which
    /// the app gives: its own, or one ``Identifier/generate()`` makes;
    /// ``init(name:cost:payers:split:date:tip:time:category:description:)`` generates it.
    public init(
        id: ID,
        name: String,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: TipRule = .noTip,
        time: TimeOfDay? = nil,
        category: Category.ID = .uncategorised,
        description: String? = nil
    ) {
        self.id = id
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

    /// An expense of `cost`, that `payers` paid and `split` shares, on `date`, under an identifier
    /// ``Identifier/generate()`` makes when this is called;
    /// ``init(id:name:cost:payers:split:date:tip:time:category:description:)`` takes the app's own.
    public init(
        name: String,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: TipRule = .noTip,
        time: TimeOfDay? = nil,
        category: Category.ID = .uncategorised,
        description: String? = nil
    ) {
        self.init(
            id: .generate(),
            name: name,
            cost: cost,
            payers: payers,
            split: split,
            date: date,
            tip: tip,
            time: time,
            category: category,
            description: description
        )
    }

    init(_ expense: EranosGenerated.Expense) {
        self.init(
            id: ID(rawValue: expense.id),
            name: expense.name,
            cost: Cost(expense.cost),
            payers: Split(expense.payers),
            split: ExpenseSplit(expense.split),
            date: CalendarDate(expense.date),
            tip: TipRule(expense.tip),
            time: expense.time.map(TimeOfDay.init),
            category: Category.ID(rawValue: expense.category),
            description: expense.description
        )
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
    case itemised([ExpenseItem])

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
            self = .itemised(items.map(ExpenseItem.init))
        }
    }
}

/// A line of an itemised bill.
public struct ExpenseItem: Hashable, Sendable {
    /// What was bought.
    public var name: String

    /// The price of one unit, in the expense's currency.
    public var unitPrice: Money

    /// How the line's total divides.
    public var split: Split

    /// How many units, from 1.
    public var quantity: Int32

    var generated: EranosGenerated.ExpenseItem {
        .init(name: name, unitPrice: unitPrice.generated, split: split.generated, quantity: quantity)
    }

    /// A line of `quantity` units of `name` at `unitPrice`, which `split` divides.
    public init(name: String, unitPrice: Money, split: Split, quantity: Int32 = 1) {
        self.name = name
        self.unitPrice = unitPrice
        self.split = split
        self.quantity = quantity
    }

    init(_ item: EranosGenerated.ExpenseItem) {
        self.init(name: item.name, unitPrice: Money(item.unitPrice), split: Split(item.split), quantity: item.quantity)
    }
}

/// A tip on top of an expense's amount, which its payers cover.
public struct TipRule: Hashable, Sendable {
    /// No tip: 0 % of the amount, shared proportionally, which an expense has until it is given
    /// another tip.
    public static let noTip = Self(EranosGenerated.tipRuleNone())

    /// How the tip is expressed.
    public var value: TipValue

    /// How the tip is shared among the participants.
    public var distribution: TipDistribution

    var generated: EranosGenerated.TipRule {
        .init(value: value.generated, distribution: distribution.generated)
    }

    /// A tip of `value`, shared by `distribution`, which is in proportion to each share unless
    /// given.
    public init(value: TipValue, distribution: TipDistribution = .proportional) {
        self.value = value
        self.distribution = distribution
    }

    init(_ tip: EranosGenerated.TipRule) {
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
