package eranos

import eranos.internal.expenseItemLineTotal
import eranos.internal.expenseMembers
import eranos.internal.expenseTipAmount
import eranos.internal.expenseTotal
import eranos.internal.tipRuleNone

/**
 * Something bought for the group: what it cost, who paid, and how it divides.
 *
 * An expense has its identifier from the moment it is made: the app's own, such as the key of its own store, or a generated
 * one. To change an expense, read it from the group, [copy] it with the changes, and pass the copy to [ExpenseGroup.replace].
 *
 * @constructor An expense of [cost] on [date], that [payers] paid and [split] shares, under the identifier [id], with no tip
 *   and in the built-in category unless given others; the constructor without [id] generates one.
 */
@Suppress("LongParameterList") // An expense has the library's fields; the public constructors default the optional ones.
public class Expense(
    /** The identifier, which the app gives: its own, or one [ExpenseId.generate] makes. */
    public val id: ExpenseId,
    /** What was bought. */
    public val name: String,
    /** What it cost. */
    public val cost: Cost,
    /** Who paid the total, amount and tip, and how much each. */
    public val payers: Split,
    /** How the amount is shared. */
    public val split: ExpenseSplit,
    /** The day it was bought. */
    public val date: CalendarDate,
    /** The tip on top of the amount: [TipRule.NONE] for an expense without one. */
    public val tip: TipRule = TipRule.NONE,
    /** The time of day it was bought. */
    public val time: TimeOfDay? = null,
    /** The category: the built-in one, [CategoryId.UNCATEGORISED], until the expense is given another. */
    public val category: CategoryId = CategoryId.UNCATEGORISED,
    /** A longer description. */
    public val description: String? = null,
) {
    /**
     * An expense of [cost] on [date], that [payers] paid and [split] shares, under an identifier [ExpenseId.generate] makes,
     * with no tip and in the built-in category unless given others; the constructor with [id] takes the app's.
     */
    public constructor(
        name: String,
        cost: Cost,
        payers: Split,
        split: ExpenseSplit,
        date: CalendarDate,
        tip: TipRule = TipRule.NONE,
        time: TimeOfDay? = null,
        category: CategoryId = CategoryId.UNCATEGORISED,
        description: String? = null,
    ) : this(ExpenseId.generate(), name, cost, payers, split, date, tip, time, category, description)

    /** The expense as the generated code holds it. */
    internal val generated: eranos.internal.Expense
        get() =
            eranos.internal.Expense(
                id.value,
                name,
                cost.generated,
                payers.generated,
                split.generated,
                date.generated,
                tip.generated,
                time?.generated,
                category.value,
                description,
            )

    /** This expense with the given fields changed; the identifier stays, so the copy can replace the expense. */
    public fun copy(
        name: String = this.name,
        cost: Cost = this.cost,
        payers: Split = this.payers,
        split: ExpenseSplit = this.split,
        date: CalendarDate = this.date,
        tip: TipRule = this.tip,
        time: TimeOfDay? = this.time,
        category: CategoryId = this.category,
        description: String? = this.description,
    ): Expense = Expense(id, name, cost, payers, split, date, tip, time, category, description)

    /**
     * The total, amount and tip, in the expense's own currency: what its payers paid.
     *
     * @throws EranosException.Expense as [tipAmount] does, and [EranosException.Money] with [MoneyFailure.Overflow] for a
     *   total out of range.
     */
    @Throws(EranosException::class)
    public fun total(): Money = converting { expenseTotal(generated).converted() }

    /**
     * The tip, in the expense's own currency.
     *
     * @throws EranosException.Expense with [ExpenseFailure.CurrencyMismatch] for a fixed tip in another currency, and
     *   [ExpenseFailure.NegativeTip] for a fixed tip below zero.
     */
    @Throws(EranosException::class)
    public fun tipAmount(): Money = converting { expenseTipAmount(generated).converted() }

    /**
     * Every member the expense involves, as a payer or a participant, in identifier order.
     *
     * @throws EranosException.Id for a member whose identifier is not one, and the failures of an item that does not build.
     */
    @Throws(EranosException::class)
    public fun members(): List<MemberId> = converting { expenseMembers(generated).map(::MemberId) }

    /**
     * Whether [other] is of the same class, with the same [id], [name], [cost], [payers], [split], [date], [tip], [time],
     * [category], and [description].
     */
    override fun equals(other: Any?): Boolean =
        other is Expense &&
            id == other.id &&
            name == other.name &&
            cost == other.cost &&
            payers == other.payers &&
            split == other.split &&
            date == other.date &&
            tip == other.tip &&
            time == other.time &&
            category == other.category &&
            description == other.description

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, name, cost, payers, split, date, tip, time, category, description)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "Expense(id=$id, name=$name, cost=$cost, payers=$payers, split=$split, date=$date, tip=$tip, time=$time, " +
            "category=$category, description=$description)"
}

/** The expense the generated code holds as [this]. */
internal fun eranos.internal.Expense.converted(): Expense =
    Expense(
        ExpenseId(id),
        name,
        cost.converted(),
        payers.converted(),
        split.converted(),
        date.converted(),
        tip.converted(),
        time?.converted(),
        CategoryId(category),
        description,
    )

/**
 * What an entry cost: an amount in the group currency, or one in another currency with the rate that converts it into the
 * group currency.
 */
public sealed interface Cost {
    /** The amount, in its own currency. */
    public val amount: Money

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
}

/** The cost as the generated code holds it. */
internal val Cost.generated: eranos.internal.Cost
    get() =
        when (this) {
            is Cost.Local -> eranos.internal.Cost.Local(amount.generated)
            is Cost.Converted -> eranos.internal.Cost.Converted(amount.generated, rate.toString())
        }

/** The cost the generated code holds as [this]. */
internal fun eranos.internal.Cost.converted(): Cost =
    when (this) {
        is eranos.internal.Cost.Local -> Cost.Local(amount.converted())
        is eranos.internal.Cost.Converted -> Cost.Converted(amount.converted(), ExchangeRate(rate))
    }

/** How an expense's cost is shared: as a whole, or line by line of an itemised bill. */
public sealed interface ExpenseSplit {
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

    /** Every line is divided by its own split; the lines add up to the amount. */
    public class Itemised(
        items: List<ExpenseItem>,
    ) : ExpenseSplit {
        /** The lines of the bill. */
        public val items: List<ExpenseItem> = items.toList()

        /** Whether [other] is of the same class, with the same [items]. */
        override fun equals(other: Any?): Boolean = other is Itemised && items == other.items

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = items.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Itemised(items=$items)"
    }
}

/** The expense split as the generated code holds it. */
internal val ExpenseSplit.generated: eranos.internal.ExpenseSplit
    get() =
        when (this) {
            is ExpenseSplit.Whole -> eranos.internal.ExpenseSplit.Whole(split.generated)
            is ExpenseSplit.Itemised -> eranos.internal.ExpenseSplit.Itemised(items.map { it.generated })
        }

/** The expense split the generated code holds as [this]. */
internal fun eranos.internal.ExpenseSplit.converted(): ExpenseSplit =
    when (this) {
        is eranos.internal.ExpenseSplit.Whole -> ExpenseSplit.Whole(split.converted())
        is eranos.internal.ExpenseSplit.Itemised -> ExpenseSplit.Itemised(items.map { it.converted() })
    }

/** A line of an itemised bill: [quantity] units of [name] at [unitPrice], which [split] divides. */
public class ExpenseItem(
    /** What was bought. */
    public val name: String,
    /** The price of one unit, in the expense's currency. */
    public val unitPrice: Money,
    /** How the line's total divides. */
    public val split: Split,
    /** How many units, to a ten-thousandth: one, [Quantity.ONE], unless given. */
    public val quantity: Quantity = Quantity.ONE,
) {
    /** The line as the generated code holds it. */
    internal val generated: eranos.internal.ExpenseItem
        get() = eranos.internal.ExpenseItem(name, unitPrice.generated, split.generated, quantity.tenThousandths)

    /**
     * The line total, as the library computes it: [unitPrice] times [quantity], rounded half away from zero to the currency's
     * minor unit, which is zero for a free line, whose product is below half a minor unit.
     *
     * @throws EranosException.Id for a member whose identifier is not one, and [EranosException.Expense] with
     *   [ExpenseFailure.EmptyItemName] for a blank name, [ExpenseFailure.NonPositiveAmount] for a unit price that is not
     *   positive, [ExpenseFailure.Overflow] for a line total out of range, and the failures of a split that does not divide
     *   the line total, among them [ExpenseFailure.AmountsDoNotAddUp] for a split of amounts on a free line.
     */
    @Throws(EranosException::class)
    public fun lineTotal(): Money = converting { expenseItemLineTotal(generated).converted() }

    /** Whether [other] is of the same class, with the same [name], [unitPrice], [split], and [quantity]. */
    override fun equals(other: Any?): Boolean =
        other is ExpenseItem && name == other.name && unitPrice == other.unitPrice && split == other.split && quantity == other.quantity

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(name, unitPrice, split, quantity)

    /** A description of every field, for logs. */
    override fun toString(): String = "ExpenseItem(name=$name, unitPrice=$unitPrice, split=$split, quantity=$quantity)"
}

/** The line the generated code holds as [this]. */
internal fun eranos.internal.ExpenseItem.converted(): ExpenseItem = ExpenseItem(name, unitPrice.converted(), split.converted(), Quantity(quantity))

/** A tip on top of an expense's amount, which its payers cover: [value], shared by [distribution]; [NONE] is no tip. */
public class TipRule(
    /** How the tip is expressed. */
    public val value: TipValue,
    /** How the tip is shared among the participants: in proportion to each share unless given. */
    public val distribution: TipDistribution = TipDistribution.PROPORTIONAL,
) {
    /** The tip as the generated code holds it. */
    internal val generated: eranos.internal.TipRule
        get() = eranos.internal.TipRule(value.generated, distribution.generated)

    /** Whether [other] is of the same class, with the same [value] and [distribution]. */
    override fun equals(other: Any?): Boolean = other is TipRule && value == other.value && distribution == other.distribution

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(value, distribution)

    /** A description of every field, for logs. */
    override fun toString(): String = "TipRule(value=$value, distribution=$distribution)"

    /** The no-tip rule. */
    public companion object {
        /** No tip: 0 % of the amount, shared proportionally, which an expense has until it is given another tip. */
        public val NONE: TipRule = tipRuleNone().converted()
    }
}

/** The tip the generated code holds as [this]. */
internal fun eranos.internal.TipRule.converted(): TipRule = TipRule(value.converted(), distribution.converted())

/** How a tip is expressed. */
public sealed interface TipValue {
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
}

/** The tip's value as the generated code holds it. */
internal val TipValue.generated: eranos.internal.TipValue
    get() =
        when (this) {
            is TipValue.Percentage -> eranos.internal.TipValue.Percentage(percentage.basisPoints)
            is TipValue.Fixed -> eranos.internal.TipValue.Fixed(amount.generated)
        }

/** The tip's value the generated code holds as [this]. */
internal fun eranos.internal.TipValue.converted(): TipValue =
    when (this) {
        is eranos.internal.TipValue.Percentage -> TipValue.Percentage(Percentage(basisPoints))
        is eranos.internal.TipValue.Fixed -> TipValue.Fixed(amount.converted())
    }

/** How a tip is shared among an expense's participants. */
public enum class TipDistribution {
    /** In proportion to each participant's share of the amount. */
    PROPORTIONAL,

    /** The same for every participant. */
    EQUAL,
}

/** The distribution as the generated code holds it. */
internal val TipDistribution.generated: eranos.internal.TipDistribution
    get() =
        when (this) {
            TipDistribution.PROPORTIONAL -> eranos.internal.TipDistribution.PROPORTIONAL
            TipDistribution.EQUAL -> eranos.internal.TipDistribution.EQUAL
        }

/** The distribution the generated code holds as [this]. */
internal fun eranos.internal.TipDistribution.converted(): TipDistribution =
    when (this) {
        eranos.internal.TipDistribution.PROPORTIONAL -> TipDistribution.PROPORTIONAL
        eranos.internal.TipDistribution.EQUAL -> TipDistribution.EQUAL
    }
