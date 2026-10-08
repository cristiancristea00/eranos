/**
 * Exchange rates, percentages, parts, and quantities.
 *
 * @packageDocumentation
 */

import { calling, checked32Bits, checked64Bits } from './converting.js';
import type { CurrencyCode } from './currency_code.js';
import { MoneyError } from './errors.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';

/** The library's refusal of a percentage, in its words, which a test holds to the core's. */
function invalidPercentage(): MoneyError {
    return new MoneyError({ kind: 'invalidPercentage' }, 'invalid percentage: expected a number with at most 2 decimal places');
}

/** The library's refusal of a number of parts, in its words, which a test holds to the core's. */
function invalidParts(): MoneyError {
    return new MoneyError({ kind: 'invalidParts' }, 'invalid parts: expected a positive number with at most 2 decimal places');
}

/** The library's refusal of a quantity, in its words, which a test holds to the core's. */
function invalidQuantity(): MoneyError {
    return new MoneyError({ kind: 'invalidQuantity' }, 'invalid quantity: expected a positive number with at most 4 decimal places');
}

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
     * @throws {@link MoneyError} with the failure `invalidPercentage` for a number that is negative, is not an integer, or
     *   does not fit a 32-bit integer
     */
    static ofBasisPoints(basisPoints: number): Percentage {
        calling(() => {
            internal.percentageCheck(checked32Bits(basisPoints, invalidPercentage));
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
     * @throws {@link MoneyError} with the failure `invalidParts` for a number that is not positive, is not an integer, or does
     *   not fit a 32-bit integer
     */
    static ofHundredths(hundredths: number): Parts {
        calling(() => {
            internal.partsCheck(checked32Bits(hundredths, invalidParts));
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

/** A number of units of a line of a bill, to a ten-thousandth: 0.452 kg of cheese is 4 520 ten-thousandths. */
export class Quantity {
    /** One unit, which a line of a bill holds unless it is given another quantity. */
    // The literal, since a module constant is evaluated before `init` loads the WebAssembly module; a test holds it to the
    // library's.
    static readonly ONE: Quantity = new Quantity(10_000n);

    private constructor(
        /** The quantity in ten-thousandths: 10 000 for one unit, 4 520 for 0.452. */
        readonly tenThousandths: bigint,
    ) {}

    /**
     * The quantity of `tenThousandths` ten-thousandths.
     *
     * @param tenThousandths - The quantity in ten-thousandths: 10 000 for one unit, 4 520 for 0.452
     * @returns The quantity
     * @throws {@link MoneyError} with the failure `invalidQuantity` for a number that is not positive or does not fit in 64
     *   bits
     */
    static ofTenThousandths(tenThousandths: bigint): Quantity {
        calling(() => {
            internal.quantityCheck(checked64Bits(tenThousandths, invalidQuantity));
        });
        return new Quantity(tenThousandths);
    }

    /**
     * The quantity of `units` whole units: 3 is 30 000 ten-thousandths.
     *
     * @param units - The number of whole units
     * @returns The quantity
     * @throws {@link MoneyError} with the failure `invalidQuantity` for a number that is not positive and one whose
     *   ten-thousandths do not fit in 64 bits
     */
    static ofWhole(units: bigint): Quantity {
        return new Quantity(calling(() => internal.quantityFromWhole(checked64Bits(units, invalidQuantity))));
    }

    /**
     * The quantity written `text`: "1", "0.452", or "2.5".
     *
     * @param text - The quantity's text
     * @returns The quantity
     * @throws {@link MoneyError} with the failure `invalidQuantity` for text that is not a positive decimal with at most four
     *   decimal places, and for a quantity whose ten-thousandths do not fit in 64 bits
     */
    static parse(text: string): Quantity {
        return new Quantity(calling(() => internal.quantityParse(text)));
    }

    /**
     * The quantity of the number `value`: 0.45205 is 0.4521.
     *
     * The library writes the number in its shortest decimal form, rounds it half away from zero to four decimal places, and
     * parses it.
     *
     * @param value - The quantity as a number
     * @returns The quantity
     * @throws {@link MoneyError} with the failure `notFinite` for NaN and the infinities, and `invalidQuantity` for a number
     *   that rounds to zero or below and one whose ten-thousandths do not fit in 64 bits
     */
    static fromNumber(value: number): Quantity {
        return new Quantity(calling(() => internal.quantityFromF64(value)));
    }

    /**
     * The quantity of `tenThousandths`, a number the library gave.
     *
     * @internal
     */
    static known(tenThousandths: bigint): Quantity {
        return new Quantity(tenThousandths);
    }

    /**
     * Whether `other` is the same quantity.
     *
     * @param other - The quantity to compare
     * @returns Whether the two are equal
     */
    equals(other: Quantity): boolean {
        return this.tenThousandths === other.tenThousandths;
    }

    /**
     * The quantity as the library writes it.
     *
     * @returns The text: "0.452"
     */
    toString(): string {
        return internal.quantityFormat(this.tenThousandths);
    }
}
