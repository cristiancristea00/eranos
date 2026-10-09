// The error every call throws. A failure converted to another module's subclass, to another failure, with another payload,
// or without the library's message would show an app another failure than the library's, so every failure the generated
// code can report is converted here, and every message is read back.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { convertedError } from '../src/errors.js';
import {
    PaymentAccountError,
    CalendarError,
    CategoryError,
    ContactError,
    CurrencyCode,
    EranosError,
    ExpenseError,
    ExpenseGroupError,
    IdError,
    InvalidJsonError,
    MismatchedField,
    MismatchedRecordError,
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
    return new Money(minorUnits, CurrencyCode.of('EUR'));
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
        generated: new internal.EranosError.Id({ failure: internal.IdFailure.Empty, detail: 'detail' }),
        type: IdError,
        failure: { kind: 'empty' },
    },
    {
        generated: new internal.EranosError.Id({ failure: internal.IdFailure.TooLong, detail: 'detail' }),
        type: IdError,
        failure: { kind: 'tooLong' },
    },
    {
        generated: new internal.EranosError.Id({ failure: internal.IdFailure.ControlCharacter, detail: 'detail' }),
        type: IdError,
        failure: { kind: 'controlCharacter' },
    },
    {
        generated: new internal.EranosError.Id({ failure: internal.IdFailure.SurroundingWhitespace, detail: 'detail' }),
        type: IdError,
        failure: { kind: 'surroundingWhitespace' },
    },
    {
        generated: new internal.EranosError.PaymentAccount({ failure: new internal.PaymentAccountFailure.EmptyName(), detail: 'detail' }),
        type: PaymentAccountError,
        failure: { kind: 'emptyName' },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({ failure: new internal.PaymentAccountFailure.EmptyUsername(), detail: 'detail' }),
        type: PaymentAccountError,
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
            failure: new internal.MoneyFailure.TooManyDecimals({ currency: internal.currencyDetails('RON'), allowed: 29n }),
            detail: 'detail',
        }),
        type: MoneyError,
        failure: { kind: 'tooManyDecimals', currency: CurrencyCode.of('RON'), allowed: 29 },
    },
    {
        generated: new internal.EranosError.Money({
            failure: new internal.MoneyFailure.CurrencyMismatch({ left: internal.currencyDetails('RON'), right: internal.currencyDetails('USD') }),
            detail: 'detail',
        }),
        type: MoneyError,
        failure: { kind: 'currencyMismatch', left: CurrencyCode.of('RON'), right: CurrencyCode.of('USD') },
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
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.InvalidQuantity(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'invalidQuantity' },
    },
    {
        generated: new internal.EranosError.Money({ failure: new internal.MoneyFailure.NotFinite(), detail: 'detail' }),
        type: MoneyError,
        failure: { kind: 'notFinite' },
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
            failure: new internal.ExpenseFailure.CurrencyMismatch({
                expected: internal.currencyDetails('USD'),
                found: internal.currencyDetails('GBP'),
            }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'currencyMismatch', expected: CurrencyCode.of('USD'), found: CurrencyCode.of('GBP') },
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
                expected: euros(3n).generated(),
                actual: euros(2n).generated(),
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
        generated: new internal.EranosError.Expense({
            failure: new internal.ExpenseFailure.ItemsDoNotAddUp({
                expected: euros(8n).generated(),
                actual: euros(7n).generated(),
            }),
            detail: 'detail',
        }),
        type: ExpenseError,
        failure: { kind: 'itemsDoNotAddUp', expected: euros(8n), actual: euros(7n) },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.NegativeDiscount(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'negativeDiscount' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.DiscountAboveHundredPercent(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'discountAboveHundredPercent' },
    },
    {
        generated: new internal.EranosError.Expense({ failure: new internal.ExpenseFailure.DiscountAbovePrice(), detail: 'detail' }),
        type: ExpenseError,
        failure: { kind: 'discountAbovePrice' },
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
            failure: new internal.ExpenseGroupFailure.MissingExchangeRate({ currency: internal.currencyDetails('RON') }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'missingExchangeRate', currency: CurrencyCode.of('RON') },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({ failure: new internal.ExpenseGroupFailure.UnexpectedExchangeRate(), detail: 'detail' }),
        type: ExpenseGroupError,
        failure: { kind: 'unexpectedExchangeRate' },
    },
    {
        generated: new internal.EranosError.ExpenseGroup({
            failure: new internal.ExpenseGroupFailure.DuplicateExchangeRate({ currency: internal.currencyDetails('GBP') }),
            detail: 'detail',
        }),
        type: ExpenseGroupError,
        failure: { kind: 'duplicateExchangeRate', currency: CurrencyCode.of('GBP') },
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
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidCharacter() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidCharacter' } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidFormat() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidFormat' } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.UnknownCountry() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'unknownCountry' } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.WrongLength({ expected: 24n, found: 22n }) }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'wrongLength', expected: 24, found: 22 } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidStructure() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidStructure' } },
    },
    {
        generated: new internal.EranosError.PaymentAccount({
            failure: new internal.PaymentAccountFailure.InvalidIban({ reason: new internal.IbanReason.InvalidChecksum() }),
            detail: 'detail',
        }),
        type: PaymentAccountError,
        failure: { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
    },
];

void describe('the conversions of every failure the generated code can report', () => {
    /**
     * A conversion that reached another module or failure, dropped what the failure holds, or lost the message would show an
     * app another failure than the library's.
     */
    void it('converts every failure of the generated code to its module and failure', () => {
        assert.equal(conversions.length, 68);
        for (const conversion of conversions) {
            const converted = convertedError(conversion.generated);
            assert.ok(converted instanceof conversion.type);
            assert.ok(converted instanceof EranosError);
            assert.deepEqual(converted.failure, conversion.failure);
            assert.equal(converted.message, 'detail');
        }
    });

    /**
     * A saved group that failed to load without its line and column would leave an app nothing to point the user at, a
     * call the program should not make, reported as a failure of the library, would be caught as one, and a mismatched
     * record that named another field would send the app to check the wrong one.
     */
    void it("converts the bindings' own failures to their standard errors", () => {
        const json = convertedError(new internal.EranosError.InvalidJson({ line: 1n, column: 2n, detail: 'detail' }));
        assert.ok(json instanceof InvalidJsonError);
        assert.equal(json.line, 1);
        assert.equal(json.column, 2);
        assert.equal(json.message, 'detail');
        assert.equal(json.name, 'InvalidJsonError');
        const released = convertedError(new internal.EranosError.Released({ detail: 'detail' }));
        assert.ok(released instanceof TypeError);
        assert.equal(released.message, 'detail');
        const fields = [
            { generated: internal.MismatchedField.IbanCountryCode, field: MismatchedField.IBAN_COUNTRY_CODE },
            { generated: internal.MismatchedField.PhoneInternational, field: MismatchedField.PHONE_INTERNATIONAL },
            { generated: internal.MismatchedField.CurrencyDetails, field: MismatchedField.CURRENCY_DETAILS },
        ];
        for (const { generated, field } of fields) {
            const mismatched = convertedError(new internal.EranosError.MismatchedRecord({ field: generated, detail: 'detail' }));
            assert.ok(mismatched instanceof MismatchedRecordError);
            assert.equal(mismatched.field, field);
            assert.equal(mismatched.message, 'detail');
            assert.equal(mismatched.name, 'MismatchedRecordError');
        }
        const cancelled = convertedError(new internal.EranosError.Cancelled({ detail: 'detail' }));
        assert.ok(cancelled instanceof DOMException);
        assert.equal(cancelled.name, 'AbortError');
        assert.equal(cancelled.message, 'detail');
        const other = new RangeError('other');
        assert.equal(convertedError(other), other);
    });
});
