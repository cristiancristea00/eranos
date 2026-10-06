// Section 13 of the tour: currencies, amounts, percentages, parts, rates, dates, times, identifiers, and language tags,
// on their own.

import assert from 'node:assert/strict';

import {
    type Cost,
    type Group,
    CalendarDate,
    Currency,
    ExchangeRate,
    Language,
    Money,
    Parts,
    Percentage,
    TimeOfDay,
    memberId,
} from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section } from '../report.js';
import { type People, eur, rate, ron } from './tour.js';

/** A currency of the library's list, as the table of currencies prints it. */
interface CurrencyRow {
    readonly code: string;
    readonly decimalPlaces: number;
    readonly name: string;
    readonly numericCode: number;
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
        const currency = Currency.of(code);
        return { code: currency.code, decimalPlaces: currency.decimalPlaces, name: currency.name, numericCode: currency.numericCode };
    });
    const unknown = refusal(() => Currency.of('XYZ'));
    printLines(
        `Currencies are the ${String(Currency.all().length)} active ISO 4217 codes with a minor unit; an unknown code is`,
        `refused (${message(unknown)}):`,
    );
    printTable(currencyColumns, rows, 2);
    assert.equal(Currency.of('BHD').decimalPlaces, 3);
}

function amounts(): void {
    const total = eur('12.34').plus(eur('0.66'));
    const change = eur('5.00').minus(eur('7.50'));
    const mixed = refusal(() => eur('1.00').plus(ron('1.00')));
    const yen = Money.parse('1500', Currency.of('JPY'));
    const dinars = new Money(Currency.of('BHD'), 1_234n);
    const nothing = new Money(Currency.of('RON'), 0n);
    const cost: Cost = { kind: 'converted', amount: eur('10.00'), rate: ExchangeRate.parse('4.9765') };
    const doubled: Cost = { kind: 'converted', amount: eur('20.00'), rate: rate(cost) };
    printLines('Amounts are exact minor units in their currency:');
    printTable(
        valueColumns,
        [
            { input: '12.34 EUR + 0.66 EUR', result: total.toString() },
            { input: '5.00 EUR − 7.50 EUR', result: `${change.toString()}, negative: ${String(0n > change.minorUnits)}` },
            { input: '1.00 EUR + 1.00 RON', result: message(mixed) },
            { input: '1.005 EUR', result: message(refusal(() => eur('1.005'))) },
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
    assert.equal(dinars.toString(), '1.234 BHD');
}

function ratios(): void {
    const percentage = Percentage.parse('12.5');
    const parts = Parts.parse('1.5');
    const exchange = ExchangeRate.parse('4.97650');
    printLines('Percentages, parts, and rates are exact decimals:');
    printTable(
        valueColumns,
        [
            { input: '12.5%', result: `${percentage.toString()}, ${String(percentage.basisPoints)} basis points` },
            { input: '333 basis points', result: Percentage.ofBasisPoints(333).toString() },
            { input: '33.333%', result: message(refusal(() => Percentage.parse('33.333'))) },
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
    assert.equal(parts.hundredths, 150);
}

function datesAndIdentifiers(group: Group, people: People): void {
    const date = CalendarDate.parse('2026-10-01');
    const time = TimeOfDay.parse('22:15');
    const parsed = memberId(people.ana);
    const found = group.memberSync(parsed)?.id;
    const language = Language.parse('pt-pt');
    printLines('Dates, times, identifiers, and language tags read and print the ISO forms:');
    printTable(
        valueColumns,
        [
            { input: '2026-10-01', result: `${date.toString()}: day ${String(date.day)}, month ${String(date.month)}, year ${String(date.year)}` },
            { input: '29 February 2027', result: message(refusal(() => CalendarDate.of(29, 2, 2027))) },
            { input: '22:15', result: `${time.toString()}: hour ${String(time.hour)}, minute ${String(time.minute)}` },
            { input: '24:00', result: message(refusal(() => TimeOfDay.of(24, 0))) },
            { input: "Ana's identifier", result: `${parsed}, the same after a round trip: ${String(people.ana === found)}` },
            { input: 'not-a-uuid', result: message(refusal(() => group.memberSync(memberId('not-a-uuid')))) },
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
}

export function values(group: Group, people: People): void {
    section('13. Values on their own');
    currencies();
    amounts();
    ratios();
    datesAndIdentifiers(group, people);
}
