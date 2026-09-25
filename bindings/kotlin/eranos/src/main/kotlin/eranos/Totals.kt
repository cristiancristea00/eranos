@file:Suppress("TooManyFunctions") // One conversion from the generated code per type of the totals.

package eranos

import eranos.internal.balanceFormat

/** The group's totals: overall, per member, per category, and per member per category, all in the group currency. */
public class Summary internal constructor(
    /** The total of every category. */
    public val categoryTotals: List<CategoryTotal>,
    /** The totals per category of every member. */
    public val memberCategoryTotals: List<MemberCategoryTotals>,
    /** The totals of every member. */
    public val members: List<MemberSummary>,
    /** What the group spent. */
    public val totalSpent: Money,
) {
    /**
     * Whether [other] is of the same class, with the same [categoryTotals], [memberCategoryTotals], [members], and
     * [totalSpent].
     */
    override fun equals(other: Any?): Boolean =
        other is Summary &&
            categoryTotals == other.categoryTotals &&
            memberCategoryTotals == other.memberCategoryTotals &&
            members == other.members &&
            totalSpent == other.totalSpent

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(categoryTotals, memberCategoryTotals, members, totalSpent)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "Summary(categoryTotals=$categoryTotals, memberCategoryTotals=$memberCategoryTotals, members=$members, " +
            "totalSpent=$totalSpent)"
}

/** The totals the generated code holds as [this]. */
internal fun eranos.internal.Summary.converted(): Summary =
    Summary(
        categoryTotals.map { it.converted() },
        memberCategoryTotals.map { it.converted() },
        members.map { it.converted() },
        totalSpent.converted(),
    )

/** A member's totals, in the group currency. */
@Suppress("LongParameterList") // A member's totals are the library's, one field each.
public class MemberSummary internal constructor(
    /** Where the member stands. */
    public val balance: Balance,
    /** The member. */
    public val member: MemberId,
    /** What the member is owed, less what they owe: positive when they get money back. */
    public val net: Money,
    /** What the member paid for expenses. */
    public val paid: Money,
    /** What the member received in transfers. */
    public val received: Money,
    /** What the member sent in transfers. */
    public val sent: Money,
    /** The member's share of the expenses: what they consumed. */
    public val share: Money,
) {
    /**
     * Whether [other] is of the same class, with the same [balance], [member], [net], [paid], [received], [sent], and
     * [share].
     */
    override fun equals(other: Any?): Boolean =
        other is MemberSummary &&
            balance == other.balance &&
            member == other.member &&
            net == other.net &&
            paid == other.paid &&
            received == other.received &&
            sent == other.sent &&
            share == other.share

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(balance, member, net, paid, received, sent, share)

    /** A description of every field, for logs. */
    override fun toString(): String =
        "MemberSummary(balance=$balance, member=$member, net=$net, paid=$paid, received=$received, sent=$sent, share=$share)"
}

/** The member's totals the generated code holds as [this]. */
internal fun eranos.internal.MemberSummary.converted(): MemberSummary =
    MemberSummary(
        balance.converted(),
        MemberId(member),
        net.converted(),
        paid.converted(),
        received.converted(),
        sent.converted(),
        share.converted(),
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
        override fun toString(): String = formatted()
    }

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
        override fun toString(): String = formatted()
    }

    /** The member owes nothing and is owed nothing. */
    public object Settled : Balance {
        /** The balance as the library writes it: "settled". */
        override fun toString(): String = formatted()
    }
}

/** The balance as the generated code holds it. */
internal val Balance.generated: eranos.internal.Balance
    get() =
        when (this) {
            is Balance.GetsBack -> eranos.internal.Balance.GetsBack(amount.generated)
            is Balance.Owes -> eranos.internal.Balance.Owes(amount.generated)
            Balance.Settled -> eranos.internal.Balance.Settled
        }

/** The balance as the library writes it; its description by the generated code if the library cannot format its amount. */
internal fun Balance.formatted(): String =
    try {
        balanceFormat(generated)
    } catch (_: eranos.internal.EranosException) {
        generated.toString()
    }

/** The balance the generated code holds as [this]. */
internal fun eranos.internal.Balance.converted(): Balance =
    when (this) {
        is eranos.internal.Balance.GetsBack -> Balance.GetsBack(amount.converted())
        is eranos.internal.Balance.Owes -> Balance.Owes(amount.converted())
        eranos.internal.Balance.Settled -> Balance.Settled
    }

/** How one expense divides, in the group currency. */
public class ExpenseShares internal constructor(
    /** What every member paid and consumed. */
    public val members: List<MemberShare>,
    /** The tip. */
    public val tip: Money,
    /** The total: the amount and the tip. */
    public val total: Money,
) {
    /** Whether [other] is of the same class, with the same [members], [tip], and [total]. */
    override fun equals(other: Any?): Boolean = other is ExpenseShares && members == other.members && tip == other.tip && total == other.total

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(members, tip, total)

    /** A description of every field, for logs. */
    override fun toString(): String = "ExpenseShares(members=$members, tip=$tip, total=$total)"
}

/** The expense's shares the generated code holds as [this]. */
internal fun eranos.internal.ExpenseShares.converted(): ExpenseShares =
    ExpenseShares(members.map { it.converted() }, tip.converted(), total.converted())

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
public class Settlement internal constructor(
    /** The payments, ordered by payer and then by recipient. */
    public val payments: List<Payment>,
    /** The strategy that produced them. */
    public val strategy: SettlementStrategy,
) {
    /** Whether [other] is of the same class, with the same [payments] and [strategy]. */
    override fun equals(other: Any?): Boolean = other is Settlement && payments == other.payments && strategy == other.strategy

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(payments, strategy)

    /** A description of every field, for logs. */
    override fun toString(): String = "Settlement(payments=$payments, strategy=$strategy)"
}

/** The settlement the generated code holds as [this]. */
internal fun eranos.internal.Settlement.converted(): Settlement = Settlement(payments.map { it.converted() }, strategy.converted())

/** One payment of a settlement: one member pays another an amount. */
public class Payment internal constructor(
    /** The amount, in the group currency. */
    public val amount: Money,
    /** The member who pays. */
    public val from: MemberId,
    /** The member who is paid. */
    public val to: MemberId,
) {
    /** Whether [other] is of the same class, with the same [amount], [from], and [to]. */
    override fun equals(other: Any?): Boolean = other is Payment && amount == other.amount && from == other.from && to == other.to

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(amount, from, to)

    /** A description of every field, for logs. */
    override fun toString(): String = "Payment(amount=$amount, from=$from, to=$to)"
}

/** The payment the generated code holds as [this]. */
internal fun eranos.internal.Payment.converted(): Payment = Payment(amount.converted(), MemberId(from), MemberId(to))

/** How balances turn into payments. */
public enum class SettlementStrategy {
    /** Everyone pays back the people who paid for them. */
    DIRECT,

    /** The largest debtor pays the largest creditor, repeatedly. */
    GREEDY,

    /** The fewest payments possible. */
    MINIMUM_TRANSFERS,
}

/** The strategy as the generated code holds it. */
internal val SettlementStrategy.generated: eranos.internal.SettlementStrategy
    get() =
        when (this) {
            SettlementStrategy.DIRECT -> eranos.internal.SettlementStrategy.DIRECT
            SettlementStrategy.GREEDY -> eranos.internal.SettlementStrategy.GREEDY
            SettlementStrategy.MINIMUM_TRANSFERS -> eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS
        }

/** The strategy the generated code holds as [this]. */
internal fun eranos.internal.SettlementStrategy.converted(): SettlementStrategy =
    when (this) {
        eranos.internal.SettlementStrategy.DIRECT -> SettlementStrategy.DIRECT
        eranos.internal.SettlementStrategy.GREEDY -> SettlementStrategy.GREEDY
        eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS -> SettlementStrategy.MINIMUM_TRANSFERS
    }
