package eranos;

import java.util.Objects;

/**
 * The identifier of a group: the text the library gave it, a UUID v7 in lowercase hyphenated form.
 *
 * <p>Every entity has an identifier type of its own, so one entity's identifier cannot stand where
 * another's is expected. An identifier is text of at most 64 bytes of UTF-8, not empty, without
 * control characters, and without whitespace at either end. Two identifiers are equal when their
 * texts are, so case matters, and identifier order is the byte order of their UTF-8 text, which is
 * creation order for generated identifiers. An identifier converts to its text and back, for
 * storing it; the library checks the text when a call carries it.
 *
 * @param value the identifier's text
 */
public record ExpenseGroupId(String value) {
    /**
     * The identifier written {@code value}.
     *
     * @param value the identifier's text
     */
    public ExpenseGroupId {
        Objects.requireNonNull(value, "value");
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
