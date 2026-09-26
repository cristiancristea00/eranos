// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.

import assert from 'node:assert/strict';

import { type Split, CalendarDate, Currency, Group, Money, SettlementStrategy, expenseTotal, formatBalance, splitMembers } from '../src/index.js';
import { load, names, participants, printLines, printTable, section, title, totals } from './report.js';

await load();
const euro = Currency.of('EUR');
const group = Group.create(euro, 'Weekend in Sinaia');
title(group.nameSync());
printLines(
    `Three friends rent a cabin and buy groceries. The group keeps its books in ${group.currencySync().toString()}, so`,
    `every total, balance, and payment below is in ${group.currencySync().toString()}.`,
);

section('1. Members');
const ana = group.addMemberSync({ firstName: 'Ana' });
const bob = group.addMemberSync({ firstName: 'Bob' });
const carol = group.addMemberSync({ firstName: 'Carol' });
printLines(`${String(group.membersSync().length)} people joined the group: ${names(group, [ana, bob, carol])}.`);

section('2. Expenses');
printLines('Each expense records who paid and who shares it. Here everyone shares everything', 'equally.');
const friday = CalendarDate.of(25, 9, 2026);
const everyone: Split = { kind: 'equal', members: [ana, bob, carol] };
group.addExpenseSync({
    cost: { kind: 'local', amount: Money.parse('240.00', euro) },
    date: friday,
    name: 'Cabin',
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'whole', split: everyone },
});
group.addExpenseSync({
    cost: { kind: 'local', amount: Money.parse('60.00', euro) },
    date: friday,
    name: 'Groceries',
    payers: { kind: 'equal', members: [bob] },
    split: { kind: 'whole', split: everyone },
});
printTable(
    ['Expense', 'Paid by', 'Shared by', 'Amount'],
    group
        .expensesSync()
        .map((expense) => [
            expense.name,
            names(group, splitMembers(expense.payers)),
            names(group, participants(expense.split)),
            expenseTotal(expense).toString(),
        ]),
    3,
);

section('3. Who paid and who consumed');
const summary = group.summarySync();
printLines(
    `The group spent ${summary.totalSpent.toString()} in total. A member's share of an expense is what they consumed,`,
    'and their balance is what they paid minus what they consumed.',
);
printTable(
    ['Member', 'Paid', 'Consumed', 'Balance'],
    group.membersSync().flatMap((member) => {
        const found = undefined === member.id ? undefined : totals(member.id, summary);
        return undefined === found ? [] : [[member.firstName, found.paid.toString(), found.share.toString(), formatBalance(found.balance)]];
    }),
    1,
);
const anaTotals = totals(ana, summary);
printLines(
    `Ana paid ${anaTotals?.paid.toString() ?? '?'} but consumed only ${anaTotals?.share.toString() ?? '?'}, so the others owe her the difference.`,
);

section('4. Settling up');
const settlement = group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS);
printLines(`The fewest payments that bring every balance to zero (${String(settlement.payments.length)} in total):`);
printTable(
    ['From', 'To', 'Amount'],
    settlement.payments.map((payment) => [names(group, [payment.from]), names(group, [payment.to]), payment.amount.toString()]),
    2,
);
printLines('Once these payments are made, nobody owes anything.');

// Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
assert.ok(Money.parse('300.00', euro).equals(summary.totalSpent));
assert.deepEqual(anaTotals?.balance, { kind: 'getsBack', amount: Money.parse('140.00', euro) });
assert.equal(settlement.payments.length, 2);
