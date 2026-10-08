package eranos;

import java.util.Objects;

/**
 * A discount off a line of an itemised bill: a percentage or a fixed amount, taken off the line as
 * a whole or off the price of each unit.
 *
 * <p>Off the line, a percentage is taken of the line total before the discount, the unit price
 * times the quantity, and rounded half away from zero to the minor unit. Off each unit, a
 * percentage of the unit price is rounded to the minor unit first, and the reduced price is then
 * multiplied by the quantity, so three units at 3.33 with 50 % off cost 4.98 with the discount off
 * each unit and 4.99 with it off the line. A discount of the whole price, 100 % or a fixed amount
 * equal to what it reduces, makes a free line, whose total is zero; the library refuses a larger
 * one in the item's totals and when the group adds its expense.
 *
 * @param value how the discount is expressed
 * @param scope what the discount is taken off
 */
public record Discount(DiscountValue value, DiscountScope scope) {
    /**
     * The discount of {@code value}, taken off {@code scope}.
     *
     * @param value how the discount is expressed
     * @param scope what the discount is taken off
     */
    public Discount {
        Objects.requireNonNull(value, "value");
        Objects.requireNonNull(scope, "scope");
    }

    /**
     * The discount of {@code value}, taken off the line, {@link DiscountScope#LINE}.
     *
     * @param value how the discount is expressed
     */
    public Discount(DiscountValue value) {
        this(value, DiscountScope.LINE);
    }

    /**
     * The discount the generated code holds as {@code discount}.
     *
     * @param discount the generated discount
     * @return the discount
     */
    static Discount converted(eranos.internal.Discount discount) {
        return new Discount(
                Conversions.converted(discount.value()), DiscountScope.converted(discount.scope()));
    }

    /**
     * The discount as the generated code holds it.
     *
     * @return the generated discount
     */
    eranos.internal.Discount generated() {
        return new eranos.internal.Discount(Conversions.generated(value), scope.generated());
    }
}
