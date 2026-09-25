package eranos;

import java.util.Objects;

/**
 * The identifier of a member: a hyphenated UUID the group gave them.
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
