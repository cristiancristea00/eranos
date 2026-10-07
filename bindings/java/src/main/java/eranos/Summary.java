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
