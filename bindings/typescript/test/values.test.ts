// The value types. Every text they show comes from the library, every value an app builds is checked by the library's rules,
// and a value the library never sends still prints something: a type that formatted on its own, let a refused value through,
// or threw on an impossible one would show an app another value than the library holds.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
    PaymentAccountError,
    CalendarDate,
    CalendarError,
    ContactError,
    CurrencyCode,
    Email,
    ExchangeRate,
    Iban,
    IdError,
    Language,
    LanguageError,
    Money,
    MoneyError,
    Parts,
    Percentage,
    PhoneNumber,
    TimeOfDay,
    Username,
    categoryId,
    expenseId,
    formatBalance,
    expenseGroupId,
    memberId,
    parseCategoryId,
    parseExpenseId,
    parseExpenseGroupId,
    parseMemberId,
    parseTransferId,
    transferId,
} from '../src/index.js';
import * as internal from '../src/internal/eranos.js';
import { initialised, refusedIdentifiers } from './flat.js';

await initialised;

/** Asserts that `call` throws an error of `type` whose failure is `failure`. */
function assertRefused(call: () => unknown, type: new (...parameters: never[]) => Error, failure?: unknown, message?: string): void {
    assert.throws(call, (error: unknown) => {
        assert.ok(error instanceof type);
        if (undefined !== failure) {
            assert.deepEqual((error as unknown as { failure: unknown }).failure, failure);
        }
        if (undefined !== message) {
            assert.equal(error.message, message);
        }
        return true;
    });
}

void describe('the currencies, amounts, ratios, dates, times, contact details, and identifiers', () => {
    /** A currency, an amount, or a check that read or wrote another way than the library's would show an app another amount than the group holds. */
    void it('reads and writes currencies and amounts as the library does', () => {
        const euro = CurrencyCode.of('EUR');
        assert.equal(euro.code, 'EUR');
        assert.equal(euro.name, 'Euro');
        assert.equal(euro.toString(), 'EUR');
        assert.equal(euro.numericCode, 978);
        assert.equal(euro.decimalPlaces, 2);
        assert.ok(CurrencyCode.all().includes(euro));
        assert.equal(CurrencyCode.of('EUR'), euro);
        assert.equal(CurrencyCode.known('EUR'), euro);
        assertRefused(() => CurrencyCode.of('XYZ'), MoneyError, { kind: 'unknownCurrency' }, 'unknown currency code');
        const lei = CurrencyCode.of('RON');

        const price = Money.parse('12.5', euro);
        assert.ok(price.equals(new Money(1_250n, euro)));
        assert.ok(!price.equals(new Money(1_251n, euro)));
        assert.ok(!price.equals(new Money(1_250n, lei)));
        assert.equal(price.toString(), '12.50 EUR');
        assert.ok(price.plus(new Money(50n, euro)).equals(new Money(1_300n, euro)));
        assert.ok(price.minus(new Money(50n, euro)).equals(new Money(1_200n, euro)));
        assertRefused(() => price.plus(new Money(1n, lei)), MoneyError, { kind: 'currencyMismatch', left: euro, right: lei });
        assertRefused(() => new Money(9_223_372_036_854_775_807n, euro).plus(new Money(1n, euro)), MoneyError);
        assertRefused(() => new Money(-9_223_372_036_854_775_808n, euro).minus(new Money(1n, euro)), MoneyError);
        assertRefused(() => Money.parse('twelve', euro), MoneyError);
    });

    /** A rate, a percentage, or a number of parts read or written another way than the library's would divide an expense another way. */
    void it('reads and writes rates, percentages, and parts as the library does', () => {
        const rate = ExchangeRate.parse('4.970');
        assert.equal(rate.toString(), '4.97');
        assert.ok(rate.equals(ExchangeRate.parse('4.97')));
        assert.ok(!rate.equals(ExchangeRate.parse('5')));
        const lei = CurrencyCode.of('RON');
        const euro = CurrencyCode.of('EUR');
        assert.ok(ExchangeRate.parse('0.25').convert(new Money(2n, lei), euro).equals(new Money(1n, euro)));
        assertRefused(() => ExchangeRate.parse('0'), MoneyError);
        assertRefused(() => ExchangeRate.parse('2').convert(new Money(9_223_372_036_854_775_807n, lei), euro), MoneyError);

        const percentage = Percentage.parse('12.5 %');
        assert.equal(percentage.basisPoints, 1_250);
        assert.equal(percentage.toString(), '12.5%');
        assert.ok(percentage.equals(Percentage.ofBasisPoints(1_250)));
        assert.ok(!percentage.equals(Percentage.ofBasisPoints(1)));
        assertRefused(() => Percentage.ofBasisPoints(-1), MoneyError);
        assertRefused(() => Percentage.parse('half'), MoneyError);

        const parts = Parts.parse('0.5');
        assert.equal(parts.hundredths, 50);
        assert.equal(parts.toString(), '0.5');
        assert.ok(parts.equals(Parts.ofHundredths(50)));
        assert.ok(!parts.equals(Parts.ofHundredths(1)));
        assertRefused(() => Parts.ofHundredths(0), MoneyError);
        assertRefused(() => Parts.parse('half'), MoneyError);
    });

    /** A date or a time read or written another way than the library's, or one that does not exist, would put an entry on another day. */
    void it('reads and writes dates and times as the library does', () => {
        const date = CalendarDate.of(2026, 9, 23);
        assert.equal(date.toString(), '2026-09-23');
        assert.equal(date.year, 2026);
        assert.equal(date.month, 9);
        assert.equal(date.day, 23);
        assert.ok(date.equals(CalendarDate.parse('2026-09-23')));
        assert.ok(!date.equals(CalendarDate.of(2026, 9, 24)));
        assertRefused(
            () => CalendarDate.of(2026, 2, 30),
            CalendarError,
            { kind: 'invalidDate' },
            'invalid date: expected an existing ISO 8601 date between years 1 and 9999',
        );
        assertRefused(() => CalendarDate.parse('2026-02-30'), CalendarError);

        const time = TimeOfDay.of(18, 30);
        assert.equal(time.toString(), '18:30');
        assert.equal(time.hour, 18);
        assert.equal(time.minute, 30);
        assert.ok(time.equals(TimeOfDay.parse('18:30')));
        assert.ok(!time.equals(TimeOfDay.of(18, 31)));
        assertRefused(() => TimeOfDay.of(24, 0), CalendarError);
        assertRefused(() => TimeOfDay.parse('25:00'), CalendarError);
    });

    /** Contact and account details kept in another form than the library's, or refused ones let through, would reach the group unchecked. */
    void it('reads contact and account details as the library does', () => {
        const email = Email.parse(' ana@example.com ');
        assert.equal(email.toString(), 'ana@example.com');
        assert.ok(email.equals(Email.parse('ana@example.com')));
        assert.ok(!email.equals(Email.parse('bob@example.com')));
        assertRefused(() => Email.parse('ana'), ContactError);

        const phone = PhoneNumber.parse('+40 712 345 678');
        assert.equal(phone.e164, '+40712345678');
        assert.equal(phone.toString(), '+40 712 345 678');
        assert.ok(phone.equals(PhoneNumber.parse('+40712345678')));
        assert.ok(!phone.equals(PhoneNumber.parse('+40712345679')));
        assertRefused(() => PhoneNumber.parse('0712345678'), ContactError);

        const iban = Iban.parse('ro49 aaaa 1b31 0075 9384 0000');
        assert.equal(iban.toString(), 'RO49AAAA1B31007593840000');
        assert.equal(iban.countryCode, 'RO');
        assert.ok(iban.equals(Iban.parse('RO49AAAA1B31007593840000')));
        assert.ok(!iban.equals(Iban.parse('GB82WEST12345698765432')));
        assertRefused(
            () => Iban.parse('RO49AAAA1B31007593840001'),
            PaymentAccountError,
            { kind: 'invalidIban', reason: { kind: 'invalidChecksum' } },
            'invalid IBAN: the check digits do not match',
        );

        const username = Username.parse(' ana.pays ');
        assert.equal(username.toString(), 'ana.pays');
        assert.ok(username.equals(Username.parse('ana.pays')));
        assert.ok(!username.equals(Username.parse('bob.pays')));
        assertRefused(() => Username.parse(' '), PaymentAccountError);

        const language = Language.parse('pt-pt');
        assert.equal(language.toString(), 'pt-PT');
        assert.ok(language.equals(Language.parse('pt-PT')));
        assert.ok(!language.equals(Language.parse('ro-RO')));
        assertRefused(
            () => Language.parse('not a tag'),
            LanguageError,
            { kind: 'invalidLanguage' },
            'invalid language tag: expected a BCP 47 tag such as `en` or `ro-RO`',
        );
    });

    /**
     * A check of an identifier that let through text the library refuses, refused it for another rule, or changed the text
     * would let an app store a key that no group takes, or read back another one than it stored.
     */
    void it('checks an identifier by the rules every kind shares and keeps its text', () => {
        const text = 'Ana Popa, café ☕ 🍰';
        for (const parse of [parseExpenseGroupId, parseMemberId, parseCategoryId, parseExpenseId, parseTransferId]) {
            assert.equal(parse(text), text);
            for (const refused of refusedIdentifiers) {
                assertRefused(() => parse(refused.text), IdError, refused.failure, refused.message);
            }
        }
        // The rule counts bytes of UTF-8, not the characters or the UTF-16 code units of the text: é takes two bytes.
        assert.equal(parseMemberId('é'.repeat(32)), 'é'.repeat(32));
        assertRefused(() => parseMemberId('é'.repeat(33)), IdError, { kind: 'tooLong' });
    });

    /** Identifiers of different entities that printed as another text would let an app store one and read back another. */
    void it('keeps an identifier as its text', () => {
        const text = '0192f1c3-0000-7000-8000-000000000001';
        assert.equal(expenseGroupId(text), text);
        assert.equal(memberId(text), text);
        assert.equal(categoryId(text), text);
        assert.equal(expenseId(text), text);
        assert.equal(transferId(text), text);
    });

    /** A value the library never sends, such as an amount in a currency it does not list, would throw from its text instead of printing. */
    void it('prints a value the library never sends', () => {
        const unlisted = CurrencyCode.known('XYZ');
        assert.equal(unlisted.code, 'XYZ');
        assert.equal(unlisted.name, 'XYZ');
        assert.equal(unlisted.numericCode, 0);
        assert.equal(unlisted.decimalPlaces, 0);
        assert.equal(new Money(5n, unlisted).toString(), '5 XYZ');
        assert.equal(Percentage.known(-1).toString(), '-1 bp');
        assert.equal(Parts.known(0).toString(), '0/100');
        assert.equal(CalendarDate.converted(internal.CalendarDate.create({ year: 2026, month: 2, day: 30 })).toString(), '2026/2/30');
        assert.equal(TimeOfDay.converted(internal.TimeOfDay.create({ hour: 25, minute: 0 })).toString(), '25h0');
        assert.equal(PhoneNumber.known('0712').toString(), '0712');
        assert.equal(Iban.known('XY').countryCode, 'XY');
        assert.equal(formatBalance({ kind: 'owes', amount: new Money(1n, unlisted) }), 'owes 1 XYZ');
    });
});
