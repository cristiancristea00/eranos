package eranos;

/** What the library's {@code LanguageError} reports: a type per failure, with what it holds. */
public sealed interface LanguageFailure {
    /** A language tag that is not a Unicode BCP 47 locale identifier (UTS #35). */
    record InvalidLanguage() implements LanguageFailure {}
}
