package eranos;

import java.util.List;
import java.util.Objects;

/**
 * What one member's shares add up to in each category.
 *
 * @param member the member
 * @param totals the member's total in every category they have a share in
 */
public record MemberCategoryTotals(MemberId member, List<CategoryTotal> totals) {
    /**
     * The member category totals of these components; the lists are copied.
     *
     * @param member the member
     * @param totals the member's total in every category they have a share in
     */
    public MemberCategoryTotals {
        Objects.requireNonNull(member, "member");
        totals = List.copyOf(totals);
    }

    /**
     * The member category totals the generated code holds as {@code memberCategoryTotals}.
     *
     * @param memberCategoryTotals the generated value
     * @return the value
     */
    static MemberCategoryTotals converted(
            eranos.internal.MemberCategoryTotals memberCategoryTotals) {
        return new MemberCategoryTotals(
                new MemberId(memberCategoryTotals.member()),
                memberCategoryTotals.totals().stream().map(CategoryTotal::converted).toList());
    }
}
