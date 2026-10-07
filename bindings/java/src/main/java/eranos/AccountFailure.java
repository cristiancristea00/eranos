package eranos;

import java.util.Objects;

/** What the library's {@code AccountError} reports: a type per failure, with what it holds. */
public sealed interface AccountFailure {
    /** An account whose name is blank. */
    record EmptyName() implements AccountFailure {}

    /**
     * An IBAN that failed validation, and why.
     *
     * @param reason the check the IBAN failed
     */
    record InvalidIban(IbanReason reason) implements AccountFailure {
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
    record EmptyUsername() implements AccountFailure {}
}
