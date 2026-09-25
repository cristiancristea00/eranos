package eranos;

import java.util.Objects;

/**
 * The exact amount one member pays of a split.
 *
 * @param amount the amount
 * @param member the member
 */
public record MemberAmount(Money amount, MemberId member) {
    /**
     * The share of {@code member}, who pays {@code amount}.
     *
     * @param amount the amount
     * @param member the member
     */
    public MemberAmount {
        Objects.requireNonNull(amount, "amount");
        Objects.requireNonNull(member, "member");
    }

    /**
     * The share the generated code holds as {@code share}.
     *
     * @param share the generated share
     * @return the share
     */
    static MemberAmount converted(eranos.internal.MemberAmount share) {
        return new MemberAmount(Money.converted(share.amount()), new MemberId(share.member()));
    }

    /**
     * The share as the generated code holds it.
     *
     * @return the generated share
     */
    eranos.internal.MemberAmount generated() {
        return new eranos.internal.MemberAmount(amount.generated(), member.value());
    }
}
