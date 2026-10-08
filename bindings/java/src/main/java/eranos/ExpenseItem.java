package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * A line of an itemised bill: {@code quantity} units of {@code name} at {@code unitPrice}, which
 * {@code split} divides.
 *
 * @param name what was bought
 * @param unitPrice the price of one unit, in the expense's currency
 * @param split how the line's total divides
 * @param quantity how many units, to a ten-thousandth
 */
public record ExpenseItem(String name, Money unitPrice, Split split, Quantity quantity) {
    /**
     * The line of {@code quantity} units.
     *
     * @param name what was bought
     * @param unitPrice the price of one unit, in the expense's currency
     * @param split how the line's total divides
     * @param quantity how many units, to a ten-thousandth
     */
    public ExpenseItem {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(unitPrice, "unitPrice");
        Objects.requireNonNull(split, "split");
        Objects.requireNonNull(quantity, "quantity");
    }

    /**
     * The line of one unit, {@link Quantity#ONE}.
     *
     * @param name what was bought
     * @param unitPrice the price of the unit, in the expense's currency
     * @param split how the line's total divides
     */
    public ExpenseItem(String name, Money unitPrice, Split split) {
        this(name, unitPrice, split, Quantity.ONE);
    }

    /**
     * The line total, as the library computes it: the unit price times the quantity, rounded half
     * away from zero to the currency's minor unit, so zero for a line whose product is below half a
     * minor unit.
     *
     * @return the line total
     * @throws EranosException.Expense with {@link ExpenseFailure.EmptyItemName} for a blank name,
     *     {@link ExpenseFailure.NonPositiveAmount} for a unit price that is not positive, {@link
     *     ExpenseFailure.AmountsDoNotAddUp} for a split of amounts that does not add up to the line
     *     total, a free line's included, {@link ExpenseFailure.Overflow} for a line total out of
     *     range, and the failures of a split that does not divide it
     * @throws EranosException.Id for a member whose identifier is not one
     */
    public Money lineTotal() throws EranosException {
        return Converting.call(() -> Money.converted(Eranos.expenseItemLineTotal(generated())));
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
                Quantity.known(item.quantity()));
    }

    /**
     * The line as the generated code holds it.
     *
     * @return the generated line
     */
    eranos.internal.ExpenseItem generated() {
        return new eranos.internal.ExpenseItem(
                name,
                unitPrice.generated(),
                Conversions.generated(split),
                quantity.tenThousandths());
    }
}
