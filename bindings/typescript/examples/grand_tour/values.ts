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
import { message, printLines, printTable, refusal, section } from '../report.js';
import { type People, eur, rate, ron } from './tour.js';

function currencies(): void {
    const rows = ['EUR', 'RON', 'JPY', 'BHD'].map((code) => {
        const currency = Currency.of(code);
        return [currency.code, currency.name, String(currency.numericCode), String(currency.decimalPlaces)];
    });
    const unknown = refusal(() => Currency.of('XYZ'));
    printLines(
        `Currencies are the ${String(Currency.all().length)} active ISO 4217 codes with a minor unit; an unknown code is`,
        `refused (${message(unknown)}):`,
    );
    printTable(['Code', 'Name', 'Numeric code', 'Decimal places'], rows, 2);
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
        ['Input', 'Result'],
        [
            ['12.34 EUR + 0.66 EUR', total.toString()],
            ['5.00 EUR − 7.50 EUR', `${change.toString()}, negative: ${String(0n > change.minorUnits)}`],
            ['1.00 EUR + 1.00 RON', message(mixed)],
            ['1.005 EUR', message(refusal(() => eur('1.005')))],
            ['1500 JPY', `${yen.toString()}, ${String(yen.minorUnits)} minor units`],
            ['1234 minor units of BHD', dinars.toString()],
            ['zero RON', `${nothing.toString()}, zero: ${String(0n === nothing.minorUnits)}`],
            ['10.00 EUR at 4.9765', `${cost.amount.toString()} at ${rate(cost).toString()}, then ${doubled.amount.toString()} at the same rate`],
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
        ['Input', 'Result'],
        [
            ['12.5%', `${percentage.toString()}, ${String(percentage.basisPoints)} basis points`],
            ['333 basis points', Percentage.ofBasisPoints(333).toString()],
            ['33.333%', message(refusal(() => Percentage.parse('33.333')))],
            ['1.5 parts', `${parts.toString()}, ${String(parts.hundredths)} hundredths`],
            ['3 whole parts', Parts.ofHundredths(300).toString()],
            ['25 hundredths of a part', Parts.ofHundredths(25).toString()],
            ['0 parts', message(refusal(() => Parts.parse('0')))],
            ['a rate of 4.97650', exchange.toString()],
            ['a rate of 0', message(refusal(() => ExchangeRate.parse('0')))],
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
        ['Input', 'Result'],
        [
            ['2026-10-01', `${date.toString()}: day ${String(date.day)}, month ${String(date.month)}, year ${String(date.year)}`],
            ['29 February 2027', message(refusal(() => CalendarDate.of(29, 2, 2027)))],
            ['22:15', `${time.toString()}: hour ${String(time.hour)}, minute ${String(time.minute)}`],
            ['24:00', message(refusal(() => TimeOfDay.of(24, 0)))],
            ["Ana's identifier", `${parsed}, the same after a round trip: ${String(people.ana === found)}`],
            ['not-a-uuid', message(refusal(() => group.memberSync(memberId('not-a-uuid'))))],
            ['pt-pt', language.toString()],
            ['pt_PT', message(refusal(() => Language.parse('pt_PT')))],
        ],
        2,
    );
    assert.deepEqual([date.day, date.month, date.year], [1, 10, 2026]);
    assert.deepEqual([time.hour, time.minute], [22, 15]);
    assert.equal(parsed, people.ana);
}

export function values(group: Group, people: People): void {
    section('13. Values on their own');
    currencies();
    amounts();
    ratios();
    datesAndIdentifiers(group, people);
}
