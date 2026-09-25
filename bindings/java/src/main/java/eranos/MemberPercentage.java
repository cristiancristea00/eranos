package eranos;

import java.util.Objects;

/**
 * The percentage one member pays.
 *
 * @param member the member
 * @param percentage the member's percentage
 */
public record MemberPercentage(MemberId member, Percentage percentage) {
    /**
     * The share of {@code member}, who pays {@code percentage}.
     *
     * @param member the member
     * @param percentage the member's percentage
     */
    public MemberPercentage {
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(percentage, "percentage");
    }

    /**
     * The share the generated code holds as {@code share}.
     *
     * @param share the generated share
     * @return the share
     */
    static MemberPercentage converted(eranos.internal.MemberPercentage share) {
        return new MemberPercentage(
                new MemberId(share.member()), Percentage.known(share.basisPoints()));
    }

    /**
     * The share as the generated code holds it.
     *
     * @return the generated share
     */
    eranos.internal.MemberPercentage generated() {
        return new eranos.internal.MemberPercentage(percentage.basisPoints(), member.value());
    }
}
