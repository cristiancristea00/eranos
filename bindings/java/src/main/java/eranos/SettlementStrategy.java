package eranos;

/** How balances turn into payments. */
public enum SettlementStrategy {
    /** Everyone pays back the people who paid for them. */
    DIRECT,

    /**
     * The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each
     * other where that saves a payment.
     */
    GREEDY,

    /** The fewest payments possible. */
    MINIMUM_TRANSFERS;

    /**
     * The strategy the generated code holds as {@code strategy}.
     *
     * @param strategy the generated strategy
     * @return the strategy
     */
    static SettlementStrategy converted(eranos.internal.SettlementStrategy strategy) {
        return switch (strategy) {
            case DIRECT -> DIRECT;
            case GREEDY -> GREEDY;
            case MINIMUM_TRANSFERS -> MINIMUM_TRANSFERS;
        };
    }

    /**
     * The strategy as the generated code holds it.
     *
     * @return the generated strategy
     */
    eranos.internal.SettlementStrategy generated() {
        return switch (this) {
            case DIRECT -> eranos.internal.SettlementStrategy.DIRECT;
            case GREEDY -> eranos.internal.SettlementStrategy.GREEDY;
            case MINIMUM_TRANSFERS -> eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS;
        };
    }
}
