package eranos;

/**
 * The field of a record that the library derives from the record's other fields, and which a record
 * gave another way: what {@link EranosException.MismatchedRecord} names.
 */
public enum MismatchedField {
    /** The country code of an IBAN, which the library derives from the IBAN. */
    IBAN_COUNTRY_CODE,

    /** The international form of a phone number, which the library writes from its E.164 form. */
    PHONE_INTERNATIONAL,

    /**
     * The name, numeric code, or decimal places of a currency, which the library holds for its
     * code; a code in lower case names no currency the library lists either.
     */
    CURRENCY_DETAILS;

    /**
     * The field the generated code holds as {@code field}.
     *
     * @param field the generated field
     * @return the field
     */
    static MismatchedField converted(eranos.internal.MismatchedField field) {
        return switch (field) {
            case IBAN_COUNTRY_CODE -> IBAN_COUNTRY_CODE;
            case PHONE_INTERNATIONAL -> PHONE_INTERNATIONAL;
            case CURRENCY_DETAILS -> CURRENCY_DETAILS;
        };
    }
}
