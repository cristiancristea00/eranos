package eranos;

import java.util.Objects;

/**
 * The parts one member pays in proportion to.
 *
 * @param member the member
 * @param parts the member's parts
 */
public record MemberParts(MemberId member, Parts parts) {
    /**
     * The share of {@code member}, who pays in proportion to {@code parts}.
     *
     * @param member the member
     * @param parts the member's parts
     */
    public MemberParts {
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(parts, "parts");
    }

    /**
     * The share the generated code holds as {@code share}.
     *
     * @param share the generated share
     * @return the share
     */
    static MemberParts converted(eranos.internal.MemberParts share) {
        return new MemberParts(new MemberId(share.member()), Parts.known(share.hundredths()));
    }

    /**
     * The share as the generated code holds it.
     *
     * @return the generated share
     */
    eranos.internal.MemberParts generated() {
        return new eranos.internal.MemberParts(parts.hundredths(), member.value());
    }
}
