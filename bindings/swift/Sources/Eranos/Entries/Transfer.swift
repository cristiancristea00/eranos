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

    /// What was paid.
    public var cost: Cost

    /// The day it was paid.
    public var date: CalendarDate

    /// The member who paid.
    public var from: Member.ID

    /// The member who was paid.
    public var to: Member.ID

    /// A note on the transfer.
    public var note: String?

    /// The time of day it was paid.
    public var time: TimeOfDay?

    var generated: EranosGenerated.Transfer {
        .init(
            id: id?.rawValue,
            cost: cost.generated,
            date: date.generated,
            from: from.rawValue,
            to: to.rawValue,
            note: note,
            time: time?.generated
        )
    }

    /// A transfer to add, of `cost` on `date`, from one member to another.
    public init(
        cost: Cost,
        date: CalendarDate,
        from: Member.ID,
        to: Member.ID,
        note: String? = nil,
        time: TimeOfDay? = nil
    ) {
        self.cost = cost
        self.date = date
        self.from = from
        self.to = to
        self.note = note
        self.time = time
    }

    init(_ transfer: EranosGenerated.Transfer) {
        self.init(
            cost: Cost(transfer.cost),
            date: CalendarDate(transfer.date),
            from: Member.ID(rawValue: transfer.from),
            to: Member.ID(rawValue: transfer.to),
            note: transfer.note,
            time: transfer.time.map(TimeOfDay.init)
        )
        id = transfer.id.map { ID(rawValue: $0) }
    }
}
