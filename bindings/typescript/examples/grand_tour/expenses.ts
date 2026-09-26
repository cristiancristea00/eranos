// Section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption.

import assert from 'node:assert/strict';

import {
    type Expense,
    type ExpenseId,
    type Group,
    type Item,
    type MemberId,
    type Split,
    Money,
    Parts,
    Percentage,
    TimeOfDay,
    TipDistribution,
    expenseMembers,
    expenseTipAmount,
    expenseTotal,
    splitMembers,
} from '../../src/index.js';
import { names, printLines, printTable, section } from '../report.js';
import { type Categories, type People, cell, eur, name, october } from './tour.js';

/** How a whole expense's amount divides, in a word or two. */
function divided(split: Split): string {
    switch (split.kind) {
        case 'amounts':
            return 'by amounts';
        case 'equal':
            return 'equally';
        case 'parts':
            return 'by parts';
        case 'percentages':
            return 'by percentage';
    }
}

/** How an expense divides, in a few words. */
function how(expense: Expense): string {
    const split = 'itemised' === expense.split.kind ? `${String(expense.split.items.length)} items` : divided(expense.split.split);
    const count = splitMembers(expense.payers).length;
    const payers = 1 === count ? '' : `, ${String(count)} payers`;
    let tip = '';
    if (0n !== expenseTipAmount(expense).minorUnits && undefined !== expense.tip) {
        tip =
            'fixed' === expense.tip.value.kind ? `, ${expense.tip.value.amount.toString()} tip` : `, ${expense.tip.value.percentage.toString()} tip`;
    }
    return split + payers + tip;
}

/** Each expense of `expenses`, with every involved member's payment and consumption. */
function printShares(group: Group, expenses: readonly ExpenseId[]): void {
    printTable(
        ['Expense', 'Split', 'Member', 'Paid', 'Consumed'],
        expenses.flatMap((id) => {
            const expense = group.expenseSync(id);
            const shares = group.expenseSharesSync(id);
            return undefined === expense || undefined === shares
                ? []
                : [
                      [
                          expense.name,
                          how(expense),
                          cell(shares.members.map((share) => name(group, share.member))),
                          cell(shares.members.map((share) => (0n === share.paid.minorUnits ? '' : share.paid.toString()))),
                          cell(shares.members.map((share) => share.share.toString())),
                      ],
                  ];
        }),
        3,
    );
}

function printItems(group: Group, id: ExpenseId): void {
    const split = group.expenseSync(id)?.split;
    if ('itemised' !== split?.kind) {
        return;
    }
    printTable(
        ['Item', 'Shared by', 'Quantity', 'Unit price', 'Line total'],
        split.items.map((item) => {
            const quantity = item.quantity ?? 1;
            const total = new Money(item.unitPrice.currency, item.unitPrice.minorUnits * BigInt(quantity));
            return [item.name, names(group, splitMembers(item.split)), String(quantity), item.unitPrice.toString(), total.toString()];
        }),
        2,
    );
}

/** The receipt of the fado night: every line with its own split. */
function fadoItems(people: People): Item[] {
    return [
        { name: 'Vinho verde', split: { kind: 'equal', members: [people.ana, people.bob, people.dan] }, unitPrice: eur('12.00'), quantity: 2 },
        { name: 'Bacalhau', split: { kind: 'equal', members: [people.carol] }, unitPrice: eur('18.00') },
        {
            name: 'Custard tarts',
            split: {
                kind: 'parts',
                shares: [
                    { member: people.ana, parts: Parts.parse('1') },
                    { member: people.bob, parts: Parts.parse('1') },
                    { member: people.carol, parts: Parts.parse('1') },
                    { member: people.dan, parts: Parts.parse('1') },
                    { member: people.eva, parts: Parts.parse('2') },
                ],
            },
            unitPrice: eur('1.50'),
            quantity: 6,
        },
        { name: 'Juice', split: { kind: 'equal', members: [people.eva] }, unitPrice: eur('3.00') },
    ];
}

/** One expense for every way of dividing one, in order. */
function everyWay(people: People, categories: Categories): Expense[] {
    const { ana, bob, carol, dan, eva } = people;
    return [
        {
            cost: { kind: 'local', amount: eur('10.00') },
            date: october(1),
            name: 'Groceries',
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol] } },
            category: categories.food,
        },
        {
            cost: { kind: 'local', amount: eur('800.00') },
            date: october(1),
            name: 'Apartment',
            payers: { kind: 'equal', members: [ana] },
            split: {
                kind: 'whole',
                split: {
                    kind: 'percentages',
                    shares: [
                        { member: ana, percentage: Percentage.parse('25') },
                        { member: bob, percentage: Percentage.parse('25') },
                        { member: carol, percentage: Percentage.parse('35') },
                        { member: dan, percentage: Percentage.parse('15') },
                    ],
                },
            },
            category: categories.lodging,
        },
        {
            cost: { kind: 'local', amount: eur('90.00') },
            date: october(2),
            name: 'Seafood dinner',
            payers: { kind: 'equal', members: [carol] },
            split: {
                kind: 'whole',
                split: {
                    kind: 'parts',
                    shares: [
                        { member: ana, parts: Parts.parse('1') },
                        { member: bob, parts: Parts.parse('1') },
                        { member: carol, parts: Parts.parse('1') },
                        { member: dan, parts: Parts.parse('1') },
                        { member: eva, parts: Parts.parse('0.5') },
                    ],
                },
            },
            category: categories.food,
        },
        {
            cost: { kind: 'local', amount: eur('88.50') },
            date: october(3),
            name: 'Train to Porto',
            payers: { kind: 'equal', members: [bob] },
            split: {
                kind: 'whole',
                split: {
                    kind: 'amounts',
                    shares: [
                        { amount: eur('24.50'), member: ana },
                        { amount: eur('24.50'), member: bob },
                        { amount: eur('24.50'), member: carol },
                        { amount: eur('15.00'), member: eva },
                    ],
                },
            },
            category: categories.transport,
        },
        {
            cost: { kind: 'local', amount: eur('60.00') },
            date: october(3),
            name: 'Museum',
            payers: { kind: 'equal', members: [ana, bob] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan] } },
        },
        {
            cost: { kind: 'local', amount: eur('54.00') },
            date: october(4),
            name: 'Fado night',
            payers: {
                kind: 'amounts',
                shares: [
                    { amount: eur('39.40'), member: carol },
                    { amount: eur('20.00'), member: dan },
                ],
            },
            split: { kind: 'itemised', items: fadoItems(people) },
            category: categories.food,
            description: 'Tasca do Chico',
            time: TimeOfDay.of(22, 15),
            tip: { value: { kind: 'percentage', percentage: Percentage.parse('10') } },
        },
        {
            cost: { kind: 'local', amount: eur('45.00') },
            date: october(5),
            name: 'Tuk-tuk tour',
            payers: { kind: 'equal', members: [dan] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan, eva] } },
            category: categories.transport,
            tip: { value: { kind: 'fixed', amount: eur('5.00') }, distribution: TipDistribution.EQUAL },
        },
    ];
}

/** The consumption of `member` in the expense `id`. */
function consumed(group: Group, id: ExpenseId, member: MemberId): Money | undefined {
    return group.expenseSharesSync(id)?.members.find((share) => share.member === member)?.share;
}

/** Prints how the fado night's tip was shared, and its receipt. */
function printFado(group: Group, fado: Expense): void {
    const tip = fado.tip;
    const rule = undefined === tip ? 'a tip' : 'fixed' === tip.value.kind ? tip.value.amount.toString() : tip.value.percentage.toString();
    const shared =
        undefined === tip ? 'another way' : TipDistribution.EQUAL === tip.distribution ? 'equally' : 'in proportion to what each member ordered';
    printLines(
        "The groceries' 10.00 EUR did not divide into thirds, so one member consumed the",
        "leftover cent; the expense's identifier picks which, so it moves from expense to",
        `expense. The fado night, at ${fado.time?.toString() ?? ''} on ${fado.date.toString()} at ${fado.description ?? ''}, was itemised; its`,
        `${rule} tip of ${expenseTipAmount(fado).toString()} was shared ${shared}:`,
    );
    if (undefined !== fado.id) {
        printItems(group, fado.id);
    }
    printLines("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
}

/** Checks a consumption: `expected` euros. */
function isEuros(expected: string, found: Money | undefined): boolean {
    return undefined !== found && eur(expected).equals(found);
}

export function expenses(group: Group, people: People, categories: Categories): ExpenseId {
    section('4. Expenses, one for every way of dividing');
    const ids = everyWay(people, categories).map((expense) => group.addExpenseSync(expense));
    const [groceries, apartment, seafood, train, museum, fado, tour] = ids;
    assert.ok(
        undefined !== groceries &&
            undefined !== apartment &&
            undefined !== seafood &&
            undefined !== train &&
            undefined !== museum &&
            undefined !== fado &&
            undefined !== tour,
        'seven expenses were added',
    );
    printLines(
        'Seven expenses cover every way of dividing one: equally, by percentage, by parts',
        '(Eva counts as half), by exact amounts, with two payers, item by item, and with a',
        "tip. Every member's consumption adds up to the expense's total, to the cent:",
    );
    printShares(group, ids);
    const fadoExpense = group.expenseSync(fado);
    assert.ok(undefined !== fadoExpense, 'the fado night was added');
    printFado(group, fadoExpense);

    const thirds = [people.ana, people.bob, people.carol].map((member) => consumed(group, groceries, member)?.minorUnits);
    assert.deepEqual(
        thirds.sort((first, second) => Number((first ?? 0n) - (second ?? 0n))),
        [333n, 333n, 334n],
    );
    assert.ok(isEuros('280.00', consumed(group, apartment, people.carol)));
    assert.ok(isEuros('10.00', consumed(group, seafood, people.eva)));
    assert.ok(isEuros('15.00', consumed(group, train, people.eva)));
    assert.ok(isEuros('30.00', group.expenseSharesSync(museum)?.members.find((share) => share.member === people.bob)?.paid));
    assert.ok(isEuros('15.00', consumed(group, museum, people.dan)));
    const museumExpense = group.expenseSync(museum);
    assert.equal(undefined === museumExpense ? 0 : expenseMembers(museumExpense).length, 4);
    assert.ok(isEuros('59.40', expenseTotal(fadoExpense)));
    assert.ok(isEuros('5.40', expenseTipAmount(fadoExpense)));
    assert.ok(isEuros('21.45', consumed(group, fado, people.carol)));
    assert.ok(isEuros('6.60', consumed(group, fado, people.eva)));
    assert.ok(isEuros('10.00', consumed(group, tour, people.eva)));
    return seafood;
}
