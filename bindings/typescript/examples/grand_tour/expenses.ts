// Section 4 of the tour: an expense for every way of dividing one, with each member's payment and consumption.

import assert from 'node:assert/strict';

import {
    type Expense,
    type ExpenseId,
    type ExpenseGroup,
    type ExpenseItem,
    type MemberId,
    type Split,
    Money,
    Parts,
    Percentage,
    TimeOfDay,
    TipDistribution,
    createExpense,
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
    readonly expense: string;
    readonly how: string;
    readonly members: string;
    readonly paid: string;
    readonly consumed: string;
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
    readonly name: string;
    readonly sharedBy: string;
    readonly quantity: number;
    readonly unitPrice: Money;
    readonly lineTotal: Money;
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
        case 'equal':
            return 'equally';
        case 'parts':
            return 'by parts';
        case 'percentages':
            return 'by percentage';
        case 'amounts':
            return 'by amounts';
    }
}

/** How an expense divides, in a few words. */
function how(expense: Expense): string {
    const split = 'whole' === expense.split.kind ? divided(expense.split.split) : `${String(expense.split.items.length)} items`;
    const count = splitMembers(expense.payers).length;
    const payers = 1 === count ? '' : `, ${String(count)} payers`;
    let tip = '';
    if (0n !== expenseTipAmount(expense).minorUnits) {
        tip =
            'percentage' === expense.tip.value.kind
                ? `, ${expense.tip.value.percentage.toString()} tip`
                : `, ${expense.tip.value.amount.toString()} tip`;
    }
    return split + payers + tip;
}

/** Each expense of `expenses`, with every involved member's payment and consumption. */
function printShares(group: ExpenseGroup, expenses: readonly ExpenseId[]): void {
    printTable(
        shareColumns,
        expenses.flatMap((id) => {
            const expense = group.expenseSync(id);
            const shares = group.expenseSharesSync(id);
            return undefined === expense || undefined === shares
                ? []
                : [
                      {
                          expense: expense.name,
                          how: how(expense),
                          members: cell(shares.members.map((share) => name(group, share.member))),
                          paid: cell(shares.members.map((share) => (0n === share.paid.minorUnits ? '' : share.paid.toString()))),
                          consumed: cell(shares.members.map((share) => share.share.toString())),
                      },
                  ];
        }),
        3,
    );
}

function printItems(group: ExpenseGroup, id: ExpenseId): void {
    const split = group.expenseSync(id)?.split;
    if ('itemised' !== split?.kind) {
        return;
    }
    printTable(
        itemColumns,
        split.items.map((item) => {
            const quantity = item.quantity ?? 1;
            return {
                name: item.name,
                sharedBy: names(group, splitMembers(item.split)),
                quantity,
                unitPrice: item.unitPrice,
                lineTotal: new Money(item.unitPrice.minorUnits * BigInt(quantity), item.unitPrice.currency),
            };
        }),
        2,
    );
}

/** The receipt of the fado night: every line with its own split. */
function fadoItems(people: People): ExpenseItem[] {
    return [
        { name: 'Vinho verde', unitPrice: eur('12.00'), split: { kind: 'equal', members: [people.ana, people.bob, people.dan] }, quantity: 2 },
        { name: 'Bacalhau', unitPrice: eur('18.00'), split: { kind: 'equal', members: [people.carol] } },
        {
            name: 'Custard tarts',
            unitPrice: eur('1.50'),
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
            quantity: 6,
        },
        { name: 'Juice', unitPrice: eur('3.00'), split: { kind: 'equal', members: [people.eva] } },
    ];
}

/** Adds one expense for every way of dividing one. */
function addExpenses(group: ExpenseGroup, people: People, categories: Categories): Expenses {
    const { ana, bob, carol, dan, eva } = people;
    const groceries = group.addExpenseSync(
        createExpense({
            name: 'Groceries',
            cost: { kind: 'local', amount: eur('10.00') },
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol] } },
            date: october(1),
            category: categories.food,
        }),
    );
    const apartment = group.addExpenseSync(
        createExpense({
            name: 'Apartment',
            cost: { kind: 'local', amount: eur('800.00') },
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
            date: october(1),
            category: categories.lodging,
        }),
    );
    const seafood = group.addExpenseSync(
        createExpense({
            name: 'Seafood dinner',
            cost: { kind: 'local', amount: eur('90.00') },
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
            date: october(2),
            category: categories.food,
        }),
    );
    const train = group.addExpenseSync(
        createExpense({
            name: 'Train to Porto',
            cost: { kind: 'local', amount: eur('88.50') },
            payers: { kind: 'equal', members: [bob] },
            split: {
                kind: 'whole',
                split: {
                    kind: 'amounts',
                    shares: [
                        { member: ana, amount: eur('24.50') },
                        { member: bob, amount: eur('24.50') },
                        { member: carol, amount: eur('24.50') },
                        { member: eva, amount: eur('15.00') },
                    ],
                },
            },
            date: october(3),
            category: categories.transport,
        }),
    );
    const museum = group.addExpenseSync(
        createExpense({
            name: 'Museum',
            cost: { kind: 'local', amount: eur('60.00') },
            payers: { kind: 'equal', members: [ana, bob] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan] } },
            date: october(3),
        }),
    );
    const fado = group.addExpenseSync(
        createExpense({
            name: 'Fado night',
            cost: { kind: 'local', amount: eur('54.00') },
            payers: {
                kind: 'amounts',
                shares: [
                    { member: carol, amount: eur('39.40') },
                    { member: dan, amount: eur('20.00') },
                ],
            },
            split: { kind: 'itemised', items: fadoItems(people) },
            date: october(4),
            tip: { value: { kind: 'percentage', percentage: Percentage.parse('10') } },
            time: TimeOfDay.of(22, 15),
            category: categories.food,
            description: 'Tasca do Chico',
        }),
    );
    const tour = group.addExpenseSync(
        createExpense({
            name: 'Tuk-tuk tour',
            cost: { kind: 'local', amount: eur('45.00') },
            payers: { kind: 'equal', members: [dan] },
            split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob, carol, dan, eva] } },
            date: october(5),
            tip: { value: { kind: 'fixed', amount: eur('5.00') }, distribution: TipDistribution.EQUAL },
            category: categories.transport,
        }),
    );
    return { apartment, fado, groceries, museum, seafood, tour, train };
}

/** The consumption of `member` in the expense `id`. */
function consumed(group: ExpenseGroup, id: ExpenseId, member: MemberId): Money | undefined {
    return group.expenseSharesSync(id)?.members.find((share) => share.member === member)?.share;
}

/** Prints how the fado night's tip was shared, and its receipt. */
function printFado(group: ExpenseGroup, fado: Expense): void {
    const tip = fado.tip;
    const rule = 'percentage' === tip.value.kind ? tip.value.percentage.toString() : tip.value.amount.toString();
    const shared = TipDistribution.EQUAL === tip.distribution ? 'equally' : 'in proportion to what each member ordered';
    printLines(
        "The groceries' 10.00 EUR did not divide into thirds, so one member consumed the",
        "leftover cent; the expense's identifier picks which, so it moves from expense to",
        `expense. The fado night, at ${fado.time?.toString() ?? ''} on ${fado.date.toString()} at ${fado.description ?? ''}, was itemised; its`,
        `${rule} tip of ${expenseTipAmount(fado).toString()} was shared ${shared}:`,
    );
    printItems(group, fado.id);
    printLines("The tuk-tuk's fixed 5.00 EUR tip was shared equally instead.");
}

/** Checks a consumption: `expected` euros. */
function isEuros(expected: string, found: Money | undefined): boolean {
    return undefined !== found && eur(expected).equals(found);
}

export function expenses(group: ExpenseGroup, people: People, categories: Categories): ExpenseId {
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
