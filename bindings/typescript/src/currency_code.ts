/**
 * The currencies an amount can be in.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import * as internal from './internal/eranos.js';

/**
 * A currency with minor units, as ISO 4217 lists it: the only currencies an amount can be in.
 *
 * A currency holds what the library says of it, so an amount in one formats without a lookup: look one up by its code with
 * {@link CurrencyCode.of}, or pick one from {@link CurrencyCode.all}. The layer keeps one object per code, which every lookup,
 * amount, failure, and group returns, so currencies compare with `===`.
 */
export class CurrencyCode {
    /** The currency of each code the layer has met, built from the library's details on the code's first use. */
    private static readonly interned = new Map<string, CurrencyCode>();

    private constructor(
        /** The three-letter code: "EUR". */
        readonly code: string,
        /** The English name: "Euro". */
        readonly name: string,
        /** The three-digit numeric code: 978 for the euro. */
        readonly numericCode: number,
        /** How many decimal places its minor unit has: 2 for the euro, 0 for the yen. */
        readonly decimalPlaces: number,
    ) {}

    /**
     * Every currency with minor units, by code.
     *
     * @returns The currencies
     */
    static all(): readonly CurrencyCode[] {
        return internal.currencies().map((details) => CurrencyCode.converted(details));
    }

    /**
     * The currency with the ISO 4217 code `code`.
     *
     * @param code - The three-letter code: "EUR"
     * @returns The currency
     * @throws {@link MoneyError} with the failure `unknownCurrency` for a code that names no currency with minor units
     */
    static of(code: string): CurrencyCode {
        return CurrencyCode.converted(calling(() => internal.currencyDetails(code)));
    }

    /**
     * The currency the generated code holds as `details`, details the library gave: the one the layer holds for its code, or,
     * on the code's first use, one built from them and kept for the code.
     *
     * @internal
     */
    static converted(details: internal.CurrencyDetails): CurrencyCode {
        const kept = CurrencyCode.interned.get(details.code);
        if (undefined !== kept) {
            return kept;
        }
        const currency = new CurrencyCode(details.code, details.name, details.numericCode, details.decimalPlaces);
        CurrencyCode.interned.set(details.code, currency);
        return currency;
    }

    /**
     * The currency as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.CurrencyDetails {
        return internal.CurrencyDetails.create({
            code: this.code,
            name: this.name,
            numericCode: this.numericCode,
            decimalPlaces: this.decimalPlaces,
        });
    }

    /**
     * The code.
     *
     * @returns The code: "EUR"
     */
    toString(): string {
        return this.code;
    }
}
