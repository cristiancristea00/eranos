package eranos;

import java.util.Objects;

/**
 * A tip on top of an expense's amount, which its payers cover.
 *
 * @param value how the tip is expressed
 * @param distribution how the tip is shared among the participants
 */
public record Tip(TipValue value, TipDistribution distribution) {
    /**
     * The tip of {@code value}, shared by {@code distribution}.
     *
     * @param value how the tip is expressed
     * @param distribution how the tip is shared among the participants
     */
    public Tip {
        Objects.requireNonNull(value, "value");
        Objects.requireNonNull(distribution, "distribution");
    }

    /**
     * The tip of {@code value}, shared in proportion to each share.
     *
     * @param value how the tip is expressed
     */
    public Tip(TipValue value) {
        this(value, TipDistribution.PROPORTIONAL);
    }

    /**
     * The tip the generated code holds as {@code tip}.
     *
     * @param tip the generated tip
     * @return the tip
     */
    static Tip converted(eranos.internal.Tip tip) {
        return new Tip(
                Conversions.converted(tip.value()), TipDistribution.converted(tip.distribution()));
    }

    /**
     * The tip as the generated code holds it.
     *
     * @return the generated tip
     */
    eranos.internal.Tip generated() {
        return new eranos.internal.Tip(Conversions.generated(value), distribution.generated());
    }
}
