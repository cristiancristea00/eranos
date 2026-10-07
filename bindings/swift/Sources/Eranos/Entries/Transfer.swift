internal import EranosGenerated

/// Money one member paid another, such as a repayment.
///
/// A transfer to add carries the identifier the app gives it, or none, and then
/// `ExpenseGroup.add(_:)` returns one the library generates.
public struct Transfer: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a transfer.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier: the app's own for a transfer to add, or absent for the library to generate
    /// one, and present for a transfer the group holds or one to replace.
    public var id: ID?

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

    /// A transfer to add, from one member to another, of `cost` on `date`, under `id`, or one the
    /// library generates.
    public init(
        id: ID? = nil,
        from: Member.ID,
        to: Member.ID,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = nil,
        note: String? = nil
    ) {
        self.id = id
        self.from = from
        self.to = to
        self.cost = cost
        self.date = date
        self.time = time
        self.note = note
    }

    init(_ transfer: EranosGenerated.Transfer) {
        self.init(
            id: transfer.id.map { ID(rawValue: $0) },
            from: Member.ID(rawValue: transfer.from),
            to: Member.ID(rawValue: transfer.to),
            cost: Cost(transfer.cost),
            date: CalendarDate(transfer.date),
            time: transfer.time.map(TimeOfDay.init),
            note: transfer.note
        )
    }
}
