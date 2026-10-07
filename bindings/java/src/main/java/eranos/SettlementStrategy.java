package eranos;

/** How balances turn into payments. */
public enum SettlementStrategy {
    /**
     * The fewest payments while at most 20 balances are left after exact opposites pay each other,
     * the greedy plan beyond; the settlement names the strategy whose plan it is.
     */
    ADAPTIVE,

    /** The fewest payments possible. */
    MINIMUM_TRANSFERS,

    /**
     * The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each
     * other where that saves a payment.
     */
    GREEDY,

    /** Everyone pays back the people who paid for them. */
    DIRECT;

    /**
     * The strategy the generated code holds as {@code strategy}.
     *
     * @param strategy the generated strategy
     * @return the strategy
     */
    static SettlementStrategy converted(eranos.internal.SettlementStrategy strategy) {
        return switch (strategy) {
            case ADAPTIVE -> ADAPTIVE;
            case MINIMUM_TRANSFERS -> MINIMUM_TRANSFERS;
            case GREEDY -> GREEDY;
            case DIRECT -> DIRECT;
        };
    }

    /**
     * The strategy as the generated code holds it.
     *
     * @return the generated strategy
     */
    eranos.internal.SettlementStrategy generated() {
        return switch (this) {
            case ADAPTIVE -> eranos.internal.SettlementStrategy.ADAPTIVE;
            case MINIMUM_TRANSFERS -> eranos.internal.SettlementStrategy.MINIMUM_TRANSFERS;
            case GREEDY -> eranos.internal.SettlementStrategy.GREEDY;
            case DIRECT -> eranos.internal.SettlementStrategy.DIRECT;
        };
    }
}
