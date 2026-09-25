package eranos

import eranos.internal.expenseMembers
import eranos.internal.expenseTipAmount
import eranos.internal.expenseTotal

/**
 * Something bought for the group: what it cost, who paid, and how it divides.
 *
 * A new expense has no identifier; [Group.add] gives it one. To change an expense, read it from the group, [copy] it with
 * the changes, and pass the copy to [Group.replace].
 */
@Suppress("LongParameterList") // An expense has the library's fields; the public constructor defaults the optional ones.
public class Expense internal constructor(
    /** The identifier: `null` for an expense to add, set for one the group holds. */
    public val id: ExpenseId?,
    /** What it cost. */
    public val cost: Cost,
    /** The day it was bought. */
    public val date: CalendarDate,
    /** What was bought. */
    public val name: String,
    /** Who paid the total, amount and tip, and how much each. */
    public val payers: Split,
    /** How the amount is shared. */
    public val split: ExpenseSplit,
    /** The category; `null` for the built-in one. */
    public val category: CategoryId?,
    /** A longer description. */
    public val description: String?,
    /** The time of day it was bought. */
    public val time: TimeOfDay?,
    /** The tip on top of the amount. */
    public val tip: Tip?,
) {
    /** An expense to add, of [cost] on [date], that [payers] paid and [split] shares. */
    public constructor(
        cost: Cost,
        date: CalendarDate,
        name: String,
        payers: Split,
        split: ExpenseSplit,
        category: CategoryId? = null,
        description: String? = null,
        time: TimeOfDay? = null,
        tip: Tip? = null,
    ) : this(null, cost, date, name, payers, split, category, description, time, tip)

    /** The expense as the generated code holds it. */
    internal val generated: eranos.internal.Expense
        get() =
            eranos.internal.Expense(
                id?.value,
                cost.generated,
                date.generated,
                name,
                payers.generated,
                split.generated,
                category?.value,
                description,
                time?.generated,
                tip?.generated,
            )

    /** This expense with the given fields changed; the identifier stays, so the copy can replace the expense. */
    public fun copy(
        cost: Cost = this.cost,
        date: CalendarDate = this.date,
        name: String = this.name,
        payers: Split = this.payers,
        split: ExpenseSplit = this.split,
        category: CategoryId? = this.category,
        description: String? = this.description,
        time: TimeOfDay? = this.time,
        tip: Tip? = this.tip,
    ): Expense = Expense(id, cost, date, name, payers, split, category, description, time, tip)

    /**
     * The tip, in the expense's own currency.
     *
     * @throws EranosException.Expense with [ExpenseFailure.CurrencyMismatch] for a fixed tip in another currency, and
     *   [ExpenseFailure.NegativeTip] for a fixed tip below zero.
     */
    @Throws(EranosException::class)
    public fun tipAmount(): Money = converting { expenseTipAmount(generated).converted() }

    /**
     * The total, amount and tip, in the expense's own currency: what its payers paid.
     *
     * @throws EranosException.Expense as [tipAmount] does, and [EranosException.Money] with [MoneyFailure.Overflow] for a
     *   total out of range.
     */
    @Throws(EranosException::class)
    public fun total(): Money = converting { expenseTotal(generated).converted() }

    /**
     * Every member the expense involves, as a payer or a participant, in identifier order.
     *
     * @throws EranosException.Id for a member whose identifier is not one, and the failures of an item that does not build.
     */
    @Throws(EranosException::class)
    public fun members(): List<MemberId> = converting { expenseMembers(generated).map(::MemberId) }

    /**
     * Whether [other] is of the same class, with the same [id], [cost], [date], [name], [payers], [split], [category],
     * [description], [time], and [tip].
     */
    override fun equals(other: Any?): Boolean =
        other is Expense &&
            id == other.id &&
            cost == other.cost &&
            date == other.date &&
            name == other.name &&
            payers == other.payers &&
            split == other.split &&
            category == other.category &&
            description == other.description &&
            time == other.time &&
            tip == other.tip

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, cost, date, name, payers, split, category, description, time, tip)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "Expense(id=$id, cost=$cost, date=$date, name=$name, payers=$payers, split=$split, category=$category, " +
            "description=$description, time=$time, tip=$tip)"
}

/** The expense the generated code holds as [this]. */
internal fun eranos.internal.Expense.converted(): Expense =
    Expense(
        id?.let(::ExpenseId),
        cost.converted(),
        date.converted(),
        name,
        payers.converted(),
        split.converted(),
        category?.let(::CategoryId),
        description,
        time?.converted(),
        tip?.converted(),
    )

/**
 * What an entry cost: an amount in the group currency, or one in another currency with the rate that converts it into the
 * group currency.
 */
public sealed interface Cost {
    /** The amount, in its own currency. */
    public val amount: Money

    /** An amount in another currency than the group's, converted at [rate]. */
    public class Converted(
        override val amount: Money,
        /** The rate that converts the amount into the group currency. */
        public val rate: ExchangeRate,
    ) : Cost {
        /** Whether [other] is of the same class, with the same [amount] and [rate]. */
        override fun equals(other: Any?): Boolean = other is Converted && amount == other.amount && rate == other.rate

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = hashOf(amount, rate)

        /** A description of every field, for logs. */
        override fun toString(): String = "Converted(amount=$amount, rate=$rate)"
    }

    /** An amount in the group currency. */
    public class Local(
        override val amount: Money,
    ) : Cost {
        /** Whether [other] is of the same class, with the same [amount]. */
        override fun equals(other: Any?): Boolean = other is Local && amount == other.amount

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = amount.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Local(amount=$amount)"
    }
}

/** The cost as the generated code holds it. */
internal val Cost.generated: eranos.internal.Cost
    get() =
        when (this) {
            is Cost.Converted -> eranos.internal.Cost.Converted(amount.generated, rate.toString())
            is Cost.Local -> eranos.internal.Cost.Local(amount.generated)
        }

/** The cost the generated code holds as [this]. */
internal fun eranos.internal.Cost.converted(): Cost =
    when (this) {
        is eranos.internal.Cost.Converted -> Cost.Converted(amount.converted(), ExchangeRate(rate))
        is eranos.internal.Cost.Local -> Cost.Local(amount.converted())
    }

/** How an expense's cost is shared: as a whole, or line by line of an itemised bill. */
public sealed interface ExpenseSplit {
    /** Every line is divided by its own split; the lines add up to the amount. */
    public class Itemised(
        items: List<Item>,
    ) : ExpenseSplit {
        /** The lines of the bill. */
        public val items: List<Item> = items.toList()

        /** Whether [other] is of the same class, with the same [items]. */
        override fun equals(other: Any?): Boolean = other is Itemised && items == other.items

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = items.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Itemised(items=$items)"
    }

    /** The whole amount is divided by one split. */
    public class Whole(
        /** How the amount divides. */
        public val split: Split,
    ) : ExpenseSplit {
        /** Whether [other] is of the same class, with the same [split]. */
        override fun equals(other: Any?): Boolean = other is Whole && split == other.split

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = split.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Whole(split=$split)"
    }
}

/** The expense split as the generated code holds it. */
internal val ExpenseSplit.generated: eranos.internal.ExpenseSplit
    get() =
        when (this) {
            is ExpenseSplit.Itemised -> eranos.internal.ExpenseSplit.Itemised(items.map { it.generated })
            is ExpenseSplit.Whole -> eranos.internal.ExpenseSplit.Whole(split.generated)
        }

/** The expense split the generated code holds as [this]. */
internal fun eranos.internal.ExpenseSplit.converted(): ExpenseSplit =
    when (this) {
        is eranos.internal.ExpenseSplit.Itemised -> ExpenseSplit.Itemised(items.map { it.converted() })
        is eranos.internal.ExpenseSplit.Whole -> ExpenseSplit.Whole(split.converted())
    }

/** A line of an itemised bill: [quantity] units of [name] at [unitPrice], which [split] divides. */
public class Item(
    /** What was bought. */
    public val name: String,
    /** How the line's total divides. */
    public val split: Split,
    /** The price of one unit, in the expense's currency. */
    public val unitPrice: Money,
    /** How many units, from 1. */
    public val quantity: Int = 1,
) {
    /** The line as the generated code holds it. */
    internal val generated: eranos.internal.Item
        get() = eranos.internal.Item(name, split.generated, unitPrice.generated, quantity)

    /** Whether [other] is of the same class, with the same [name], [split], [unitPrice], and [quantity]. */
    override fun equals(other: Any?): Boolean =
        other is Item && name == other.name && split == other.split && unitPrice == other.unitPrice && quantity == other.quantity

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(name, split, unitPrice, quantity)

    /** A description of every field, for logs. */
    override fun toString(): String = "Item(name=$name, split=$split, unitPrice=$unitPrice, quantity=$quantity)"
}

/** The line the generated code holds as [this]. */
internal fun eranos.internal.Item.converted(): Item = Item(name, split.converted(), unitPrice.converted(), quantity)

/** A tip on top of an expense's amount, which its payers cover: [value], shared by [distribution]. */
public class Tip(
    /** How the tip is expressed. */
    public val value: TipValue,
    /** How the tip is shared among the participants: in proportion to each share unless given. */
    public val distribution: TipDistribution = TipDistribution.PROPORTIONAL,
) {
    /** The tip as the generated code holds it. */
    internal val generated: eranos.internal.Tip
        get() = eranos.internal.Tip(value.generated, distribution.generated)

    /** Whether [other] is of the same class, with the same [value] and [distribution]. */
    override fun equals(other: Any?): Boolean = other is Tip && value == other.value && distribution == other.distribution

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(value, distribution)

    /** A description of every field, for logs. */
    override fun toString(): String = "Tip(value=$value, distribution=$distribution)"
}

/** The tip the generated code holds as [this]. */
internal fun eranos.internal.Tip.converted(): Tip = Tip(value.converted(), distribution.converted())

/** How a tip is expressed. */
public sealed interface TipValue {
    /** A fixed amount in the expense's currency. */
    public class Fixed(
        /** The amount. */
        public val amount: Money,
    ) : TipValue {
        /** Whether [other] is of the same class, with the same [amount]. */
        override fun equals(other: Any?): Boolean = other is Fixed && amount == other.amount

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = amount.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Fixed(amount=$amount)"
    }

    /** A percentage of the expense's amount. */
    public class Percentage(
        /** The percentage. */
        public val percentage: eranos.Percentage,
    ) : TipValue {
        /** Whether [other] is of the same class, with the same [percentage]. */
        override fun equals(other: Any?): Boolean = other is Percentage && percentage == other.percentage

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = percentage.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Percentage(percentage=$percentage)"
    }
}

/** The tip's value as the generated code holds it. */
internal val TipValue.generated: eranos.internal.TipValue
    get() =
        when (this) {
            is TipValue.Fixed -> eranos.internal.TipValue.Fixed(amount.generated)
            is TipValue.Percentage -> eranos.internal.TipValue.Percentage(percentage.basisPoints)
        }

/** The tip's value the generated code holds as [this]. */
internal fun eranos.internal.TipValue.converted(): TipValue =
    when (this) {
        is eranos.internal.TipValue.Fixed -> TipValue.Fixed(amount.converted())
        is eranos.internal.TipValue.Percentage -> TipValue.Percentage(Percentage(basisPoints))
    }

/** How a tip is shared among an expense's participants. */
public enum class TipDistribution {
    /** The same for every participant. */
    EQUAL,

    /** In proportion to each participant's share of the amount. */
    PROPORTIONAL,
}

/** The distribution as the generated code holds it. */
internal val TipDistribution.generated: eranos.internal.TipDistribution
    get() =
        when (this) {
            TipDistribution.EQUAL -> eranos.internal.TipDistribution.EQUAL
            TipDistribution.PROPORTIONAL -> eranos.internal.TipDistribution.PROPORTIONAL
        }

/** The distribution the generated code holds as [this]. */
internal fun eranos.internal.TipDistribution.converted(): TipDistribution =
    when (this) {
        eranos.internal.TipDistribution.EQUAL -> TipDistribution.EQUAL
        eranos.internal.TipDistribution.PROPORTIONAL -> TipDistribution.PROPORTIONAL
    }
