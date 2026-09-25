package eranos;

import java.util.Objects;

/**
 * What one member paid of an expense and what their share of it is.
 *
 * @param member the member
 * @param paid what the member paid, tip included
 * @param share the member's share, tip included
 */
public record MemberShare(MemberId member, Money paid, Money share) {
    /**
     * The member share of these components.
     *
     * @param member the member
     * @param paid what the member paid, tip included
     * @param share the member's share, tip included
     */
    public MemberShare {
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(paid, "paid");
        Objects.requireNonNull(share, "share");
    }

    /**
     * The member share the generated code holds as {@code memberShare}.
     *
     * @param memberShare the generated value
     * @return the value
     */
    static MemberShare converted(eranos.internal.MemberShare memberShare) {
        return new MemberShare(
                new MemberId(memberShare.member()),
                Money.converted(memberShare.paid()),
                Money.converted(memberShare.share()));
    }
}
