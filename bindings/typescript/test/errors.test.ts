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
    GroupError,
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
    return new Money(Currency.known('EUR'), minorUnits);
}

/** A module's subclass of {@link EranosError}, whose instances hold the module's failure. */
type ModuleError = abstract new (...values: never[]) => EranosError & { readonly failure: unknown };

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the class and failure it becomes. */
const conversions: ReadonlyArray<readonly [unknown, ModuleError, unknown]> = [
    [new internal.EranosError.Account({ detail: 'detail', failure: new internal.AccountFailure.EmptyName() }), AccountError, { kind: 'emptyName' }],
    [
        new internal.EranosError.Account({ detail: 'detail', failure: new internal.AccountFailure.EmptyUsername() }),
        AccountError,
        { kind: 'emptyUsername' },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    ],
    [new internal.EranosError.Calendar({ detail: 'detail', failure: internal.CalendarFailure.InvalidDate }), CalendarError, { kind: 'invalidDate' }],
    [new internal.EranosError.Calendar({ detail: 'detail', failure: internal.CalendarFailure.InvalidTime }), CalendarError, { kind: 'invalidTime' }],
    [new internal.EranosError.Category({ detail: 'detail', failure: internal.CategoryFailure.EmptyName }), CategoryError, { kind: 'emptyName' }],
    [new internal.EranosError.Contact({ detail: 'detail', failure: internal.ContactFailure.InvalidEmail }), ContactError, { kind: 'invalidEmail' }],
    [
        new internal.EranosError.Contact({ detail: 'detail', failure: internal.ContactFailure.InvalidPhoneNumber }),
        ContactError,
        { kind: 'invalidPhoneNumber' },
    ],
    [
        new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.AmountsDoNotAddUp({
                actual: internal.Money.create({ currency: 'EUR', minorUnits: 2n }),
                expected: internal.Money.create({ currency: 'EUR', minorUnits: 3n }),
            }),
        }),
        ExpenseError,
        { kind: 'amountsDoNotAddUp', actual: euros(2n), expected: euros(3n) },
    ],
    [
        new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.CurrencyMismatch({ expected: 'USD', found: 'GBP' }),
        }),
        ExpenseError,
        { kind: 'currencyMismatch', expected: Currency.known('USD'), found: Currency.known('GBP') },
    ],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.DuplicateMember({ member: 'member-6' }) }),
        ExpenseError,
        { kind: 'duplicateMember', member: memberId('member-6') },
    ],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptyItemName() }),
        ExpenseError,
        { kind: 'emptyItemName' },
    ],
    [new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptyName() }), ExpenseError, { kind: 'emptyName' }],
    [new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptySplit() }), ExpenseError, { kind: 'emptySplit' }],
    [
        new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.ItemsDoNotAddUp({
                actual: internal.Money.create({ currency: 'EUR', minorUnits: 7n }),
                expected: internal.Money.create({ currency: 'EUR', minorUnits: 8n }),
            }),
        }),
        ExpenseError,
        { kind: 'itemsDoNotAddUp', actual: euros(7n), expected: euros(8n) },
    ],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NegativeTip() }),
        ExpenseError,
        { kind: 'negativeTip' },
    ],
    [new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NoItems() }), ExpenseError, { kind: 'noItems' }],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NonPositiveAmount() }),
        ExpenseError,
        { kind: 'nonPositiveAmount' },
    ],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NonPositiveShare({ member: 'member-9' }) }),
        ExpenseError,
        { kind: 'nonPositiveShare', member: memberId('member-9') },
    ],
    [new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.Overflow() }), ExpenseError, { kind: 'overflow' }],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.PercentagesDoNotAddUp({ totalBasisPoints: 10n }) }),
        ExpenseError,
        { kind: 'percentagesDoNotAddUp', totalBasisPoints: 10 },
    ],
    [
        new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.ZeroQuantity() }),
        ExpenseError,
        { kind: 'zeroQuantity' },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.ArchivedMember({ member: 'member-11' }) }),
        GroupError,
        { kind: 'archivedMember', member: memberId('member-11') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.BuiltInCategory() }),
        GroupError,
        { kind: 'builtInCategory' },
    ],
    [new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.Busy() }), GroupError, { kind: 'busy' }],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateCategory({ category: 'category-12' }) }),
        GroupError,
        { kind: 'duplicateCategory', category: categoryId('category-12') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateCategoryName({ existing: 'existing-13' }) }),
        GroupError,
        { kind: 'duplicateCategoryName', existing: categoryId('existing-13') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateExchangeRate({ currency: 'GBP' }) }),
        GroupError,
        { kind: 'duplicateExchangeRate', currency: Currency.known('GBP') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateExpense({ expense: 'expense-15' }) }),
        GroupError,
        { kind: 'duplicateExpense', expense: expenseId('expense-15') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateMember({ member: 'member-16' }) }),
        GroupError,
        { kind: 'duplicateMember', member: memberId('member-16') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateTransfer({ transfer: 'transfer-17' }) }),
        GroupError,
        { kind: 'duplicateTransfer', transfer: transferId('transfer-17') },
    ],
    [new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.EmptyName() }), GroupError, { kind: 'emptyName' }],
    [
        new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.MemberInUse({ expenses: ['expenses-18'], member: 'member-19', transfers: ['transfers-20'] }),
        }),
        GroupError,
        { kind: 'memberInUse', expenses: [expenseId('expenses-18')], member: memberId('member-19'), transfers: [transferId('transfers-20')] },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.MissingExchangeRate({ currency: 'RON' }) }),
        GroupError,
        { kind: 'missingExchangeRate', currency: Currency.known('RON') },
    ],
    [new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.Overflow() }), GroupError, { kind: 'overflow' }],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnexpectedExchangeRate() }),
        GroupError,
        { kind: 'unexpectedExchangeRate' },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownCategory({ category: 'category-22' }) }),
        GroupError,
        { kind: 'unknownCategory', category: categoryId('category-22') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownExpense({ expense: 'expense-23' }) }),
        GroupError,
        { kind: 'unknownExpense', expense: expenseId('expense-23') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownMember({ member: 'member-24' }) }),
        GroupError,
        { kind: 'unknownMember', member: memberId('member-24') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownTransfer({ transfer: 'transfer-25' }) }),
        GroupError,
        { kind: 'unknownTransfer', transfer: transferId('transfer-25') },
    ],
    [
        new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnsupportedFormat({ format: 26n }) }),
        GroupError,
        { kind: 'unsupportedFormat', format: 26 },
    ],
    [new internal.EranosError.Id({ detail: 'detail', failure: internal.IdFailure.InvalidId }), IdError, { kind: 'invalidId' }],
    [
        new internal.EranosError.Language({ detail: 'detail', failure: internal.LanguageFailure.InvalidLanguage }),
        LanguageError,
        { kind: 'invalidLanguage' },
    ],
    [new internal.EranosError.Member({ detail: 'detail', failure: internal.MemberFailure.EmptyFirstName }), MemberError, { kind: 'emptyFirstName' }],
    [
        new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.CurrencyMismatch({ left: 'RON', right: 'USD' }) }),
        MoneyError,
        { kind: 'currencyMismatch', left: Currency.known('RON'), right: Currency.known('USD') },
    ],
    [new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidAmount() }), MoneyError, { kind: 'invalidAmount' }],
    [
        new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidExchangeRate() }),
        MoneyError,
        { kind: 'invalidExchangeRate' },
    ],
    [new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidParts() }), MoneyError, { kind: 'invalidParts' }],
    [
        new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidPercentage() }),
        MoneyError,
        { kind: 'invalidPercentage' },
    ],
    [new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.Overflow() }), MoneyError, { kind: 'overflow' }],
    [
        new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.TooManyDecimals({ allowed: 29n, currency: 'RON' }) }),
        MoneyError,
        { kind: 'tooManyDecimals', allowed: 29, currency: Currency.known('RON') },
    ],
    [
        new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.UnknownCurrency() }),
        MoneyError,
        { kind: 'unknownCurrency' },
    ],
    [
        new internal.EranosError.Settlement({ detail: 'detail', failure: new internal.SettlementFailure.OutOfMemory({ balances: 31n }) }),
        SettlementError,
        { kind: 'outOfMemory', balances: 31 },
    ],
    [
        new internal.EranosError.Transfer({ detail: 'detail', failure: internal.TransferFailure.NonPositiveAmount }),
        TransferError,
        { kind: 'nonPositiveAmount' },
    ],
    [new internal.EranosError.Transfer({ detail: 'detail', failure: internal.TransferFailure.SameMember }), TransferError, { kind: 'sameMember' }],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidCharacter() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'invalidCharacter' } },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidFormat() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'invalidFormat' } },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidStructure() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'invalidStructure' } },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.UnknownCountry() }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'unknownCountry' } },
    ],
    [
        new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.WrongLength({ expected: 24n, found: 22n }) }),
        }),
        AccountError,
        { kind: 'invalidIban', reason: { kind: 'wrongLength', expected: 24, found: 22 } },
    ],
];

void describe('the conversions of every failure the generated code can report', () => {
    /**
     * A conversion that reached another module or failure, dropped what the failure holds, or lost the message would show an
     * app another failure than the library's.
     */
    void it('converts every failure of the generated code to its module and failure', () => {
        assert.equal(conversions.length, 61);
        for (const [generated, type, failure] of conversions) {
            const converted = convertedError(generated);
            assert.ok(converted instanceof type);
            assert.ok(converted instanceof EranosError);
            assert.deepEqual(converted.failure, failure);
            assert.equal(converted.message, 'detail');
        }
    });

    /**
     * A saved group that failed to load without its line and column would leave an app nothing to point the user at, and a
     * call the program should not make, reported as a failure of the library, would be caught as one.
     */
    void it("converts the bindings' own failures to their standard errors", () => {
        const json = convertedError(new internal.EranosError.InvalidJson({ column: 2n, detail: 'detail', line: 1n }));
        assert.ok(json instanceof InvalidJsonError);
        assert.deepEqual([json.column, json.line, json.message, json.name], [2, 1, 'detail', 'InvalidJsonError']);
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
        assert.deepEqual([cancelled.name, cancelled.message], ['AbortError', 'detail']);
        const other = new RangeError('other');
        assert.equal(convertedError(other), other);
    });
});
