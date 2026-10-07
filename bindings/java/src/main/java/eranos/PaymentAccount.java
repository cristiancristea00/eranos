package eranos;

import java.util.Objects;

/**
 * A place to send a member money, with a label: "Main account".
 *
 * @param name the label
 * @param details the IBAN or the username
 */
public record PaymentAccount(String name, PaymentAccountDetails details) {
    /**
     * The account at {@code details}, labelled {@code name}.
     *
     * @param name the label
     * @param details the IBAN or the username
     */
    public PaymentAccount {
        Objects.requireNonNull(name, "name");
        Objects.requireNonNull(details, "details");
    }

    /**
     * The account the generated code holds as {@code account}.
     *
     * @param account the generated account
     * @return the account
     */
    static PaymentAccount converted(eranos.internal.PaymentAccount account) {
        return new PaymentAccount(account.name(), Conversions.converted(account.details()));
    }

    /**
     * The account as the generated code holds it.
     *
     * @return the generated account
     */
    eranos.internal.PaymentAccount generated() {
        return new eranos.internal.PaymentAccount(name, Conversions.generated(details));
    }
}
