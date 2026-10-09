package eranos

import eranos.internal.expenseItemDiscountAmount
import eranos.internal.expenseItemLineTotal
import eranos.internal.expenseItemLineTotalBeforeDiscount
import eranos.internal.expenseMembers
import eranos.internal.expenseSplitParticipants
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
    /** A note on it. */
    public val note: String? = null,
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
        note: String? = null,
    ) : this(ExpenseId.generate(), name, cost, payers, split, date, tip, time, category, note)

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
                note,
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
        note: String? = this.note,
    ): Expense = Expense(id, name, cost, payers, split, date, tip, time, category, note)

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
     * [category], and [note].
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
            note == other.note

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(id, name, cost, payers, split, date, tip, time, category, note)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "Expense(id=$id, name=$name, cost=$cost, payers=$payers, split=$split, date=$date, tip=$tip, time=$time, " +
            "category=$category, note=$note)"
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
        note,
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
    /**
     * Every member who shares the cost, once each, in identifier order.
     *
     * @throws EranosException.Id for a member whose identifier is not one, and the failures of an item that does not build.
     */
    @Throws(EranosException::class)
    public fun participants(): List<MemberId>

    /** The whole amount is divided by one split. */
    public class Whole(
        /** How the amount divides. */
        public val split: Split,
    ) : ExpenseSplit {
        /** Whether [other] is of the same class, with the same [split]. */
        override fun equals(other: Any?): Boolean = other is Whole && split == other.split

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = split.hashCode()

        /** The members of [split], once each, in identifier order. */
        override fun participants(): List<MemberId> = converting { expenseSplitParticipants(generated).map(::MemberId) }

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

        /** The members of every line's split, once each, in identifier order. */
        override fun participants(): List<MemberId> = converting { expenseSplitParticipants(generated).map(::MemberId) }

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

/** A line of an itemised bill: [quantity] units of [name] at [unitPrice], less any [discount], which [split] divides. */
public class ExpenseItem(
    /** What was bought. */
    public val name: String,
    /** The price of one unit, in the expense's currency. */
    public val unitPrice: Money,
    /** How the line's total divides. */
    public val split: Split,
    /** How many units, to a ten-thousandth: one, [Quantity.ONE], unless given. */
    public val quantity: Quantity = Quantity.ONE,
    /** The discount off the line or off each unit: none, for a line at full price, unless given. */
    public val discount: Discount? = null,
) {
    /** The line as the generated code holds it. */
    internal val generated: eranos.internal.ExpenseItem
        get() = eranos.internal.ExpenseItem(name, unitPrice.generated, split.generated, quantity.tenThousandths, discount?.generated)

    /**
     * The line total before the discount, as the library computes it: [unitPrice] times [quantity], rounded half away from
     * zero to the currency's minor unit, which is zero when the product is below half a minor unit.
     *
     * @throws EranosException as [lineTotal] does.
     */
    @Throws(EranosException::class)
    public fun lineTotalBeforeDiscount(): Money = converting { expenseItemLineTotalBeforeDiscount(generated).converted() }

    /**
     * What the [discount] takes off the line, as the library computes it: [lineTotalBeforeDiscount] minus [lineTotal], zero
     * without a discount.
     *
     * @throws EranosException as [lineTotal] does.
     */
    @Throws(EranosException::class)
    public fun discountAmount(): Money = converting { expenseItemDiscountAmount(generated).converted() }

    /**
     * The line total, as the library computes it: [unitPrice] times [quantity], rounded half away from zero to the currency's
     * minor unit, less the [discount], which is zero for a free line, whose product is below half a minor unit or whose
     * discount takes the whole price.
     *
     * @throws EranosException.Id for a member whose identifier is not one, [EranosException.Money] with
     *   [MoneyFailure.UnknownCurrency] for a fixed discount in a currency the library does not list, and
     *   [EranosException.Expense] with [ExpenseFailure.EmptyItemName] for a blank name, [ExpenseFailure.NonPositiveAmount]
     *   for a unit price that is not positive, [ExpenseFailure.Overflow] for a line total out of range, a discount the line
     *   cannot take ([ExpenseFailure.CurrencyMismatch] for a fixed discount in another currency than the unit price,
     *   [ExpenseFailure.NegativeDiscount], [ExpenseFailure.DiscountAboveHundredPercent], and
     *   [ExpenseFailure.DiscountAbovePrice]), and the failures of a split that does not divide the line total, among them
     *   [ExpenseFailure.AmountsDoNotAddUp] for a split of amounts on a free line.
     */
    @Throws(EranosException::class)
    public fun lineTotal(): Money = converting { expenseItemLineTotal(generated).converted() }

    /** Whether [other] is of the same class, with the same [name], [unitPrice], [split], [quantity], and [discount]. */
    override fun equals(other: Any?): Boolean =
        other is ExpenseItem &&
            name == other.name &&
            unitPrice == other.unitPrice &&
            split == other.split &&
            quantity == other.quantity &&
            discount == other.discount

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(name, unitPrice, split, quantity, discount)

    /** A description of every field, for logs. */
    override fun toString(): String = "ExpenseItem(name=$name, unitPrice=$unitPrice, split=$split, quantity=$quantity, discount=$discount)"
}

/** The line the generated code holds as [this]. */
internal fun eranos.internal.ExpenseItem.converted(): ExpenseItem =
    ExpenseItem(name, unitPrice.converted(), split.converted(), Quantity(quantity), discount?.converted())

/**
 * A discount off a line of an itemised bill: [value], taken off [scope].
 *
 * Off the line, a percentage is taken of the line total before the discount and rounded half away from zero to the minor
 * unit. Off each unit, a percentage of the unit price is rounded to the minor unit first, and the reduced price is then
 * multiplied by the quantity, so three units at 3.33 with 50 % off cost 4.98 with the discount off each unit and 4.99 with
 * it off the line. A discount of the whole price, 100 % or a fixed amount equal to what it reduces, makes a free line, whose
 * total is zero; a larger one is refused by every call that reads the item.
 */
public class Discount(
    /** How the discount is expressed. */
    public val value: DiscountValue,
    /** What the discount is taken off: the line as a whole, [DiscountScope.LINE], unless given. */
    public val scope: DiscountScope = DiscountScope.LINE,
) {
    /** The discount as the generated code holds it. */
    internal val generated: eranos.internal.Discount
        get() = eranos.internal.Discount(value.generated, scope.generated)

    /** Whether [other] is of the same class, with the same [value] and [scope]. */
    override fun equals(other: Any?): Boolean = other is Discount && value == other.value && scope == other.scope

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(value, scope)

    /** A description of every field, for logs. */
    override fun toString(): String = "Discount(value=$value, scope=$scope)"
}

/** The discount the generated code holds as [this]. */
internal fun eranos.internal.Discount.converted(): Discount = Discount(value.converted(), scope.converted())

/** How a discount is expressed. */
public sealed interface DiscountValue {
    /** A percentage of what the discount reduces, at most 100 %. */
    public class Percentage(
        /** The percentage. */
        public val percentage: eranos.Percentage,
    ) : DiscountValue {
        /** Whether [other] is of the same class, with the same [percentage]. */
        override fun equals(other: Any?): Boolean = other is Percentage && percentage == other.percentage

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = percentage.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Percentage(percentage=$percentage)"
    }

    /** A fixed amount in the item's currency, at most what the discount reduces. */
    public class Fixed(
        /** The amount. */
        public val amount: Money,
    ) : DiscountValue {
        /** Whether [other] is of the same class, with the same [amount]. */
        override fun equals(other: Any?): Boolean = other is Fixed && amount == other.amount

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = amount.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Fixed(amount=$amount)"
    }
}

/** The discount's value as the generated code holds it. */
internal val DiscountValue.generated: eranos.internal.DiscountValue
    get() =
        when (this) {
            is DiscountValue.Percentage -> eranos.internal.DiscountValue.Percentage(percentage.basisPoints)
            is DiscountValue.Fixed -> eranos.internal.DiscountValue.Fixed(amount.generated)
        }

/** The discount's value the generated code holds as [this]. */
internal fun eranos.internal.DiscountValue.converted(): DiscountValue =
    when (this) {
        is eranos.internal.DiscountValue.Percentage -> DiscountValue.Percentage(Percentage(basisPoints))
        is eranos.internal.DiscountValue.Fixed -> DiscountValue.Fixed(amount.converted())
    }

/** What a discount is taken off. */
public enum class DiscountScope {
    /** The line total, the unit price times the quantity: one discount for the whole line. */
    LINE,

    /** The price of each unit, before it is multiplied by the quantity. */
    UNIT,
}

/** The scope as the generated code holds it. */
internal val DiscountScope.generated: eranos.internal.DiscountScope
    get() =
        when (this) {
            DiscountScope.LINE -> eranos.internal.DiscountScope.LINE
            DiscountScope.UNIT -> eranos.internal.DiscountScope.UNIT
        }

/** The scope the generated code holds as [this]. */
internal fun eranos.internal.DiscountScope.converted(): DiscountScope =
    when (this) {
        eranos.internal.DiscountScope.LINE -> DiscountScope.LINE
        eranos.internal.DiscountScope.UNIT -> DiscountScope.UNIT
    }

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

    /** The same for every participant with a share of the amount above zero. */
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
