package eranos;

import java.util.Objects;

/**
 * The identifier of an expense: the text the group gave it, a UUID v7 in lowercase hyphenated form.
 *
 * <p>Its text follows the rules of every identifier, on {@link ExpenseGroupId}.
 *
 * @param value the identifier's text
 */
public record ExpenseId(String value) {
    /**
     * The identifier written {@code value}.
     *
     * @param value the identifier's text
     */
    public ExpenseId {
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
