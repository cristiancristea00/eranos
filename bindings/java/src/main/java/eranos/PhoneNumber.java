package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * A phone number, valid in its country's numbering plan, with the two forms the library writes it
 * in.
 */
public final class PhoneNumber {
    private final String e164;
    private final String international;

    private PhoneNumber(String e164, String international) {
        this.e164 = e164;
        this.international = international;
    }

    /**
     * The number written {@code text}, starting with {@code +} and the country code.
     *
     * @param text the number's text
     * @return the number
     * @throws EranosException.Contact with {@link ContactFailure.InvalidPhoneNumber}
     */
    public static PhoneNumber parse(String text) throws EranosException {
        return Converting.call(() -> converted(Eranos.phoneNumberParse(text)));
    }

    /**
     * The number the generated code holds as {@code number}, with both forms as the library wrote
     * them.
     *
     * @param number the generated number
     * @return the number
     */
    static PhoneNumber converted(eranos.internal.PhoneNumber number) {
        return new PhoneNumber(number.e164(), number.international());
    }

    /**
     * The number as the generated code holds it.
     *
     * @return the generated number
     */
    eranos.internal.PhoneNumber generated() {
        return new eranos.internal.PhoneNumber(e164, international);
    }

    /**
     * The number in E.164 form.
     *
     * @return the number: "+40712345678"
     */
    public String e164() {
        return e164;
    }

    /**
     * The number grouped as its country groups it.
     *
     * @return the text: "+40 712 345 678"
     */
    public String international() {
        return international;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof PhoneNumber number
                && e164.equals(number.e164)
                && international.equals(number.international);
    }

    @Override
    public int hashCode() {
        return Objects.hash(e164, international);
    }

    /**
     * The number grouped as its country groups it, as {@link #international} returns it.
     *
     * @return the text: "+40 712 345 678"
     */
    @Override
    public String toString() {
        return international;
    }
}
