package eranos;

import java.util.Objects;

/**
 * What the library's {@code PaymentAccountError} reports: a type per failure, with what it holds.
 */
public sealed interface PaymentAccountFailure {
    /** An account whose name is blank. */
    record EmptyName() implements PaymentAccountFailure {}

    /**
     * An IBAN that failed validation, and why.
     *
     * @param reason the check the IBAN failed
     */
    record InvalidIban(IbanReason reason) implements PaymentAccountFailure {
        /**
         * The failure of these components.
         *
         * @param reason the check the IBAN failed
         */
        public InvalidIban {
            Objects.requireNonNull(reason, "reason");
        }
    }

    /** A username that is blank. */
    record EmptyUsername() implements PaymentAccountFailure {}
}
