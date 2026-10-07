package eranos;

import java.util.Objects;

/**
 * The exact amount one member pays of a split.
 *
 * @param member the member
 * @param amount the amount
 */
public record MemberAmount(MemberId member, Money amount) {
    /**
     * The share of {@code member}, who pays {@code amount}.
     *
     * @param member the member
     * @param amount the amount
     */
    public MemberAmount {
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(amount, "amount");
    }

    /**
     * The share the generated code holds as {@code share}.
     *
     * @param share the generated share
     * @return the share
     */
    static MemberAmount converted(eranos.internal.MemberAmount share) {
        return new MemberAmount(new MemberId(share.member()), Money.converted(share.amount()));
    }

    /**
     * The share as the generated code holds it.
     *
     * @return the generated share
     */
    eranos.internal.MemberAmount generated() {
        return new eranos.internal.MemberAmount(member.value(), amount.generated());
    }
}
