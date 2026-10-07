package eranos;

import java.util.List;
import java.util.Objects;

/**
 * How one expense divides, in the group currency.
 *
 * @param members what every member paid and consumed
 * @param total the total: the amount and the tip
 * @param tip the tip
 */
public record ExpenseShares(List<MemberShare> members, Money total, Money tip) {
    /**
     * The expense shares of these components; the lists are copied.
     *
     * @param members what every member paid and consumed
     * @param total the total
     * @param tip the tip
     */
    public ExpenseShares {
        members = List.copyOf(members);
        Objects.requireNonNull(total, "total");
        Objects.requireNonNull(tip, "tip");
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
                Money.converted(expenseShares.total()),
                Money.converted(expenseShares.tip()));
    }
}
