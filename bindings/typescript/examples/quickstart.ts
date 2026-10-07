// A weekend trip in a few lines: three friends, two shared expenses, the totals, and who pays whom. The port of the
// Rust example `quickstart`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Balance,
    type Split,
    CalendarDate,
    CurrencyCode,
    ExpenseGroup,
    Money,
    SettlementStrategy,
    expenseTotal,
    formatBalance,
    splitMembers,
} from '../src/index.js';
import { type Columns, load, names, participants, printLines, printTable, section, title, totals } from './report.js';

/** An expense, as the table of expenses prints it. */
interface ExpenseRow {
    readonly name: string;
    readonly payers: string;
    readonly participants: string;
    readonly amount: Money;
}

const expenseColumns: Columns<ExpenseRow> = [
    { header: 'Expense', text: (row) => row.name },
    { header: 'Paid by', text: (row) => row.payers },
    { header: 'Shared by', text: (row) => row.participants },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

/** A member's totals, as the table of members prints them. */
interface MemberRow {
    readonly name: string;
    readonly paid: Money;
    readonly consumed: Money;
    readonly balance: Balance;
}

const memberColumns: Columns<MemberRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** A payment of the plan, as the table of payments prints it. */
interface PaymentRow {
    readonly from: string;
    readonly to: string;
    readonly amount: Money;
}

const paymentColumns: Columns<PaymentRow> = [
    { header: 'From', text: (row) => row.from },
    { header: 'To', text: (row) => row.to },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

await load();
const euro = CurrencyCode.of('EUR');
const group = ExpenseGroup.create('Weekend in Sinaia', euro);
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
const friday = CalendarDate.of(2026, 9, 25);
const everyone: Split = { kind: 'equal', members: [ana, bob, carol] };
group.addExpenseSync({
    name: 'Cabin',
    cost: { kind: 'local', amount: Money.parse('240.00', euro) },
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'whole', split: everyone },
    date: friday,
});
group.addExpenseSync({
    name: 'Groceries',
    cost: { kind: 'local', amount: Money.parse('60.00', euro) },
    payers: { kind: 'equal', members: [bob] },
    split: { kind: 'whole', split: everyone },
    date: friday,
});
printTable(
    expenseColumns,
    group.expensesSync().map((expense) => ({
        name: expense.name,
        payers: names(group, splitMembers(expense.payers)),
        participants: names(group, participants(expense.split)),
        amount: expenseTotal(expense),
    })),
    3,
);

section('3. Who paid and who consumed');
const summary = group.summarySync();
printLines(
    `The group spent ${summary.totalSpent.toString()} in total. A member's share of an expense is what they consumed,`,
    'and their balance is what they paid minus what they consumed.',
);
printTable(
    memberColumns,
    group.membersSync().flatMap((member) => {
        const found = undefined === member.id ? undefined : totals(member.id, summary);
        return undefined === found ? [] : [{ name: member.firstName, paid: found.paid, consumed: found.share, balance: found.balance }];
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
    paymentColumns,
    settlement.payments.map((payment) => ({ from: names(group, [payment.from]), to: names(group, [payment.to]), amount: payment.amount })),
    2,
);
printLines('Once these payments are made, nobody owes anything.');

// Everyone's share is 100.00: Ana gets back 140.00 from Bob (40.00) and Carol (100.00).
assert.ok(Money.parse('300.00', euro).equals(summary.totalSpent));
assert.deepEqual(anaTotals?.balance, { kind: 'getsBack', amount: Money.parse('140.00', euro) });
assert.equal(settlement.payments.length, 2);
