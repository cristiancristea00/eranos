package eranos;

import java.util.Objects;

/**
 * The identifier of a expense: a hyphenated UUID the group gave it.
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
