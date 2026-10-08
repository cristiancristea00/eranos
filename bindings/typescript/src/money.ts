/**
 * Amounts of money.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import { CurrencyCode } from './currency_code.js';
import * as internal from './internal/eranos.js';

/**
 * An amount: a whole number of minor units of a currency, cents for the euro.
 *
 * An amount prints as the library writes it, "12.50 EUR", and adds and subtracts with its checks: both amounts in one
 * currency, and the result within range.
 */
export class Money {
    /**
     * The amount of `minorUnits` minor units of `currency`.
     *
     * @param minorUnits - The amount in minor units: cents for the euro, none for the yen
     * @param currency - The currency
     */
    constructor(
        /** The amount in minor units: cents for the euro, none for the yen. */
        readonly minorUnits: bigint,
        /** The currency. */
        readonly currency: CurrencyCode,
    ) {}

    /**
     * The amount written `text` in `currency`: "12.50", "12.5", or "-3".
     *
     * @param text - The amount's text
     * @param currency - The currency
     * @returns The amount
     * @throws {@link MoneyError} with the failure `invalidAmount` for text that is not a decimal number, and
     *   `tooManyDecimals` for more decimal places than the currency has
     */
    static parse(text: string, currency: CurrencyCode): Money {
        return Money.converted(calling(() => internal.moneyParse(text, currency.code)));
    }

    /**
     * The amount of the number `value` in `currency`: 1.005 in EUR is 1.01 EUR.
     *
     * The library writes the number in its shortest decimal form, rounds it half away from zero to the currency's decimal
     * places, and parses it, so every language gets the same amount from the same double. Rounding the text rather than the
     * number keeps the cents the user typed: the double nearest 1.005 lies below it.
     *
     * @param value - The amount as a number
     * @param currency - The currency
     * @returns The amount
     * @throws {@link MoneyError} with the failure `notFinite` for NaN and the infinities, and `overflow` for an amount that
     *   does not fit in 64-bit minor units
     */
    static fromNumber(value: number, currency: CurrencyCode): Money {
        return Money.converted(calling(() => internal.moneyFromF64(value, currency.code)));
    }

    /**
     * The amount the generated code holds as `money`.
     *
     * @internal
     */
    static converted(money: internal.Money): Money {
        return new Money(money.minorUnits, CurrencyCode.converted(money.currency));
    }

    /**
     * The amount as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.Money {
        return internal.Money.create({ minorUnits: this.minorUnits, currency: this.currency.generated() });
    }

    /**
     * This amount plus `other`.
     *
     * @param other - The amount to add
     * @returns The sum
     * @throws {@link MoneyError} with the failure `currencyMismatch` for amounts in two currencies, and `overflow` for a sum
     *   out of range
     */
    plus(other: Money): Money {
        return Money.converted(calling(() => internal.moneyAdd(this.generated(), other.generated())));
    }

    /**
     * This amount minus `other`.
     *
     * @param other - The amount to subtract
     * @returns The difference
     * @throws {@link MoneyError} with the failure `currencyMismatch` for amounts in two currencies, and `overflow` for a
     *   difference out of range
     */
    minus(other: Money): Money {
        return Money.converted(calling(() => internal.moneySubtract(this.generated(), other.generated())));
    }

    /**
     * Whether `other` is the same amount in the same currency.
     *
     * @param other - The amount to compare
     * @returns Whether the two are equal
     */
    equals(other: Money): boolean {
        return this.minorUnits === other.minorUnits && this.currency.code === other.currency.code;
    }

    /**
     * The amount as the library writes it.
     *
     * @returns The text: "12.50 EUR"
     */
    toString(): string {
        return internal.moneyFormat(this.generated());
    }
}
