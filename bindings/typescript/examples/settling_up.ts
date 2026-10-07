// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Balance,
    type MemberId,
    type Settlement,
    CalendarDate,
    Currency,
    ExpenseGroup,
    Money,
    SettlementStrategy,
    expenseTotal,
    formatBalance,
    splitMembers,
} from '../src/index.js';
import { type Columns, load, names, participants, printLines, printTable, section, totals } from './report.js';

/** An expense, as the table of expenses prints it. */
interface ExpenseRow {
    readonly name: string;
    readonly payer: string;
    readonly participants: string;
    readonly amount: Money;
}

const expenseColumns: Columns<ExpenseRow> = [
    { header: 'Expense', text: (row) => row.name },
    { header: 'Paid by', text: (row) => row.payer },
    { header: 'Shared by', text: (row) => row.participants },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

/** A member's totals, as the table of balances prints them. */
interface BalanceRow {
    readonly name: string;
    readonly paid: Money;
    readonly consumed: Money;
    readonly balance: Balance;
}

const balanceColumns: Columns<BalanceRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** A payment of a plan, as the table of payments prints it. */
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

/** A strategy with what it promises, as the table of strategies prints it. */
interface StrategyRow {
    readonly strategy: string;
    readonly guarantee: string;
    readonly cost: string;
    readonly payments: number;
}

const strategyColumns: Columns<StrategyRow> = [
    { header: 'Strategy', text: (row) => row.strategy },
    { header: 'Guarantee', text: (row) => row.guarantee },
    { header: 'Cost', text: (row) => row.cost },
    { header: 'Payments here', text: (row) => String(row.payments) },
];

/** The number of payments of each strategy's plan. */
interface Counts {
    readonly adaptive: number;
    readonly minimum: number;
    readonly greedy: number;
    readonly direct: number;
}

/** The name of `strategy` as the apps show it. */
function strategyName(strategy: SettlementStrategy): string {
    switch (strategy) {
        case SettlementStrategy.ADAPTIVE:
            return 'Adaptive';
        case SettlementStrategy.MINIMUM_TRANSFERS:
            return 'MinimumTransfers';
        case SettlementStrategy.GREEDY:
            return 'Greedy';
        case SettlementStrategy.DIRECT:
            return 'Direct';
    }
}

/** Records an expense of `amount` EUR paid by `payer` and split equally. */
function spend(group: ExpenseGroup, name: string, amount: string, payer: MemberId, members: MemberId[]): void {
    group.addExpenseSync({
        name,
        cost: { kind: 'local', amount: Money.parse(amount, Currency.of('EUR')) },
        payers: { kind: 'equal', members: [payer] },
        split: { kind: 'whole', split: { kind: 'equal', members } },
        date: CalendarDate.of(2026, 9, 20),
    });
}

function printBalances(group: ExpenseGroup): void {
    const summary = group.summarySync();
    printTable(
        balanceColumns,
        group.membersSync().flatMap((member) => {
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found ? [] : [{ name: member.firstName, paid: found.paid, consumed: found.share, balance: found.balance }];
        }),
        1,
    );
}

function printPayments(group: ExpenseGroup, settlement: Settlement): void {
    printTable(
        paymentColumns,
        settlement.payments.map((payment) => ({ from: names(group, [payment.from]), to: names(group, [payment.to]), amount: payment.amount })),
        2,
    );
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each. */
function strategies(group: ExpenseGroup): Counts {
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

    section('6. Adaptive');
    const adaptive = group.settlementSync(SettlementStrategy.ADAPTIVE);
    printLines(
        "The fewest payments while they are quick to find, greedy's beyond: up to 20 open",
        'balances the minimum solver settles them, past that greedy does, and the plan',
        'names the strategy whose plan it is. Six balances are well within the limit, so',
        `the plan is the one ${strategyName(adaptive.strategy)} gives, the same ${String(adaptive.payments.length)} payments.`,
    );

    section('7. The strategies side by side');
    printTable(
        strategyColumns,
        [
            {
                strategy: 'MinimumTransfers',
                guarantee: 'the fewest payments',
                cost: 'O(2ⁿ·n) for n open balances',
                payments: minimum.payments.length,
            },
            { strategy: 'Greedy', guarantee: 'at most n − 1 payments', cost: 'O(n log n)', payments: greedy.payments.length },
            { strategy: 'Direct', guarantee: 'one payment per pair in debt', cost: 'O(pairs)', payments: direct.payments.length },
            {
                strategy: 'Adaptive',
                guarantee: 'the fewest up to 20 balances',
                cost: 'as MinimumTransfers, then Greedy',
                payments: adaptive.payments.length,
            },
        ],
        3,
    );
    return {
        adaptive: adaptive.payments.length,
        minimum: minimum.payments.length,
        greedy: greedy.payments.length,
        direct: direct.payments.length,
    };
}

/**
 * Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which
 * strategy's plan it took; returns the number of payments.
 */
function festival(): number {
    section('8. A larger group');
    const euro = Currency.of('EUR');
    using group = ExpenseGroup.create('Festival', euro);
    const friends: MemberId[] = [];
    for (let number = 1; 21 >= number; number += 1) {
        friends.push(group.addMemberSync({ firstName: `Friend ${String(number)}` }));
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
    // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
    for (const [index, payer] of friends.entries()) {
        const number = BigInt(index + 1);
        group.addExpenseSync({
            name: 'Round',
            cost: { kind: 'local', amount: new Money(21n * number * number, euro) },
            payers: { kind: 'equal', members: [payer] },
            split: { kind: 'whole', split: { kind: 'equal', members: friends } },
            date: CalendarDate.of(2026, 9, 18),
        });
    }
    const open = group.summarySync().members.filter((member) => 0n !== member.net.minorUnits).length;
    const adaptive = group.settlementSync(SettlementStrategy.ADAPTIVE);
    const greedy = group.settlementSync(SettlementStrategy.GREEDY);
    printLines(
        'Twenty-one friends at a festival each bought one round for everyone, each round',
        `at another price, so all ${String(open)} end with a balance and no two of them cancel out.`,
        'That is more than the 20 the adaptive strategy settles exactly, so it takes the',
        `plan ${strategyName(adaptive.strategy)} gives and says so: the same ${String(adaptive.payments.length)} payments.`,
    );
    assert.equal(open, 21);
    assert.equal(adaptive.strategy, SettlementStrategy.GREEDY);
    assert.deepEqual(adaptive.payments, greedy.payments);
    return adaptive.payments.length;
}

await load();
using group = ExpenseGroup.create('Ski week', Currency.of('EUR'));
const ana = group.addMemberSync({ firstName: 'Ana' });
const bob = group.addMemberSync({ firstName: 'Bob' });
const carol = group.addMemberSync({ firstName: 'Carol' });
const dan = group.addMemberSync({ firstName: 'Dan' });
const eva = group.addMemberSync({ firstName: 'Eva' });
const filip = group.addMemberSync({ firstName: 'Filip' });

printLines('Ski week: four ways to settle up', '════════════════════════════════');
printLines('Six friends spent a week in the mountains. The library can turn their balances', 'into payments in four ways, and the app picks one.');

section('1. What was spent');
spend(group, 'Cabin', '12.00', dan, [dan, eva, filip]);
spend(group, 'Lift passes', '9.00', bob, [bob, carol, filip]);
spend(group, 'Groceries', '6.00', carol, [bob, dan]);
spend(group, 'Coffee', '2.00', ana, [dan, eva]);
spend(group, 'Snacks', '1.00', filip, [ana]);
printTable(
    expenseColumns,
    group.expensesSync().map((expense) => ({
        name: expense.name,
        payer: names(group, splitMembers(expense.payers)),
        participants: names(group, participants(expense.split)),
        amount: expenseTotal(expense),
    })),
    3,
);

section('2. Balances');
printLines('Four members are owed money and two owe it:');
printBalances(group);

const counts = strategies(group);
const festivalPayments = festival();

section('9. Settling up for real');
const plan = group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS);
for (const payment of plan.payments) {
    group.addTransferSync({
        from: payment.from,
        to: payment.to,
        cost: { kind: 'local', amount: payment.amount },
        date: CalendarDate.of(2026, 9, 27),
    });
}
printLines(
    'Each payment of the minimum plan, once made, is recorded as a transfer. Afterwards',
    'every balance is zero and a new settlement has nothing left to pay:',
);
printBalances(group);
printLines(`Payments still needed: ${String(group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS).payments.length)}.`);

// {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that one;
// greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
assert.deepEqual(counts, { adaptive: 4, minimum: 4, greedy: 5, direct: 7 });
// Greedy settles the festival's 21 balances in 19 payments.
assert.equal(festivalPayments, 19);
const summary = group.summarySync();
for (const member of [ana, bob, carol, dan, eva, filip]) {
    assert.deepEqual(totals(member, summary)?.balance, { kind: 'settled' });
}
