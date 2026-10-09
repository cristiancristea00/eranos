package eranos;

import java.util.Objects;
import java.util.Optional;

/**
 * Money one member paid another, such as a repayment.
 *
 * <p>A transfer has an identifier from the moment its builder starts: the one given to {@link
 * #builder(TransferId, MemberId, MemberId, Cost, CalendarDate)}, or one that {@link
 * #builder(MemberId, MemberId, Cost, CalendarDate)} generates, and keeps it through every edit. To
 * change a transfer, read it from the group, edit it through {@link #toBuilder}, and pass the
 * result to {@link ExpenseGroup#replaceTransfer}.
 */
public final class Transfer {
    private final TransferId id;
    private final MemberId sender;
    private final MemberId receiver;
    private final Cost cost;
    private final CalendarDate date;
    private final TimeOfDay time;
    private final String note;

    private Transfer(Builder builder) {
        id = builder.id;
        sender = builder.sender;
        receiver = builder.receiver;
        cost = builder.cost;
        date = builder.date;
        time = builder.time;
        note = builder.note;
    }

    /**
     * A builder of a transfer to add, of {@code cost} on {@code date}, from one member to another,
     * under an identifier generated when it is called; {@link #builder(TransferId, MemberId,
     * MemberId, Cost, CalendarDate)} takes one instead.
     *
     * @param sender the member who paid
     * @param receiver the member who was paid
     * @param cost what was paid
     * @param date the day it was paid
     * @return the builder
     */
    public static Builder builder(
            MemberId sender, MemberId receiver, Cost cost, CalendarDate date) {
        return builder(TransferId.generate(), sender, receiver, cost, date);
    }

    /**
     * A builder of a transfer to add, of {@code cost} on {@code date}, from one member to another,
     * under the identifier {@code id}, such as the key an app's own store gives it; {@link
     * #builder(MemberId, MemberId, Cost, CalendarDate)} generates one instead.
     *
     * @param id the identifier
     * @param sender the member who paid
     * @param receiver the member who was paid
     * @param cost what was paid
     * @param date the day it was paid
     * @return the builder
     */
    public static Builder builder(
            TransferId id, MemberId sender, MemberId receiver, Cost cost, CalendarDate date) {
        return new Builder(id, sender, receiver, cost, date);
    }

    /**
     * A builder of the transfer that records {@code payment} of a settlement, made on {@code date}:
     * from its sender to its receiver, of its amount in the group currency, under an identifier
     * generated when it is called; {@link #builder(TransferId, Payment, CalendarDate)} takes one
     * instead.
     *
     * @param payment the payment
     * @param date the day it was paid
     * @return the builder
     */
    public static Builder builder(Payment payment, CalendarDate date) {
        return builder(TransferId.generate(), payment, date);
    }

    /**
     * A builder of the transfer that records {@code payment} of a settlement, made on {@code date},
     * under the identifier {@code id}; {@link #builder(Payment, CalendarDate)} generates one
     * instead.
     *
     * @param id the identifier
     * @param payment the payment
     * @param date the day it was paid
     * @return the builder
     */
    public static Builder builder(TransferId id, Payment payment, CalendarDate date) {
        return builder(
                id, payment.sender(), payment.receiver(), new Cost.Local(payment.amount()), date);
    }

    /**
     * A builder that starts from this transfer and keeps its identifier, so the transfer it builds
     * can replace this one.
     *
     * @return the builder
     */
    public Builder toBuilder() {
        return new Builder(id, sender, receiver, cost, date).time(time).note(note);
    }

    /**
     * The transfer the generated code holds as {@code transfer}.
     *
     * @param transfer the generated transfer
     * @return the transfer
     */
    static Transfer converted(eranos.internal.Transfer transfer) {
        return new Builder(
                        new TransferId(transfer.id()),
                        new MemberId(transfer.sender()),
                        new MemberId(transfer.receiver()),
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
                id.value(),
                sender.value(),
                receiver.value(),
                Conversions.generated(cost),
                date.generated(),
                null == time ? null : time.generated(),
                note);
    }

    /**
     * The identifier.
     *
     * @return the identifier
     */
    public TransferId id() {
        return id;
    }

    /**
     * The member who paid.
     *
     * @return the member
     */
    public MemberId sender() {
        return sender;
    }

    /**
     * The member who was paid.
     *
     * @return the member
     */
    public MemberId receiver() {
        return receiver;
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
                && id.equals(transfer.id)
                && sender.equals(transfer.sender)
                && receiver.equals(transfer.receiver)
                && cost.equals(transfer.cost)
                && date.equals(transfer.date)
                && Objects.equals(time, transfer.time)
                && Objects.equals(note, transfer.note);
    }

    @Override
    public int hashCode() {
        return Objects.hash(id, sender, receiver, cost, date, time, note);
    }

    @Override
    public String toString() {
        return "Transfer[id="
                + id
                + ", sender="
                + sender
                + ", receiver="
                + receiver
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
        private TransferId id;
        private MemberId sender;
        private MemberId receiver;
        private Cost cost;
        private CalendarDate date;
        private TimeOfDay time;
        private String note;

        private Builder(
                TransferId id, MemberId sender, MemberId receiver, Cost cost, CalendarDate date) {
            id(id).sender(sender).receiver(receiver).cost(cost).date(date);
        }

        /**
         * Sets the identifier.
         *
         * @param id the identifier
         * @return this builder
         */
        public Builder id(TransferId id) {
            this.id = Objects.requireNonNull(id, "id");
            return this;
        }

        /**
         * Sets the member who paid.
         *
         * @param sender the member
         * @return this builder
         */
        public Builder sender(MemberId sender) {
            this.sender = Objects.requireNonNull(sender, "sender");
            return this;
        }

        /**
         * Sets the member who was paid.
         *
         * @param receiver the member
         * @return this builder
         */
        public Builder receiver(MemberId receiver) {
            this.receiver = Objects.requireNonNull(receiver, "receiver");
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
