package eranos;

import java.util.Objects;

/**
 * A member's totals, in the group currency.
 *
 * @param balance where the member stands
 * @param member the member
 * @param net what the member is owed, less what they owe: positive when they get money back
 * @param paid what the member paid for expenses
 * @param received what the member received in transfers
 * @param sent what the member sent in transfers
 * @param share the member's share of the expenses: what they consumed
 */
public record MemberSummary(
        Balance balance,
        MemberId member,
        Money net,
        Money paid,
        Money received,
        Money sent,
        Money share) {
    /**
     * The member summary of these components.
     *
     * @param balance where the member stands
     * @param member the member
     * @param net what the member is owed, less what they owe
     * @param paid what the member paid for expenses
     * @param received what the member received in transfers
     * @param sent what the member sent in transfers
     * @param share the member's share of the expenses
     */
    public MemberSummary {
        Objects.requireNonNull(balance, "balance");
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(net, "net");
        Objects.requireNonNull(paid, "paid");
        Objects.requireNonNull(received, "received");
        Objects.requireNonNull(sent, "sent");
        Objects.requireNonNull(share, "share");
    }

    /**
     * The member summary the generated code holds as {@code memberSummary}.
     *
     * @param memberSummary the generated value
     * @return the value
     */
    static MemberSummary converted(eranos.internal.MemberSummary memberSummary) {
        return new MemberSummary(
                Conversions.converted(memberSummary.balance()),
                new MemberId(memberSummary.member()),
                Money.converted(memberSummary.net()),
                Money.converted(memberSummary.paid()),
                Money.converted(memberSummary.received()),
                Money.converted(memberSummary.sent()),
                Money.converted(memberSummary.share()));
    }
}
