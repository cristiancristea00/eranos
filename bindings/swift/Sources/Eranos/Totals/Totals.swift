internal import EranosGenerated

/// The group's totals: overall, per member, per category, and per member per category, all in the
/// group currency.
public struct Summary: Hashable, Sendable {
    /// What the group spent.
    public let totalSpent: Money

    /// The totals of every member.
    public let members: [MemberSummary]

    /// The total of every category.
    public let categoryTotals: [CategoryTotal]

    /// The totals per category of every member.
    public let memberCategoryTotals: [MemberCategoryTotals]

    init(_ summary: EranosGenerated.Summary) {
        totalSpent = Money(summary.totalSpent)
        members = summary.members.map(MemberSummary.init)
        categoryTotals = summary.categoryTotals.map(CategoryTotal.init)
        memberCategoryTotals = summary.memberCategoryTotals.map(MemberCategoryTotals.init)
    }
}

/// A member's totals, in the group currency.
public struct MemberSummary: Hashable, Sendable {
    /// The member.
    public let member: Member.ID

    /// What the member paid for expenses.
    public let paid: Money

    /// The member's share of the expenses: what they consumed.
    public let share: Money

    /// What the member sent in transfers.
    public let sent: Money

    /// What the member received in transfers.
    public let received: Money

    /// What the member is owed, less what they owe: positive when they get money back.
    public let net: Money

    /// Where the member stands.
    public let balance: Balance

    init(_ summary: EranosGenerated.MemberSummary) {
        member = Member.ID(rawValue: summary.member)
        paid = Money(summary.paid)
        share = Money(summary.share)
        sent = Money(summary.sent)
        received = Money(summary.received)
        net = Money(summary.net)
        balance = Balance(summary.balance)
    }
}

/// What the expenses of a category add up to, in the group currency.
public struct CategoryTotal: Hashable, Sendable {
    /// The category.
    public let category: Category.ID

    /// The total.
    public let total: Money

    init(_ total: EranosGenerated.CategoryTotal) {
        category = Category.ID(rawValue: total.category)
        self.total = Money(total.total)
    }
}

/// What one member's shares add up to in each category.
public struct MemberCategoryTotals: Hashable, Sendable {
    /// The member.
    public let member: Member.ID

    /// The member's total in every category they have a share in.
    public let totals: [CategoryTotal]

    init(_ totals: EranosGenerated.MemberCategoryTotals) {
        member = Member.ID(rawValue: totals.member)
        self.totals = totals.totals.map(CategoryTotal.init)
    }
}

/// Where a member stands with the group.
public enum Balance: Hashable, Sendable, CustomStringConvertible {
    /// The member owes the others the amount.
    case owes(Money)

    /// The member owes nothing and is owed nothing.
    case settled

    /// The others owe the member the amount.
    case getsBack(Money)

    /// The balance as the library writes it: "owes 12.00 EUR".
    public var description: String {
        (try? EranosGenerated.balanceFormat(balance: generated)) ?? String(describing: generated)
    }

    var generated: EranosGenerated.Balance {
        switch self {
        case let .owes(amount):
            .owes(amount: amount.generated)
        case .settled:
            .settled
        case let .getsBack(amount):
            .getsBack(amount: amount.generated)
        }
    }

    init(_ balance: EranosGenerated.Balance) {
        switch balance {
        case let .owes(amount):
            self = .owes(Money(amount))
        case .settled:
            self = .settled
        case let .getsBack(amount):
            self = .getsBack(Money(amount))
        }
    }
}

/// How one expense divides, in the group currency.
public struct ExpenseShares: Hashable, Sendable {
    /// What every member paid and consumed.
    public let members: [MemberShare]

    /// The total: the amount and the tip.
    public let total: Money

    /// The tip.
    public let tip: Money

    init(_ shares: EranosGenerated.ExpenseShares) {
        members = shares.members.map(MemberShare.init)
        total = Money(shares.total)
        tip = Money(shares.tip)
    }
}

/// What one member paid of an expense and what their share of it is.
public struct MemberShare: Hashable, Sendable {
    /// The member.
    public let member: Member.ID

    /// What the member paid, tip included.
    public let paid: Money

    /// The member's share, tip included.
    public let share: Money

    init(_ share: EranosGenerated.MemberShare) {
        member = Member.ID(rawValue: share.member)
        paid = Money(share.paid)
        self.share = Money(share.share)
    }
}

/// The payments that settle every balance.
public struct SettlementPlan: Hashable, Sendable {
    /// The payments, ordered by payer and then by recipient.
    public let payments: [Payment]

    /// The strategy that produced them: the one asked for, or, for the adaptive strategy, the one
    /// whose plan it chose.
    public let strategy: SettlementStrategy

    init(_ settlement: EranosGenerated.SettlementPlan) {
        payments = settlement.payments.map(Payment.init)
        strategy = SettlementStrategy(settlement.strategy)
    }
}

/// One payment of a settlement: one member pays another an amount.
public struct Payment: Hashable, Sendable {
    /// The member who pays.
    public let from: Member.ID

    /// The member who is paid.
    public let to: Member.ID

    /// The amount, in the group currency.
    public let amount: Money

    init(_ payment: EranosGenerated.Payment) {
        from = Member.ID(rawValue: payment.from)
        to = Member.ID(rawValue: payment.to)
        amount = Money(payment.amount)
    }
}

/// How balances turn into payments.
public enum SettlementStrategy: Hashable, Sendable, CaseIterable {
    /// The fewest payments while at most 20 balances are left after exact opposites pay each other,
    /// the greedy plan beyond; the settlement's ``SettlementPlan/strategy`` names the strategy whose plan
    /// it is.
    case adaptive

    /// The fewest payments possible.
    case minimumTransfers

    /// The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each other where that saves a payment.
    case greedy

    /// Everyone pays back the people who paid for them.
    case direct

    var generated: EranosGenerated.SettlementStrategy {
        switch self {
        case .adaptive:
            .adaptive
        case .minimumTransfers:
            .minimumTransfers
        case .greedy:
            .greedy
        case .direct:
            .direct
        }
    }

    init(_ strategy: EranosGenerated.SettlementStrategy) {
        switch strategy {
        case .adaptive:
            self = .adaptive
        case .minimumTransfers:
            self = .minimumTransfers
        case .greedy:
            self = .greedy
        case .direct:
            self = .direct
        }
    }
}
