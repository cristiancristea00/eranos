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
 * A currency comes from the library's list, so an amount in one always formats: look one up by its code with
 * {@link CurrencyCode.of}, or pick one from {@link CurrencyCode.all}. Every lookup of a code returns the same currency, so
 * currencies compare with `===`.
 */
export class CurrencyCode {
    /** The currencies of {@link CurrencyCode.all} by their codes, read from the library on first use. */
    private static listed: ReadonlyMap<string, CurrencyCode> | undefined;

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
        return [...CurrencyCode.byCode().values()];
    }

    /**
     * The currency with the ISO 4217 code `code`.
     *
     * @param code - The three-letter code: "EUR"
     * @returns The currency
     * @throws {@link MoneyError} with the failure `unknownCurrency` for a code that names no currency with minor units
     */
    static of(code: string): CurrencyCode {
        const details = calling(() => internal.currencyDetails(code));
        return CurrencyCode.known(details.code);
    }

    /**
     * The currency of `code`, a code the library gave, so always one of {@link CurrencyCode.all}; any other code stands for
     * itself, so that an amount in it keeps the code it came with.
     *
     * @internal
     */
    static known(code: string): CurrencyCode {
        return CurrencyCode.byCode().get(code) ?? new CurrencyCode(code, code, 0, 0);
    }

    /** The currencies of the library's list by their codes. */
    private static byCode(): ReadonlyMap<string, CurrencyCode> {
        CurrencyCode.listed ??= new Map(
            internal
                .currencies()
                .map((details) => [details.code, new CurrencyCode(details.code, details.name, details.numericCode, details.decimalPlaces)]),
        );
        return CurrencyCode.listed;
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
