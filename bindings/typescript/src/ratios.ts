/**
 * Exchange rates, percentages, and parts.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import type { CurrencyCode } from './currency_code.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';

/** The rate that converts an amount in one currency into another: a positive decimal. */
export class ExchangeRate {
    private constructor(
        /** The rate as the library writes it. */
        private readonly text: string,
    ) {}

    /**
     * The rate written `text`: "4.97" or "0.2012".
     *
     * @param text - The rate's text
     * @returns The rate
     * @throws {@link MoneyError} with the failure `invalidExchangeRate` for text that is not a positive decimal with at
     *   most 18 decimal places
     */
    static parse(text: string): ExchangeRate {
        return new ExchangeRate(calling(() => internal.exchangeRateParse(text)));
    }

    /**
     * The rate of the number `value`: 4.9765 is "4.9765".
     *
     * The library writes the number in its shortest decimal form, rounds it half away from zero to 18 decimal places, and
     * parses it, so the rate keeps the digits of that form: 25.5 is "25.5", and `0.1 + 0.2` is "0.30000000000000004".
     *
     * @param value - The rate as a number
     * @returns The rate
     * @throws {@link MoneyError} with the failure `notFinite` for NaN and the infinities, and `invalidExchangeRate` for a
     *   number that is not positive at 18 decimal places or has more significant digits than fit in 64 bits
     */
    static fromNumber(value: number): ExchangeRate {
        return new ExchangeRate(calling(() => internal.exchangeRateFromF64(value)));
    }

    /**
     * The rate the library wrote as `text`.
     *
     * @internal
     */
    static known(text: string): ExchangeRate {
        return new ExchangeRate(text);
    }

    /**
     * `amount` converted at this rate into `currency`, rounded to its minor unit, halves away from zero.
     *
     * @param amount - The amount to convert
     * @param currency - The currency to convert it into
     * @returns The converted amount
     * @throws {@link MoneyError} with the failure `overflow` for a result out of range
     */
    convert(amount: Money, currency: CurrencyCode): Money {
        return Money.converted(calling(() => internal.exchangeRateConvert(this.text, amount.generated(), currency.code)));
    }

    /**
     * Whether `other` is the same rate.
     *
     * @param other - The rate to compare
     * @returns Whether the two are equal
     */
    equals(other: ExchangeRate): boolean {
        return this.text === other.text;
    }

    /**
     * The rate as the library writes it.
     *
     * @returns The text: "4.97"
     */
    toString(): string {
        return this.text;
    }
}

/** A share of a whole, to a hundredth of a per cent: 12.5 % is 1 250 basis points. */
export class Percentage {
    private constructor(
        /** The percentage in basis points: 1 250 for 12.5 %. */
        readonly basisPoints: number,
    ) {}

    /**
     * The percentage of `basisPoints` hundredths of a per cent.
     *
     * @param basisPoints - The percentage in basis points: 1 250 for 12.5 %
     * @returns The percentage
     * @throws {@link MoneyError} with the failure `invalidPercentage` for a negative number
     */
    static ofBasisPoints(basisPoints: number): Percentage {
        calling(() => {
            internal.percentageCheck(basisPoints);
        });
        return new Percentage(basisPoints);
    }

    /**
     * The percentage written `text`: "12.5", "12.5%", or "100".
     *
     * @param text - The percentage's text
     * @returns The percentage
     * @throws {@link MoneyError} with the failure `invalidPercentage` for text that is not a non-negative decimal with at
     *   most two decimal places
     */
    static parse(text: string): Percentage {
        return new Percentage(calling(() => internal.percentageParse(text)));
    }

    /**
     * The percentage of the number `value`, a number of per cent: 12.345 is 12.35 %.
     *
     * The library writes the number in its shortest decimal form, rounds it half away from zero to two decimal places, and
     * parses it: `0.1 + 0.2`, which is 0.30000000000000004 as a double, is 0.3 %.
     *
     * @param value - The percentage as a number of per cent
     * @returns The percentage
     * @throws {@link MoneyError} with the failure `notFinite` for NaN and the infinities, and `invalidPercentage` for a
     *   negative number and one above 21 474 836.47 %, whose basis points do not fit a 32-bit integer
     */
    static fromNumber(value: number): Percentage {
        return new Percentage(calling(() => internal.percentageFromF64(value)));
    }

    /**
     * The percentage of `basisPoints`, a number the library gave.
     *
     * @internal
     */
    static known(basisPoints: number): Percentage {
        return new Percentage(basisPoints);
    }

    /**
     * Whether `other` is the same percentage.
     *
     * @param other - The percentage to compare
     * @returns Whether the two are equal
     */
    equals(other: Percentage): boolean {
        return this.basisPoints === other.basisPoints;
    }

    /**
     * The percentage as the library writes it.
     *
     * @returns The text: "12.5%"
     */
    toString(): string {
        return internal.percentageFormat(this.basisPoints);
    }
}

/** A number of parts, to a hundredth: a member who pays half a part pays half of what a member with one part pays. */
export class Parts {
    private constructor(
        /** The parts in hundredths: 50 for half a part. */
        readonly hundredths: number,
    ) {}

    /**
     * The parts of `hundredths` hundredths.
     *
     * @param hundredths - The parts in hundredths: 50 for half a part
     * @returns The parts
     * @throws {@link MoneyError} with the failure `invalidParts` for a number that is not positive
     */
    static ofHundredths(hundredths: number): Parts {
        calling(() => {
            internal.partsCheck(hundredths);
        });
        return new Parts(hundredths);
    }

    /**
     * The parts written `text`: "1", "0.5", or "2.25".
     *
     * @param text - The parts' text
     * @returns The parts
     * @throws {@link MoneyError} with the failure `invalidParts` for text that is not a positive decimal with at most two
     *   decimal places
     */
    static parse(text: string): Parts {
        return new Parts(calling(() => internal.partsParse(text)));
    }

    /**
     * The parts of the number `value`: 1.005 is 1.01 parts.
     *
     * The library writes the number in its shortest decimal form, rounds it half away from zero to two decimal places, and
     * parses it.
     *
     * @param value - The parts as a number
     * @returns The parts
     * @throws {@link MoneyError} with the failure `notFinite` for NaN and the infinities, and `invalidParts` for a number that
     *   rounds to zero or below and one above 21 474 836.47 parts, whose hundredths do not fit a 32-bit integer
     */
    static fromNumber(value: number): Parts {
        return new Parts(calling(() => internal.partsFromF64(value)));
    }

    /**
     * The parts of `hundredths`, a number the library gave.
     *
     * @internal
     */
    static known(hundredths: number): Parts {
        return new Parts(hundredths);
    }

    /**
     * Whether `other` is the same number of parts.
     *
     * @param other - The parts to compare
     * @returns Whether the two are equal
     */
    equals(other: Parts): boolean {
        return this.hundredths === other.hundredths;
    }

    /**
     * The parts as the library writes them.
     *
     * @returns The text: "0.5"
     */
    toString(): string {
        return internal.partsFormat(this.hundredths);
    }
}
