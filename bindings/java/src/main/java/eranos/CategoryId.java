package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * The identifier of a category: the text the app gave it, or a UUID v7 in lowercase hyphenated form
 * that the library generated.
 *
 * <p>Its text follows the rules of every identifier, on {@link ExpenseGroupId}.
 *
 * @param value the identifier's text
 */
public record CategoryId(String value) {
    /**
     * The identifier written {@code value}, as it is: the library checks it when a call carries it.
     *
     * @param value the identifier's text
     */
    public CategoryId {
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
    public static CategoryId parse(String text) throws EranosException {
        return Converting.call(() -> new CategoryId(Eranos.identifierParse(text)));
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
