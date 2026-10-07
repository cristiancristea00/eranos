package eranos

/**
 * Money one member paid another, such as a repayment.
 *
 * A new transfer has no identifier; [ExpenseGroup.add] gives it one. To change a transfer, read it from the group, [copy] it with
 * the changes, and pass the copy to [ExpenseGroup.replace].
 */
@Suppress("LongParameterList") // A transfer has the library's fields; the public constructor defaults the optional ones.
public class Transfer internal constructor(
    /** The identifier: `null` for a transfer to add, set for one the group holds. */
    public val id: TransferId?,
    /** The member who paid. */
    public val from: MemberId,
    /** The member who was paid. */
    public val to: MemberId,
    /** What was paid. */
    public val cost: Cost,
    /** The day it was paid. */
    public val date: CalendarDate,
    /** The time of day it was paid. */
    public val time: TimeOfDay?,
    /** A note on the transfer. */
    public val note: String?,
) {
    /** A transfer to add, of [cost] on [date], from one member to another. */
    public constructor(
        from: MemberId,
        to: MemberId,
        cost: Cost,
        date: CalendarDate,
        time: TimeOfDay? = null,
        note: String? = null,
    ) : this(null, from, to, cost, date, time, note)

    /** The transfer as the generated code holds it. */
    internal val generated: eranos.internal.Transfer
        get() = eranos.internal.Transfer(id?.value, from.value, to.value, cost.generated, date.generated, time?.generated, note)

    /** This transfer with the given fields changed; the identifier stays, so the copy can replace the transfer. */
    public fun copy(
        from: MemberId = this.from,
        to: MemberId = this.to,
        cost: Cost = this.cost,
        date: CalendarDate = this.date,
        time: TimeOfDay? = this.time,
        note: String? = this.note,
    ): Transfer = Transfer(id, from, to, cost, date, time, note)

    /** Whether [other] is of the same class, with the same [id], [from], [to], [cost], [date], [time], and [note]. */
    override fun equals(other: Any?): Boolean =
        other is Transfer &&
            id == other.id &&
            from == other.from &&
            to == other.to &&
            cost == other.cost &&
            date == other.date &&
            time == other.time &&
            note == other.note

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, from, to, cost, date, time, note)

    /** A description of every field, for logs. */
    override fun toString(): String = "Transfer(id=$id, from=$from, to=$to, cost=$cost, date=$date, time=$time, note=$note)"
}

/** The transfer the generated code holds as [this]. */
internal fun eranos.internal.Transfer.converted(): Transfer =
    Transfer(
        id?.let(::TransferId),
        MemberId(from),
        MemberId(to),
        cost.converted(),
        date.converted(),
        time?.converted(),
        note,
    )
