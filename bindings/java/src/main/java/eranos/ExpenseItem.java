package eranos;

import eranos.internal.Eranos;

import java.util.Objects;
import java.util.Optional;

/**
 * A line of an itemised bill: a quantity of units of what was bought at a unit price, which a split
 * divides, with an optional discount off the line or off each unit.
 *
 * <p>Its builder starts it at one unit, {@link Quantity#ONE}, and without a discount. To change a
 * line, edit it through {@link #toBuilder}. The library checks a line when it reads its totals and
 * when the group adds or replaces its expense.
 */
public final class ExpenseItem {
    private final String name;
    private final Money unitPrice;
    private final Split split;
    private final Quantity quantity;
    private final Discount discount;

    private ExpenseItem(Builder builder) {
        name = builder.name;
        unitPrice = builder.unitPrice;
        split = builder.split;
        quantity = builder.quantity;
        discount = builder.discount;
    }

    /**
     * A builder of the line of {@code name} at {@code unitPrice}, which {@code split} divides.
     *
     * @param name what was bought
     * @param unitPrice the price of one unit, in the expense's currency
     * @param split how the line's total divides
     * @return the builder
     */
    public static Builder builder(String name, Money unitPrice, Split split) {
        return new Builder(name, unitPrice, split);
    }

    /**
     * A builder that starts from this line.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(name, unitPrice, split).quantity(quantity).discount(discount);
    }

    /**
     * The line the generated code holds as {@code item}.
     *
     * @param item the generated line
     * @return the line
     */
    static ExpenseItem converted(eranos.internal.ExpenseItem item) {
        return new Builder(
                        item.name(),
                        Money.converted(item.unitPrice()),
                        Conversions.converted(item.split()))
                .quantity(Quantity.known(item.quantity()))
                .discount(null == item.discount() ? null : Discount.converted(item.discount()))
                .build();
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
                quantity.tenThousandths(),
                null == discount ? null : discount.generated());
    }

    /**
     * What was bought.
     *
     * @return the name
     */
    public String name() {
        return name;
    }

    /**
     * The price of one unit, in the expense's currency.
     *
     * @return the unit price
     */
    public Money unitPrice() {
        return unitPrice;
    }

    /**
     * How the line's total divides.
     *
     * @return the split
     */
    public Split split() {
        return split;
    }

    /**
     * How many units, to a ten-thousandth.
     *
     * @return the quantity: {@link Quantity#ONE} for a line that was given none
     */
    public Quantity quantity() {
        return quantity;
    }

    /**
     * The line total before the discount, as the library computes it: the unit price times the
     * quantity, rounded half away from zero to the currency's minor unit, so zero for a line whose
     * product is below half a minor unit.
     *
     * @return the line total before the discount
     * @throws EranosException.Expense as {@link #lineTotal} does
     * @throws EranosException.Id as {@link #lineTotal} does
     */
    public Money lineTotalBeforeDiscount() throws EranosException {
        return Converting.call(
                () -> Money.converted(Eranos.expenseItemLineTotalBeforeDiscount(generated())));
    }

    /**
     * The discount off the line or off each unit.
     *
     * @return the discount, if the line has one
     */
    public Optional<Discount> discount() {
        return Optional.ofNullable(discount);
    }

    /**
     * What the discount takes off the line, as the library computes it: the line total before the
     * discount minus the line total.
     *
     * @return the discount's amount: zero for a line without a discount
     * @throws EranosException.Expense as {@link #lineTotal} does
     * @throws EranosException.Id as {@link #lineTotal} does
     */
    public Money discountAmount() throws EranosException {
        return Converting.call(
                () -> Money.converted(Eranos.expenseItemDiscountAmount(generated())));
    }

    /**
     * The line total, as the library computes it: the unit price times the quantity, rounded half
     * away from zero to the currency's minor unit, less the discount, so zero for a free line,
     * whose product is below half a minor unit or whose discount takes the whole price.
     *
     * @return the line total
     * @throws EranosException.Expense with {@link ExpenseFailure.EmptyItemName} for a blank name,
     *     {@link ExpenseFailure.NonPositiveAmount} for a unit price that is not positive, {@link
     *     ExpenseFailure.CurrencyMismatch} for a fixed discount in another currency, {@link
     *     ExpenseFailure.NegativeDiscount} for a fixed discount below zero, {@link
     *     ExpenseFailure.DiscountAboveHundredPercent} for a percentage above 100 %, {@link
     *     ExpenseFailure.DiscountAbovePrice} for a fixed discount larger than what it reduces,
     *     {@link ExpenseFailure.AmountsDoNotAddUp} for a split of amounts that does not add up to
     *     the line total, a free line's included, {@link ExpenseFailure.Overflow} for a line total
     *     out of range, and the failures of a split that does not divide it
     * @throws EranosException.Id for a member whose identifier is not one
     */
    public Money lineTotal() throws EranosException {
        return Converting.call(() -> Money.converted(Eranos.expenseItemLineTotal(generated())));
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof ExpenseItem item
                && name.equals(item.name)
                && unitPrice.equals(item.unitPrice)
                && split.equals(item.split)
                && quantity.equals(item.quantity)
                && Objects.equals(discount, item.discount);
    }

    @Override
    public int hashCode() {
        return Objects.hash(name, unitPrice, split, quantity, discount);
    }

    @Override
    public String toString() {
        return "ExpenseItem[name="
                + name
                + ", unitPrice="
                + unitPrice
                + ", split="
                + split
                + ", quantity="
                + quantity
                + ", discount="
                + discount
                + "]";
    }

    /** A builder of a line, whose every setter replaces what was set before. */
    public static final class Builder {
        private String name;
        private Money unitPrice;
        private Split split;
        private Quantity quantity = Quantity.ONE;
        private Discount discount;

        private Builder(String name, Money unitPrice, Split split) {
            name(name).unitPrice(unitPrice).split(split);
        }

        /**
         * Sets what was bought.
         *
         * @param name the name
         * @return this builder
         */
        public Builder name(String name) {
            this.name = Objects.requireNonNull(name, "name");
            return this;
        }

        /**
         * Sets the price of one unit, in the expense's currency.
         *
         * @param unitPrice the unit price
         * @return this builder
         */
        public Builder unitPrice(Money unitPrice) {
            this.unitPrice = Objects.requireNonNull(unitPrice, "unitPrice");
            return this;
        }

        /**
         * Sets how the line's total divides.
         *
         * @param split the split
         * @return this builder
         */
        public Builder split(Split split) {
            this.split = Objects.requireNonNull(split, "split");
            return this;
        }

        /**
         * Sets how many units, to a ten-thousandth; {@link Quantity#ONE}, which the builder starts
         * at, makes a line of one unit.
         *
         * @param quantity the quantity
         * @return this builder
         */
        public Builder quantity(Quantity quantity) {
            this.quantity = Objects.requireNonNull(quantity, "quantity");
            return this;
        }

        /**
         * Sets the discount off the line or off each unit; {@code null} leaves the line without
         * one.
         *
         * @param discount the discount, or {@code null}
         * @return this builder
         */
        public Builder discount(Discount discount) {
            this.discount = discount;
            return this;
        }

        /**
         * The line; the library checks it when it reads its totals and when the group adds or
         * replaces its expense.
         *
         * @return the line
         */
        public ExpenseItem build() {
            return new ExpenseItem(this);
        }
    }
}
