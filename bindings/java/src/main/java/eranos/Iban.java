package eranos;

import eranos.internal.Eranos;

import java.util.Objects;

/**
 * An international bank account number, checked against the SWIFT IBAN registry, with the country
 * code the library reads from it.
 */
public final class Iban {
    private final String text;
    private final String countryCode;

    private Iban(String text, String countryCode) {
        this.text = text;
        this.countryCode = countryCode;
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
        return Converting.call(
                () -> {
                    eranos.internal.Iban iban = Eranos.ibanParse(text);
                    return new Iban(iban.iban(), iban.countryCode());
                });
    }

    /**
     * The IBAN the library wrote as {@code text}, with the country code it gave.
     *
     * @param text the IBAN without spaces, in capitals
     * @param countryCode the two-letter country code
     * @return the IBAN
     */
    static Iban known(String text, String countryCode) {
        return new Iban(text, countryCode);
    }

    /**
     * The two-letter country code.
     *
     * @return the code: "RO"
     */
    public String countryCode() {
        return countryCode;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Iban iban
                && text.equals(iban.text)
                && countryCode.equals(iban.countryCode);
    }

    @Override
    public int hashCode() {
        return Objects.hash(text, countryCode);
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
