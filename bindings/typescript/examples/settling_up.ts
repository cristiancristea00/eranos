// Settling up with each strategy on a group where they differ, then recording the chosen plan as transfers until
// everyone is settled. The port of the Rust example `settling_up`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Balance,
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
import { type Columns, load, names, participants, printLines, printTable, section, totals } from './report.js';

/** An expense, as the table of expenses prints it. */
interface ExpenseRow {
    readonly amount: Money;
    readonly name: string;
    readonly participants: string;
    readonly payer: string;
}

const expenseColumns: Columns<ExpenseRow> = [
    { header: 'Expense', text: (row) => row.name },
    { header: 'Paid by', text: (row) => row.payer },
    { header: 'Shared by', text: (row) => row.participants },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

/** A member's totals, as the table of balances prints them. */
interface BalanceRow {
    readonly balance: Balance;
    readonly consumed: Money;
    readonly name: string;
    readonly paid: Money;
}

const balanceColumns: Columns<BalanceRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** A payment of a plan, as the table of payments prints it. */
interface PaymentRow {
    readonly amount: Money;
    readonly from: string;
    readonly to: string;
}

const paymentColumns: Columns<PaymentRow> = [
    { header: 'From', text: (row) => row.from },
    { header: 'To', text: (row) => row.to },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

/** A strategy with what it promises, as the table of strategies prints it. */
interface StrategyRow {
    readonly cost: string;
    readonly guarantee: string;
    readonly payments: number;
    readonly strategy: string;
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
    readonly direct: number;
    readonly greedy: number;
    readonly minimum: number;
}

/** The name of `strategy` as the apps show it. */
function strategyName(strategy: SettlementStrategy): string {
    switch (strategy) {
        case SettlementStrategy.ADAPTIVE:
            return 'Adaptive';
        case SettlementStrategy.DIRECT:
            return 'Direct';
        case SettlementStrategy.GREEDY:
            return 'Greedy';
        case SettlementStrategy.MINIMUM_TRANSFERS:
            return 'MinimumTransfers';
    }
}

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
        balanceColumns,
        group.membersSync().flatMap((member) => {
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found ? [] : [{ balance: found.balance, consumed: found.share, name: member.firstName, paid: found.paid }];
        }),
        1,
    );
}

function printPayments(group: Group, settlement: Settlement): void {
    printTable(
        paymentColumns,
        settlement.payments.map((payment) => ({ amount: payment.amount, from: names(group, [payment.from]), to: names(group, [payment.to]) })),
        2,
    );
}

/** Prints each strategy's plan with what it promises; returns the number of payments of each. */
function strategies(group: Group): Counts {
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
                cost: 'O(2ⁿ·n) for n open balances',
                guarantee: 'the fewest payments',
                payments: minimum.payments.length,
                strategy: 'MinimumTransfers',
            },
            { cost: 'O(n log n)', guarantee: 'at most n − 1 payments', payments: greedy.payments.length, strategy: 'Greedy' },
            { cost: 'O(pairs)', guarantee: 'one payment per pair in debt', payments: direct.payments.length, strategy: 'Direct' },
            {
                cost: 'as MinimumTransfers, then Greedy',
                guarantee: 'the fewest up to 20 balances',
                payments: adaptive.payments.length,
                strategy: 'Adaptive',
            },
        ],
        3,
    );
    return {
        adaptive: adaptive.payments.length,
        direct: direct.payments.length,
        greedy: greedy.payments.length,
        minimum: minimum.payments.length,
    };
}

/**
 * Settles a group of 21 friends, more open balances than the adaptive strategy settles exactly, and prints which
 * strategy's plan it took; returns the number of payments.
 */
function festival(): number {
    section('8. A larger group');
    const euro = Currency.of('EUR');
    using group = Group.create(euro, 'Festival');
    const friends: MemberId[] = [];
    for (let number = 1; 21 >= number; number += 1) {
        friends.push(group.addMemberSync({ firstName: `Friend ${String(number)}` }));
    }
    // Friend i buys a round of 0.21·i² EUR for all 21, so each share is i² cents with nothing to round, and friend i's
    // balance is 21·i² − 3311 cents: never zero, and never the opposite of another's.
    for (const [index, payer] of friends.entries()) {
        const number = BigInt(index + 1);
        group.addExpenseSync({
            cost: { kind: 'local', amount: new Money(euro, 21n * number * number) },
            date: CalendarDate.of(18, 9, 2026),
            name: 'Round',
            payers: { kind: 'equal', members: [payer] },
            split: { kind: 'whole', split: { kind: 'equal', members: friends } },
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
using group = Group.create(Currency.of('EUR'), 'Ski week');
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
        amount: expenseTotal(expense),
        name: expense.name,
        participants: names(group, participants(expense.split)),
        payer: names(group, splitMembers(expense.payers)),
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

// {Ana, Dan, Eva} and {Bob, Carol, Filip} each sum to zero, so four payments suffice, and the adaptive plan is that one;
// greedy pays the largest amounts first and needs five; the unsimplified debts between pairs need seven.
assert.deepEqual(counts, { adaptive: 4, direct: 7, greedy: 5, minimum: 4 });
// Greedy settles the festival's 21 balances in 19 payments.
assert.equal(festivalPayments, 19);
const summary = group.summarySync();
for (const member of [ana, bob, carol, dan, eva, filip]) {
    assert.deepEqual(totals(member, summary)?.balance, { kind: 'settled' });
}
