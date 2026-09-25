package eranos;

/** What the library's {@code ContactError} reports: a type per failure, with what it holds. */
public sealed interface ContactFailure {
    /**
     * An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an
     * {@code @}, a malformed one, one too long, or whitespace or an invisible character.
     */
    record InvalidEmail() implements ContactFailure {}

    /**
     * A phone number that does not start with {@code +}, holds a letter or an extension, or is not
     * valid in its country's numbering plan.
     */
    record InvalidPhoneNumber() implements ContactFailure {}
}
