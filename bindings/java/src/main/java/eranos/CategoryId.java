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
     * The identifier of the built-in category, Uncategorised, which every group has and every
     * expense is in until it is given another: the text of the nil UUID, {@code
     * 00000000-0000-0000-0000-000000000000}.
     */
    public static final CategoryId UNCATEGORISED = new CategoryId(Eranos.categoryIdUncategorised());

    /**
     * The identifier written {@code value}, as it is: the library checks it when a call carries it.
     *
     * @param value the identifier's text
     */
    public CategoryId {
        Objects.requireNonNull(value, "value");
    }

    /**
     * A new identifier, a UUID v7 in lowercase hyphenated form.
     *
     * <p>Every kind of identifier draws from the one generator of the library, so identifiers
     * generated one after another are in increasing order, whatever their kinds.
     *
     * @return the identifier
     */
    public static CategoryId generate() {
        return new CategoryId(Eranos.identifierGenerate());
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
