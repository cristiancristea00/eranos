package eranos;

import java.util.Objects;

/**
 * A member's totals, in the group currency.
 *
 * @param member the member
 * @param paid what the member paid for expenses
 * @param share the member's share of the expenses: what they consumed
 * @param sent what the member sent in transfers
 * @param received what the member received in transfers
 * @param net what the member is owed, less what they owe: positive when they get money back
 * @param balance where the member stands
 */
public record MemberSummary(
        MemberId member,
        Money paid,
        Money share,
        Money sent,
        Money received,
        Money net,
        Balance balance) {
    /**
     * The member summary of these components.
     *
     * @param member the member
     * @param paid what the member paid for expenses
     * @param share the member's share of the expenses
     * @param sent what the member sent in transfers
     * @param received what the member received in transfers
     * @param net what the member is owed, less what they owe
     * @param balance where the member stands
     */
    public MemberSummary {
        Objects.requireNonNull(member, "member");
        Objects.requireNonNull(paid, "paid");
        Objects.requireNonNull(share, "share");
        Objects.requireNonNull(sent, "sent");
        Objects.requireNonNull(received, "received");
        Objects.requireNonNull(net, "net");
        Objects.requireNonNull(balance, "balance");
    }

    /**
     * The member summary the generated code holds as {@code memberSummary}.
     *
     * @param memberSummary the generated value
     * @return the value
     */
    static MemberSummary converted(eranos.internal.MemberSummary memberSummary) {
        return new MemberSummary(
                new MemberId(memberSummary.member()),
                Money.converted(memberSummary.paid()),
                Money.converted(memberSummary.share()),
                Money.converted(memberSummary.sent()),
                Money.converted(memberSummary.received()),
                Money.converted(memberSummary.net()),
                Conversions.converted(memberSummary.balance()));
    }
}
