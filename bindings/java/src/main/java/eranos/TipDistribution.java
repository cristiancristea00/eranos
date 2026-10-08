package eranos;

/** How a tip is shared among an expense's participants. */
public enum TipDistribution {
    /** In proportion to each participant's share of the amount. */
    PROPORTIONAL,

    /** The same for every participant with a share of the amount above zero. */
    EQUAL;

    /**
     * The distribution the generated code holds as {@code distribution}.
     *
     * @param distribution the generated distribution
     * @return the distribution
     */
    static TipDistribution converted(eranos.internal.TipDistribution distribution) {
        return switch (distribution) {
            case PROPORTIONAL -> PROPORTIONAL;
            case EQUAL -> EQUAL;
        };
    }

    /**
     * The distribution as the generated code holds it.
     *
     * @return the generated distribution
     */
    eranos.internal.TipDistribution generated() {
        return switch (this) {
            case PROPORTIONAL -> eranos.internal.TipDistribution.PROPORTIONAL;
            case EQUAL -> eranos.internal.TipDistribution.EQUAL;
        };
    }
}
