internal import EranosGenerated

/// Something bought for the group: what it cost, who paid, and how it divides.
///
/// An expense has an identifier from the moment it is made: the app's own, which
/// ``init(id:name:cost:payers:split:date:tip:time:category:note:)`` takes, or one that
/// ``init(name:cost:payers:split:date:tip:time:category:note:)`` generates. To change an
/// expense, read it from the group, change its properties, and pass it to
/// `ExpenseGroup.replace(_:)`.
public struct Expense: Hashable, Sendable, Identifiable, Codable {
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

    /// A note on it.
    public var note: String?

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
            note: note
        )
    }

    /// An expense of `cost`, that `payers` paid and `split` shares, on `date`, under `id`, which
    /// the app gives: its own, or one ``Identifier/generate()`` makes;
    /// ``init(name:cost:payers:split:date:tip:time:category:note:)`` generates it.
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
        note: String? = nil
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
        self.note = note
    }

    /// An expense of `cost`, that `payers` paid and `split` shares, on `date`, under an identifier
    /// ``Identifier/generate()`` makes when this is called;
    /// ``init(id:name:cost:payers:split:date:tip:time:category:note:)`` takes the app's own.
    public init(
        name: String,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: TipRule = .noTip,
        time: TimeOfDay? = nil,
        category: Category.ID = .uncategorised,
        note: String? = nil
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
            note: note
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
            note: expense.note
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
public enum Cost: Hashable, Sendable, Codable {
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
public enum ExpenseSplit: Hashable, Sendable, Codable {
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

    /// Every member who shares the cost, once each, in identifier order.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for a member whose identifier is not one, and the
    ///   failures of an item that does not build.
    public func participants() throws(EranosError) -> [Member.ID] {
        let participants = try converting { try EranosGenerated.expenseSplitParticipants(split: generated) }
        return participants.map { Member.ID(rawValue: $0) }
    }
}

/// A line of an itemised bill.
public struct ExpenseItem: Hashable, Sendable, Codable {
    /// What was bought.
    public var name: String

    /// The price of one unit, in the expense's currency.
    public var unitPrice: Money

    /// How the line's total divides.
    public var split: Split

    /// How many units: ``Quantity/one`` unless given another.
    public var quantity: Quantity

    /// The discount off the line or off each unit: none, a line at full price, unless given one.
    public var discount: Discount?

    var generated: EranosGenerated.ExpenseItem {
        .init(
            name: name,
            unitPrice: unitPrice.generated,
            split: split.generated,
            quantity: quantity.tenThousandths,
            discount: discount?.generated
        )
    }

    /// A line of `quantity` units of `name` at `unitPrice`, which `split` divides, with `discount`
    /// off the line or off each unit.
    public init(name: String, unitPrice: Money, split: Split, quantity: Quantity = .one, discount: Discount? = nil) {
        self.name = name
        self.unitPrice = unitPrice
        self.split = split
        self.quantity = quantity
        self.discount = discount
    }

    init(_ item: EranosGenerated.ExpenseItem) {
        self.init(
            name: item.name,
            unitPrice: Money(item.unitPrice),
            split: Split(item.split),
            quantity: Quantity(known: item.quantity),
            discount: item.discount.map(Discount.init)
        )
    }

    /// The line's total before its discount, as the library computes it: the unit price times the
    /// quantity, rounded half away from zero to the currency's minor unit, so 0.335 at 1.00 EUR is
    /// 0.34 EUR, and a product below half a minor unit is 0.00 EUR.
    ///
    /// - Throws: As ``lineTotal()`` does.
    public func lineTotalBeforeDiscount() throws(EranosError) -> Money {
        let total = try converting { try EranosGenerated.expenseItemLineTotalBeforeDiscount(item: generated) }
        return Money(total)
    }

    /// What the discount takes off the line, as the library computes it: the line's total before
    /// the discount minus its line total, 0.00 EUR for a line without a discount.
    ///
    /// - Throws: As ``lineTotal()`` does.
    public func discountAmount() throws(EranosError) -> Money {
        let amount = try converting { try EranosGenerated.expenseItemDiscountAmount(item: generated) }
        return Money(amount)
    }

    /// The line's total, as the library computes it: the unit price times the quantity, rounded
    /// half away from zero to the currency's minor unit, less the discount, so 0.335 at 1.00 EUR is
    /// 0.34 EUR. A product below half a minor unit, or a discount of the whole price, makes a free
    /// line, 0.00 EUR.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for a member whose identifier is not one,
    ///   ``EranosError/money(_:detail:)`` with ``MoneyFailure/unknownCurrency`` for a unit price or
    ///   a fixed discount whose currency details are not the library's for its code, and the
    ///   failures of an item that does not build: ``EranosError/expense(_:detail:)`` with
    ///   ``ExpenseFailure/emptyItemName`` for a blank name, ``ExpenseFailure/nonPositiveAmount``
    ///   for a unit price that is not positive, ``ExpenseFailure/currencyMismatch(expected:found:)``
    ///   for a fixed discount in another currency, ``ExpenseFailure/negativeDiscount`` for a fixed
    ///   discount below zero, ``ExpenseFailure/discountAboveHundredPercent`` for a percentage above
    ///   100 %, ``ExpenseFailure/discountAbovePrice`` for a discount larger than the price it
    ///   reduces, those of a split that does not divide the line total, among them
    ///   ``ExpenseFailure/amountsDoNotAddUp(expected:actual:)``, and ``ExpenseFailure/overflow`` for
    ///   a line total out of range.
    public func lineTotal() throws(EranosError) -> Money {
        let total = try converting { try EranosGenerated.expenseItemLineTotal(item: generated) }
        return Money(total)
    }
}

/// A discount off a line of an itemised bill: a percentage or a fixed amount, taken off the line
/// as a whole or off the price of each unit.
///
/// Off the line, a percentage is taken of the line's total before the discount and rounded half
/// away from zero to the minor unit. Off each unit, a percentage of the unit price is rounded to
/// the minor unit first, and the reduced price is then multiplied by the quantity, so three units
/// at 3.33 EUR with 50 % off cost 4.98 EUR with the discount off each unit and 4.99 EUR with it
/// off the line. A discount of the whole price, 100 % or a fixed amount equal to what it reduces,
/// makes a free line; a larger one is refused when the item is built.
public struct Discount: Hashable, Sendable, Codable {
    /// How the discount is expressed.
    public var value: DiscountValue

    /// What the discount is taken off: ``DiscountScope/line`` unless given another.
    public var scope: DiscountScope

    var generated: EranosGenerated.Discount {
        .init(value: value.generated, scope: scope.generated)
    }

    /// A discount of `value`, taken off `scope`, which is the line as a whole unless given.
    public init(value: DiscountValue, scope: DiscountScope = .line) {
        self.value = value
        self.scope = scope
    }

    init(_ discount: EranosGenerated.Discount) {
        self.init(value: DiscountValue(discount.value), scope: DiscountScope(discount.scope))
    }
}

/// How a discount is expressed.
public enum DiscountValue: Hashable, Sendable, Codable {
    /// A percentage of what the discount reduces, at most 100 %.
    case percentage(Percentage)

    /// A fixed amount in the item's currency, at most what the discount reduces.
    case fixed(Money)

    var generated: EranosGenerated.DiscountValue {
        switch self {
        case let .percentage(percentage):
            .percentage(basisPoints: percentage.generated)
        case let .fixed(amount):
            .fixed(amount: amount.generated)
        }
    }

    init(_ value: EranosGenerated.DiscountValue) {
        switch value {
        case let .percentage(basisPoints):
            self = .percentage(Percentage(known: basisPoints))
        case let .fixed(amount):
            self = .fixed(Money(amount))
        }
    }
}

/// What a discount is taken off.
public enum DiscountScope: Hashable, Sendable, Codable, CaseIterable {
    /// The line's total, the unit price times the quantity.
    case line

    /// The price of each unit, before it is multiplied by the quantity.
    case unit

    var generated: EranosGenerated.DiscountScope {
        switch self {
        case .line:
            .line
        case .unit:
            .unit
        }
    }

    init(_ scope: EranosGenerated.DiscountScope) {
        switch scope {
        case .line:
            self = .line
        case .unit:
            self = .unit
        }
    }
}

/// A tip on top of an expense's amount, which its payers cover.
public struct TipRule: Hashable, Sendable, Codable {
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
public enum TipValue: Hashable, Sendable, Codable {
    /// A percentage of the expense's amount.
    case percentage(Percentage)

    /// A fixed amount in the expense's currency.
    case fixed(Money)

    var generated: EranosGenerated.TipValue {
        switch self {
        case let .percentage(percentage):
            .percentage(basisPoints: percentage.generated)
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
public enum TipDistribution: Hashable, Sendable, Codable, CaseIterable {
    /// In proportion to each participant's share of the amount.
    case proportional

    /// The same for every participant with a share of the amount above zero.
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
