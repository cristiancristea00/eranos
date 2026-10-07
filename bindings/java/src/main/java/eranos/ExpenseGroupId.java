package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * The identifier of a group: the text the app gave it, or a UUID v7 in lowercase hyphenated form
 * that the library generated.
 *
 * <p>Every entity has an identifier type of its own, so one entity's identifier cannot stand where
 * another's is expected. An identifier is text of at most 64 bytes of UTF-8, not empty, without
 * control characters, and without whitespace at either end. Two identifiers are equal when their
 * texts are, so case matters, and identifier order is the byte order of their UTF-8 text, which is
 * creation order for generated identifiers; {@link String#compareTo}, over UTF-16 code units,
 * orders text above U+FFFF otherwise. An identifier converts to its text and back, for storing it:
 * {@link #parse} checks the text by these rules, and the record's constructor leaves it to the
 * library to check when a call carries it.
 *
 * @param value the identifier's text
 */
public record ExpenseGroupId(String value) {
    /**
     * The identifier written {@code value}, as it is: the library checks it when a call carries it.
     *
     * @param value the identifier's text
     */
    public ExpenseGroupId {
        Objects.requireNonNull(value, "value");
    }

    /**
     * The identifier written {@code text}, checked by the rules of every identifier: not trimmed,
     * and in its case.
     *
     * @param text the identifier's text
     * @return the identifier
     * @throws EranosException.Id with the first rule the text breaks: {@link IdFailure.Empty},
     *     {@link IdFailure.TooLong} for more than 64 bytes of UTF-8, {@link
     *     IdFailure.ControlCharacter}, and {@link IdFailure.SurroundingWhitespace}
     */
    public static ExpenseGroupId parse(String text) throws EranosException {
        return Converting.call(() -> new ExpenseGroupId(Eranos.identifierParse(text)));
    }

    /**
     * The identifier's text.
     *
     * @return the text
     */
    @Override
    public String toString() {
        return value;
    }
}
