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
import { type Columns, names, printLines, printTable, section } from '../report.js';
import { type Categories, type People, cell, eur, name, october } from './tour.js';

/** The seven expenses, one for every way of dividing one. */
interface Expenses {
    readonly apartment: ExpenseId;
    readonly fado: ExpenseId;
    readonly groceries: ExpenseId;
    readonly museum: ExpenseId;
    readonly seafood: ExpenseId;
    readonly tour: ExpenseId;
    readonly train: ExpenseId;
}

/** An expense with every involved member's payment and consumption, as the table of shares prints it. */
interface ShareRow {
    readonly consumed: string;
    readonly expense: string;
    readonly how: string;
    readonly members: string;
    readonly paid: string;
}

const shareColumns: Columns<ShareRow> = [
    { header: 'Expense', text: (row) => row.expense },
    { header: 'Split', text: (row) => row.how },
    { header: 'Member', text: (row) => row.members },
    { header: 'Paid', text: (row) => row.paid },
    { header: 'Consumed', text: (row) => row.consumed },
];

/** A line of a receipt, as the table of items prints it. */
interface ItemRow {
    readonly lineTotal: Money;
    readonly name: string;
    readonly quantity: number;
    readonly sharedBy: string;
    readonly unitPrice: Money;
}

const itemColumns: Columns<ItemRow> = [
    { header: 'Item', text: (row) => row.name },
    { header: 'Shared by', text: (row) => row.sharedBy },
    { header: 'Quantity', text: (row) => String(row.quantity) },
    { header: 'Unit price', text: (row) => row.unitPrice.toString() },
    { header: 'Line total', text: (row) => row.lineTotal.toString() },
];

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
        shareColumns,
        expenses.flatMap((id) => {
            const expense = group.expenseSync(id);
            const shares = group.expenseSharesSync(id);
            return undefined === expense || undefined === shares
                ? []
                : [
                      {
                          consumed: cell(shares.members.map((share) => share.share.toString())),
                          expense: expense.name,
                          how: how(expense),
                          members: cell(shares.members.map((share) => name(group, share.member))),
                          paid: cell(shares.members.map((share) => (0n === share.paid.minorUnits ? '' : share.paid.toString()))),
                      },
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
        itemColumns,
        split.items.map((item) => {
            const quantity = item.quantity ?? 1;
            return {
                lineTotal: new Money(item.unitPrice.currency, item.unitPrice.minorUnits * BigInt(quantity)),
                name: item.name,
                quantity,
                sharedBy: names(group, splitMembers(item.split)),
                unitPrice: item.unitPrice,
            };
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

/** Adds one expense for every way of dividing one. */
function addExpenses(group: Group, people: People, categories: Categories): Expenses {
    const { ana, bob, carol, dan, eva } = people;
    const groceries = group.addExpenseSync({
        cost: { kind: 'local', amount: eur('10.00') },
        date: october(1),
        name: 'Groceries',
        payers: { kind: 'equal', members: [ana] },
        split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol] } },
        category: categories.food,
    });
    const apartment = group.addExpenseSync({
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
    });
    const seafood = group.addExpenseSync({
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
    });
    const train = group.addExpenseSync({
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
    });
    const museum = group.addExpenseSync({
        cost: { kind: 'local', amount: eur('60.00') },
        date: october(3),
        name: 'Museum',
        payers: { kind: 'equal', members: [ana, bob] },
        split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan] } },
    });
    const fado = group.addExpenseSync({
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
    });
    const tour = group.addExpenseSync({
        cost: { kind: 'local', amount: eur('45.00') },
        date: october(5),
        name: 'Tuk-tuk tour',
        payers: { kind: 'equal', members: [dan] },
        split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan, eva] } },
        category: categories.transport,
        tip: { value: { kind: 'fixed', amount: eur('5.00') }, distribution: TipDistribution.EQUAL },
    });
    return { apartment, fado, groceries, museum, seafood, tour, train };
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
    const { apartment, fado, groceries, museum, seafood, tour, train } = addExpenses(group, people, categories);
    printLines(
        'Seven expenses cover every way of dividing one: equally, by percentage, by parts',
        '(Eva counts as half), by exact amounts, with two payers, item by item, and with a',
        "tip. Every member's consumption adds up to the expense's total, to the cent:",
    );
    printShares(group, [groceries, apartment, seafood, train, museum, fado, tour]);
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
