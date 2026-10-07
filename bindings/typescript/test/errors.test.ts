// The error every call throws. A failure converted to another module's subclass, to another failure, with another payload,
// or without the library's message would show an app another failure than the library's, so every failure the generated
// code can report is converted here, and every message is read back.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { convertedError } from '../src/errors.js';
import {
    AccountError,
    CalendarError,
    CategoryError,
    ContactError,
    Currency,
    EranosError,
    ExpenseError,
    ExpenseGroupError,
    IdError,
    InvalidJsonError,
    LanguageError,
    MemberError,
    Money,
    MoneyError,
    SettlementError,
    TransferError,
    categoryId,
    expenseId,
    memberId,
    transferId,
} from '../src/index.js';
import * as internal from '../src/internal/eranos.js';
import { initialised } from './flat.js';

await initialised;

/** `minorUnits` cents. */
function euros(minorUnits: bigint): Money {
    return new Money(minorUnits, Currency.known('EUR'));
}

/** A module's subclass of {@link EranosError}, whose instances hold the module's failure. */
type ModuleError = abstract new (...values: never[]) => EranosError & { readonly failure: unknown };

/** A failure the generated code can report, with the class and the failure it becomes. */
interface Conversion {
    readonly generated: unknown;
    readonly type: ModuleError;
    readonly failure: unknown;
}

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the class and failure it becomes. */
const conversions: readonly Conversion[] = [
    {
        generated: new internal.EranosError.Id({ failure: internal.IdFailure.InvalidId, detail: 'detail' }),
        type: IdError,
        failure: { kind: 'invalidId' },
    },
    {
        generated: new internal.EranosError.Account({ failure: new internal.AccountFailure.EmptyName(), detail: 'detail' }),
        type: AccountError,
        failure: { kind: 'emptyName' },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    },
    {
        generated: new internal.EranosError.Account({ failure: new internal.AccountFailure.EmptyUsername(), detail: 'detail' }),
        type: AccountError,
        failure: { kind: 'emptyUsername' },
    },
    {
        generated: new internal.EranosError.Calendar({ failure: internal.CalendarFailure.InvalidDate, detail: 'detail' }),
        type: CalendarError,
        failure: { kind: 'invalidDate' },
    },
    {
        generated: new internal.EranosError.Calendar({ failure: internal.CalendarFailure.InvalidTime, detail: 'detail' }),
        type: CalendarError,
        failure: { kind: 'invalidTime' },
    },
    {
        generated: new internal.EranosError.Category({ failure: internal.CategoryFailure.EmptyName, detail: 'detail' }),
        type: CategoryError,
        failure: { kind: 'emptyName' },
    },
    {
        generated: new internal.EranosError.Contact({ failure: internal.ContactFailure.InvalidEmail, detail: 'detail' }),
        type: ContactError,
        failure: { kind: 'invalidEmail' },
    },
    {
        generated: new internal.EranosError.Contact({ failure: internal.ContactFailure.InvalidPhoneNumber, detail: 'detail' }),
        type: ContactError,
        failure: { kind: 'invalidPhoneNumber' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.InvalidAmount(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'invalidAmount' },
    },
    {
        generated: new internal.EranosError.Money({
            failure: new internal.MoneyFailure.TooManyDecimals({ currency: 'RON', allowed: 29n }),
            detail: 'detail',
        }),
        type: MoneyError,
        failure: { kind: 'tooManyDecimals', currency: Currency.known('RON'), allowed: 29 },
    },
    {
        generated: new internal.EranosError.Money({
            failure: new internal.MoneyFailure.CurrencyMismatch({ left: 'RON', right: 'USD' }),
            detail: 'detail',
        }),
        type: MoneyError,
        failure: { kind: 'currencyMismatch', left: Currency.known('RON'), right: Currency.known('USD') },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.UnknownCurrency(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'unknownCurrency' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.InvalidExchangeRate(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'invalidExchangeRate' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.InvalidParts(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'invalidParts' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.InvalidPercentage(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'invalidPercentage' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.Overflow(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'overflow' },
    },
    {
        generated: new internal.EranosError.Member({ failure: internal.MemberFailure.EmptyFirstName, detail: 'detail' }),
        type: MemberError,
        failure: { kind: 'emptyFirstName' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.EmptyName(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'emptyName' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.NonPositiveAmount(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'nonPositiveAmount' },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.CurrencyMismatch({ expected: 'USD', found: 'GBP' }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'currencyMismatch', expected: Currency.known('USD'), found: Currency.known('GBP') },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.EmptySplit(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'emptySplit' },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.DuplicateMember({ member: 'member-6' }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'duplicateMember', member: memberId('member-6') },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.NonPositiveShare({ member: 'member-9' }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'nonPositiveShare', member: memberId('member-9') },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.AmountsDoNotAddUp({
                expected: internal.Money.create({ minorUnits: 3n, currency: 'EUR' }),
                actual: internal.Money.create({ minorUnits: 2n, currency: 'EUR' }),
            }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'amountsDoNotAddUp', expected: euros(3n), actual: euros(2n) },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.PercentagesDoNotAddUp({ totalBasisPoints: 10n }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'percentagesDoNotAddUp', totalBasisPoints: 10 },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.NoItems(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'noItems' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.EmptyItemName(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'emptyItemName' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.ZeroQuantity(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'zeroQuantity' },
    },
    {
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.ItemsDoNotAddUp({
                expected: internal.Money.create({ minorUnits: 8n, currency: 'EUR' }),
                actual: internal.Money.create({ minorUnits: 7n, currency: 'EUR' }),
            }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'itemsDoNotAddUp', expected: euros(8n), actual: euros(7n) },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.NegativeTip(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'negativeTip' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.Overflow(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'overflow' },
    },
    {
        generated: new internal.EranosError.Settlement({ failure: new internal.SettlementFailure.OutOfMemory({ balances: 31n }), detail: 'detail' }),
        type: SettlementError,
        failure: { kind: 'outOfMemory', balances: 31 },
    },
    {
        generated: new internal.EranosError.Transfer({ failure: internal.TransferFailure.SameMember, detail: 'detail' }),
        type: TransferError,
        failure: { kind: 'sameMember' },
    },
    {
        generated: new internal.EranosError.Transfer({ failure: internal.TransferFailure.NonPositiveAmount, detail: 'detail' }),
        type: TransferError,
        failure: { kind: 'nonPositiveAmount' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.UnsupportedFormat({ format: 26n }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'unsupportedFormat', format: 26 },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.EmptyName(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'emptyName' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.MissingExchangeRate({ currency: 'RON' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'missingExchangeRate', currency: Currency.known('RON') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.UnexpectedExchangeRate(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'unexpectedExchangeRate' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateExchangeRate({ currency: 'GBP' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateExchangeRate', currency: Currency.known('GBP') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateMember({ member: 'member-16' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateMember', member: memberId('member-16') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.UnknownMember({ member: 'member-24' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'unknownMember', member: memberId('member-24') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.ArchivedMember({ member: 'member-11' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'archivedMember', member: memberId('member-11') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.MemberInUse({ member: 'member-19', expenses: ['expenses-18'], transfers: ['transfers-20'] }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: {
            kind: 'memberInUse',
            member: memberId('member-19'),
            expenses: [expenseId('expenses-18')],
            transfers: [transferId('transfers-20')],
        },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateCategory({ category: 'category-12' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateCategory', category: categoryId('category-12') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateCategoryName({ existing: 'existing-13' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateCategoryName', existing: categoryId('existing-13') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.UnknownCategory({ category: 'category-22' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'unknownCategory', category: categoryId('category-22') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.BuiltInCategory(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'builtInCategory' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateExpense({ expense: 'expense-15' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateExpense', expense: expenseId('expense-15') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.UnknownExpense({ expense: 'expense-23' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'unknownExpense', expense: expenseId('expense-23') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateTransfer({ transfer: 'transfer-17' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateTransfer', transfer: transferId('transfer-17') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.UnknownTransfer({ transfer: 'transfer-25' }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'unknownTransfer', transfer: transferId('transfer-25') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.Busy(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'busy' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.Overflow(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'overflow' },
    },
    {
        generated: new internal.EranosError.Language({ failure: internal.LanguageFailure.InvalidLanguage, detail: 'detail' }),
        type: LanguageError,
        failure: { kind: 'invalidLanguage' },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidCharacter() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidCharacter' } },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidFormat() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidFormat' } },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.UnknownCountry() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'unknownCountry' } },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.WrongLength({ expected: 24n, found: 22n }) }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'wrongLength', expected: 24, found: 22 } },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidStructure() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidStructure' } },
    },
    {
        generated: new internal.EranosError.Account({
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
            detail: 'detail',
        }),
        type: AccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    },
];

void describe('the conversions of every failure the generated code can report', () => {
    /**
     * A conversion that reached another module or failure, dropped what the failure holds, or lost the message would show an
     * app another failure than the library's.
     */
    void it('converts every failure of the generated code to its module and failure', () => {
        assert.equal(conversions.length, 61);
        for (const conversion of conversions) {
            const converted = convertedError(conversion.generated);
            assert.ok(converted instanceof conversion.type);
            assert.ok(converted instanceof EranosError);
            assert.deepEqual(converted.failure, conversion.failure);
            assert.equal(converted.message, 'detail');
        }
    });

    /**
     * A saved group that failed to load without its line and column would leave an app nothing to point the user at, and a
     * call the program should not make, reported as a failure of the library, would be caught as one.
     */
    void it("converts the bindings' own failures to their standard errors", () => {
        const json = convertedError(new internal.EranosError.InvalidJson({ line: 1n, column: 2n, detail: 'detail' }));
        assert.ok(json instanceof InvalidJsonError);
        assert.equal(json.line, 1);
        assert.equal(json.column, 2);
        assert.equal(json.message, 'detail');
        assert.equal(json.name, 'InvalidJsonError');
        for (const generated of [
            new internal.EranosError.IdentifierGiven({ detail: 'detail' }),
            new internal.EranosError.IdentifierMissing({ detail: 'detail' }),
            new internal.EranosError.Released({ detail: 'detail' }),
        ]) {
            const converted = convertedError(generated);
            assert.ok(converted instanceof TypeError);
            assert.equal(converted.message, 'detail');
        }
        const cancelled = convertedError(new internal.EranosError.Cancelled({ detail: 'detail' }));
        assert.ok(cancelled instanceof DOMException);
        assert.equal(cancelled.name, 'AbortError');
        assert.equal(cancelled.message, 'detail');
        const other = new RangeError('other');
        assert.equal(convertedError(other), other);
    });
});
