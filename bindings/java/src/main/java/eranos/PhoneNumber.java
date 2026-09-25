package eranos;

import eranos.internal.Eranos;

/** A phone number, valid in its country's numbering plan. */
public final class PhoneNumber {
    private final String e164;

    private PhoneNumber(String e164) {
        this.e164 = e164;
    }

    /**
     * The number written {@code text}, starting with {@code +} and the country code.
     *
     * @param text the number's text
     * @return the number
     * @throws EranosException.Contact with {@link ContactFailure.InvalidPhoneNumber}
     */
    public static PhoneNumber parse(String text) throws EranosException {
        return Converting.call(() -> new PhoneNumber(Eranos.phoneNumberParse(text)));
    }

    /**
     * The number the library wrote as {@code e164}.
     *
     * @param e164 the number in E.164 form
     * @return the number
     */
    static PhoneNumber known(String e164) {
        return new PhoneNumber(e164);
    }

    /**
     * The number in E.164 form.
     *
     * @return the number: "+40712345678"
     */
    public String e164() {
        return e164;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof PhoneNumber number && e164.equals(number.e164);
    }

    @Override
    public int hashCode() {
        return e164.hashCode();
    }

    /**
     * The number grouped as its country groups it.
     *
     * @return the text: "+40 712 345 678"
     */
    @Override
    public String toString() {
        try {
            return Eranos.phoneNumberInternational(e164);
        } catch (eranos.internal.EranosException exception) {
            return e164;
        }
    }
}
