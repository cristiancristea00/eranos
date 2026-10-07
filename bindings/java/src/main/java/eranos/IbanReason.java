package eranos;

/** Why the library refused an IBAN: a type per check it failed. */
public sealed interface IbanReason {
    /** A character other than an ASCII letter, a digit, or whitespace (which is dropped). */
    record InvalidCharacter() implements IbanReason {}

    /** Not two letters followed by two check digits. */
    record InvalidFormat() implements IbanReason {}

    /** A country code without an entry in the SWIFT IBAN registry. */
    record UnknownCountry() implements IbanReason {}

    /**
     * The wrong number of characters for the country.
     *
     * @param expected how many characters the country's IBANs have
     * @param found how many characters the IBAN has
     */
    record WrongLength(long expected, long found) implements IbanReason {
        /**
         * The failure of these components.
         *
         * @param expected how many characters the country's IBANs have
         * @param found how many characters the IBAN has
         */
        public WrongLength {}
    }

    /**
     * A letter where the country's account number has a digit, or a digit where it has a letter, by
     * the BBAN structure of the SWIFT IBAN registry.
     */
    record InvalidStructure() implements IbanReason {}

    /**
     * The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97
     * check failed: the number contains a typo.
     */
    record InvalidChecksum() implements IbanReason {}
}
