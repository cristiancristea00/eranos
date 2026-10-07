package eranos;

import java.util.List;
import java.util.Objects;

/**
 * The payments that settle every balance.
 *
 * @param payments the payments, ordered by payer and then by recipient
 * @param strategy the strategy that produced them: the one asked for, or, for the adaptive
 *     strategy, the one whose plan it chose
 */
public record SettlementPlan(List<Payment> payments, SettlementStrategy strategy) {
    /**
     * The settlement of these components; the lists are copied.
     *
     * @param payments the payments, ordered by payer and then by recipient
     * @param strategy the strategy that produced them: the one asked for, or, for the adaptive
     *     strategy, the one whose plan it chose
     */
    public SettlementPlan {
        payments = List.copyOf(payments);
        Objects.requireNonNull(strategy, "strategy");
    }

    /**
     * The settlement the generated code holds as {@code settlement}.
     *
     * @param settlement the generated value
     * @return the value
     */
    static SettlementPlan converted(eranos.internal.SettlementPlan settlement) {
        return new SettlementPlan(
                settlement.payments().stream().map(Payment::converted).toList(),
                SettlementStrategy.converted(settlement.strategy()));
    }
}
