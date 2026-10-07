package eranos;

import java.util.Objects;

/**
 * A line of an itemised bill: {@code quantity} units of {@code name} at {@code unitPrice}, which
 * {@code split} divides.
 *
 * @param name what was bought
 * @param unitPrice the price of one unit, in the expense's currency
 * @param split how the line's total divides
 * @param quantity how many units, from 1
 */
public record ExpenseItem(String name, Money unitPrice, Split split, int quantity) {
    /**
     * The line of {@code quantity} units.
     *
     * @param name what was bought
     * @param unitPrice the price of one unit, in the expense's currency
     * @param split how the line's total divides
     * @param quantity how many units, from 1
     */
    public ExpenseItem {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(unitPrice, "unitPrice");
        Objects.requireNonNull(split, "split");
    }

    /**
     * The line of one unit.
     *
     * @param name what was bought
     * @param unitPrice the price of the unit, in the expense's currency
     * @param split how the line's total divides
     */
    public ExpenseItem(String name, Money unitPrice, Split split) {
        this(name, unitPrice, split, 1);
    }

    /**
     * The line the generated code holds as {@code item}.
     *
     * @param item the generated line
     * @return the line
     */
    static ExpenseItem converted(eranos.internal.ExpenseItem item) {
        return new ExpenseItem(
                item.name(),
                Money.converted(item.unitPrice()),
                Conversions.converted(item.split()),
                item.quantity());
    }

    /**
     * The line as the generated code holds it.
     *
     * @return the generated line
     */
    eranos.internal.ExpenseItem generated() {
        return new eranos.internal.ExpenseItem(
                name, unitPrice.generated(), Conversions.generated(split), quantity);
    }
}
