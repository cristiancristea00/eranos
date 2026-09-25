package eranos;

/** What the library's {@code MemberError} reports: a type per failure, with what it holds. */
public sealed interface MemberFailure {
    /** A first name that is blank. */
    record EmptyFirstName() implements MemberFailure {}
}
