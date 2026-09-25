/**
 * Amounts of money.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import { Currency } from './currency.js';
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
     * @param currency - The currency
     * @param minorUnits - The amount in minor units: cents for the euro, none for the yen
     */
    constructor(
        /** The currency. */
        readonly currency: Currency,
        /** The amount in minor units: cents for the euro, none for the yen. */
        readonly minorUnits: bigint,
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
    static parse(text: string, currency: Currency): Money {
        return Money.converted(calling(() => internal.moneyParse(text, currency.code)));
    }

    /**
     * The amount the generated code holds as `money`.
     *
     * @internal
     */
    static converted(money: internal.Money): Money {
        return new Money(Currency.known(money.currency), money.minorUnits);
    }

    /**
     * The amount as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.Money {
        return internal.Money.create({ currency: this.currency.code, minorUnits: this.minorUnits });
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
        return this.currency.code === other.currency.code && this.minorUnits === other.minorUnits;
    }

    /**
     * The amount as the library writes it.
     *
     * @returns The text: "12.50 EUR"
     */
    toString(): string {
        try {
            return internal.moneyFormat(this.generated());
        } catch {
            return `${this.minorUnits.toString()} ${this.currency.code}`;
        }
    }
}
