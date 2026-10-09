internal import EranosGenerated

/// Money one member paid another, such as a repayment.
///
/// A transfer has an identifier from the moment it is made: the app's own, which
/// ``init(id:sender:receiver:cost:date:time:note:)`` takes, or one that
/// ``init(sender:receiver:cost:date:time:note:)`` generates.
public struct Transfer: Hashable, Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a transfer.
    public typealias ID = Identifier<Self> // swiftlint:disable:this type_name

    /// The identifier, which the app gives: its own, or one ``Identifier/generate()`` makes.
    public var id: ID

    /// The member who paid.
    public var sender: Member.ID

    /// The member who was paid.
    public var receiver: Member.ID

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
            sender: sender.rawValue,
            receiver: receiver.rawValue,
            cost: cost.generated,
            date: date.generated,
            time: time?.generated,
            note: note
        )
    }

    /// A transfer from one member to another, of `cost` on `date`, under `id`, which the app gives:
    /// its own, or one ``Identifier/generate()`` makes;
    /// ``init(sender:receiver:cost:date:time:note:)`` generates it.
    public init(
        id: ID,
        sender: Member.ID,
        receiver: Member.ID,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = nil,
        note: String? = nil
    ) {
        self.id = id
        self.sender = sender
        self.receiver = receiver
        self.cost = cost
        self.date = date
        self.time = time
        self.note = note
    }

    /// A transfer from one member to another, of `cost` on `date`, under an identifier
    /// ``Identifier/generate()`` makes when this is called;
    /// ``init(id:sender:receiver:cost:date:time:note:)`` takes the app's own.
    public init(
        sender: Member.ID,
        receiver: Member.ID,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = nil,
        note: String? = nil
    ) {
        self.init(id: .generate(), sender: sender, receiver: receiver, cost: cost, date: date, time: time, note: note)
    }

    /// The transfer that records `payment` of a settlement, made on `date`: from its sender to its
    /// receiver, of its amount in the group currency, under `id`, which the app gives;
    /// ``init(payment:date:time:note:)`` generates it.
    public init(id: ID, payment: Payment, date: CalendarDate, time: TimeOfDay? = nil, note: String? = nil) {
        self.init(id: id, sender: payment.sender, receiver: payment.receiver, cost: .local(payment.amount), date: date, time: time, note: note)
    }

    /// The transfer that records `payment` of a settlement, made on `date`, under an identifier
    /// ``Identifier/generate()`` makes when this is called; ``init(id:payment:date:time:note:)``
    /// takes the app's own.
    public init(payment: Payment, date: CalendarDate, time: TimeOfDay? = nil, note: String? = nil) {
        self.init(id: .generate(), payment: payment, date: date, time: time, note: note)
    }

    init(_ transfer: EranosGenerated.Transfer) {
        self.init(
            id: ID(rawValue: transfer.id),
            sender: Member.ID(rawValue: transfer.sender),
            receiver: Member.ID(rawValue: transfer.receiver),
            cost: Cost(transfer.cost),
            date: CalendarDate(transfer.date),
            time: transfer.time.map(TimeOfDay.init),
            note: transfer.note
        )
    }
}
