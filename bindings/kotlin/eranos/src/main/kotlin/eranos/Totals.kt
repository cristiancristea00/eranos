@file:Suppress("TooManyFunctions") // One conversion from the generated code per type of the totals.

package eranos

import eranos.internal.balanceFormat

/** The group's totals: overall, per member, per category, and per member per category, all in the group currency. */
public class Summary internal constructor(
    /** What the group spent. */
    public val totalSpent: Money,
    /** The totals of every member. */
    public val members: List<MemberSummary>,
    /** The total of every category. */
    public val categoryTotals: List<CategoryTotal>,
    /** The totals per category of every member. */
    public val memberCategoryTotals: List<MemberCategoryTotals>,
) {
    /**
     * Whether [other] is of the same class, with the same [totalSpent], [members], [categoryTotals], and
     * [memberCategoryTotals].
     */
    override fun equals(other: Any?): Boolean =
        other is Summary &&
            totalSpent == other.totalSpent &&
            members == other.members &&
            categoryTotals == other.categoryTotals &&
            memberCategoryTotals == other.memberCategoryTotals

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(totalSpent, members, categoryTotals, memberCategoryTotals)

    /** The totals of the member with the identifier [id]: all zero, and settled, for a member without entries. */
    public fun member(id: MemberId): MemberSummary {
        val zero = Money.zero(totalSpent.currency)
        return members.find { it.member == id } ?: MemberSummary(id, zero, zero, zero, zero, zero, Balance.Settled)
    }

    /** The spending in the category with the identifier [id]: zero for a category without expenses. */
    public fun categoryTotal(id: CategoryId): Money = categoryTotals.find { it.category == id }?.total ?: Money.zero(totalSpent.currency)

    /**
     * The share of the member with the identifier [member] of the spending in the category with the identifier [category]:
     * zero when they have none there.
     */
    public fun memberCategoryTotal(
        member: MemberId,
        category: CategoryId,
    ): Money {
        val totals = memberCategoryTotals.find { it.member == member }?.totals.orEmpty()
        return totals.find { it.category == category }?.total ?: Money.zero(totalSpent.currency)
    }

    /** A description of every field, for logs. */
    override fun toString(): String =
        "Summary(totalSpent=$totalSpent, members=$members, categoryTotals=$categoryTotals, " +
            "memberCategoryTotals=$memberCategoryTotals)"
}

/** The totals the generated code holds as [this]. */
internal fun eranos.internal.Summary.converted(): Summary =
    Summary(
        totalSpent.converted(),
        members.map { it.converted() },
        categoryTotals.map { it.converted() },
        memberCategoryTotals.map { it.converted() },
    )

/** A member's totals, in the group currency. */
@Suppress("LongParameterList") // A member's totals are the library's, one field each.
public class MemberSummary internal constructor(
    /** The member. */
    public val member: MemberId,
    /** What the member paid for expenses. */
    public val paid: Money,
    /** The member's share of the expenses: what they consumed. */
    public val share: Money,
    /** What the member sent in transfers. */
    public val sent: Money,
    /** What the member received in transfers. */
    public val received: Money,
    /** What the member is owed, less what they owe: positive when they get money back. */
    public val net: Money,
    /** Where the member stands. */
    public val balance: Balance,
) {
    /**
     * Whether [other] is of the same class, with the same [member], [paid], [share], [sent], [received], [net], and
     * [balance].
     */
    override fun equals(other: Any?): Boolean =
        other is MemberSummary &&
            member == other.member &&
            paid == other.paid &&
            share == other.share &&
            sent == other.sent &&
            received == other.received &&
            net == other.net &&
            balance == other.balance

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(member, paid, share, sent, received, net, balance)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "MemberSummary(member=$member, paid=$paid, share=$share, sent=$sent, received=$received, net=$net, balance=$balance)"
}

/** The member's totals the generated code holds as [this]. */
internal fun eranos.internal.MemberSummary.converted(): MemberSummary =
    MemberSummary(
        MemberId(member),
        paid.converted(),
        share.converted(),
        sent.converted(),
        received.converted(),
        net.converted(),
        balance.converted(),
    )

/** What the expenses of a category add up to, in the group currency. */
public class CategoryTotal internal constructor(
    /** The category. */
    public val category: CategoryId,
    /** The total. */
    public val total: Money,
) {
    /** Whether [other] is of the same class, with the same [category] and [total]. */
    override fun equals(other: Any?): Boolean = other is CategoryTotal && category == other.category && total == other.total

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(category, total)

    /** A description of every field, for logs. */
    override fun toString(): String = "CategoryTotal(category=$category, total=$total)"
}

/** The category's total the generated code holds as [this]. */
internal fun eranos.internal.CategoryTotal.converted(): CategoryTotal = CategoryTotal(CategoryId(category), total.converted())

/** What one member's shares add up to in each category. */
public class MemberCategoryTotals internal constructor(
    /** The member. */
    public val member: MemberId,
    /** The member's total in every category they have a share in. */
    public val totals: List<CategoryTotal>,
) {
    /** Whether [other] is of the same class, with the same [member] and [totals]. */
    override fun equals(other: Any?): Boolean = other is MemberCategoryTotals && member == other.member && totals == other.totals

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(member, totals)

    /** A description of every field, for logs. */
    override fun toString(): String = "MemberCategoryTotals(member=$member, totals=$totals)"
}

/** The member's category totals the generated code holds as [this]. */
internal fun eranos.internal.MemberCategoryTotals.converted(): MemberCategoryTotals =
    MemberCategoryTotals(MemberId(member), totals.map { it.converted() })

/** Where a member stands with the group; it prints as the library writes it, "owes 12.00 EUR". */
public sealed interface Balance {
    /** The member owes the others [amount]. */
    public class Owes(
        /** What the member owes. */
        public val amount: Money,
    ) : Balance {
        /** Whether [other] is of the same class, with the same [amount]. */
        override fun equals(other: Any?): Boolean = other is Owes && amount == other.amount

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = amount.hashCode()

        /** The balance as the library writes it: "owes 12.00 EUR". */
        override fun toString(): String = balanceFormat(generated)
    }

    /** The member owes nothing and is owed nothing. */
    public object Settled : Balance {
        /** The balance as the library writes it: "settled". */
        override fun toString(): String = balanceFormat(generated)
    }

    /** The others owe the member [amount]. */
    public class GetsBack(
        /** What the member gets back. */
        public val amount: Money,
    ) : Balance {
        /** Whether [other] is of the same class, with the same [amount]. */
        override fun equals(other: Any?): Boolean = other is GetsBack && amount == other.amount

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = amount.hashCode()

        /** The balance as the library writes it: "gets back 12.00 EUR". */
        override fun toString(): String = balanceFormat(generated)
    }
}

/** The balance as the generated code holds it. */
internal val Balance.generated: eranos.internal.Balance
    get() =
        when (this) {
            is Balance.Owes -> eranos.internal.Balance.Owes(amount.generated)
            Balance.Settled -> eranos.internal.Balance.Settled
            is Balance.GetsBack -> eranos.internal.Balance.GetsBack(amount.generated)
        }

/** The balance the generated code holds as [this]. */
internal fun eranos.internal.Balance.converted(): Balance =
    when (this) {
        is eranos.internal.Balance.Owes -> Balance.Owes(amount.converted())
        eranos.internal.Balance.Settled -> Balance.Settled
        is eranos.internal.Balance.GetsBack -> Balance.GetsBack(amount.converted())
    }

/** How one expense divides, in the group currency. */
public class ExpenseShares internal constructor(
    /** What every member paid and consumed. */
    public val members: List<MemberShare>,
    /** The total: the amount and the tip. */
    public val total: Money,
    /** The tip. */
    public val tip: Money,
) {
    /** Whether [other] is of the same class, with the same [members], [total], and [tip]. */
    override fun equals(other: Any?): Boolean = other is ExpenseShares && members == other.members && total == other.total && tip == other.tip

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(members, total, tip)

    /** A description of every field, for logs. */
    override fun toString(): String = "ExpenseShares(members=$members, total=$total, tip=$tip)"
}

/** The expense's shares the generated code holds as [this]. */
internal fun eranos.internal.ExpenseShares.converted(): ExpenseShares =
    ExpenseShares(members.map { it.converted() }, total.converted(), tip.converted())

/** What one member paid of an expense and what their share of it is. */
public class MemberShare internal constructor(
    /** The member. */
    public val member: MemberId,
    /** What the member paid, tip included. */
    public val paid: Money,
    /** The member's share, tip included. */
    public val share: Money,
) {
    /** Whether [other] is of the same class, with the same [member], [paid], and [share]. */
    override fun equals(other: Any?): Boolean = other is MemberShare && member == other.member && paid == other.paid && share == other.share

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(member, paid, share)

    /** A description of every field, for logs. */
    override fun toString(): String = "MemberShare(member=$member, paid=$paid, share=$share)"
}

/** The member's share the generated code holds as [this]. */
internal fun eranos.internal.MemberShare.converted(): MemberShare = MemberShare(MemberId(member), paid.converted(), share.converted())

/** The payments that settle every balance. */
public class SettlementPlan internal constructor(
    /** The payments, ordered by payer and then by recipient. */
    public val payments: List<Payment>,
    /** The strategy that produced them: the one asked for, or, for [SettlementStrategy.ADAPTIVE], the one whose plan it chose. */
    public val strategy: SettlementStrategy,
) {
    /** Whether [other] is of the same class, with the same [payments] and [strategy]. */
    override fun equals(other: Any?): Boolean = other is SettlementPlan && payments == other.payments && strategy == other.strategy

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(payments, strategy)

    /** A description of every field, for logs. */
    override fun toString(): String = "SettlementPlan(payments=$payments, strategy=$strategy)"
}

/** The settlement the generated code holds as [this]. */
internal fun eranos.internal.SettlementPlan.converted(): SettlementPlan = SettlementPlan(payments.map { it.converted() }, strategy.converted())

/** One payment of a settlement: one member pays another an amount. */
public class Payment internal constructor(
    /** The member who pays. */
    public val sender: MemberId,
    /** The member who is paid. */
    public val receiver: MemberId,
    /** The amount, in the group currency. */
    public val amount: Money,
) {
    /** Whether [other] is of the same class, with the same [sender], [receiver], and [amount]. */
    override fun equals(other: Any?): Boolean = other is Payment && sender == other.sender && receiver == other.receiver && amount == other.amount

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(sender, receiver, amount)

    /** A description of every field, for logs. */
    override fun toString(): String = "Payment(sender=$sender, receiver=$receiver, amount=$amount)"
}

/** The payment the generated code holds as [this]. */
internal fun eranos.internal.Payment.converted(): Payment = Payment(MemberId(sender), MemberId(receiver), amount.converted())

/** How balances turn into payments. */
public enum class SettlementStrategy {
    /**
     * The fewest payments while at most 20 balances are left after exact opposites pay each other, the greedy plan beyond;
     * the settlement names the strategy whose plan it is: [MINIMUM_TRANSFERS] or [GREEDY].
     */
    ADAPTIVE,

    /** The fewest payments possible. */
    MINIMUM_TRANSFERS,

    /** The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each other where that saves a payment. */
    GREEDY,

    /** Everyone pays back the people who paid for them. */
    DIRECT,
}

/** The strategy as the generated code holds it. */
internal val SettlementStrategy.generated: eranos.internal.SettlementStrategy
    get() =
        when (this) {
            SettlementStrategy.ADAPTIVE -> eranos.internal.SettlementStrategy.ADAPTIVE
            SettlementStrategy.MINIMUM_TRANSFERS -> eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS
            SettlementStrategy.GREEDY -> eranos.internal.SettlementStrategy.GREEDY
            SettlementStrategy.DIRECT -> eranos.internal.SettlementStrategy.DIRECT
        }

/** The strategy the generated code holds as [this]. */
internal fun eranos.internal.SettlementStrategy.converted(): SettlementStrategy =
    when (this) {
        eranos.internal.SettlementStrategy.ADAPTIVE -> SettlementStrategy.ADAPTIVE
        eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS -> SettlementStrategy.MINIMUM_TRANSFERS
        eranos.internal.SettlementStrategy.GREEDY -> SettlementStrategy.GREEDY
        eranos.internal.SettlementStrategy.DIRECT -> SettlementStrategy.DIRECT
    }
