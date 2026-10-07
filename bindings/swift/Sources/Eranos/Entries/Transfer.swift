internal import EranosGenerated

/// Money one member paid another, such as a repayment.
///
/// A transfer has an identifier from the moment it is made: the app's own, which
/// ``init(id:from:to:cost:date:time:note:)`` takes, or one that
/// ``init(from:to:cost:date:time:note:)`` generates.
public struct Transfer: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a transfer.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier, which the app gives: its own, or one ``Identifier/generate()`` makes.
    public var id: ID

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
            id: id.rawValue,
            from: from.rawValue,
            to: to.rawValue,
            cost: cost.generated,
            date: date.generated,
            time: time?.generated,
            note: note
        )
    }

    /// A transfer from one member to another, of `cost` on `date`, under `id`, which the app gives:
    /// its own, or one ``Identifier/generate()`` makes; ``init(from:to:cost:date:time:note:)``
    /// generates it.
    public init(
        id: ID,
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

    /// A transfer from one member to another, of `cost` on `date`, under an identifier
    /// ``Identifier/generate()`` makes when this is called;
    /// ``init(id:from:to:cost:date:time:note:)`` takes the app's own.
    public init(
        from: Member.ID,
        to: Member.ID,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = nil,
        note: String? = nil
    ) {
        self.init(id: .generate(), from: from, to: to, cost: cost, date: date, time: time, note: note)
    }

    init(_ transfer: EranosGenerated.Transfer) {
        self.init(
            id: ID(rawValue: transfer.id),
            from: Member.ID(rawValue: transfer.from),
            to: Member.ID(rawValue: transfer.to),
            cost: Cost(transfer.cost),
            date: CalendarDate(transfer.date),
            time: transfer.time.map(TimeOfDay.init),
            note: transfer.note
        )
    }
}
