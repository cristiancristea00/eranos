package eranos;

/** What the library's {@code IdError} reports: a type per failure, with what it holds. */
public sealed interface IdFailure {
    /** Empty text. */
    record Empty() implements IdFailure {}

    /** Text longer than 64 bytes of UTF-8. */
    record TooLong() implements IdFailure {}

    /** Text that holds a control character, such as a tab or a line break. */
    record ControlCharacter() implements IdFailure {}

    /** Text that starts or ends with whitespace. */
    record SurroundingWhitespace() implements IdFailure {}
}
