package eranos;

import eranos.internal.CurrencyDetails;
import eranos.internal.Eranos;

import java.util.List;
import java.util.Objects;

/**
 * A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
 *
 * <p>A currency comes from the library: look one up by its code with {@link #of}, or pick one from
 * {@link #all}. An amount, the group currency, and a failure carry the details the library sends
 * with them, so a currency is never looked up by its code to show an amount.
 */
public final class CurrencyCode {
    private final String code;
    private final String name;
    private final int numericCode;
    private final int decimalPlaces;

    private CurrencyCode(String code, String name, int numericCode, int decimalPlaces) {
        this.code = code;
        this.name = name;
        this.numericCode = numericCode;
        this.decimalPlaces = decimalPlaces;
    }

    /** The library's list of currencies, read on first use. */
    private static final class Listed {
        private Listed() {}

        /** Every currency with minor units, by code. */
        static final List<CurrencyCode> ALL =
                Eranos.currencies().stream().map(CurrencyCode::converted).toList();
    }

    /**
     * Every currency with minor units, by code.
     *
     * @return the currencies
     */
    public static List<CurrencyCode> all() {
        return Listed.ALL;
    }

    /**
     * The currency with the ISO 4217 code {@code code}.
     *
     * @param code the three-letter code: "EUR"
     * @return the currency
     * @throws EranosException.Money with {@link MoneyFailure.UnknownCurrency} for a code that names
     *     no currency with minor units
     */
    public static CurrencyCode of(String code) throws EranosException {
        return Converting.call(() -> converted(Eranos.currencyDetails(code)));
    }

    /**
     * The currency the library describes as {@code details}, with the details as they are: its code
     * is not looked up.
     *
     * <p>The decimal places cross as a {@code byte}, which holds the library's unsigned 8-bit
     * number, so they are read unsigned.
     *
     * @param details the description
     * @return the currency
     */
    static CurrencyCode converted(CurrencyDetails details) {
        return new CurrencyCode(
                details.code(),
                details.name(),
                details.numericCode(),
                Byte.toUnsignedInt(details.decimalPlaces()));
    }

    /**
     * The currency as the generated code holds it.
     *
     * @return the generated details
     */
    CurrencyDetails generated() {
        return new CurrencyDetails(code, name, numericCode, (byte) decimalPlaces);
    }

    /**
     * The three-letter code.
     *
     * @return the code: "EUR"
     */
    public String code() {
        return code;
    }

    /**
     * The English name.
     *
     * @return the name: "Euro"
     */
    public String name() {
        return name;
    }

    /**
     * The three-digit numeric code.
     *
     * @return 978 for the euro
     */
    public int numericCode() {
        return numericCode;
    }

    /**
     * How many decimal places its minor unit has.
     *
     * @return 2 for the euro, 0 for the yen
     */
    public int decimalPlaces() {
        return decimalPlaces;
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof CurrencyCode currency
                && code.equals(currency.code)
                && name.equals(currency.name)
                && numericCode == currency.numericCode
                && decimalPlaces == currency.decimalPlaces;
    }

    @Override
    public int hashCode() {
        return Objects.hash(code, name, numericCode, decimalPlaces);
    }

    /**
     * The code.
     *
     * @return the code: "EUR"
     */
    @Override
    public String toString() {
        return code;
    }
}
