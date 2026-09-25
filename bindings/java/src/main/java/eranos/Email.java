package eranos;

import eranos.internal.Eranos;

/** An email address, checked by the library's rules. */
public final class Email {
    private final String text;

    private Email(String text) {
        this.text = text;
    }

    /**
     * The address written {@code text}.
     *
     * @param text the address's text
     * @return the address
     * @throws EranosException.Contact with {@link ContactFailure.InvalidEmail} for text that is not
     *     a local part and a domain around an {@code @}
     */
    public static Email parse(String text) throws EranosException {
        return Converting.call(() -> new Email(Eranos.emailParse(text)));
    }

    /**
     * The address the library wrote as {@code text}.
     *
     * @param text the address's text
     * @return the address
     */
    static Email known(String text) {
        return new Email(text);
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Email email && text.equals(email.text);
    }

    @Override
    public int hashCode() {
        return text.hashCode();
    }

    /**
     * The address, as entered but without surrounding whitespace.
     *
     * @return the text
     */
    @Override
    public String toString() {
        return text;
    }
}
