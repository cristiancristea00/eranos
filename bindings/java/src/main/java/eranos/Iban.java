package eranos;

import eranos.internal.Eranos;

/** An international bank account number, checked against the SWIFT IBAN registry. */
public final class Iban {
    private final String text;

    private Iban(String text) {
        this.text = text;
    }

    /**
     * The IBAN written {@code text}, with or without spaces, in any letter case.
     *
     * @param text the IBAN's text
     * @return the IBAN
     * @throws EranosException.PaymentAccount with {@link PaymentAccountFailure.InvalidIban} and the
     *     check it failed
     */
    public static Iban parse(String text) throws EranosException {
        return Converting.call(() -> new Iban(Eranos.ibanParse(text)));
    }

    /**
     * The IBAN the library wrote as {@code text}.
     *
     * @param text the IBAN without spaces, in capitals
     * @return the IBAN
     */
    static Iban known(String text) {
        return new Iban(text);
    }

    /**
     * The two-letter country code.
     *
     * @return the code: "RO"
     */
    public String countryCode() {
        try {
            return Eranos.ibanCountryCode(text);
        } catch (eranos.internal.EranosException exception) {
            return text.substring(0, Math.min(2, text.length()));
        }
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Iban iban && text.equals(iban.text);
    }

    @Override
    public int hashCode() {
        return text.hashCode();
    }

    /**
     * The IBAN without spaces, in capitals.
     *
     * @return the text: "RO49AAAA1B31007593840000"
     */
    @Override
    public String toString() {
        return text;
    }
}
