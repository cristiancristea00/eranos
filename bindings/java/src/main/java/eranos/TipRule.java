package eranos;

import java.util.Objects;

/**
 * A tip on top of an expense's amount, which its payers cover.
 *
 * @param value how the tip is expressed
 * @param distribution how the tip is shared among the participants
 */
public record TipRule(TipValue value, TipDistribution distribution) {
    /**
     * The tip of {@code value}, shared by {@code distribution}.
     *
     * @param value how the tip is expressed
     * @param distribution how the tip is shared among the participants
     */
    public TipRule {
        Objects.requireNonNull(value, "value");
        Objects.requireNonNull(distribution, "distribution");
    }

    /**
     * The tip of {@code value}, shared in proportion to each share.
     *
     * @param value how the tip is expressed
     */
    public TipRule(TipValue value) {
        this(value, TipDistribution.PROPORTIONAL);
    }

    /**
     * The tip the generated code holds as {@code tip}.
     *
     * @param tip the generated tip
     * @return the tip
     */
    static TipRule converted(eranos.internal.TipRule tip) {
        return new TipRule(
                Conversions.converted(tip.value()), TipDistribution.converted(tip.distribution()));
    }

    /**
     * The tip as the generated code holds it.
     *
     * @return the generated tip
     */
    eranos.internal.TipRule generated() {
        return new eranos.internal.TipRule(Conversions.generated(value), distribution.generated());
    }
}
