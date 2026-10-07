package eranos;

import eranos.internal.Eranos;

import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * Something bought for the group: what it cost, who paid, and how it divides.
 *
 * <p>A new expense has no identifier; {@link Group#addExpense} gives it one. To change an expense,
 * read it from the group, edit it through {@link #toBuilder}, and pass the result to {@link
 * Group#replaceExpense}.
 */
public final class Expense {
    private final ExpenseId id;
    private final String name;
    private final Cost cost;
    private final Split payers;
    private final ExpenseSplit split;
    private final CalendarDate date;
    private final Tip tip;
    private final TimeOfDay time;
    private final CategoryId category;
    private final String description;

    private Expense(Builder builder) {
        id = builder.id;
        name = builder.name;
        cost = builder.cost;
        payers = builder.payers;
        split = builder.split;
        date = builder.date;
        tip = builder.tip;
        time = builder.time;
        category = builder.category;
        description = builder.description;
    }

    /**
     * A builder of an expense to add, of {@code cost} on {@code date}, that {@code payers} paid and
     * {@code split} shares.
     *
     * @param name what was bought
     * @param cost what it cost
     * @param payers who paid the total, amount and tip, and how much each
     * @param split how the amount is shared
     * @param date the day it was bought
     * @return the builder
     */
    public static Builder builder(
            String name, Cost cost, Split payers, ExpenseSplit split, CalendarDate date) {
        return new Builder(null, name, cost, payers, split, date);
    }

    /**
     * A builder that starts from this expense and keeps its identifier, so the expense it builds
     * can replace this one.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(id, name, cost, payers, split, date)
                .tip(tip)
                .time(time)
                .category(category)
                .description(description);
    }

    /**
     * The expense the generated code holds as {@code expense}.
     *
     * @param expense the generated expense
     * @return the expense
     */
    static Expense converted(eranos.internal.Expense expense) {
        return new Builder(
                        null == expense.id() ? null : new ExpenseId(expense.id()),
                        expense.name(),
                        Conversions.converted(expense.cost()),
                        Conversions.converted(expense.payers()),
                        Conversions.converted(expense.split()),
                        CalendarDate.converted(expense.date()))
                .tip(null == expense.tip() ? null : Tip.converted(expense.tip()))
                .time(null == expense.time() ? null : TimeOfDay.converted(expense.time()))
                .category(null == expense.category() ? null : new CategoryId(expense.category()))
                .description(expense.description())
                .build();
    }

    /**
     * The expense as the generated code holds it.
     *
     * @return the generated expense
     */
    eranos.internal.Expense generated() {
        return new eranos.internal.Expense(
                null == id ? null : id.value(),
                name,
                Conversions.generated(cost),
                Conversions.generated(payers),
                Conversions.generated(split),
                date.generated(),
                null == tip ? null : tip.generated(),
                null == time ? null : time.generated(),
                null == category ? null : category.value(),
                description);
    }

    /**
     * The identifier.
     *
     * @return the identifier, empty for an expense to add
     */
    public Optional<ExpenseId> id() {
        return Optional.ofNullable(id);
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
     * What it cost.
     *
     * @return the cost
     */
    public Cost cost() {
        return cost;
    }

    /**
     * The total, amount and tip, in the expense's own currency: what its payers paid.
     *
     * @return the total
     * @throws EranosException.Expense as {@link #tipAmount} does, and {@link EranosException.Money}
     *     with {@link MoneyFailure.Overflow} for a total out of range
     */
    public Money total() throws EranosException {
        return Converting.call(() -> Money.converted(Eranos.expenseTotal(generated())));
    }

    /**
     * Who paid the total, amount and tip, and how much each.
     *
     * @return the payers
     */
    public Split payers() {
        return payers;
    }

    /**
     * How the amount is shared.
     *
     * @return the split
     */
    public ExpenseSplit split() {
        return split;
    }

    /**
     * The day it was bought.
     *
     * @return the date
     */
    public CalendarDate date() {
        return date;
    }

    /**
     * The tip on top of the amount.
     *
     * @return the tip, if the expense has one
     */
    public Optional<Tip> tip() {
        return Optional.ofNullable(tip);
    }

    /**
     * The tip, in the expense's own currency.
     *
     * @return the tip
     * @throws EranosException.Expense with {@link ExpenseFailure.CurrencyMismatch} for a fixed tip
     *     in another currency, and {@link ExpenseFailure.NegativeTip} for a fixed tip below zero
     */
    public Money tipAmount() throws EranosException {
        return Converting.call(() -> Money.converted(Eranos.expenseTipAmount(generated())));
    }

    /**
     * The time of day it was bought.
     *
     * @return the time, if the expense has one
     */
    public Optional<TimeOfDay> time() {
        return Optional.ofNullable(time);
    }

    /**
     * The category.
     *
     * @return the category, empty for the built-in one
     */
    public Optional<CategoryId> category() {
        return Optional.ofNullable(category);
    }

    /**
     * A longer description.
     *
     * @return the description, if the expense has one
     */
    public Optional<String> description() {
        return Optional.ofNullable(description);
    }

    /**
     * Every member the expense involves, as a payer or a participant, in identifier order.
     *
     * @return the members
     * @throws EranosException.Id for a member whose identifier is not one, and the failures of an
     *     item that does not build
     */
    public List<MemberId> members() throws EranosException {
        return Converting.call(
                () -> Eranos.expenseMembers(generated()).stream().map(MemberId::new).toList());
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Expense expense
                && Objects.equals(id, expense.id)
                && name.equals(expense.name)
                && cost.equals(expense.cost)
                && payers.equals(expense.payers)
                && split.equals(expense.split)
                && date.equals(expense.date)
                && Objects.equals(tip, expense.tip)
                && Objects.equals(time, expense.time)
                && Objects.equals(category, expense.category)
                && Objects.equals(description, expense.description);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, name, cost, payers, split, date, tip, time, category, description);
    }

    @Override
    public String toString() {
        return "Expense[id="
                + id
                + ", name="
                + name
                + ", cost="
                + cost
                + ", payers="
                + payers
                + ", split="
                + split
                + ", date="
                + date
                + ", tip="
                + tip
                + ", time="
                + time
                + ", category="
                + category
                + ", description="
                + description
                + "]";
    }

    /** A builder of an expense, whose every setter replaces what was set before. */
    public static final class Builder {
        private final ExpenseId id;
        private String name;
        private Cost cost;
        private Split payers;
        private ExpenseSplit split;
        private CalendarDate date;
        private Tip tip;
        private TimeOfDay time;
        private CategoryId category;
        private String description;

        private Builder(
                ExpenseId id,
                String name,
                Cost cost,
                Split payers,
                ExpenseSplit split,
                CalendarDate date) {
            this.id = id;
            name(name).cost(cost).payers(payers).split(split).date(date);
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
         * Sets what it cost.
         *
         * @param cost the cost
         * @return this builder
         */
        public Builder cost(Cost cost) {
            this.cost = Objects.requireNonNull(cost, "cost");
            return this;
        }

        /**
         * Sets who paid the total, amount and tip, and how much each.
         *
         * @param payers the payers
         * @return this builder
         */
        public Builder payers(Split payers) {
            this.payers = Objects.requireNonNull(payers, "payers");
            return this;
        }

        /**
         * Sets how the amount is shared.
         *
         * @param split the split
         * @return this builder
         */
        public Builder split(ExpenseSplit split) {
            this.split = Objects.requireNonNull(split, "split");
            return this;
        }

        /**
         * Sets the day it was bought.
         *
         * @param date the date
         * @return this builder
         */
        public Builder date(CalendarDate date) {
            this.date = Objects.requireNonNull(date, "date");
            return this;
        }

        /**
         * Sets the tip; {@code null} leaves the expense without one.
         *
         * @param tip the tip, or {@code null}
         * @return this builder
         */
        public Builder tip(Tip tip) {
            this.tip = tip;
            return this;
        }

        /**
         * Sets the time of day it was bought; {@code null} leaves the expense without one.
         *
         * @param time the time, or {@code null}
         * @return this builder
         */
        public Builder time(TimeOfDay time) {
            this.time = time;
            return this;
        }

        /**
         * Sets the category; {@code null} leaves the expense in the built-in one.
         *
         * @param category the category, or {@code null}
         * @return this builder
         */
        public Builder category(CategoryId category) {
            this.category = category;
            return this;
        }

        /**
         * Sets the description; {@code null} leaves the expense without one.
         *
         * @param description the description, or {@code null}
         * @return this builder
         */
        public Builder description(String description) {
            this.description = description;
            return this;
        }

        /**
         * The expense; the group checks it when it is added or replaces another.
         *
         * @return the expense
         */
        public Expense build() {
            return new Expense(this);
        }
    }
}
