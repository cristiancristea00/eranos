package eranos;

/** How a tip is shared among an expense's participants. */
public enum TipDistribution {
    /** The same for every participant. */
    EQUAL,

    /** In proportion to each participant's share of the amount. */
    PROPORTIONAL;

    /**
     * The distribution the generated code holds as {@code distribution}.
     *
     * @param distribution the generated distribution
     * @return the distribution
     */
    static TipDistribution converted(eranos.internal.TipDistribution distribution) {
        return switch (distribution) {
            case EQUAL -> EQUAL;
            case PROPORTIONAL -> PROPORTIONAL;
        };
    }

    /**
     * The distribution as the generated code holds it.
     *
     * @return the generated distribution
     */
    eranos.internal.TipDistribution generated() {
        return switch (this) {
            case EQUAL -> eranos.internal.TipDistribution.EQUAL;
            case PROPORTIONAL -> eranos.internal.TipDistribution.PROPORTIONAL;
        };
    }
}
