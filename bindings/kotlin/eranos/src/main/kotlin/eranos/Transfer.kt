package eranos

/**
 * Money one member paid another, such as a repayment.
 *
 * A new transfer has no identifier; [Group.add] gives it one. To change a transfer, read it from the group, [copy] it with
 * the changes, and pass the copy to [Group.replace].
 */
@Suppress("LongParameterList") // A transfer has the library's fields; the public constructor defaults the optional ones.
public class Transfer internal constructor(
    /** The identifier: `null` for a transfer to add, set for one the group holds. */
    public val id: TransferId?,
    /** What was paid. */
    public val cost: Cost,
    /** The day it was paid. */
    public val date: CalendarDate,
    /** The member who paid. */
    public val from: MemberId,
    /** The member who was paid. */
    public val to: MemberId,
    /** A note on the transfer. */
    public val note: String?,
    /** The time of day it was paid. */
    public val time: TimeOfDay?,
) {
    /** A transfer to add, of [cost] on [date], from one member to another. */
    public constructor(
        cost: Cost,
        date: CalendarDate,
        from: MemberId,
        to: MemberId,
        note: String? = null,
        time: TimeOfDay? = null,
    ) : this(null, cost, date, from, to, note, time)

    /** The transfer as the generated code holds it. */
    internal val generated: eranos.internal.Transfer
        get() = eranos.internal.Transfer(id?.value, cost.generated, date.generated, from.value, to.value, note, time?.generated)

    /** This transfer with the given fields changed; the identifier stays, so the copy can replace the transfer. */
    public fun copy(
        cost: Cost = this.cost,
        date: CalendarDate = this.date,
        from: MemberId = this.from,
        to: MemberId = this.to,
        note: String? = this.note,
        time: TimeOfDay? = this.time,
    ): Transfer = Transfer(id, cost, date, from, to, note, time)

    /** Whether [other] is of the same class, with the same [id], [cost], [date], [from], [to], [note], and [time]. */
    override fun equals(other: Any?): Boolean =
        other is Transfer &&
            id == other.id &&
            cost == other.cost &&
            date == other.date &&
            from == other.from &&
            to == other.to &&
            note == other.note &&
            time == other.time

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, cost, date, from, to, note, time)

    /** A description of every field, for logs. */
    override fun toString(): String = "Transfer(id=$id, cost=$cost, date=$date, from=$from, to=$to, note=$note, time=$time)"
}

/** The transfer the generated code holds as [this]. */
internal fun eranos.internal.Transfer.converted(): Transfer =
    Transfer(
        id?.let(::TransferId),
        cost.converted(),
        date.converted(),
        MemberId(from),
        MemberId(to),
        note,
        time?.converted(),
    )
