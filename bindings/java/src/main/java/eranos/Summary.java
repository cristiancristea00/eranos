package eranos;

import java.util.List;
import java.util.Objects;

/**
 * The group's totals: overall, per member, per category, and per member per category, all in the
 * group currency.
 *
 * @param totalSpent what the group spent
 * @param members the totals of every member
 * @param categoryTotals the total of every category
 * @param memberCategoryTotals the totals per category of every member
 */
public record Summary(
        Money totalSpent,
        List<MemberSummary> members,
        List<CategoryTotal> categoryTotals,
        List<MemberCategoryTotals> memberCategoryTotals) {
    /**
     * The summary of these components; the lists are copied.
     *
     * @param totalSpent what the group spent
     * @param members the totals of every member
     * @param categoryTotals the total of every category
     * @param memberCategoryTotals the totals per category of every member
     */
    public Summary {
        Objects.requireNonNull(totalSpent, "totalSpent");
        members = List.copyOf(members);
        categoryTotals = List.copyOf(categoryTotals);
        memberCategoryTotals = List.copyOf(memberCategoryTotals);
    }

    /**
     * The totals of the member with the identifier {@code id}: all zero, and settled, for a member
     * without entries.
     *
     * @param id the member's identifier
     * @return the member's totals
     */
    public MemberSummary member(MemberId id) {
        Money zero = Money.zero(totalSpent.currency());
        return members.stream()
                .filter(totals -> totals.member().equals(id))
                .findFirst()
                .orElseGet(
                        () ->
                                new MemberSummary(
                                        id, zero, zero, zero, zero, zero, new Balance.Settled()));
    }

    /**
     * The spending in the category with the identifier {@code id}: zero for a category without
     * expenses.
     *
     * @param id the category's identifier
     * @return the spending
     */
    public Money categoryTotal(CategoryId id) {
        return categoryTotals.stream()
                .filter(total -> total.category().equals(id))
                .map(CategoryTotal::total)
                .findFirst()
                .orElse(Money.zero(totalSpent.currency()));
    }

    /**
     * The share of the member with the identifier {@code member} of the spending in the category
     * with the identifier {@code category}: zero when they have none there.
     *
     * @param member the member's identifier
     * @param category the category's identifier
     * @return the member's share of the spending
     */
    public Money memberCategoryTotal(MemberId member, CategoryId category) {
        return memberCategoryTotals.stream()
                .filter(totals -> totals.member().equals(member))
                .flatMap(totals -> totals.totals().stream())
                .filter(total -> total.category().equals(category))
                .map(CategoryTotal::total)
                .findFirst()
                .orElse(Money.zero(totalSpent.currency()));
    }

    /**
     * The summary the generated code holds as {@code summary}.
     *
     * @param summary the generated value
     * @return the value
     */
    static Summary converted(eranos.internal.Summary summary) {
        return new Summary(
                Money.converted(summary.totalSpent()),
                summary.members().stream().map(MemberSummary::converted).toList(),
                summary.categoryTotals().stream().map(CategoryTotal::converted).toList(),
                summary.memberCategoryTotals().stream()
                        .map(MemberCategoryTotals::converted)
                        .toList());
    }
}
