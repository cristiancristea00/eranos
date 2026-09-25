package eranos;

import java.util.List;
import java.util.Objects;

/**
 * How one expense divides, in the group currency.
 *
 * @param members what every member paid and consumed
 * @param tip the tip
 * @param total the total: the amount and the tip
 */
public record ExpenseShares(List<MemberShare> members, Money tip, Money total) {
    /**
     * The expense shares of these components; the lists are copied.
     *
     * @param members what every member paid and consumed
     * @param tip the tip
     * @param total the total
     */
    public ExpenseShares {
        members = List.copyOf(members);
        Objects.requireNonNull(tip, "tip");
        Objects.requireNonNull(total, "total");
    }

    /**
     * The expense shares the generated code holds as {@code expenseShares}.
     *
     * @param expenseShares the generated value
     * @return the value
     */
    static ExpenseShares converted(eranos.internal.ExpenseShares expenseShares) {
        return new ExpenseShares(
                expenseShares.members().stream().map(MemberShare::converted).toList(),
                Money.converted(expenseShares.tip()),
                Money.converted(expenseShares.total()));
    }
}
