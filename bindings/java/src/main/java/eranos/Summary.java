package eranos;

import java.util.List;
import java.util.Objects;

/**
 * The group's totals: overall, per member, per category, and per member per category, all in the
 * group currency.
 *
 * @param categoryTotals the total of every category
 * @param memberCategoryTotals the totals per category of every member
 * @param members the totals of every member
 * @param totalSpent what the group spent
 */
public record Summary(
        List<CategoryTotal> categoryTotals,
        List<MemberCategoryTotals> memberCategoryTotals,
        List<MemberSummary> members,
        Money totalSpent) {
    /**
     * The summary of these components; the lists are copied.
     *
     * @param categoryTotals the total of every category
     * @param memberCategoryTotals the totals per category of every member
     * @param members the totals of every member
     * @param totalSpent what the group spent
     */
    public Summary {
        categoryTotals = List.copyOf(categoryTotals);
        memberCategoryTotals = List.copyOf(memberCategoryTotals);
        members = List.copyOf(members);
        Objects.requireNonNull(totalSpent, "totalSpent");
    }

    /**
     * The summary the generated code holds as {@code summary}.
     *
     * @param summary the generated value
     * @return the value
     */
    static Summary converted(eranos.internal.Summary summary) {
        return new Summary(
                summary.categoryTotals().stream().map(CategoryTotal::converted).toList(),
                summary.memberCategoryTotals().stream()
                        .map(MemberCategoryTotals::converted)
                        .toList(),
                summary.members().stream().map(MemberSummary::converted).toList(),
                Money.converted(summary.totalSpent()));
    }
}
