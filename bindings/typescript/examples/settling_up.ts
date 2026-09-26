// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type MemberId,
    type Settlement,
    CalendarDate,
    Currency,
    Group,
    Money,
    SettlementStrategy,
    expenseTotal,
    formatBalance,
    splitMembers,
} from '../src/index.js';
import { load, names, participants, printLines, printTable, section, totals } from './report.js';

/** Records an expense of `amount` EUR paid by `payer` and split equally. */
function spend(group: Group, name: string, amount: string, payer: MemberId, members: MemberId[]): void {
    group.addExpenseSync({
        cost: { kind: 'local', amount: Money.parse(amount, Currency.of('EUR')) },
        date: CalendarDate.of(20, 9, 2026),
        name,
        payers: { kind: 'equal', members: [payer] },
        split: { kind: 'whole', split: { kind: 'equal', members } },
    });
}

function printBalances(group: Group): void {
    const summary = group.summarySync();
    printTable(
        ['Member', 'Paid', 'Consumed', 'Balance'],
        group.membersSync().flatMap((member) => {
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found ? [] : [[member.firstName, found.paid.toString(), found.share.toString(), formatBalance(found.balance)]];
        }),
        1,
    );
}

function printPayments(group: Group, settlement: Settlement): void {
    printTable(
        ['From', 'To', 'Amount'],
        settlement.payments.map((payment) => [names(group, [payment.from]), names(group, [payment.to]), payment.amount.toString()]),
        2,
    );
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each, in the order minimum, greedy, direct. */
function strategies(group: Group): number[] {
    section('3. Minimum transfers');
    const minimum = group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS);
    printLines(
        'The fewest payments possible. Finding them is NP-hard; the solver does it exactly by',
        'splitting the balances into as many groups that sum to zero as it can: here',
        '{Ana, Dan, Eva} and {Bob, Carol, Filip}, each settled with one payment fewer than',
        `its size. ${String(minimum.payments.length)} payments:`,
    );
    printPayments(group, minimum);

    section('4. Greedy');
    const greedy = group.settlementSync(SettlementStrategy.GREEDY);
    printLines(
        'The largest debtor pays the largest creditor, again and again: fast, at most one',
        `payment fewer than the members with a balance, but not always the minimum. ${String(greedy.payments.length)}`,
        'payments here:',
    );
    printPayments(group, greedy);

    section('5. Direct');
    const direct = group.settlementSync(SettlementStrategy.DIRECT);
    printLines(
        'No simplification: everyone pays back the people who paid for them, with the debts',
        `between each pair of members netted. The most payments (${String(direct.payments.length)}), but each one traces`,
        'back to shared expenses:',
    );
    printPayments(group, direct);

    section('6. The strategies side by side');
    printTable(
        ['Strategy', 'Guarantee', 'Cost', 'Payments here'],
        [
            ['MinimumTransfers', 'the fewest payments', 'O(2ⁿ·n) for n open balances', String(minimum.payments.length)],
            ['Greedy', 'at most n − 1 payments', 'O(n log n)', String(greedy.payments.length)],
            ['Direct', 'one payment per pair in debt', 'O(pairs)', String(direct.payments.length)],
        ],
        3,
    );
    return [minimum.payments.length, greedy.payments.length, direct.payments.length];
}

await load();
using group = Group.create(Currency.of('EUR'), 'Ski week');
const ana = group.addMemberSync({ firstName: 'Ana' });
const bob = group.addMemberSync({ firstName: 'Bob' });
const carol = group.addMemberSync({ firstName: 'Carol' });
const dan = group.addMemberSync({ firstName: 'Dan' });
const eva = group.addMemberSync({ firstName: 'Eva' });
const filip = group.addMemberSync({ firstName: 'Filip' });

printLines('Ski week: three ways to settle up', '═════════════════════════════════');
printLines(
    'Six friends spent a week in the mountains. The library can turn their balances',
    'into payments in three ways; the app picks one, and they differ here.',
);

section('1. What was spent');
spend(group, 'Cabin', '12.00', dan, [dan, eva, filip]);
spend(group, 'Lift passes', '9.00', bob, [bob, carol, filip]);
spend(group, 'Groceries', '6.00', carol, [bob, dan]);
spend(group, 'Coffee', '2.00', ana, [dan, eva]);
spend(group, 'Snacks', '1.00', filip, [ana]);
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

section('2. Balances');
printLines('Four members are owed money and two owe it:');
printBalances(group);

const payments = strategies(group);

section('7. Settling up for real');
const plan = group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS);
for (const payment of plan.payments) {
    group.addTransferSync({
        cost: { kind: 'local', amount: payment.amount },
        date: CalendarDate.of(27, 9, 2026),
        from: payment.from,
        to: payment.to,
    });
}
printLines(
    'Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards',
    'every balance is zero and a new settlement has nothing left to pay:',
);
printBalances(group);
printLines(`Payments still needed: ${String(group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS).payments.length)}.`);

// {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice; greedy pays the largest amounts
// first and needs five; the unsimplified debts between pairs need seven.
assert.deepEqual(payments, [4, 5, 7]);
const summary = group.summarySync();
for (const member of [ana, bob, carol, dan, eva, filip]) {
    assert.deepEqual(totals(member, summary)?.balance, { kind: 'settled' });
}
