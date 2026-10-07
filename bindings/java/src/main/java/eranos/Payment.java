package eranos;

import java.util.Objects;

/**
 * One payment of a settlement: one member pays another an amount.
 *
 * @param from the member who pays
 * @param to the member who is paid
 * @param amount the amount, in the group currency
 */
public record Payment(MemberId from, MemberId to, Money amount) {
    /**
     * The payment of these components.
     *
     * @param from the member who pays
     * @param to the member who is paid
     * @param amount the amount, in the group currency
     */
    public Payment {
        Objects.requireNonNull(from, "from");
        Objects.requireNonNull(to, "to");
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
                new MemberId(payment.from()),
                new MemberId(payment.to()),
                Money.converted(payment.amount()));
    }
}
