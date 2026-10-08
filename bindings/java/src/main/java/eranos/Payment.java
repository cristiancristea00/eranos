package eranos;

import java.util.Objects;

/**
 * One payment of a settlement: one member pays another an amount.
 *
 * @param sender the member who pays
 * @param receiver the member who is paid
 * @param amount the amount, in the group currency
 */
public record Payment(MemberId sender, MemberId receiver, Money amount) {
    /**
     * The payment of these components.
     *
     * @param sender the member who pays
     * @param receiver the member who is paid
     * @param amount the amount, in the group currency
     */
    public Payment {
        Objects.requireNonNull(sender, "sender");
        Objects.requireNonNull(receiver, "receiver");
        Objects.requireNonNull(amount, "amount");
    }

    /**
     * The payment the generated code holds as {@code payment}.
     *
     * @param payment the generated value
     * @return the value
     */
    static Payment converted(eranos.internal.Payment payment) {
        return new Payment(
                new MemberId(payment.sender()),
                new MemberId(payment.receiver()),
                Money.converted(payment.amount()));
    }
}
