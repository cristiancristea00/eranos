// A trip across currencies: the group keeps its books in EUR, expenses and a cash repayment in RON and HUF carry a rate
// fixed when they are entered, and the group later moves its books to RON. The port of the Rust example `multi_currency`,
// which prints the same report.

import assert from 'node:assert/strict';

import {
    type Balance,
    type Cost,
    type MemberId,
    type Split,
    CalendarDate,
    CurrencyCode,
    ExchangeRate,
    ExpenseGroup,
    Money,
    createExpense,
    createMember,
    createTransfer,
    expenseTotal,
    formatBalance,
    splitMembers,
} from '../src/index.js';
import { type Columns, load, names, printLines, printTable, section, totals } from './report.js';

/** An expense or a transfer, as entered and in the group currency, as the table of entries prints it. */
interface EntryRow {
    readonly name: string;
    readonly payer: string;
    readonly original: Money;
    readonly rate: ExchangeRate;
    readonly converted: Money;
}

const entryColumns: Columns<EntryRow> = [
    { header: 'Entry', text: (row) => row.name },
    { header: 'Paid by', text: (row) => row.payer },
    { header: 'Entered as', text: (row) => row.original.toString() },
    { header: 'Rate', text: (row) => row.rate.toString() },
    { header: 'In the group currency', text: (row) => row.converted.toString() },
];

/** A member's totals, as the table of balances prints them. */
interface BalanceRow {
    readonly name: string;
    readonly paid: Money;
    readonly consumed: Money;
    readonly sent: Money;
    readonly received: Money;
    readonly balance: Balance;
}

const balanceColumns: Columns<BalanceRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Sent', text: (row) => row.sent.toString() },
    { header: 'Received', text: (row) => row.received.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** The rate `cost` was entered at: 1 for a cost in the group currency. */
function rate(cost: Cost): ExchangeRate {
    return 'converted' === cost.kind ? cost.rate : ExchangeRate.parse('1');
}

/** Every expense and transfer, as entered and in the group currency. */
function printEntries(group: ExpenseGroup): void {
    const zero = new Money(0n, group.currencySync());
    const expenses = group.expensesSync().map((expense): EntryRow => ({
        name: expense.name,
        payer: names(group, splitMembers(expense.payers)),
        original: expenseTotal(expense),
        rate: rate(expense.cost),
        converted: group.expenseSharesSync(expense.id)?.total ?? zero,
    }));
    const transfers = group.transfersSync().map((transfer): EntryRow => ({
        name: `Transfer to ${names(group, [transfer.receiver])}`,
        payer: names(group, [transfer.sender]),
        original: transfer.cost.amount,
        rate: rate(transfer.cost),
        converted: group.transferAmountSync(transfer.id) ?? zero,
    }));
    printTable(entryColumns, [...expenses, ...transfers], 2);
}

function printBalances(group: ExpenseGroup): void {
    const summary = group.summarySync();
    printTable(
        balanceColumns,
        group.membersSync().flatMap((member) => {
            const found = totals(member.id, summary);
            return undefined === found
                ? []
                : [
                      {
                          name: member.firstName,
                          paid: found.paid,
                          consumed: found.share,
                          sent: found.sent,
                          received: found.received,
                          balance: found.balance,
                      },
                  ];
        }),
        1,
    );
}

function expenses(group: ExpenseGroup, ana: MemberId, bob: MemberId): void {
    section('1. Expenses in three currencies');
    const both: Split = { kind: 'equal', members: [ana, bob] };
    group.addExpenseSync(
        createExpense({
            name: 'Fuel',
            cost: { kind: 'converted', amount: Money.parse('400.00', CurrencyCode.of('RON')), rate: ExchangeRate.parse('0.2') },
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'whole', split: both },
            date: CalendarDate.of(2026, 8, 1),
        }),
    );
    group.addExpenseSync(
        createExpense({
            name: 'Dinner',
            cost: { kind: 'converted', amount: Money.parse('24000.00', CurrencyCode.of('HUF')), rate: ExchangeRate.parse('0.0025') },
            payers: { kind: 'equal', members: [bob] },
            split: { kind: 'whole', split: both },
            date: CalendarDate.of(2026, 8, 2),
        }),
    );
    group.addExpenseSync(
        createExpense({
            name: 'Ferry',
            cost: { kind: 'local', amount: Money.parse('60.00', CurrencyCode.of('EUR')) },
            payers: { kind: 'equal', members: [bob] },
            split: { kind: 'whole', split: both },
            date: CalendarDate.of(2026, 8, 3),
        }),
    );
    printEntries(group);
    printLines(
        `The rate says how many ${group.currencySync().toString()} one unit of the entry's currency is worth: 400.00 RON at`,
        '0.2 is 80.00 EUR. The ferry was paid in the group currency, a local cost at 1.',
    );
}

function repayment(group: ExpenseGroup, ana: MemberId, bob: MemberId): void {
    section('3. A repayment in cash, in another currency');
    group.addTransferSync(
        createTransfer({
            sender: ana,
            receiver: bob,
            cost: { kind: 'converted', amount: Money.parse('100.00', CurrencyCode.of('RON')), rate: ExchangeRate.parse('0.2') },
            date: CalendarDate.of(2026, 8, 4),
            note: 'Cash at the border',
        }),
    );
    printLines(
        'Ana pays Bob 100.00 RON in cash, which at 0.2 is exactly the 20.00 EUR she owed.',
        `Transfers move balances but are not spending, so the total stays at ${group.summarySync().totalSpent.toString()}:`,
    );
    printBalances(group);
}

function newCurrency(group: ExpenseGroup): void {
    section('4. Moving the books to another currency');
    printLines(
        'Changing the group currency needs a rate into the new currency for every other',
        'currency in use: 1 EUR = 5 RON and 1 HUF = 0.0125 RON. Entries in those currencies',
        'take the new rate; entries already in RON become local costs, at 1.',
    );
    group.changeCurrencySync(CurrencyCode.of('RON'), [
        { currency: CurrencyCode.of('EUR'), rate: ExchangeRate.parse('5') },
        { currency: CurrencyCode.of('HUF'), rate: ExchangeRate.parse('0.0125') },
    ]);
    printEntries(group);
    printLines(`In ${group.currencySync().toString()}, the trip cost ${group.summarySync().totalSpent.toString()}, and everyone is still settled:`);
    printBalances(group);
}

await load();
const euro = CurrencyCode.of('EUR');
const leu = CurrencyCode.of('RON');
using group = ExpenseGroup.create('Danube road trip', euro);
const ana = group.addMemberSync(createMember({ firstName: 'Ana' }));
const bob = group.addMemberSync(createMember({ firstName: 'Bob' }));

printLines(`${group.nameSync()}: expenses in several currencies`, '════════════════════════════════════════════════');
printLines(
    `Ana and Bob drive from Romania to Hungary. The group keeps its books in ${group.currencySync().toString()}; an`,
    'expense in another currency carries the exchange rate used when it was entered,',
    'and is converted once, rounding half away from zero to the cent.',
);
expenses(group, ana, bob);

section('2. Balances');
printLines(
    `The group spent ${group.summarySync().totalSpent.toString()}. Bob paid 120.00 of it but consumed half, so he gets back the`,
    'difference:',
);
printBalances(group);
assert.deepEqual(totals(bob, group.summarySync())?.balance, { kind: 'getsBack', amount: Money.parse('20.00', euro) });

repayment(group, ana, bob);
assert.deepEqual(totals(bob, group.summarySync())?.balance, { kind: 'settled' });

newCurrency(group);
// 400 RON of fuel, 300 of dinner, 300 of ferry; still settled.
const summary = group.summarySync();
assert.ok(Money.parse('1000.00', leu).equals(summary.totalSpent));
assert.deepEqual(totals(bob, summary)?.balance, { kind: 'settled' });
