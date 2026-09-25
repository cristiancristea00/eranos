package eranos;

/** What the library's {@code CalendarError} reports: a type per failure, with what it holds. */
public sealed interface CalendarFailure {
    /** A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text. */
    record InvalidDate() implements CalendarFailure {}

    /** A time of day that does not exist, has seconds, or is not ISO 8601 text. */
    record InvalidTime() implements CalendarFailure {}
}
