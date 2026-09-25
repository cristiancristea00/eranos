package eranos;

/** What the library's {@code TransferError} reports: a type per failure, with what it holds. */
public sealed interface TransferFailure {
    /** The amount is zero or negative. */
    record NonPositiveAmount() implements TransferFailure {}

    /** The sender and the recipient are the same member. */
    record SameMember() implements TransferFailure {}
}
