package eranos

/**
 * How an amount divides among members: equally, by exact amounts, by parts, or by percentages.
 *
 * The library divides every amount so that the shares add up to it exactly, and gives any minor unit left over by the rounding
 * to a member it picks from the expense's identifier.
 */
public sealed interface Split {
    /** The members the split divides among, in the order it lists them. */
    public val members: List<MemberId>

    /** Every member pays an exact amount; the amounts add up to what they divide. */
    public class Amounts(
        shares: List<MemberAmount>,
    ) : Split {
        /** What every member pays. */
        public val shares: List<MemberAmount> = shares.toList()

        override val members: List<MemberId>
            get() = shares.map { it.member }

        /** Whether [other] is of the same class, with the same [shares]. */
        override fun equals(other: Any?): Boolean = other is Amounts && shares == other.shares

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = shares.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Amounts(shares=$shares)"
    }

    /** Every member pays the same. */
    public class Equal(
        members: List<MemberId>,
    ) : Split {
        override val members: List<MemberId> = members.toList()

        /** Whether [other] is of the same class, with the same [members]. */
        override fun equals(other: Any?): Boolean = other is Equal && members == other.members

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = members.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Equal(members=$members)"
    }

    /** Every member pays in proportion to their parts. */
    public class Parts(
        shares: List<MemberParts>,
    ) : Split {
        /** The parts of every member. */
        public val shares: List<MemberParts> = shares.toList()

        override val members: List<MemberId>
            get() = shares.map { it.member }

        /** Whether [other] is of the same class, with the same [shares]. */
        override fun equals(other: Any?): Boolean = other is Parts && shares == other.shares

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = shares.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Parts(shares=$shares)"
    }

    /** Every member pays a percentage; the percentages add up to 100 %. */
    public class Percentages(
        shares: List<MemberPercentage>,
    ) : Split {
        /** The percentage of every member. */
        public val shares: List<MemberPercentage> = shares.toList()

        override val members: List<MemberId>
            get() = shares.map { it.member }

        /** Whether [other] is of the same class, with the same [shares]. */
        override fun equals(other: Any?): Boolean = other is Percentages && shares == other.shares

        /** A hash of what [equals] compares. */
        override fun hashCode(): Int = shares.hashCode()

        /** A description of every field, for logs. */
        override fun toString(): String = "Percentages(shares=$shares)"
    }
}

/** The split as the generated code holds it. */
internal val Split.generated: eranos.internal.Split
    get() =
        when (this) {
            is Split.Amounts -> eranos.internal.Split.Amounts(shares.map { it.generated })
            is Split.Equal -> eranos.internal.Split.Equal(members.map { it.value })
            is Split.Parts -> eranos.internal.Split.Parts(shares.map { it.generated })
            is Split.Percentages -> eranos.internal.Split.Percentages(shares.map { it.generated })
        }

/** The split the generated code holds as [this]. */
internal fun eranos.internal.Split.converted(): Split =
    when (this) {
        is eranos.internal.Split.Amounts -> Split.Amounts(shares.map { it.converted() })
        is eranos.internal.Split.Equal -> Split.Equal(members.map(::MemberId))
        is eranos.internal.Split.Parts -> Split.Parts(shares.map { it.converted() })
        is eranos.internal.Split.Percentages -> Split.Percentages(shares.map { it.converted() })
    }

/** The exact amount one member pays of a split. */
public class MemberAmount(
    /** The amount. */
    public val amount: Money,
    /** The member. */
    public val member: MemberId,
) {
    /** The share as the generated code holds it. */
    internal val generated: eranos.internal.MemberAmount
        get() = eranos.internal.MemberAmount(amount.generated, member.value)

    /** Whether [other] is of the same class, with the same [amount] and [member]. */
    override fun equals(other: Any?): Boolean = other is MemberAmount && amount == other.amount && member == other.member

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(amount, member)

    /** A description of every field, for logs. */
    override fun toString(): String = "MemberAmount(amount=$amount, member=$member)"
}

/** The share the generated code holds as [this]. */
internal fun eranos.internal.MemberAmount.converted(): MemberAmount = MemberAmount(amount.converted(), MemberId(member))

/** The parts one member pays in proportion to. */
public class MemberParts(
    /** The member. */
    public val member: MemberId,
    /** The member's parts. */
    public val parts: Parts,
) {
    /** The share as the generated code holds it. */
    internal val generated: eranos.internal.MemberParts
        get() = eranos.internal.MemberParts(parts.hundredths, member.value)

    /** Whether [other] is of the same class, with the same [member] and [parts]. */
    override fun equals(other: Any?): Boolean = other is MemberParts && member == other.member && parts == other.parts

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(member, parts)

    /** A description of every field, for logs. */
    override fun toString(): String = "MemberParts(member=$member, parts=$parts)"
}

/** The share the generated code holds as [this]. */
internal fun eranos.internal.MemberParts.converted(): MemberParts = MemberParts(MemberId(member), Parts(hundredths))

/** The percentage one member pays. */
public class MemberPercentage(
    /** The member. */
    public val member: MemberId,
    /** The member's percentage. */
    public val percentage: Percentage,
) {
    /** The share as the generated code holds it. */
    internal val generated: eranos.internal.MemberPercentage
        get() = eranos.internal.MemberPercentage(percentage.basisPoints, member.value)

    /** Whether [other] is of the same class, with the same [member] and [percentage]. */
    override fun equals(other: Any?): Boolean = other is MemberPercentage && member == other.member && percentage == other.percentage

    /** A hash of what [equals] compares. */
    override fun hashCode(): Int = hashOf(member, percentage)

    /** A description of every field, for logs. */
    override fun toString(): String = "MemberPercentage(member=$member, percentage=$percentage)"
}

/** The share the generated code holds as [this]. */
internal fun eranos.internal.MemberPercentage.converted(): MemberPercentage = MemberPercentage(MemberId(member), Percentage(basisPoints))
