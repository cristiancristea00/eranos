package eranos;

import java.util.Objects;

/**
 * The identifier of a group: a hyphenated UUID the library gave it.
 *
 * <p>Every entity has an identifier type of its own, so one entity's identifier cannot stand where
 * another's is expected. An identifier converts to its text and back, for storing it; the library
 * checks the text when a call carries it.
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
