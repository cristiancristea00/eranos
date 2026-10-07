// Every way to divide an expense: equally, by percentage, by parts (including half a part for a child), by exact
// amounts, and with several payers. The port of the Rust example `split_methods`, which prints the same report.

import assert from 'node:assert/strict';

import { type ExpenseId, type MemberId, CalendarDate, Currency, Group, Money, Parts, Percentage, formatBalance } from '../src/index.js';
import { type Columns, load, names, printLines, printTable, refusal, section, totals } from './report.js';

/** A member's rule, payment, and consumption in one expense, as the table of shares prints them. */
interface ShareRow {
    readonly name: string;
    readonly rule: string;
    readonly paid: Money;
    readonly consumed: Money;
}

const shareColumns: Columns<ShareRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Rule', text: (row) => row.rule },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
];

/** A member's totals over the holiday, as the table of totals prints them. */
interface TotalRow {
    readonly name: string;
    readonly paid: Money;
    readonly consumed: Money;
    readonly balance: string;
}

const totalColumns: Columns<TotalRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Balance', text: (row) => row.balance },
];

function eur(text: string): Money {
    return Money.parse(text, Currency.of('EUR'));
}

/** The family and the day every expense happened. */
interface Family {
    readonly ana: MemberId;
    readonly bob: MemberId;
    readonly mia: MemberId;
    readonly day: CalendarDate;
}

/** The rule of an expense shared equally by the three. */
function thirds(family: Family): Map<MemberId, string> {
    return new Map([
        [family.ana, 'an equal third'],
        [family.bob, 'an equal third'],
        [family.mia, 'an equal third'],
    ]);
}

/** Prints how one expense divides: every involved member's rule, what they paid, and what they consumed. */
function report(group: Group, id: ExpenseId, rules: ReadonlyMap<MemberId, string>): void {
    const expense = group.expenseSync(id);
    const shares = group.expenseSharesSync(id);
    if (undefined === expense || undefined === shares) {
        return;
    }
    printLines(`${expense.name}, ${shares.total.toString()}:`);
    printTable(
        shareColumns,
        shares.members.map((share) => ({
            name: names(group, [share.member]),
            rule: rules.get(share.member) ?? '(payer only)',
            paid: share.paid,
            consumed: share.share,
        })),
        2,
    );
}

function equally(group: Group, family: Family): void {
    section('1. Equally');
    printLines(
        'Everyone pays the same. 10.00 does not divide by three, so one member pays the',
        'leftover cent; which member it is rotates from expense to expense, so over a trip',
        'nobody always pays it.',
    );
    const taxi = group.addExpenseSync({
        name: 'Taxi',
        cost: { kind: 'local', amount: eur('10.00') },
        payers: { kind: 'equal', members: [family.ana] },
        split: { kind: 'whole', split: { kind: 'equal', members: [family.ana, family.bob, family.mia] } },
        date: family.day,
    });
    report(group, taxi, thirds(family));
}

function byPercentage(group: Group, family: Family): void {
    section('2. By percentage');
    printLines(
        'Everyone pays a percentage of the amount; the percentages must add up to exactly',
        "100 %. Mia's room is smaller, so she takes 20 %.",
    );
    const rent = group.addExpenseSync({
        name: 'Flat',
        cost: { kind: 'local', amount: eur('500.00') },
        payers: { kind: 'equal', members: [family.bob] },
        split: {
            kind: 'whole',
            split: {
                kind: 'percentages',
                shares: [
                    { member: family.ana, percentage: Percentage.parse('40') },
                    { member: family.bob, percentage: Percentage.parse('40') },
                    { member: family.mia, percentage: Percentage.parse('20') },
                ],
            },
        },
        date: family.day,
    });
    report(
        group,
        rent,
        new Map([
            [family.ana, '40%'],
            [family.bob, '40%'],
            [family.mia, '20%'],
        ]),
    );
}

function byParts(group: Group, family: Family): ExpenseId {
    section('3. By parts');
    printLines(
        'Everyone pays in proportion to their parts. Mia, a child, counts as half an adult:',
        '2.5 parts in total, so one part is 125.00 / 2.5 = 50.00.',
    );
    const food = group.addExpenseSync({
        name: 'Food',
        cost: { kind: 'local', amount: eur('125.00') },
        payers: { kind: 'equal', members: [family.ana] },
        split: {
            kind: 'whole',
            split: {
                kind: 'parts',
                shares: [
                    { member: family.ana, parts: Parts.parse('1') },
                    { member: family.bob, parts: Parts.parse('1') },
                    { member: family.mia, parts: Parts.parse('0.5') },
                ],
            },
        },
        date: family.day,
    });
    report(
        group,
        food,
        new Map([
            [family.ana, '1 part'],
            [family.bob, '1 part'],
            [family.mia, '0.5 parts'],
        ]),
    );
    return food;
}

function byAmounts(group: Group, family: Family): void {
    section('4. By exact amounts');
    printLines('Everyone pays a fixed amount, such as a ticket price; the amounts must add up to', 'exactly the expense amount.');
    const tickets = group.addExpenseSync({
        name: 'Museum',
        cost: { kind: 'local', amount: eur('35.00') },
        payers: { kind: 'equal', members: [family.bob] },
        split: {
            kind: 'whole',
            split: {
                kind: 'amounts',
                shares: [
                    { member: family.ana, amount: eur('15.00') },
                    { member: family.bob, amount: eur('15.00') },
                    { member: family.mia, amount: eur('5.00') },
                ],
            },
        },
        date: family.day,
    });
    report(
        group,
        tickets,
        new Map([
            [family.ana, '15.00 EUR'],
            [family.bob, '15.00 EUR'],
            [family.mia, '5.00 EUR'],
        ]),
    );
}

function severalPayers(group: Group, family: Family): void {
    section('5. Several payers');
    printLines(
        'The dinner bill went on two cards: Ana paid 50.00 and Bob 40.00. The payers are',
        'divided with the same methods as the participants.',
    );
    const dinner = group.addExpenseSync({
        name: 'Dinner',
        cost: { kind: 'local', amount: eur('90.00') },
        payers: {
            kind: 'amounts',
            shares: [
                { member: family.ana, amount: eur('50.00') },
                { member: family.bob, amount: eur('40.00') },
            ],
        },
        split: { kind: 'whole', split: { kind: 'equal', members: [family.ana, family.bob, family.mia] } },
        date: family.day,
    });
    report(group, dinner, thirds(family));
}

function mistakes(group: Group, family: Family): void {
    section('6. Mistakes are refused');
    printLines('A percentage split that adds up to 90 % is refused when the expense is built:');
    const refused = refusal(() =>
        group.addExpenseSync({
            name: 'Invalid',
            cost: { kind: 'local', amount: eur('10.00') },
            payers: { kind: 'equal', members: [family.ana] },
            split: {
                kind: 'whole',
                split: {
                    kind: 'percentages',
                    shares: [
                        { member: family.ana, percentage: Percentage.parse('60') },
                        { member: family.bob, percentage: Percentage.parse('30') },
                    ],
                },
            },
            date: family.day,
        }),
    );
    assert.ok(undefined !== refused, 'the split must be refused');
    printLines(`  error: ${refused.message}`);
}

function printTotals(group: Group): void {
    section('7. The holiday in total');
    const summary = group.summarySync();
    printLines(`${summary.totalSpent.toString()} spent over five expenses:`);
    printTable(
        totalColumns,
        group.membersSync().flatMap((member) => {
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found
                ? []
                : [{ name: member.firstName, paid: found.paid, consumed: found.share, balance: formatBalance(found.balance) }];
        }),
        1,
    );
}

await load();
using group = Group.create('Family holiday', Currency.of('EUR'));
const family: Family = {
    ana: group.addMemberSync({ firstName: 'Ana' }),
    bob: group.addMemberSync({ firstName: 'Bob' }),
    mia: group.addMemberSync({ firstName: 'Mia' }),
    day: CalendarDate.of(2026, 7, 14),
};

printLines('Family holiday: every way to split an expense', '═════════════════════════════════════════════');
printLines(
    'Ana, Bob, and their daughter Mia share a holiday. Each expense below uses a',
    'different split method; whatever the method, the shares add up to exactly the',
    'amount, to the cent.',
);
equally(group, family);
byPercentage(group, family);
const food = byParts(group, family);
byAmounts(group, family);
severalPayers(group, family);
mistakes(group, family);
printTotals(group);

// Parts: 125.00 over 2.5 parts is 50.00 per part.
const consumed = group.expenseSharesSync(food)?.members.find((share) => share.member === family.mia)?.share;
assert.ok(undefined !== consumed && eur('25.00').equals(consumed));
