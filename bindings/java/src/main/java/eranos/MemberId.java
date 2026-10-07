package eranos;

import java.util.Objects;

/**
 * The identifier of a member: the text the group gave them, a UUID v7 in lowercase hyphenated form.
 *
 * <p>Its text follows the rules of every identifier, on {@link ExpenseGroupId}.
 *
 * @param value the identifier's text
 */
public record MemberId(String value) {
    /**
     * The identifier written {@code value}.
     *
     * @param value the identifier's text
     */
    public MemberId {
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
