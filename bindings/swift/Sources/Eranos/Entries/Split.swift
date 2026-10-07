internal import EranosGenerated

/// How an amount divides among members: equally, by parts, by percentages, or by exact amounts.
///
/// The library divides every amount so that the shares add up to it exactly, and gives any
/// minor unit left over by the rounding to a member it picks from the expense's identifier.
public enum Split: Hashable, Sendable {
    /// Every member pays the same.
    case equal([Member.ID])

    /// Every member pays in proportion to their parts.
    case parts([MemberParts])

    /// Every member pays a percentage; the percentages add up to 100 %.
    case percentages([MemberPercentage])

    /// Every member pays an exact amount; the amounts add up to what they divide.
    case amounts([MemberAmount])

    /// The members the split divides among, in the order it lists them.
    public var members: [Member.ID] {
        switch self {
        case let .equal(members):
            members
        case let .parts(shares):
            shares.map(\.member)
        case let .percentages(shares):
            shares.map(\.member)
        case let .amounts(shares):
            shares.map(\.member)
        }
    }

    var generated: EranosGenerated.Split {
        switch self {
        case let .equal(members):
            .equal(members: members.map(\.rawValue))
        case let .parts(shares):
            .parts(shares: shares.map(\.generated))
        case let .percentages(shares):
            .percentages(shares: shares.map(\.generated))
        case let .amounts(shares):
            .amounts(shares: shares.map(\.generated))
        }
    }

    init(_ split: EranosGenerated.Split) {
        switch split {
        case let .equal(members):
            self = .equal(members.map { Member.ID(rawValue: $0) })
        case let .parts(shares):
            self = .parts(shares.map(MemberParts.init))
        case let .percentages(shares):
            self = .percentages(shares.map(MemberPercentage.init))
        case let .amounts(shares):
            self = .amounts(shares.map(MemberAmount.init))
        }
    }
}

/// The exact amount one member pays of a split.
public struct MemberAmount: Hashable, Sendable {
    /// The member.
    public var member: Member.ID

    /// The amount.
    public var amount: Money

    var generated: EranosGenerated.MemberAmount {
        .init(member: member.rawValue, amount: amount.generated)
    }

    /// The share of `member`, who pays `amount`.
    public init(member: Member.ID, amount: Money) {
        self.member = member
        self.amount = amount
    }

    init(_ share: EranosGenerated.MemberAmount) {
        self.init(member: Member.ID(rawValue: share.member), amount: Money(share.amount))
    }
}

/// The parts one member pays in proportion to.
public struct MemberParts: Hashable, Sendable {
    /// The member.
    public var member: Member.ID

    /// The member's parts.
    public var parts: Parts

    var generated: EranosGenerated.MemberParts {
        .init(member: member.rawValue, hundredths: parts.hundredths)
    }

    /// The share of `member`, who pays in proportion to `parts`.
    public init(member: Member.ID, parts: Parts) {
        self.member = member
        self.parts = parts
    }

    init(_ share: EranosGenerated.MemberParts) {
        self.init(member: Member.ID(rawValue: share.member), parts: Parts(known: share.hundredths))
    }
}

/// The percentage one member pays.
public struct MemberPercentage: Hashable, Sendable {
    /// The member.
    public var member: Member.ID

    /// The member's percentage.
    public var percentage: Percentage

    var generated: EranosGenerated.MemberPercentage {
        .init(member: member.rawValue, basisPoints: percentage.basisPoints)
    }

    /// The share of `member`, who pays `percentage`.
    public init(member: Member.ID, percentage: Percentage) {
        self.member = member
        self.percentage = percentage
    }

    init(_ share: EranosGenerated.MemberPercentage) {
        self.init(member: Member.ID(rawValue: share.member), percentage: Percentage(known: share.basisPoints))
    }
}
