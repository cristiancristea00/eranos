package eranos;

import eranos.internal.Eranos;

/** A username in a payment app. */
public final class Username {
    private final String text;

    private Username(String text) {
        this.text = text;
    }

    /**
     * The username written {@code text}.
     *
     * @param text the username's text
     * @return the username
     * @throws EranosException.PaymentAccount with {@link PaymentAccountFailure.EmptyUsername} for
     *     blank text
     */
    public static Username parse(String text) throws EranosException {
        return Converting.call(() -> new Username(Eranos.usernameParse(text)));
    }

    /**
     * The username the library wrote as {@code text}.
     *
     * @param text the username's text
     * @return the username
     */
    static Username known(String text) {
        return new Username(text);
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Username username && text.equals(username.text);
    }

    @Override
    public int hashCode() {
        return text.hashCode();
    }

    /**
     * The username, as entered but without surrounding whitespace.
     *
     * @return the text
     */
    @Override
    public String toString() {
        return text;
    }
}
