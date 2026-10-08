package eranos;

/** What a discount is taken off. */
public enum DiscountScope {
    /** The line total, the unit price times the quantity: one discount for the whole line. */
    LINE,

    /** The price of each unit, before it is multiplied by the quantity. */
    UNIT;

    /**
     * The scope the generated code holds as {@code scope}.
     *
     * @param scope the generated scope
     * @return the scope
     */
    static DiscountScope converted(eranos.internal.DiscountScope scope) {
        return switch (scope) {
            case LINE -> LINE;
            case UNIT -> UNIT;
        };
    }

    /**
     * The scope as the generated code holds it.
     *
     * @return the generated scope
     */
    eranos.internal.DiscountScope generated() {
        return switch (this) {
            case LINE -> eranos.internal.DiscountScope.LINE;
            case UNIT -> eranos.internal.DiscountScope.UNIT;
        };
    }
}
