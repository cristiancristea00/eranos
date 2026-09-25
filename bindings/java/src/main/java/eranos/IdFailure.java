package eranos;

/** What the library's {@code IdError} reports: a type per failure, with what it holds. */
public sealed interface IdFailure {
    /** Text that is not a UUID. */
    record InvalidId() implements IdFailure {}
}
