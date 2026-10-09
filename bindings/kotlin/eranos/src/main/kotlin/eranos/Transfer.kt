package eranos

/**
 * Money one member paid another, such as a repayment.
 *
 * A transfer has its identifier from the moment it is made: the app's own, such as the key of its own store, or a generated
 * one. To change a transfer, read it from the group, [copy] it with the changes, and pass the copy to [ExpenseGroup.replace].
 *
 * @constructor A transfer of [cost] on [date], from one member to another, under the identifier [id]; the constructor
 *   without [id] generates one.
 */
@Suppress("LongParameterList") // A transfer has the library's fields; the public constructors default the optional ones.
public class Transfer(
    /** The identifier, which the app gives: its own, or one [TransferId.generate] makes. */
    public val id: TransferId,
    /** The member who paid. */
    public val sender: MemberId,
    /** The member who was paid. */
    public val receiver: MemberId,
    /** What was paid. */
    public val cost: Cost,
    /** The day it was paid. */
    public val date: CalendarDate,
    /** The time of day it was paid. */
    public val time: TimeOfDay? = null,
    /** A note on the transfer. */
    public val note: String? = null,
) {
    /**
     * A transfer of [cost] on [date], from one member to another, under an identifier [TransferId.generate] makes; the
     * constructor with [id] takes the app's.
     */
    public constructor(
        sender: MemberId,
        receiver: MemberId,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = null,
        note: String? = null,
    ) : this(TransferId.generate(), sender, receiver, cost, date, time, note)

    /**
     * The transfer that records [payment] of a settlement, made on [date]: from its sender to its receiver, of its amount in
     * the group currency, under [id], which the app gives; the constructor without [id] generates it.
     */
    public constructor(
        id: TransferId,
        payment: Payment,
        date: CalendarDate,
        time: TimeOfDay? = null,
        note: String? = null,
    ) : this(id, payment.sender, payment.receiver, Cost.Local(payment.amount), date, time, note)

    /**
     * The transfer that records [payment] of a settlement, made on [date], under an identifier [TransferId.generate] makes;
     * the constructor with an identifier takes the app's.
     */
    public constructor(
        payment: Payment,
        date: CalendarDate,
        time: TimeOfDay? = null,
        note: String? = null,
    ) : this(TransferId.generate(), payment, date, time, note)

    /** The transfer as the generated code holds it. */
    internal val generated: eranos.internal.Transfer
        get() = eranos.internal.Transfer(id.value, sender.value, receiver.value, cost.generated, date.generated, time?.generated, note)

    /** This transfer with the given fields changed; the identifier stays, so the copy can replace the transfer. */
    public fun copy(
        sender: MemberId = this.sender,
        receiver: MemberId = this.receiver,
        cost: Cost = this.cost,
        date: CalendarDate = this.date,
        time: TimeOfDay? = this.time,
        note: String? = this.note,
    ): Transfer = Transfer(id, sender, receiver, cost, date, time, note)

    /** Whether [other] is of the same class, with the same [id], [sender], [receiver], [cost], [date], [time], and [note]. */
    override fun equals(other: Any?): Boolean =
        other is Transfer &&
            id == other.id &&
            sender == other.sender &&
            receiver == other.receiver &&
            cost == other.cost &&
            date == other.date &&
            time == other.time &&
            note == other.note

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, sender, receiver, cost, date, time, note)

    /** A description of every field, for logs. */
    override fun toString(): String = "Transfer(id=$id, sender=$sender, receiver=$receiver, cost=$cost, date=$date, time=$time, note=$note)"
}

/** The transfer the generated code holds as [this]. */
internal fun eranos.internal.Transfer.converted(): Transfer =
    Transfer(
        TransferId(id),
        MemberId(sender),
        MemberId(receiver),
        cost.converted(),
        date.converted(),
        time?.converted(),
        note,
    )
