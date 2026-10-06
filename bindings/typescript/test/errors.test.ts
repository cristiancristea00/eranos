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

/** A failure the generated code can report, with the class and the failure it becomes. */
interface Conversion {
    readonly failure: unknown;
    readonly generated: unknown;
    readonly type: ModuleError;
}

/** Every failure the generated code can report, with every reason of a refused IBAN, beside the class and failure it becomes. */
const conversions: readonly Conversion[] = [
    {
        failure: { kind: 'emptyName' },
        generated: new internal.EranosError.Account({ detail: 'detail', failure: new internal.AccountFailure.EmptyName() }),
        type: AccountError,
    },
    {
        failure: { kind: 'emptyUsername' },
        generated: new internal.EranosError.Account({ detail: 'detail', failure: new internal.AccountFailure.EmptyUsername() }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidDate' },
        generated: new internal.EranosError.Calendar({ detail: 'detail', failure: internal.CalendarFailure.InvalidDate }),
        type: CalendarError,
    },
    {
        failure: { kind: 'invalidTime' },
        generated: new internal.EranosError.Calendar({ detail: 'detail', failure: internal.CalendarFailure.InvalidTime }),
        type: CalendarError,
    },
    {
        failure: { kind: 'emptyName' },
        generated: new internal.EranosError.Category({ detail: 'detail', failure: internal.CategoryFailure.EmptyName }),
        type: CategoryError,
    },
    {
        failure: { kind: 'invalidEmail' },
        generated: new internal.EranosError.Contact({ detail: 'detail', failure: internal.ContactFailure.InvalidEmail }),
        type: ContactError,
    },
    {
        failure: { kind: 'invalidPhoneNumber' },
        generated: new internal.EranosError.Contact({ detail: 'detail', failure: internal.ContactFailure.InvalidPhoneNumber }),
        type: ContactError,
    },
    {
        failure: { kind: 'amountsDoNotAddUp', actual: euros(2n), expected: euros(3n) },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.AmountsDoNotAddUp({
                actual: internal.Money.create({ currency: 'EUR', minorUnits: 2n }),
                expected: internal.Money.create({ currency: 'EUR', minorUnits: 3n }),
            }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'currencyMismatch', expected: Currency.known('USD'), found: Currency.known('GBP') },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.CurrencyMismatch({ expected: 'USD', found: 'GBP' }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'duplicateMember', member: memberId('member-6') },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.DuplicateMember({ member: 'member-6' }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'emptyItemName' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptyItemName() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'emptyName' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptyName() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'emptySplit' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.EmptySplit() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'itemsDoNotAddUp', actual: euros(7n), expected: euros(8n) },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.ItemsDoNotAddUp({
                actual: internal.Money.create({ currency: 'EUR', minorUnits: 7n }),
                expected: internal.Money.create({ currency: 'EUR', minorUnits: 8n }),
            }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'negativeTip' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NegativeTip() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'noItems' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NoItems() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'nonPositiveAmount' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.NonPositiveAmount() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'nonPositiveShare', member: memberId('member-9') },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.NonPositiveShare({ member: 'member-9' }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'overflow' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.Overflow() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'percentagesDoNotAddUp', totalBasisPoints: 10 },
        generated: new internal.EranosError.Expense({
            detail: 'detail',
            failure: new internal.ExpenseFailure.PercentagesDoNotAddUp({ totalBasisPoints: 10n }),
        }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'zeroQuantity' },
        generated: new internal.EranosError.Expense({ detail: 'detail', failure: new internal.ExpenseFailure.ZeroQuantity() }),
        type: ExpenseError,
    },
    {
        failure: { kind: 'archivedMember', member: memberId('member-11') },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.ArchivedMember({ member: 'member-11' }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'builtInCategory' },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.BuiltInCategory() }),
        type: GroupError,
    },
    {
        failure: { kind: 'busy' },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.Busy() }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateCategory', category: categoryId('category-12') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.DuplicateCategory({ category: 'category-12' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateCategoryName', existing: categoryId('existing-13') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.DuplicateCategoryName({ existing: 'existing-13' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateExchangeRate', currency: Currency.known('GBP') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.DuplicateExchangeRate({ currency: 'GBP' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateExpense', expense: expenseId('expense-15') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.DuplicateExpense({ expense: 'expense-15' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateMember', member: memberId('member-16') },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.DuplicateMember({ member: 'member-16' }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'duplicateTransfer', transfer: transferId('transfer-17') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.DuplicateTransfer({ transfer: 'transfer-17' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'emptyName' },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.EmptyName() }),
        type: GroupError,
    },
    {
        failure: {
            kind: 'memberInUse',
            expenses: [expenseId('expenses-18')],
            member: memberId('member-19'),
            transfers: [transferId('transfers-20')],
        },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.MemberInUse({ expenses: ['expenses-18'], member: 'member-19', transfers: ['transfers-20'] }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'missingExchangeRate', currency: Currency.known('RON') },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.MissingExchangeRate({ currency: 'RON' }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'overflow' },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.Overflow() }),
        type: GroupError,
    },
    {
        failure: { kind: 'unexpectedExchangeRate' },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnexpectedExchangeRate() }),
        type: GroupError,
    },
    {
        failure: { kind: 'unknownCategory', category: categoryId('category-22') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.UnknownCategory({ category: 'category-22' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'unknownExpense', expense: expenseId('expense-23') },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownExpense({ expense: 'expense-23' }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'unknownMember', member: memberId('member-24') },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnknownMember({ member: 'member-24' }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'unknownTransfer', transfer: transferId('transfer-25') },
        generated: new internal.EranosError.Group({
            detail: 'detail',
            failure: new internal.GroupFailure.UnknownTransfer({ transfer: 'transfer-25' }),
        }),
        type: GroupError,
    },
    {
        failure: { kind: 'unsupportedFormat', format: 26 },
        generated: new internal.EranosError.Group({ detail: 'detail', failure: new internal.GroupFailure.UnsupportedFormat({ format: 26n }) }),
        type: GroupError,
    },
    {
        failure: { kind: 'invalidId' },
        generated: new internal.EranosError.Id({ detail: 'detail', failure: internal.IdFailure.InvalidId }),
        type: IdError,
    },
    {
        failure: { kind: 'invalidLanguage' },
        generated: new internal.EranosError.Language({ detail: 'detail', failure: internal.LanguageFailure.InvalidLanguage }),
        type: LanguageError,
    },
    {
        failure: { kind: 'emptyFirstName' },
        generated: new internal.EranosError.Member({ detail: 'detail', failure: internal.MemberFailure.EmptyFirstName }),
        type: MemberError,
    },
    {
        failure: { kind: 'currencyMismatch', left: Currency.known('RON'), right: Currency.known('USD') },
        generated: new internal.EranosError.Money({
            detail: 'detail',
            failure: new internal.MoneyFailure.CurrencyMismatch({ left: 'RON', right: 'USD' }),
        }),
        type: MoneyError,
    },
    {
        failure: { kind: 'invalidAmount' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidAmount() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'invalidExchangeRate' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidExchangeRate() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'invalidParts' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidParts() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'invalidPercentage' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.InvalidPercentage() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'overflow' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.Overflow() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'tooManyDecimals', allowed: 29, currency: Currency.known('RON') },
        generated: new internal.EranosError.Money({
            detail: 'detail',
            failure: new internal.MoneyFailure.TooManyDecimals({ allowed: 29n, currency: 'RON' }),
        }),
        type: MoneyError,
    },
    {
        failure: { kind: 'unknownCurrency' },
        generated: new internal.EranosError.Money({ detail: 'detail', failure: new internal.MoneyFailure.UnknownCurrency() }),
        type: MoneyError,
    },
    {
        failure: { kind: 'outOfMemory', balances: 31 },
        generated: new internal.EranosError.Settlement({ detail: 'detail', failure: new internal.SettlementFailure.OutOfMemory({ balances: 31n }) }),
        type: SettlementError,
    },
    {
        failure: { kind: 'nonPositiveAmount' },
        generated: new internal.EranosError.Transfer({ detail: 'detail', failure: internal.TransferFailure.NonPositiveAmount }),
        type: TransferError,
    },
    {
        failure: { kind: 'sameMember' },
        generated: new internal.EranosError.Transfer({ detail: 'detail', failure: internal.TransferFailure.SameMember }),
        type: TransferError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'invalidCharacter' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidCharacter() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'invalidFormat' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidFormat() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'invalidStructure' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidStructure() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'unknownCountry' } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.UnknownCountry() }),
        }),
        type: AccountError,
    },
    {
        failure: { kind: 'invalidIban', reason: { kind: 'wrongLength', expected: 24, found: 22 } },
        generated: new internal.EranosError.Account({
            detail: 'detail',
            failure: new internal.AccountFailure.InvalidIban({ reason: new internal.IbanReason.WrongLength({ expected: 24n, found: 22n }) }),
        }),
        type: AccountError,
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
        const json = convertedError(new internal.EranosError.InvalidJson({ column: 2n, detail: 'detail', line: 1n }));
        assert.ok(json instanceof InvalidJsonError);
        assert.equal(json.column, 2);
        assert.equal(json.line, 1);
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
