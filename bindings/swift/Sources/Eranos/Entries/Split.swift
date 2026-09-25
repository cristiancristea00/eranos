internal import EranosGenerated

/// How an amount divides among members: equally, by exact amounts, by parts, or by percentages.
///
/// The library divides every amount so that the shares add up to it exactly, and gives any
/// minor unit left over by the rounding to a member it picks from the expense's identifier.
public enum Split: Hashable, Sendable {
    /// Every member pays an exact amount; the amounts add up to what they divide.
    case amounts([MemberAmount])

    /// Every member pays the same.
    case equal([Member.ID])

    /// Every member pays in proportion to their parts.
    case parts([MemberParts])

    /// Every member pays a percentage; the percentages add up to 100 %.
    case percentages([MemberPercentage])

    /// The members the split divides among, in the order it lists them.
    public var members: [Member.ID] {
        switch self {
        case let .amounts(shares):
            shares.map(\.member)
        case let .equal(members):
            members
        case let .parts(shares):
            shares.map(\.member)
        case let .percentages(shares):
            shares.map(\.member)
        }
    }

    var generated: EranosGenerated.Split {
        switch self {
        case let .amounts(shares):
            .amounts(shares: shares.map(\.generated))
        case let .equal(members):
            .equal(members: members.map(\.rawValue))
        case let .parts(shares):
            .parts(shares: shares.map(\.generated))
        case let .percentages(shares):
            .percentages(shares: shares.map(\.generated))
        }
    }

    init(_ split: EranosGenerated.Split) {
        switch split {
        case let .amounts(shares):
            self = .amounts(shares.map(MemberAmount.init))
        case let .equal(members):
            self = .equal(members.map { Member.ID(rawValue: $0) })
        case let .parts(shares):
            self = .parts(shares.map(MemberParts.init))
        case let .percentages(shares):
            self = .percentages(shares.map(MemberPercentage.init))
        }
    }
}

/// The exact amount one member pays of a split.
public struct MemberAmount: Hashable, Sendable {
    /// The amount.
    public var amount: Money

    /// The member.
    public var member: Member.ID

    var generated: EranosGenerated.MemberAmount {
        .init(amount: amount.generated, member: member.rawValue)
    }

    /// The share of `member`, who pays `amount`.
    public init(amount: Money, member: Member.ID) {
        self.amount = amount
        self.member = member
    }

    init(_ share: EranosGenerated.MemberAmount) {
        self.init(amount: Money(share.amount), member: Member.ID(rawValue: share.member))
    }
}

/// The parts one member pays in proportion to.
public struct MemberParts: Hashable, Sendable {
    /// The member.
    public var member: Member.ID

    /// The member's parts.
    public var parts: Parts

    var generated: EranosGenerated.MemberParts {
        .init(hundredths: parts.hundredths, member: member.rawValue)
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
        .init(basisPoints: percentage.basisPoints, member: member.rawValue)
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
