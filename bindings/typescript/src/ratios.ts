/**
 * Exchange rates, percentages, and parts.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import type { Currency } from './currency.js';
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
    convert(amount: Money, currency: Currency): Money {
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
        calling(() => internal.percentageFormat(basisPoints));
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
        try {
            return internal.percentageFormat(this.basisPoints);
        } catch {
            return `${this.basisPoints.toString()} bp`;
        }
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
        calling(() => internal.partsFormat(hundredths));
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
        try {
            return internal.partsFormat(this.hundredths);
        } catch {
            return `${this.hundredths.toString()}/100`;
        }
    }
}
