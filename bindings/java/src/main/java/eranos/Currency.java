package eranos;

import eranos.internal.CurrencyDetails;
import eranos.internal.Eranos;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.function.Function;
import java.util.stream.Collectors;

/**
 * A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
 *
 * <p>A currency comes from the library's list, so an amount in one always formats: look one up by
 * its code with {@link #of}, or pick one from {@link #all}.
 */
public final class Currency {
    private final String code;
    private final String name;
    private final int numericCode;
    private final int decimalPlaces;

    private Currency(String code, String name, int numericCode, int decimalPlaces) {
        this.code = code;
        this.name = name;
        this.numericCode = numericCode;
        this.decimalPlaces = decimalPlaces;
    }

    /** The library's list of currencies, read on first use. */
    private static final class Listed {
        private Listed() {}

        /** Every currency with minor units, by code. */
        static final List<Currency> ALL =
                Eranos.currencies().stream().map(Currency::converted).toList();

        /** The currencies of {@link #ALL} by their codes. */
        static final Map<String, Currency> BY_CODE =
                ALL.stream()
                        .collect(Collectors.toUnmodifiableMap(Currency::code, Function.identity()));
    }

    /**
     * Every currency with minor units, by code.
     *
     * @return the currencies
     */
    public static List<Currency> all() {
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
    public static Currency of(String code) throws EranosException {
        return Converting.call(() -> converted(Eranos.currencyDetails(code)));
    }

    /**
     * The currency of {@code code}, a code the library gave, so always one of {@link #all}; any
     * other code stands for itself, so that an amount in it keeps the code it came with.
     *
     * @param code the code
     * @return the currency
     */
    static Currency known(String code) {
        Currency listed = Listed.BY_CODE.get(code);
        return null == listed ? new Currency(code, code, 0, 0) : listed;
    }

    /**
     * The currency the library describes as {@code details}.
     *
     * @param details the description
     * @return the currency
     */
    static Currency converted(CurrencyDetails details) {
        return new Currency(
                details.code(), details.name(), details.numericCode(), details.decimalPlaces());
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
        return other instanceof Currency currency
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
