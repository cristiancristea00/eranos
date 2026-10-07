package eranos;

import java.util.Objects;
import java.util.Optional;

/**
 * Money one member paid another, such as a repayment.
 *
 * <p>A new transfer has no identifier; {@link Group#addTransfer} gives it one. To change a
 * transfer, read it from the group, edit it through {@link #toBuilder}, and pass the result to
 * {@link Group#replaceTransfer}.
 */
public final class Transfer {
    private final TransferId id;
    private final MemberId from;
    private final MemberId to;
    private final Cost cost;
    private final CalendarDate date;
    private final TimeOfDay time;
    private final String note;

    private Transfer(Builder builder) {
        id = builder.id;
        from = builder.from;
        to = builder.to;
        cost = builder.cost;
        date = builder.date;
        time = builder.time;
        note = builder.note;
    }

    /**
     * A builder of a transfer to add, of {@code cost} on {@code date}, from one member to another.
     *
     * @param from the member who paid
     * @param to the member who was paid
     * @param cost what was paid
     * @param date the day it was paid
     * @return the builder
     */
    public static Builder builder(MemberId from, MemberId to, Cost cost, CalendarDate date) {
        return new Builder(null, from, to, cost, date);
    }

    /**
     * A builder that starts from this transfer and keeps its identifier, so the transfer it builds
     * can replace this one.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(id, from, to, cost, date).time(time).note(note);
    }

    /**
     * The transfer the generated code holds as {@code transfer}.
     *
     * @param transfer the generated transfer
     * @return the transfer
     */
    static Transfer converted(eranos.internal.Transfer transfer) {
        return new Builder(
                        null == transfer.id() ? null : new TransferId(transfer.id()),
                        new MemberId(transfer.from()),
                        new MemberId(transfer.to()),
                        Conversions.converted(transfer.cost()),
                        CalendarDate.converted(transfer.date()))
                .time(null == transfer.time() ? null : TimeOfDay.converted(transfer.time()))
                .note(transfer.note())
                .build();
    }

    /**
     * The transfer as the generated code holds it.
     *
     * @return the generated transfer
     */
    eranos.internal.Transfer generated() {
        return new eranos.internal.Transfer(
                null == id ? null : id.value(),
                from.value(),
                to.value(),
                Conversions.generated(cost),
                date.generated(),
                null == time ? null : time.generated(),
                note);
    }

    /**
     * The identifier.
     *
     * @return the identifier, empty for a transfer to add
     */
    public Optional<TransferId> id() {
        return Optional.ofNullable(id);
    }

    /**
     * The member who paid.
     *
     * @return the member
     */
    public MemberId from() {
        return from;
    }

    /**
     * The member who was paid.
     *
     * @return the member
     */
    public MemberId to() {
        return to;
    }

    /**
     * What was paid.
     *
     * @return the cost
     */
    public Cost cost() {
        return cost;
    }

    /**
     * The day it was paid.
     *
     * @return the date
     */
    public CalendarDate date() {
        return date;
    }

    /**
     * The time of day it was paid.
     *
     * @return the time, if the transfer has one
     */
    public Optional<TimeOfDay> time() {
        return Optional.ofNullable(time);
    }

    /**
     * A note on the transfer.
     *
     * @return the note, if the transfer has one
     */
    public Optional<String> note() {
        return Optional.ofNullable(note);
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Transfer transfer
                && Objects.equals(id, transfer.id)
                && from.equals(transfer.from)
                && to.equals(transfer.to)
                && cost.equals(transfer.cost)
                && date.equals(transfer.date)
                && Objects.equals(time, transfer.time)
                && Objects.equals(note, transfer.note);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, from, to, cost, date, time, note);
    }

    @Override
    public String toString() {
        return "Transfer[id="
                + id
                + ", from="
                + from
                + ", to="
                + to
                + ", cost="
                + cost
                + ", date="
                + date
                + ", time="
                + time
                + ", note="
                + note
                + "]";
    }

    /** A builder of a transfer, whose every setter replaces what was set before. */
    public static final class Builder {
        private final TransferId id;
        private MemberId from;
        private MemberId to;
        private Cost cost;
        private CalendarDate date;
        private TimeOfDay time;
        private String note;

        private Builder(TransferId id, MemberId from, MemberId to, Cost cost, CalendarDate date) {
            this.id = id;
            from(from).to(to).cost(cost).date(date);
        }

        /**
         * Sets the member who paid.
         *
         * @param from the member
         * @return this builder
         */
        public Builder from(MemberId from) {
            this.from = Objects.requireNonNull(from, "from");
            return this;
        }

        /**
         * Sets the member who was paid.
         *
         * @param to the member
         * @return this builder
         */
        public Builder to(MemberId to) {
            this.to = Objects.requireNonNull(to, "to");
            return this;
        }

        /**
         * Sets what was paid.
         *
         * @param cost the cost
         * @return this builder
         */
        public Builder cost(Cost cost) {
            this.cost = Objects.requireNonNull(cost, "cost");
            return this;
        }

        /**
         * Sets the day it was paid.
         *
         * @param date the date
         * @return this builder
         */
        public Builder date(CalendarDate date) {
            this.date = Objects.requireNonNull(date, "date");
            return this;
        }

        /**
         * Sets the time of day it was paid; {@code null} leaves the transfer without one.
         *
         * @param time the time, or {@code null}
         * @return this builder
         */
        public Builder time(TimeOfDay time) {
            this.time = time;
            return this;
        }

        /**
         * Sets the note; {@code null} leaves the transfer without one.
         *
         * @param note the note, or {@code null}
         * @return this builder
         */
        public Builder note(String note) {
            this.note = note;
            return this;
        }

        /**
         * The transfer; the group checks it when it is added or replaces another.
         *
         * @return the transfer
         */
        public Transfer build() {
            return new Transfer(this);
        }
    }
}
