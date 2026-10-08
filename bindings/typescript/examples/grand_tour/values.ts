// Section 13 of the tour: currencies, amounts, percentages, parts, rates, quantities, dates, times, identifiers, and language
// tags, on their own.

import assert from 'node:assert/strict';

import {
    type Cost,
    CalendarDate,
    CurrencyCode,
    ExchangeRate,
    IdError,
    Language,
    Money,
    MoneyError,
    Parts,
    Percentage,
    Quantity,
    TimeOfDay,
    expenseItemLineTotal,
    parseMemberId,
} from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section } from '../report.js';
import { type People, eur, rate, ron } from './tour.js';

/** A currency of the library's list, as the table of currencies prints it. */
interface CurrencyRow {
    readonly code: string;
    readonly name: string;
    readonly numericCode: number;
    readonly decimalPlaces: number;
}

const currencyColumns: Columns<CurrencyRow> = [
    { header: 'Code', text: (row) => row.code },
    { header: 'Name', text: (row) => row.name },
    { header: 'Numeric code', text: (row) => String(row.numericCode) },
    { header: 'Decimal places', text: (row) => String(row.decimalPlaces) },
];

/** An input with what the library makes of it, as the tables of values print it. */
interface ValueRow {
    readonly input: string;
    readonly result: string;
}

const valueColumns: Columns<ValueRow> = [
    { header: 'Input', text: (row) => row.input },
    { header: 'Result', text: (row) => row.result },
];

function currencies(): void {
    const rows = ['EUR', 'RON', 'JPY', 'BHD'].map((code): CurrencyRow => {
        const currency = CurrencyCode.of(code);
        return { code: currency.code, name: currency.name, numericCode: currency.numericCode, decimalPlaces: currency.decimalPlaces };
    });
    const unknown = refusal(() => CurrencyCode.of('XYZ'));
    printLines(
        `Currencies are the ${String(CurrencyCode.all().length)} active ISO 4217 codes with a minor unit; an unknown code is`,
        `refused (${message(unknown)}):`,
    );
    printTable(currencyColumns, rows, 2);
    assert.equal(CurrencyCode.of('BHD').decimalPlaces, 3);
}

function amounts(): void {
    const total = eur('12.34').plus(eur('0.66'));
    const change = eur('5.00').minus(eur('7.50'));
    const mixed = refusal(() => eur('1.00').plus(ron('1.00')));
    const yen = Money.parse('1500', CurrencyCode.of('JPY'));
    const dinars = new Money(1_234n, CurrencyCode.of('BHD'));
    const nothing = new Money(0n, CurrencyCode.of('RON'));
    const cost: Cost = { kind: 'converted', amount: eur('10.00'), rate: ExchangeRate.parse('4.9765') };
    const doubled: Cost = { kind: 'converted', amount: eur('20.00'), rate: rate(cost) };
    // The double nearest 1.005 lies just below it; its shortest form, 1.005, is what rounds.
    const fromDouble = Money.fromNumber(1.005, CurrencyCode.of('EUR'));
    const notANumber = refusal(() => Money.fromNumber(Number.NaN, CurrencyCode.of('EUR')));
    printLines('Amounts are exact minor units in their currency:');
    printTable(
        valueColumns,
        [
            { input: '12.34 EUR + 0.66 EUR', result: total.toString() },
            { input: '5.00 EUR − 7.50 EUR', result: `${change.toString()}, negative: ${String(0n > change.minorUnits)}` },
            { input: '1.00 EUR + 1.00 RON', result: message(mixed) },
            { input: '1.005 EUR', result: message(refusal(() => eur('1.005'))) },
            { input: '1.005 as a double, in EUR', result: fromDouble.toString() },
            { input: 'NaN as a double, in EUR', result: message(notANumber) },
            { input: '1500 JPY', result: `${yen.toString()}, ${String(yen.minorUnits)} minor units` },
            { input: '1234 minor units of BHD', result: dinars.toString() },
            { input: 'zero RON', result: `${nothing.toString()}, zero: ${String(0n === nothing.minorUnits)}` },
            {
                input: '10.00 EUR at 4.9765',
                result: `${cost.amount.toString()} at ${rate(cost).toString()}, then ${doubled.amount.toString()} at the same rate`,
            },
        ],
        2,
    );
    assert.ok(eur('13.00').equals(total));
    assert.equal(change.minorUnits, -250n);
    assert.ok(undefined !== mixed);
    assert.ok(eur('1.01').equals(fromDouble));
    assert.ok(undefined !== notANumber);
    assert.equal(dinars.toString(), '1.234 BHD');
}

function ratios(): void {
    const percentage = Percentage.parse('12.5');
    const parts = Parts.parse('1.5');
    const exchange = ExchangeRate.parse('4.97650');
    // 0.30000000000000004 as a double, which rounds to two decimal places.
    const sum = Percentage.fromNumber(0.1 + 0.2);
    printLines('Percentages, parts, and rates are exact decimals:');
    printTable(
        valueColumns,
        [
            { input: '12.5%', result: `${percentage.toString()}, ${String(percentage.basisPoints)} basis points` },
            { input: '333 basis points', result: Percentage.ofBasisPoints(333).toString() },
            { input: '33.333%', result: message(refusal(() => Percentage.parse('33.333'))) },
            { input: '0.1 + 0.2 as a double, a percentage', result: sum.toString() },
            { input: '1.5 parts', result: `${parts.toString()}, ${String(parts.hundredths)} hundredths` },
            { input: '3 whole parts', result: Parts.ofHundredths(300).toString() },
            { input: '25 hundredths of a part', result: Parts.ofHundredths(25).toString() },
            { input: '0 parts', result: message(refusal(() => Parts.parse('0'))) },
            { input: 'a rate of 4.97650', result: exchange.toString() },
            { input: 'a rate of 0', result: message(refusal(() => ExchangeRate.parse('0'))) },
        ],
        2,
    );
    assert.equal(percentage.basisPoints, 1_250);
    assert.equal(sum.basisPoints, 30);
    assert.equal(parts.hundredths, 150);
}

function quantities(people: People): void {
    const weight = Quantity.parse('0.452');
    // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
    const olives = expenseItemLineTotal({
        name: 'Olives',
        unitPrice: eur('1.00'),
        split: { kind: 'equal', members: [people.ana] },
        quantity: Quantity.parse('0.335'),
    });
    const fiveDecimals = refusal(() => Quantity.parse('0.00005'));
    const zero = refusal(() => Quantity.parse('0'));
    printLines('Quantities have four decimal places; line totals round half away from zero:');
    printTable(
        valueColumns,
        [
            { input: 'a quantity of 0.452', result: `${weight.toString()}, ${String(weight.tenThousandths)} ten-thousandths` },
            { input: 'a quantity of 0.00005', result: message(fiveDecimals) },
            { input: 'a quantity of 0', result: message(zero) },
            { input: '0.335 at 1.00 EUR', result: olives.toString() },
        ],
        2,
    );
    assert.equal(weight.tenThousandths, 4_520n);
    assert.ok(fiveDecimals instanceof MoneyError && 'invalidQuantity' === fiveDecimals.failure.kind);
    assert.ok(zero instanceof MoneyError && 'invalidQuantity' === zero.failure.kind);
    assert.ok(eur('0.34').equals(olives));
}

function datesAndIdentifiers(people: People): void {
    const date = CalendarDate.parse('2026-10-01');
    const time = TimeOfDay.parse('22:15');
    const text: string = people.ana;
    const parsed = parseMemberId(text);
    const empty = refusal(() => parseMemberId(''));
    const surroundingWhitespace = refusal(() => parseMemberId(' ana'));
    const controlCharacter = refusal(() => parseMemberId('a\tb'));
    const tooLong = refusal(() => parseMemberId('a'.repeat(65)));
    const language = Language.parse('pt-pt');
    printLines('Dates, times, identifiers, and language tags read and print the ISO forms:');
    printTable(
        valueColumns,
        [
            { input: '2026-10-01', result: `${date.toString()}: day ${String(date.day)}, month ${String(date.month)}, year ${String(date.year)}` },
            { input: '29 February 2027', result: message(refusal(() => CalendarDate.of(2027, 2, 29))) },
            { input: '22:15', result: `${time.toString()}: hour ${String(time.hour)}, minute ${String(time.minute)}` },
            { input: '24:00', result: message(refusal(() => TimeOfDay.of(24, 0))) },
            { input: "Ana's identifier", result: `${parsed}, the same after a round trip: ${String(people.ana === parsed)}` },
            { input: 'empty text', result: message(empty) },
            { input: 'ana after a space', result: message(surroundingWhitespace) },
            { input: 'a tab between a and b', result: message(controlCharacter) },
            { input: '65 letters a', result: message(tooLong) },
            { input: 'pt-pt', result: language.toString() },
            { input: 'pt_PT', result: message(refusal(() => Language.parse('pt_PT'))) },
        ],
        2,
    );
    assert.equal(date.day, 1);
    assert.equal(date.month, 10);
    assert.equal(date.year, 2026);
    assert.equal(time.hour, 22);
    assert.equal(time.minute, 15);
    assert.equal(parsed, people.ana);
    assert.ok(empty instanceof IdError && 'empty' === empty.failure.kind);
    assert.ok(surroundingWhitespace instanceof IdError && 'surroundingWhitespace' === surroundingWhitespace.failure.kind);
    assert.ok(controlCharacter instanceof IdError && 'controlCharacter' === controlCharacter.failure.kind);
    assert.ok(tooLong instanceof IdError && 'tooLong' === tooLong.failure.kind);
}

export function values(people: People): void {
    section('13. Values on their own');
    currencies();
    amounts();
    ratios();
    quantities(people);
    datesAndIdentifiers(people);
}
