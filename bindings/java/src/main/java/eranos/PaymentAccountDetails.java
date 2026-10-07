package eranos;

import java.util.Objects;

/** Where a member can be paid: a bank account or a payment-app username. */
public sealed interface PaymentAccountDetails {
    /**
     * A bank account.
     *
     * @param iban the account's IBAN
     */
    record Iban(eranos.Iban iban) implements PaymentAccountDetails {
        /**
         * The bank account of {@code iban}.
         *
         * @param iban the account's IBAN
         */
        public Iban {
            Objects.requireNonNull(iban, "iban");
        }
    }

    /**
     * A username in a payment app.
     *
     * @param username the username
     */
    record Username(eranos.Username username) implements PaymentAccountDetails {
        /**
         * The payment-app account of {@code username}.
         *
         * @param username the username
         */
        public Username {
            Objects.requireNonNull(username, "username");
        }
    }
}
