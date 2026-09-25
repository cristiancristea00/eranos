package eranos;

/** What the library's {@code CategoryError} reports: a type per failure, with what it holds. */
public sealed interface CategoryFailure {
    /** A name that is blank. */
    record EmptyName() implements CategoryFailure {}
}
