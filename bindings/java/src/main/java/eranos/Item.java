package eranos;

import java.util.Objects;

/**
 * A line of an itemised bill: {@code quantity} units of {@code name} at {@code unitPrice}, which
 * {@code split} divides.
 *
 * @param name what was bought
 * @param split how the line's total divides
 * @param unitPrice the price of one unit, in the expense's currency
 * @param quantity how many units, from 1
 */
public record Item(String name, Split split, Money unitPrice, int quantity) {
    /**
     * The line of {@code quantity} units.
     *
     * @param name what was bought
     * @param split how the line's total divides
     * @param unitPrice the price of one unit, in the expense's currency
     * @param quantity how many units, from 1
     */
    public Item {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(split, "split");
        Objects.requireNonNull(unitPrice, "unitPrice");
    }

    /**
     * The line of one unit.
     *
     * @param name what was bought
     * @param split how the line's total divides
     * @param unitPrice the price of the unit, in the expense's currency
     */
    public Item(String name, Split split, Money unitPrice) {
        this(name, split, unitPrice, 1);
    }

    /**
     * The line the generated code holds as {@code item}.
     *
     * @param item the generated line
     * @return the line
     */
    static Item converted(eranos.internal.Item item) {
        return new Item(
                item.name(),
                Conversions.converted(item.split()),
                Money.converted(item.unitPrice()),
                item.quantity());
    }

    /**
     * The line as the generated code holds it.
     *
     * @return the generated line
     */
    eranos.internal.Item generated() {
        return new eranos.internal.Item(
                name, Conversions.generated(split), unitPrice.generated(), quantity);
    }
}
