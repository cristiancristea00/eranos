/**
 * Why a call failed: {@link EranosError} and a subclass per module of the library, each holding what the module
 * reports, with the library's message.
 *
 * @packageDocumentation
 */

import { CurrencyCode } from './currency_code.js';
import { type CategoryId, type ExpenseId, type MemberId, type TransferId, categoryId, expenseId, memberId, transferId } from './identifiers.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';

/**
 * Why a call failed: the base of a subclass per module of the library, holding what the module reports, with the
 * library's message.
 *
 * Every call of the package that the library refuses throws a subclass, so an `instanceof` takes a module's failures
 * together, and a `switch` over its `failure.kind` takes one of them:
 *
 * ```ts
 * try {
 *     group.removeMemberSync(member);
 * } catch (error: unknown) {
 *     if (!(error instanceof ExpenseGroupError) || error.failure.kind !== 'memberInUse') {
 *         throw error;
 *     }
 *     group.archiveMemberSync(member); // The member still has entries; archive them instead.
 * }
 * ```
 *
 * A call the program should not make throws a `TypeError` instead: a read through an {@link ExpenseGroupReader} after its
 * release.
 */
export abstract class EranosError extends Error {}

/** The library's `IdError`, with the message it words. */
export class IdError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: IdFailure,
        message: string,
    ) {
        super(message);
        this.name = 'IdError';
    }
}

/** The library's `PaymentAccountError`, with the message it words. */
export class PaymentAccountError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: PaymentAccountFailure,
        message: string,
    ) {
        super(message);
        this.name = 'PaymentAccountError';
    }
}

/** The library's `CalendarError`, with the message it words. */
export class CalendarError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: CalendarFailure,
        message: string,
    ) {
        super(message);
        this.name = 'CalendarError';
    }
}

/** The library's `CategoryError`, with the message it words. */
export class CategoryError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: CategoryFailure,
        message: string,
    ) {
        super(message);
        this.name = 'CategoryError';
    }
}

/** The library's `ContactError`, with the message it words. */
export class ContactError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: ContactFailure,
        message: string,
    ) {
        super(message);
        this.name = 'ContactError';
    }
}

/** The library's `MoneyError`, with the message it words. */
export class MoneyError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: MoneyFailure,
        message: string,
    ) {
        super(message);
        this.name = 'MoneyError';
    }
}

/** The library's `MemberError`, with the message it words. */
export class MemberError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: MemberFailure,
        message: string,
    ) {
        super(message);
        this.name = 'MemberError';
    }
}

/** The library's `ExpenseError`, with the message it words. */
export class ExpenseError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: ExpenseFailure,
        message: string,
    ) {
        super(message);
        this.name = 'ExpenseError';
    }
}

/** The library's `SettlementError`, with the message it words. */
export class SettlementError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: SettlementFailure,
        message: string,
    ) {
        super(message);
        this.name = 'SettlementError';
    }
}

/** The library's `TransferError`, with the message it words. */
export class TransferError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: TransferFailure,
        message: string,
    ) {
        super(message);
        this.name = 'TransferError';
    }
}

/** The library's `ExpenseGroupError`, with the message it words. */
export class ExpenseGroupError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: ExpenseGroupFailure,
        message: string,
    ) {
        super(message);
        this.name = 'ExpenseGroupError';
    }
}

/** The library's `LanguageError`, with the message it words. */
export class LanguageError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: LanguageFailure,
        message: string,
    ) {
        super(message);
        this.name = 'LanguageError';
    }
}

/**
 * A saved group that is not JSON of the saved form, at a line and a column; the message names the fault but never the text
 * there, which can hold an account number.
 */
export class InvalidJsonError extends EranosError {
    /**
     * The error of the fault at `line` and `column`, which the library words as `message`.
     *
     * @param line - The line of the fault, from 1
     * @param column - The column of the fault, from 1
     * @param message - The library's message
     */
    constructor(
        /** The line of the fault, from 1. */
        readonly line: number,
        /** The column of the fault, from 1. */
        readonly column: number,
        message: string,
    ) {
        super(message);
        this.name = 'InvalidJsonError';
    }
}

/** What the library's `IdError` reports: a type per failure, with what it holds. */
export type IdFailure =
    | {
          /** Empty text. */
          readonly kind: 'empty';
      }
    | {
          /** Text longer than 64 bytes of UTF-8. */
          readonly kind: 'tooLong';
      }
    | {
          /** Text that holds a control character, such as a tab or a line break. */
          readonly kind: 'controlCharacter';
      }
    | {
          /** Text that starts or ends with whitespace. */
          readonly kind: 'surroundingWhitespace';
      };

/** What the library's `PaymentAccountError` reports: a type per failure, with what it holds. */
export type PaymentAccountFailure =
    | {
          /** An account whose name is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /** An IBAN that failed validation, and why. */
          readonly kind: 'invalidIban';
          /** The check the IBAN failed. */
          readonly reason: IbanReason;
      }
    | {
          /** A username that is blank. */
          readonly kind: 'emptyUsername';
      };

/** What the library's `CalendarError` reports: a type per failure, with what it holds. */
export type CalendarFailure =
    | {
          /** A calendar date that does not exist, is outside years 1 to 9999, or is not ISO 8601 text. */
          readonly kind: 'invalidDate';
      }
    | {
          /** A time of day that does not exist, has seconds, or is not ISO 8601 text. */
          readonly kind: 'invalidTime';
      };

/** What the library's `CategoryError` reports: a type per failure, with what it holds. */
export interface CategoryFailure {
    /** A name that is blank. */
    readonly kind: 'emptyName';
}

/** What the library's `ContactError` reports: a type per failure, with what it holds. */
export type ContactFailure =
    | {
          /**
           * An email address that breaks the RFC 5321 and 5322 rules: no local part or domain around an `@`, a malformed
           * one, one too long, or whitespace or an invisible character.
           */
          readonly kind: 'invalidEmail';
      }
    | {
          /**
           * A phone number that does not start with `+`, holds a letter or an extension, or is not valid in its country's
           * numbering plan.
           */
          readonly kind: 'invalidPhoneNumber';
      };

/** What the library's `MoneyError` reports: a type per failure, with what it holds. */
export type MoneyFailure =
    | {
          /** An amount that is not a decimal number. */
          readonly kind: 'invalidAmount';
      }
    | {
          /** An amount with more decimal places than its currency has. */
          readonly kind: 'tooManyDecimals';
          /** The currency. */
          readonly currency: CurrencyCode;
          /** How many decimal places the currency has. */
          readonly allowed: number;
      }
    | {
          /** Arithmetic between amounts of two different currencies. */
          readonly kind: 'currencyMismatch';
          /** The currency of the first amount. */
          readonly left: CurrencyCode;
          /** The currency of the second amount. */
          readonly right: CurrencyCode;
      }
    | {
          /**
           * A currency the library does not list: a code that is not an active ISO 4217 code with minor units, or an amount's
           * currency that holds other details than the library's for its code.
           */
          readonly kind: 'unknownCurrency';
      }
    | {
          /** An exchange rate that is not a positive decimal number with at most 18 decimal places. */
          readonly kind: 'invalidExchangeRate';
      }
    | {
          /** A number of parts that is not a positive decimal number with at most two decimal places. */
          readonly kind: 'invalidParts';
      }
    | {
          /** A percentage that is not a decimal number with at most two decimal places. */
          readonly kind: 'invalidPercentage';
      }
    | {
          /**
           * A quantity that is not a positive decimal number with at most four decimal places, or whose ten-thousandths do not
           * fit in 64 bits.
           */
          readonly kind: 'invalidQuantity';
      }
    | {
          /** A number that is NaN or infinite, which no amount, percentage, number of parts, quantity, or exchange rate is. */
          readonly kind: 'notFinite';
      }
    | {
          /** A number or a result that does not fit its range. */
          readonly kind: 'overflow';
      };

/** What the library's `MemberError` reports: a type per failure, with what it holds. */
export interface MemberFailure {
    /** A first name that is blank. */
    readonly kind: 'emptyFirstName';
}

/** What the library's `ExpenseError` reports: a type per failure, with what it holds. */
export type ExpenseFailure =
    | {
          /** An expense whose name is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /** The amount or unit price is zero or negative. */
          readonly kind: 'nonPositiveAmount';
      }
    | {
          /** An amount inside the expense is in a different currency. */
          readonly kind: 'currencyMismatch';
          /** The expense's currency. */
          readonly expected: CurrencyCode;
          /** The currency of the amount inside it. */
          readonly found: CurrencyCode;
      }
    | {
          /** A split lists no member. */
          readonly kind: 'emptySplit';
      }
    | {
          /** A member appears more than once in one split. */
          readonly kind: 'duplicateMember';
          /** The member who appears more than once. */
          readonly member: MemberId;
      }
    | {
          /** A member's percentage or amount is zero or negative. */
          readonly kind: 'nonPositiveShare';
          /** The member whose share is zero or negative. */
          readonly member: MemberId;
      }
    | {
          /** The amounts of a split do not add up to the amount being split. */
          readonly kind: 'amountsDoNotAddUp';
          /** The amount being split. */
          readonly expected: Money;
          /** What the amounts add up to. */
          readonly actual: Money;
      }
    | {
          /** The percentages of a split do not add up to exactly 100 %. */
          readonly kind: 'percentagesDoNotAddUp';
          /** What the percentages add up to, in basis points. */
          readonly totalBasisPoints: number;
      }
    | {
          /** An itemised expense without items. */
          readonly kind: 'noItems';
      }
    | {
          /** An item whose name is blank. */
          readonly kind: 'emptyItemName';
      }
    | {
          /** The item line totals do not add up to the expense amount. */
          readonly kind: 'itemsDoNotAddUp';
          /** The expense amount. */
          readonly expected: Money;
          /** What the line totals add up to. */
          readonly actual: Money;
      }
    | {
          /** A fixed discount below zero. */
          readonly kind: 'negativeDiscount';
      }
    | {
          /** A percentage discount above 100 %. */
          readonly kind: 'discountAboveHundredPercent';
      }
    | {
          /**
           * A fixed discount larger than what it reduces: the line total before the discount, or the unit price for a
           * discount off each unit.
           */
          readonly kind: 'discountAbovePrice';
      }
    | {
          /** A fixed tip below zero. */
          readonly kind: 'negativeTip';
      }
    | {
          /** A total that does not fit the range of an amount. */
          readonly kind: 'overflow';
      };

/** What the library's `SettlementError` reports: a type per failure, with what it holds. */
export interface SettlementFailure {
    /**
     * The exact solver needs 2ⁿ bytes for `n` open balances, and that table does not fit in the address space or the
     * allocator refused it.
     */
    readonly kind: 'outOfMemory';
    /** How many open balances the plan has. */
    readonly balances: number;
}

/** What the library's `TransferError` reports: a type per failure, with what it holds. */
export type TransferFailure =
    | {
          /** The sender and the recipient are the same member. */
          readonly kind: 'sameMember';
      }
    | {
          /** The amount is zero or negative. */
          readonly kind: 'nonPositiveAmount';
      };

/** What the library's `ExpenseGroupError` reports: a type per failure, with what it holds. */
export type ExpenseGroupFailure =
    | {
          /**
           * A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier
           * one.
           */
          readonly kind: 'unsupportedFormat';
          /** The format the saved group names. */
          readonly format: number;
      }
    | {
          /** A group name that is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /**
           * An entry in another currency than the group's has no exchange rate into the group currency: its cost is local,
           * or a currency change gave no rate for its currency.
           */
          readonly kind: 'missingExchangeRate';
          /** The currency without a rate. */
          readonly currency: CurrencyCode;
      }
    | {
          /** An entry in the group currency has a converted cost; only a local cost is in the group currency. */
          readonly kind: 'unexpectedExchangeRate';
      }
    | {
          /** The same currency appears twice among the rates of a currency change. */
          readonly kind: 'duplicateExchangeRate';
          /** The currency that appears twice. */
          readonly currency: CurrencyCode;
      }
    | {
          /** A member with this ID already exists. */
          readonly kind: 'duplicateMember';
          /** The identifier another member has. */
          readonly member: MemberId;
      }
    | {
          /** No member with this ID. */
          readonly kind: 'unknownMember';
          /** The identifier no member has. */
          readonly member: MemberId;
      }
    | {
          /** An archived member cannot join a new expense. */
          readonly kind: 'archivedMember';
          /** The archived member. */
          readonly member: MemberId;
      }
    | {
          /** The member is still referenced and cannot be removed; archive it instead. */
          readonly kind: 'memberInUse';
          /** The member. */
          readonly member: MemberId;
          /** The expenses that name the member. */
          readonly expenses: readonly ExpenseId[];
          /** The transfers that name the member. */
          readonly transfers: readonly TransferId[];
      }
    | {
          /** A category with this ID already exists. */
          readonly kind: 'duplicateCategory';
          /** The identifier another category has. */
          readonly category: CategoryId;
      }
    | {
          /** Another category already has this name, compared by Unicode canonical caseless matching. */
          readonly kind: 'duplicateCategoryName';
          /** The category that has the name. */
          readonly existing: CategoryId;
      }
    | {
          /** No category with this ID. */
          readonly kind: 'unknownCategory';
          /** The identifier no category has. */
          readonly category: CategoryId;
      }
    | {
          /** The built-in Uncategorised category cannot be renamed or removed. */
          readonly kind: 'builtInCategory';
      }
    | {
          /** An expense with this ID already exists. */
          readonly kind: 'duplicateExpense';
          /** The identifier another expense has. */
          readonly expense: ExpenseId;
      }
    | {
          /** No expense with this ID. */
          readonly kind: 'unknownExpense';
          /** The identifier no expense has. */
          readonly expense: ExpenseId;
      }
    | {
          /** A transfer with this ID already exists. */
          readonly kind: 'duplicateTransfer';
          /** The identifier another transfer has. */
          readonly transfer: TransferId;
      }
    | {
          /** No transfer with this ID. */
          readonly kind: 'unknownTransfer';
          /** The identifier no transfer has. */
          readonly transfer: TransferId;
      }
    | {
          /** A shared group's lock was asked for without waiting, while another call held it. */
          readonly kind: 'busy';
      }
    | {
          /** A total would leave the range of an amount. */
          readonly kind: 'overflow';
      };

/** What the library's `LanguageError` reports: a type per failure, with what it holds. */
export interface LanguageFailure {
    /** A language tag that is not a Unicode BCP 47 locale identifier (UTS #35). */
    readonly kind: 'invalidLanguage';
}

/** Why the library refused an IBAN: a type per check it failed. */
export type IbanReason =
    | {
          /** A character other than an ASCII letter, a digit, or whitespace (which is dropped). */
          readonly kind: 'invalidCharacter';
      }
    | {
          /** Not two letters followed by two check digits. */
          readonly kind: 'invalidFormat';
      }
    | {
          /** A country code without an entry in the SWIFT IBAN registry. */
          readonly kind: 'unknownCountry';
      }
    | {
          /** The wrong number of characters for the country. */
          readonly kind: 'wrongLength';
          /** How many characters the country's IBANs have. */
          readonly expected: number;
          /** How many characters the IBAN has. */
          readonly found: number;
      }
    | {
          /**
           * A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN
           * structure of the SWIFT IBAN registry.
           */
          readonly kind: 'invalidStructure';
      }
    | {
          /**
           * The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the
           * number contains a typo.
           */
          readonly kind: 'invalidChecksum';
      };

/**
 * The error this package throws for `error`, which the generated code threw: the library's failure as an
 * {@link EranosError}, a call the program should not make as a `TypeError`, a cancelled call as an `AbortError`, and
 * anything else as it is.
 *
 * @internal
 */
export function convertedError(error: unknown): unknown {
    if (!internal.EranosError.instanceOf(error)) {
        return error;
    }
    switch (error.tag) {
        case internal.EranosError_Tags.Id:
            return new IdError(convertedIdFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.PaymentAccount:
            return new PaymentAccountError(convertedPaymentAccountFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Calendar:
            return new CalendarError(convertedCalendarFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Category:
            return new CategoryError(convertedCategoryFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Contact:
            return new ContactError(convertedContactFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Money:
            return new MoneyError(convertedMoneyFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Member:
            return new MemberError(convertedMemberFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Expense:
            return new ExpenseError(convertedExpenseFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Settlement:
            return new SettlementError(convertedSettlementFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Transfer:
            return new TransferError(convertedTransferFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.ExpenseGroup:
            return new ExpenseGroupError(convertedGroupFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Language:
            return new LanguageError(convertedLanguageFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.InvalidJson:
            return new InvalidJsonError(Number(error.inner.line), Number(error.inner.column), error.inner.detail);
        case internal.EranosError_Tags.Cancelled:
            return new DOMException(error.inner.detail, 'AbortError');
        case internal.EranosError_Tags.Released:
            return new TypeError(error.inner.detail);
    }
}

/** The failure the generated code reports as `failure`. */
function convertedIdFailure(failure: internal.IdFailure): IdFailure {
    switch (failure) {
        case internal.IdFailure.Empty:
            return { kind: 'empty' };
        case internal.IdFailure.TooLong:
            return { kind: 'tooLong' };
        case internal.IdFailure.ControlCharacter:
            return { kind: 'controlCharacter' };
        case internal.IdFailure.SurroundingWhitespace:
            return { kind: 'surroundingWhitespace' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedPaymentAccountFailure(failure: internal.PaymentAccountFailure): PaymentAccountFailure {
    switch (failure.tag) {
        case internal.PaymentAccountFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.PaymentAccountFailure_Tags.InvalidIban:
            return { kind: 'invalidIban', reason: convertedIbanReason(failure.inner.reason) };
        case internal.PaymentAccountFailure_Tags.EmptyUsername:
            return { kind: 'emptyUsername' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedCalendarFailure(failure: internal.CalendarFailure): CalendarFailure {
    switch (failure) {
        case internal.CalendarFailure.InvalidDate:
            return { kind: 'invalidDate' };
        case internal.CalendarFailure.InvalidTime:
            return { kind: 'invalidTime' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedCategoryFailure(failure: internal.CategoryFailure): CategoryFailure {
    switch (failure) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.CategoryFailure.EmptyName:
            return { kind: 'emptyName' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedContactFailure(failure: internal.ContactFailure): ContactFailure {
    switch (failure) {
        case internal.ContactFailure.InvalidEmail:
            return { kind: 'invalidEmail' };
        case internal.ContactFailure.InvalidPhoneNumber:
            return { kind: 'invalidPhoneNumber' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedMoneyFailure(failure: internal.MoneyFailure): MoneyFailure {
    switch (failure.tag) {
        case internal.MoneyFailure_Tags.InvalidAmount:
            return { kind: 'invalidAmount' };
        case internal.MoneyFailure_Tags.TooManyDecimals:
            return { kind: 'tooManyDecimals', currency: CurrencyCode.converted(failure.inner.currency), allowed: Number(failure.inner.allowed) };
        case internal.MoneyFailure_Tags.CurrencyMismatch:
            return { kind: 'currencyMismatch', left: CurrencyCode.converted(failure.inner.left), right: CurrencyCode.converted(failure.inner.right) };
        case internal.MoneyFailure_Tags.UnknownCurrency:
            return { kind: 'unknownCurrency' };
        case internal.MoneyFailure_Tags.InvalidExchangeRate:
            return { kind: 'invalidExchangeRate' };
        case internal.MoneyFailure_Tags.InvalidParts:
            return { kind: 'invalidParts' };
        case internal.MoneyFailure_Tags.InvalidPercentage:
            return { kind: 'invalidPercentage' };
        case internal.MoneyFailure_Tags.InvalidQuantity:
            return { kind: 'invalidQuantity' };
        case internal.MoneyFailure_Tags.NotFinite:
            return { kind: 'notFinite' };
        case internal.MoneyFailure_Tags.Overflow:
            return { kind: 'overflow' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedMemberFailure(failure: internal.MemberFailure): MemberFailure {
    switch (failure) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.MemberFailure.EmptyFirstName:
            return { kind: 'emptyFirstName' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedExpenseFailure(failure: internal.ExpenseFailure): ExpenseFailure {
    switch (failure.tag) {
        case internal.ExpenseFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.ExpenseFailure_Tags.NonPositiveAmount:
            return { kind: 'nonPositiveAmount' };
        case internal.ExpenseFailure_Tags.CurrencyMismatch:
            return {
                kind: 'currencyMismatch',
                expected: CurrencyCode.converted(failure.inner.expected),
                found: CurrencyCode.converted(failure.inner.found),
            };
        case internal.ExpenseFailure_Tags.EmptySplit:
            return { kind: 'emptySplit' };
        case internal.ExpenseFailure_Tags.DuplicateMember:
            return { kind: 'duplicateMember', member: memberId(failure.inner.member) };
        case internal.ExpenseFailure_Tags.NonPositiveShare:
            return { kind: 'nonPositiveShare', member: memberId(failure.inner.member) };
        case internal.ExpenseFailure_Tags.AmountsDoNotAddUp:
            return { kind: 'amountsDoNotAddUp', expected: Money.converted(failure.inner.expected), actual: Money.converted(failure.inner.actual) };
        case internal.ExpenseFailure_Tags.PercentagesDoNotAddUp:
            return { kind: 'percentagesDoNotAddUp', totalBasisPoints: Number(failure.inner.totalBasisPoints) };
        case internal.ExpenseFailure_Tags.NoItems:
            return { kind: 'noItems' };
        case internal.ExpenseFailure_Tags.EmptyItemName:
            return { kind: 'emptyItemName' };
        case internal.ExpenseFailure_Tags.ItemsDoNotAddUp:
            return { kind: 'itemsDoNotAddUp', expected: Money.converted(failure.inner.expected), actual: Money.converted(failure.inner.actual) };
        case internal.ExpenseFailure_Tags.NegativeDiscount:
            return { kind: 'negativeDiscount' };
        case internal.ExpenseFailure_Tags.DiscountAboveHundredPercent:
            return { kind: 'discountAboveHundredPercent' };
        case internal.ExpenseFailure_Tags.DiscountAbovePrice:
            return { kind: 'discountAbovePrice' };
        case internal.ExpenseFailure_Tags.NegativeTip:
            return { kind: 'negativeTip' };
        case internal.ExpenseFailure_Tags.Overflow:
            return { kind: 'overflow' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedSettlementFailure(failure: internal.SettlementFailure): SettlementFailure {
    switch (failure.tag) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.SettlementFailure_Tags.OutOfMemory:
            return { kind: 'outOfMemory', balances: Number(failure.inner.balances) };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedTransferFailure(failure: internal.TransferFailure): TransferFailure {
    switch (failure) {
        case internal.TransferFailure.SameMember:
            return { kind: 'sameMember' };
        case internal.TransferFailure.NonPositiveAmount:
            return { kind: 'nonPositiveAmount' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedGroupFailure(failure: internal.ExpenseGroupFailure): ExpenseGroupFailure {
    switch (failure.tag) {
        case internal.ExpenseGroupFailure_Tags.UnsupportedFormat:
            return { kind: 'unsupportedFormat', format: Number(failure.inner.format) };
        case internal.ExpenseGroupFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.ExpenseGroupFailure_Tags.MissingExchangeRate:
            return { kind: 'missingExchangeRate', currency: CurrencyCode.converted(failure.inner.currency) };
        case internal.ExpenseGroupFailure_Tags.UnexpectedExchangeRate:
            return { kind: 'unexpectedExchangeRate' };
        case internal.ExpenseGroupFailure_Tags.DuplicateExchangeRate:
            return { kind: 'duplicateExchangeRate', currency: CurrencyCode.converted(failure.inner.currency) };
        case internal.ExpenseGroupFailure_Tags.DuplicateMember:
            return { kind: 'duplicateMember', member: memberId(failure.inner.member) };
        case internal.ExpenseGroupFailure_Tags.UnknownMember:
            return { kind: 'unknownMember', member: memberId(failure.inner.member) };
        case internal.ExpenseGroupFailure_Tags.ArchivedMember:
            return { kind: 'archivedMember', member: memberId(failure.inner.member) };
        case internal.ExpenseGroupFailure_Tags.MemberInUse:
            return {
                kind: 'memberInUse',
                member: memberId(failure.inner.member),
                expenses: failure.inner.expenses.map(expenseId),
                transfers: failure.inner.transfers.map(transferId),
            };
        case internal.ExpenseGroupFailure_Tags.DuplicateCategory:
            return { kind: 'duplicateCategory', category: categoryId(failure.inner.category) };
        case internal.ExpenseGroupFailure_Tags.DuplicateCategoryName:
            return { kind: 'duplicateCategoryName', existing: categoryId(failure.inner.existing) };
        case internal.ExpenseGroupFailure_Tags.UnknownCategory:
            return { kind: 'unknownCategory', category: categoryId(failure.inner.category) };
        case internal.ExpenseGroupFailure_Tags.BuiltInCategory:
            return { kind: 'builtInCategory' };
        case internal.ExpenseGroupFailure_Tags.DuplicateExpense:
            return { kind: 'duplicateExpense', expense: expenseId(failure.inner.expense) };
        case internal.ExpenseGroupFailure_Tags.UnknownExpense:
            return { kind: 'unknownExpense', expense: expenseId(failure.inner.expense) };
        case internal.ExpenseGroupFailure_Tags.DuplicateTransfer:
            return { kind: 'duplicateTransfer', transfer: transferId(failure.inner.transfer) };
        case internal.ExpenseGroupFailure_Tags.UnknownTransfer:
            return { kind: 'unknownTransfer', transfer: transferId(failure.inner.transfer) };
        case internal.ExpenseGroupFailure_Tags.Busy:
            return { kind: 'busy' };
        case internal.ExpenseGroupFailure_Tags.Overflow:
            return { kind: 'overflow' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedLanguageFailure(failure: internal.LanguageFailure): LanguageFailure {
    switch (failure) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.LanguageFailure.InvalidLanguage:
            return { kind: 'invalidLanguage' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedIbanReason(failure: internal.IbanReason): IbanReason {
    switch (failure.tag) {
        case internal.IbanReason_Tags.InvalidCharacter:
            return { kind: 'invalidCharacter' };
        case internal.IbanReason_Tags.InvalidFormat:
            return { kind: 'invalidFormat' };
        case internal.IbanReason_Tags.UnknownCountry:
            return { kind: 'unknownCountry' };
        case internal.IbanReason_Tags.WrongLength:
            return { kind: 'wrongLength', expected: Number(failure.inner.expected), found: Number(failure.inner.found) };
        case internal.IbanReason_Tags.InvalidStructure:
            return { kind: 'invalidStructure' };
        case internal.IbanReason_Tags.InvalidChecksum:
            return { kind: 'invalidChecksum' };
    }
}
