package eranos;

import java.util.Objects;

/**
 * One payment of a settlement: one member pays another an amount.
 *
 * @param amount the amount, in the group currency
 * @param from the member who pays
 * @param to the member who is paid
 */
public record Payment(Money amount, MemberId from, MemberId to) {
    /**
     * The payment of these components.
     *
     * @param amount the amount, in the group currency
     * @param from the member who pays
     * @param to the member who is paid
     */
    public Payment {
        Objects.requireNonNull(amount, "amount");
        Objects.requireNonNull(from, "from");
        Objects.requireNonNull(to, "to");
    }

    /**
     * The payment the generated code holds as {@code payment}.
     *
     * @param payment the generated value
     * @return the value
     */
    static Payment converted(eranos.internal.Payment payment) {
        return new Payment(
                Money.converted(payment.amount()),
                new MemberId(payment.from()),
                new MemberId(payment.to()));
    }
}
