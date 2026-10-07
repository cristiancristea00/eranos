internal import EranosGenerated

/// Money one member paid another, such as a repayment.
///
/// A new transfer has no identifier; `Group.add(_:)` gives it one.
public struct Transfer: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a transfer.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: absent for a transfer to add, present for one the group holds.
    public private(set) var id: ID?

    /// The member who paid.
    public var from: Member.ID

    /// The member who was paid.
    public var to: Member.ID

    /// What was paid.
    public var cost: Cost

    /// The day it was paid.
    public var date: CalendarDate

    /// The time of day it was paid.
    public var time: TimeOfDay?

    /// A note on the transfer.
    public var note: String?

    var generated: EranosGenerated.Transfer {
        .init(
            id: id?.rawValue,
            from: from.rawValue,
            to: to.rawValue,
            cost: cost.generated,
            date: date.generated,
            time: time?.generated,
            note: note
        )
    }

    /// A transfer to add, from one member to another, of `cost` on `date`.
    public init(
        from: Member.ID,
        to: Member.ID,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = nil,
        note: String? = nil
    ) {
        self.from = from
        self.to = to
        self.cost = cost
        self.date = date
        self.time = time
        self.note = note
    }

    init(_ transfer: EranosGenerated.Transfer) {
        self.init(
            from: Member.ID(rawValue: transfer.from),
            to: Member.ID(rawValue: transfer.to),
            cost: Cost(transfer.cost),
            date: CalendarDate(transfer.date),
            time: transfer.time.map(TimeOfDay.init),
            note: transfer.note
        )
        id = transfer.id.map { ID(rawValue: $0) }
    }
}
