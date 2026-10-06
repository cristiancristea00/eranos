// What the sections of the tour share: the members and categories they name, the amounts and dates of the trip, and
// the tables several sections print.

import {
    type Balance,
    type CategoryId,
    type Cost,
    type Group,
    type MemberId,
    CalendarDate,
    Currency,
    ExchangeRate,
    Money,
    expenseTotal,
    formatBalance,
    splitMembers,
} from '../../src/index.js';
import { type Columns, names, printTable, refusal, totals } from '../report.js';

/** The members of the trip. */
export interface People {
    readonly ana: MemberId;
    readonly bob: MemberId;
    readonly carol: MemberId;
    readonly dan: MemberId;
    readonly eva: MemberId;
    readonly filip: MemberId;
}

/** The categories the group adds to the built-in one. */
export interface Categories {
    readonly food: CategoryId;
    readonly lodging: CategoryId;
    readonly transport: CategoryId;
}

/** A change the group refuses, with the library's message, as the tables of refusals print it. */
export interface RefusalRow {
    readonly change: string;
    readonly error: string;
}

export const refusalColumns: Columns<RefusalRow> = [
    { header: 'Change', text: (row) => row.change },
    { header: 'Refused with', text: (row) => row.error },
];

/** A payment of a plan, as the tables of payments print it. */
export interface PaymentRow {
    readonly amount: Money;
    readonly from: string;
    readonly to: string;
}

export const paymentColumns: Columns<PaymentRow> = [
    { header: 'From', text: (row) => row.from },
    { header: 'To', text: (row) => row.to },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

/** An expense or a transfer, as entered and in the group currency, as the table of entries prints it. */
interface EntryRow {
    readonly converted: Money;
    readonly name: string;
    readonly original: Money;
    readonly payer: string;
    readonly rate: ExchangeRate;
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
    readonly balance: Balance;
    readonly consumed: Money;
    readonly name: string;
    readonly paid: Money;
    readonly received: Money;
    readonly sent: Money;
}

const balanceColumns: Columns<BalanceRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Paid', text: (row) => row.paid.toString() },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Sent', text: (row) => row.sent.toString() },
    { header: 'Received', text: (row) => row.received.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** A category's expenses and total, as the table of categories prints them. */
interface CategoryRow {
    readonly expenses: number;
    readonly name: string;
    readonly total: Money;
}

const categoryColumns: Columns<CategoryRow> = [
    { header: 'Category', text: (row) => row.name },
    { header: 'Expenses', text: (row) => String(row.expenses) },
    { header: 'Total', text: (row) => row.total.toString() },
];

export function eur(text: string): Money {
    return Money.parse(text, Currency.of('EUR'));
}

export function ron(text: string): Money {
    return Money.parse(text, Currency.of('RON'));
}

/** A date of the trip, in October 2026. */
export function october(date: number): CalendarDate {
    return CalendarDate.of(date, 10, 2026);
}

export function name(group: Group, member: MemberId): string {
    return names(group, [member]);
}

/** `lines`, one below the other in a table cell. */
export function cell(lines: readonly string[]): string {
    return lines.join('\n');
}

/** "valid: …" with `shown` of the value `make` returns, or "refused: …" with the message. */
export function outcome<Value>(make: () => Value, shown: (value: Value) => string): string {
    let result = '';
    const refused = refusal(() => {
        result = `valid: ${shown(make())}`;
    });
    return undefined === refused ? result : `refused: ${refused.message}`;
}

/** The sum of `amounts`, all in `currency`. */
export function sum(currency: Currency, amounts: readonly Money[]): Money {
    return amounts.reduce((total, amount) => total.plus(amount), new Money(currency, 0n));
}

/** The rate `cost` was entered at: 1 for a cost in the group currency. */
export function rate(cost: Cost): ExchangeRate {
    return 'converted' === cost.kind ? cost.rate : ExchangeRate.parse('1');
}

/** The identifier of the built-in category, which the group lists first. */
export function builtInCategory(group: Group): CategoryId | undefined {
    return group.categoriesSync()[0]?.id;
}

export function printBalances(group: Group): void {
    const summary = group.summarySync();
    const zero = new Money(group.currencySync(), 0n);
    printTable(
        balanceColumns,
        group.membersSync().map((member): BalanceRow => {
            // The summary lists the members with a total; every total of the others is zero.
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found
                ? { balance: { kind: 'settled' }, consumed: zero, name: member.firstName, paid: zero, received: zero, sent: zero }
                : {
                      balance: found.balance,
                      consumed: found.share,
                      name: member.firstName,
                      paid: found.paid,
                      received: found.received,
                      sent: found.sent,
                  };
        }),
        1,
    );
}

export function printCategories(group: Group): void {
    const expenses = group.expensesSync();
    const builtIn = builtInCategory(group);
    printTable(
        categoryColumns,
        group.summarySync().categoryTotals.map((total) => ({
            expenses: expenses.filter((expense) => (expense.category ?? builtIn) === total.category).length,
            name: group.categorySync(total.category)?.name ?? '?',
            total: total.total,
        })),
        1,
    );
}

/** Every expense and transfer in a currency `shown` accepts, as entered and in the group currency. */
export function printEntries(group: Group, shown: (currency: Currency) => boolean): void {
    const zero = new Money(group.currencySync(), 0n);
    const expenses = group
        .expensesSync()
        .filter((expense) => shown(expense.cost.amount.currency))
        .map((expense): EntryRow => ({
            converted: (undefined === expense.id ? undefined : group.expenseSharesSync(expense.id))?.total ?? zero,
            name: expense.name,
            original: expenseTotal(expense),
            payer: names(group, splitMembers(expense.payers)),
            rate: rate(expense.cost),
        }));
    const transfers = group
        .transfersSync()
        .filter((transfer) => shown(transfer.cost.amount.currency))
        .map((transfer): EntryRow => ({
            converted: (undefined === transfer.id ? undefined : group.transferAmountSync(transfer.id)) ?? zero,
            name: `Transfer to ${name(group, transfer.to)}`,
            original: transfer.cost.amount,
            payer: name(group, transfer.from),
            rate: rate(transfer.cost),
        }));
    printTable(entryColumns, [...expenses, ...transfers], 2);
}
