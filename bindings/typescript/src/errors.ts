/**
 * Why a call failed: {@link EranosError} and a subclass per module of the library, each holding what the module
 * reports, with the library's message.
 *
 * @packageDocumentation
 */

import { Currency } from './currency.js';
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
 *     if (!(error instanceof GroupError) || error.failure.kind !== 'memberInUse') {
 *         throw error;
 *     }
 *     group.archiveMemberSync(member); // The member still has entries; archive them instead.
 * }
 * ```
 *
 * A call the program should not make throws a `TypeError` instead: for an entry to add that has an identifier, for one
 * to replace that has none, and for a read through a {@link GroupReader} after its release.
 */
export abstract class EranosError extends Error {}

/** The library's `AccountError`, with the message it words. */
export class AccountError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: AccountFailure,
        message: string,
    ) {
        super(message);
        this.name = 'AccountError';
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

/** The library's `GroupError`, with the message it words. */
export class GroupError extends EranosError {
    /**
     * The error of `failure`, which the library words as `message`.
     *
     * @param failure - What the module reports
     * @param message - The library's message
     */
    constructor(
        /** What the module reports. */
        readonly failure: GroupFailure,
        message: string,
    ) {
        super(message);
        this.name = 'GroupError';
    }
}

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

/**
 * A saved group that is not JSON of the saved form, at a line and a column; the message names the fault but never the text
 * there, which can hold an account number.
 */
export class InvalidJsonError extends EranosError {
    /**
     * The error of the fault at `line` and `column`, which the library words as `message`.
     *
     * @param column - The column of the fault, from 1
     * @param line - The line of the fault, from 1
     * @param message - The library's message
     */
    constructor(
        /** The column of the fault, from 1. */
        readonly column: number,
        /** The line of the fault, from 1. */
        readonly line: number,
        message: string,
    ) {
        super(message);
        this.name = 'InvalidJsonError';
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

/** What the library's `AccountError` reports: a type per failure, with what it holds. */
export type AccountFailure =
    | {
          /** An account whose name is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /** A username that is blank. */
          readonly kind: 'emptyUsername';
      }
    | {
          /** An IBAN that failed validation, and why. */
          readonly kind: 'invalidIban';
          /** The check the IBAN failed. */
          readonly reason: IbanReason;
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

/** What the library's `ExpenseError` reports: a type per failure, with what it holds. */
export type ExpenseFailure =
    | {
          /** The amounts of a split do not add up to the amount being split. */
          readonly kind: 'amountsDoNotAddUp';
          /** What the amounts add up to. */
          readonly actual: Money;
          /** The amount being split. */
          readonly expected: Money;
      }
    | {
          /** An amount inside the expense is in a different currency. */
          readonly kind: 'currencyMismatch';
          /** The expense's currency. */
          readonly expected: Currency;
          /** The currency of the amount inside it. */
          readonly found: Currency;
      }
    | {
          /** A member appears more than once in one split. */
          readonly kind: 'duplicateMember';
          /** The member who appears more than once. */
          readonly member: MemberId;
      }
    | {
          /** An item whose name is blank. */
          readonly kind: 'emptyItemName';
      }
    | {
          /** An expense whose name is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /** A split lists no member. */
          readonly kind: 'emptySplit';
      }
    | {
          /** The item line totals do not add up to the expense amount. */
          readonly kind: 'itemsDoNotAddUp';
          /** What the line totals add up to. */
          readonly actual: Money;
          /** The expense amount. */
          readonly expected: Money;
      }
    | {
          /** A fixed tip below zero. */
          readonly kind: 'negativeTip';
      }
    | {
          /** An itemised expense without items. */
          readonly kind: 'noItems';
      }
    | {
          /** The amount or unit price is zero or negative. */
          readonly kind: 'nonPositiveAmount';
      }
    | {
          /** A member's percentage or amount is zero or negative. */
          readonly kind: 'nonPositiveShare';
          /** The member whose share is zero or negative. */
          readonly member: MemberId;
      }
    | {
          /** A total that does not fit the range of an amount. */
          readonly kind: 'overflow';
      }
    | {
          /** The percentages of a split do not add up to exactly 100 %. */
          readonly kind: 'percentagesDoNotAddUp';
          /** What the percentages add up to, in basis points. */
          readonly totalBasisPoints: number;
      }
    | {
          /** An item quantity of zero. */
          readonly kind: 'zeroQuantity';
      };

/** What the library's `GroupError` reports: a type per failure, with what it holds. */
export type GroupFailure =
    | {
          /** An archived member cannot join a new expense. */
          readonly kind: 'archivedMember';
          /** The archived member. */
          readonly member: MemberId;
      }
    | {
          /** The built-in Uncategorised category cannot be renamed or removed. */
          readonly kind: 'builtInCategory';
      }
    | {
          /** A shared group's lock was asked for without waiting, while another call held it. */
          readonly kind: 'busy';
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
          /** The same currency appears twice among the rates of a currency change. */
          readonly kind: 'duplicateExchangeRate';
          /** The currency that appears twice. */
          readonly currency: Currency;
      }
    | {
          /** An expense with this ID already exists. */
          readonly kind: 'duplicateExpense';
          /** The identifier another expense has. */
          readonly expense: ExpenseId;
      }
    | {
          /** A member with this ID already exists. */
          readonly kind: 'duplicateMember';
          /** The identifier another member has. */
          readonly member: MemberId;
      }
    | {
          /** A transfer with this ID already exists. */
          readonly kind: 'duplicateTransfer';
          /** The identifier another transfer has. */
          readonly transfer: TransferId;
      }
    | {
          /** A group name that is blank. */
          readonly kind: 'emptyName';
      }
    | {
          /** The member is still referenced and cannot be removed; archive it instead. */
          readonly kind: 'memberInUse';
          /** The expenses that name the member. */
          readonly expenses: readonly ExpenseId[];
          /** The member. */
          readonly member: MemberId;
          /** The transfers that name the member. */
          readonly transfers: readonly TransferId[];
      }
    | {
          /**
           * An entry in another currency than the group's has no exchange rate into the group currency: its cost is local,
           * or a currency change gave no rate for its currency.
           */
          readonly kind: 'missingExchangeRate';
          /** The currency without a rate. */
          readonly currency: Currency;
      }
    | {
          /** A total would leave the range of an amount. */
          readonly kind: 'overflow';
      }
    | {
          /** An entry in the group currency has a converted cost; only a local cost is in the group currency. */
          readonly kind: 'unexpectedExchangeRate';
      }
    | {
          /** No category with this ID. */
          readonly kind: 'unknownCategory';
          /** The identifier no category has. */
          readonly category: CategoryId;
      }
    | {
          /** No expense with this ID. */
          readonly kind: 'unknownExpense';
          /** The identifier no expense has. */
          readonly expense: ExpenseId;
      }
    | {
          /** No member with this ID. */
          readonly kind: 'unknownMember';
          /** The identifier no member has. */
          readonly member: MemberId;
      }
    | {
          /** No transfer with this ID. */
          readonly kind: 'unknownTransfer';
          /** The identifier no transfer has. */
          readonly transfer: TransferId;
      }
    | {
          /**
           * A saved group in a format this version does not read: a newer one, whose fields it would drop, or an earlier
           * one.
           */
          readonly kind: 'unsupportedFormat';
          /** The format the saved group names. */
          readonly format: number;
      };

/** What the library's `IdError` reports: a type per failure, with what it holds. */
export interface IdFailure {
    /** Text that is not a UUID. */
    readonly kind: 'invalidId';
}

/** What the library's `LanguageError` reports: a type per failure, with what it holds. */
export interface LanguageFailure {
    /** A language tag that is not a Unicode BCP 47 locale identifier (UTS #35). */
    readonly kind: 'invalidLanguage';
}

/** What the library's `MemberError` reports: a type per failure, with what it holds. */
export interface MemberFailure {
    /** A first name that is blank. */
    readonly kind: 'emptyFirstName';
}

/** What the library's `MoneyError` reports: a type per failure, with what it holds. */
export type MoneyFailure =
    | {
          /** Arithmetic between amounts of two different currencies. */
          readonly kind: 'currencyMismatch';
          /** The currency of the first amount. */
          readonly left: Currency;
          /** The currency of the second amount. */
          readonly right: Currency;
      }
    | {
          /** An amount that is not a decimal number. */
          readonly kind: 'invalidAmount';
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
          /** A number or a result that does not fit its range. */
          readonly kind: 'overflow';
      }
    | {
          /** An amount with more decimal places than its currency has. */
          readonly kind: 'tooManyDecimals';
          /** How many decimal places the currency has. */
          readonly allowed: number;
          /** The currency. */
          readonly currency: Currency;
      }
    | {
          /** A currency code that is not an active ISO 4217 code. */
          readonly kind: 'unknownCurrency';
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
          /** The amount is zero or negative. */
          readonly kind: 'nonPositiveAmount';
      }
    | {
          /** The sender and the recipient are the same member. */
          readonly kind: 'sameMember';
      };

/** Why the library refused an IBAN: a type per check it failed. */
export type IbanReason =
    | {
          /** A character other than an ASCII letter, a digit, or whitespace (which is dropped). */
          readonly kind: 'invalidCharacter';
      }
    | {
          /**
           * The check digits are 00, 01, or 99, which ISO 13616 never issues, or the ISO 7064 mod-97 check failed: the
           * number contains a typo.
           */
          readonly kind: 'invalidChecksum';
      }
    | {
          /** Not two letters followed by two check digits. */
          readonly kind: 'invalidFormat';
      }
    | {
          /**
           * A letter where the country's account number has a digit, or a digit where it has a letter, by the BBAN
           * structure of the SWIFT IBAN registry.
           */
          readonly kind: 'invalidStructure';
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
        case internal.EranosError_Tags.Account:
            return new AccountError(convertedAccountFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Calendar:
            return new CalendarError(convertedCalendarFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Cancelled:
            return new DOMException(error.inner.detail, 'AbortError');
        case internal.EranosError_Tags.Category:
            return new CategoryError(convertedCategoryFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Contact:
            return new ContactError(convertedContactFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Expense:
            return new ExpenseError(convertedExpenseFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Group:
            return new GroupError(convertedGroupFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Id:
            return new IdError(convertedIdFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.IdentifierGiven:
            return new TypeError(error.inner.detail);
        case internal.EranosError_Tags.IdentifierMissing:
            return new TypeError(error.inner.detail);
        case internal.EranosError_Tags.InvalidJson:
            return new InvalidJsonError(Number(error.inner.column), Number(error.inner.line), error.inner.detail);
        case internal.EranosError_Tags.Language:
            return new LanguageError(convertedLanguageFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Member:
            return new MemberError(convertedMemberFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Money:
            return new MoneyError(convertedMoneyFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Released:
            return new TypeError(error.inner.detail);
        case internal.EranosError_Tags.Settlement:
            return new SettlementError(convertedSettlementFailure(error.inner.failure), error.inner.detail);
        case internal.EranosError_Tags.Transfer:
            return new TransferError(convertedTransferFailure(error.inner.failure), error.inner.detail);
    }
}

/** The failure the generated code reports as `failure`. */
function convertedAccountFailure(failure: internal.AccountFailure): AccountFailure {
    switch (failure.tag) {
        case internal.AccountFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.AccountFailure_Tags.EmptyUsername:
            return { kind: 'emptyUsername' };
        case internal.AccountFailure_Tags.InvalidIban:
            return { kind: 'invalidIban', reason: convertedIbanReason(failure.inner.reason) };
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
function convertedExpenseFailure(failure: internal.ExpenseFailure): ExpenseFailure {
    switch (failure.tag) {
        case internal.ExpenseFailure_Tags.AmountsDoNotAddUp:
            return { kind: 'amountsDoNotAddUp', actual: Money.converted(failure.inner.actual), expected: Money.converted(failure.inner.expected) };
        case internal.ExpenseFailure_Tags.CurrencyMismatch:
            return { kind: 'currencyMismatch', expected: Currency.known(failure.inner.expected), found: Currency.known(failure.inner.found) };
        case internal.ExpenseFailure_Tags.DuplicateMember:
            return { kind: 'duplicateMember', member: memberId(failure.inner.member) };
        case internal.ExpenseFailure_Tags.EmptyItemName:
            return { kind: 'emptyItemName' };
        case internal.ExpenseFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.ExpenseFailure_Tags.EmptySplit:
            return { kind: 'emptySplit' };
        case internal.ExpenseFailure_Tags.ItemsDoNotAddUp:
            return { kind: 'itemsDoNotAddUp', actual: Money.converted(failure.inner.actual), expected: Money.converted(failure.inner.expected) };
        case internal.ExpenseFailure_Tags.NegativeTip:
            return { kind: 'negativeTip' };
        case internal.ExpenseFailure_Tags.NoItems:
            return { kind: 'noItems' };
        case internal.ExpenseFailure_Tags.NonPositiveAmount:
            return { kind: 'nonPositiveAmount' };
        case internal.ExpenseFailure_Tags.NonPositiveShare:
            return { kind: 'nonPositiveShare', member: memberId(failure.inner.member) };
        case internal.ExpenseFailure_Tags.Overflow:
            return { kind: 'overflow' };
        case internal.ExpenseFailure_Tags.PercentagesDoNotAddUp:
            return { kind: 'percentagesDoNotAddUp', totalBasisPoints: Number(failure.inner.totalBasisPoints) };
        case internal.ExpenseFailure_Tags.ZeroQuantity:
            return { kind: 'zeroQuantity' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedGroupFailure(failure: internal.GroupFailure): GroupFailure {
    switch (failure.tag) {
        case internal.GroupFailure_Tags.ArchivedMember:
            return { kind: 'archivedMember', member: memberId(failure.inner.member) };
        case internal.GroupFailure_Tags.BuiltInCategory:
            return { kind: 'builtInCategory' };
        case internal.GroupFailure_Tags.Busy:
            return { kind: 'busy' };
        case internal.GroupFailure_Tags.DuplicateCategory:
            return { kind: 'duplicateCategory', category: categoryId(failure.inner.category) };
        case internal.GroupFailure_Tags.DuplicateCategoryName:
            return { kind: 'duplicateCategoryName', existing: categoryId(failure.inner.existing) };
        case internal.GroupFailure_Tags.DuplicateExchangeRate:
            return { kind: 'duplicateExchangeRate', currency: Currency.known(failure.inner.currency) };
        case internal.GroupFailure_Tags.DuplicateExpense:
            return { kind: 'duplicateExpense', expense: expenseId(failure.inner.expense) };
        case internal.GroupFailure_Tags.DuplicateMember:
            return { kind: 'duplicateMember', member: memberId(failure.inner.member) };
        case internal.GroupFailure_Tags.DuplicateTransfer:
            return { kind: 'duplicateTransfer', transfer: transferId(failure.inner.transfer) };
        case internal.GroupFailure_Tags.EmptyName:
            return { kind: 'emptyName' };
        case internal.GroupFailure_Tags.MemberInUse:
            return {
                kind: 'memberInUse',
                expenses: failure.inner.expenses.map(expenseId),
                member: memberId(failure.inner.member),
                transfers: failure.inner.transfers.map(transferId),
            };
        case internal.GroupFailure_Tags.MissingExchangeRate:
            return { kind: 'missingExchangeRate', currency: Currency.known(failure.inner.currency) };
        case internal.GroupFailure_Tags.Overflow:
            return { kind: 'overflow' };
        case internal.GroupFailure_Tags.UnexpectedExchangeRate:
            return { kind: 'unexpectedExchangeRate' };
        case internal.GroupFailure_Tags.UnknownCategory:
            return { kind: 'unknownCategory', category: categoryId(failure.inner.category) };
        case internal.GroupFailure_Tags.UnknownExpense:
            return { kind: 'unknownExpense', expense: expenseId(failure.inner.expense) };
        case internal.GroupFailure_Tags.UnknownMember:
            return { kind: 'unknownMember', member: memberId(failure.inner.member) };
        case internal.GroupFailure_Tags.UnknownTransfer:
            return { kind: 'unknownTransfer', transfer: transferId(failure.inner.transfer) };
        case internal.GroupFailure_Tags.UnsupportedFormat:
            return { kind: 'unsupportedFormat', format: Number(failure.inner.format) };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedIdFailure(failure: internal.IdFailure): IdFailure {
    switch (failure) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.IdFailure.InvalidId:
            return { kind: 'invalidId' };
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
function convertedMemberFailure(failure: internal.MemberFailure): MemberFailure {
    switch (failure) {
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- one kind of failure; a second leaves the switch without a return
        case internal.MemberFailure.EmptyFirstName:
            return { kind: 'emptyFirstName' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedMoneyFailure(failure: internal.MoneyFailure): MoneyFailure {
    switch (failure.tag) {
        case internal.MoneyFailure_Tags.CurrencyMismatch:
            return { kind: 'currencyMismatch', left: Currency.known(failure.inner.left), right: Currency.known(failure.inner.right) };
        case internal.MoneyFailure_Tags.InvalidAmount:
            return { kind: 'invalidAmount' };
        case internal.MoneyFailure_Tags.InvalidExchangeRate:
            return { kind: 'invalidExchangeRate' };
        case internal.MoneyFailure_Tags.InvalidParts:
            return { kind: 'invalidParts' };
        case internal.MoneyFailure_Tags.InvalidPercentage:
            return { kind: 'invalidPercentage' };
        case internal.MoneyFailure_Tags.Overflow:
            return { kind: 'overflow' };
        case internal.MoneyFailure_Tags.TooManyDecimals:
            return { kind: 'tooManyDecimals', allowed: Number(failure.inner.allowed), currency: Currency.known(failure.inner.currency) };
        case internal.MoneyFailure_Tags.UnknownCurrency:
            return { kind: 'unknownCurrency' };
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
        case internal.TransferFailure.NonPositiveAmount:
            return { kind: 'nonPositiveAmount' };
        case internal.TransferFailure.SameMember:
            return { kind: 'sameMember' };
    }
}

/** The failure the generated code reports as `failure`. */
function convertedIbanReason(failure: internal.IbanReason): IbanReason {
    switch (failure.tag) {
        case internal.IbanReason_Tags.InvalidCharacter:
            return { kind: 'invalidCharacter' };
        case internal.IbanReason_Tags.InvalidChecksum:
            return { kind: 'invalidChecksum' };
        case internal.IbanReason_Tags.InvalidFormat:
            return { kind: 'invalidFormat' };
        case internal.IbanReason_Tags.InvalidStructure:
            return { kind: 'invalidStructure' };
        case internal.IbanReason_Tags.UnknownCountry:
            return { kind: 'unknownCountry' };
        case internal.IbanReason_Tags.WrongLength:
            return { kind: 'wrongLength', expected: Number(failure.inner.expected), found: Number(failure.inner.found) };
    }
}
