// What the sections of the tour share: the members and categories they name, the amounts and dates of the trip, and
// the tables several sections print.

import {
    type Balance,
    type CategoryId,
    type Cost,
    type ExpenseGroup,
    type MemberId,
    CalendarDate,
    CurrencyCode,
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
    readonly from: string;
    readonly to: string;
    readonly amount: Money;
}

export const paymentColumns: Columns<PaymentRow> = [
    { header: 'From', text: (row) => row.from },
    { header: 'To', text: (row) => row.to },
    { header: 'Amount', text: (row) => row.amount.toString() },
];

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

/** A category's expenses and total, as the table of categories prints them. */
interface CategoryRow {
    readonly name: string;
    readonly expenses: number;
    readonly total: Money;
}

const categoryColumns: Columns<CategoryRow> = [
    { header: 'Category', text: (row) => row.name },
    { header: 'Expenses', text: (row) => String(row.expenses) },
    { header: 'Total', text: (row) => row.total.toString() },
];

export function eur(text: string): Money {
    return Money.parse(text, CurrencyCode.of('EUR'));
}

export function ron(text: string): Money {
    return Money.parse(text, CurrencyCode.of('RON'));
}

/** A date of the trip, in October 2026. */
export function october(date: number): CalendarDate {
    return CalendarDate.of(2026, 10, date);
}

export function name(group: ExpenseGroup, member: MemberId): string {
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
export function sum(currency: CurrencyCode, amounts: readonly Money[]): Money {
    return amounts.reduce((total, amount) => total.plus(amount), new Money(0n, currency));
}

/** The rate `cost` was entered at: 1 for a cost in the group currency. */
export function rate(cost: Cost): ExchangeRate {
    return 'converted' === cost.kind ? cost.rate : ExchangeRate.parse('1');
}

/** The identifier of the built-in category, which the group lists first. */
export function builtInCategory(group: ExpenseGroup): CategoryId | undefined {
    return group.categoriesSync()[0]?.id;
}

export function printBalances(group: ExpenseGroup): void {
    const summary = group.summarySync();
    const zero = new Money(0n, group.currencySync());
    printTable(
        balanceColumns,
        group.membersSync().map((member): BalanceRow => {
            // The summary lists the members with a total; every total of the others is zero.
            const found = totals(member.id, summary);
            return undefined === found
                ? { name: member.firstName, paid: zero, consumed: zero, sent: zero, received: zero, balance: { kind: 'settled' } }
                : {
                      name: member.firstName,
                      paid: found.paid,
                      consumed: found.share,
                      sent: found.sent,
                      received: found.received,
                      balance: found.balance,
                  };
        }),
        1,
    );
}

export function printCategories(group: ExpenseGroup): void {
    const expenses = group.expensesSync();
    const builtIn = builtInCategory(group);
    printTable(
        categoryColumns,
        group.summarySync().categoryTotals.map((total) => ({
            name: group.categorySync(total.category)?.name ?? '?',
            expenses: expenses.filter((expense) => (expense.category ?? builtIn) === total.category).length,
            total: total.total,
        })),
        1,
    );
}

/** Every expense and transfer in a currency `shown` accepts, as entered and in the group currency. */
export function printEntries(group: ExpenseGroup, shown: (currency: CurrencyCode) => boolean): void {
    const zero = new Money(0n, group.currencySync());
    const expenses = group
        .expensesSync()
        .filter((expense) => shown(expense.cost.amount.currency))
        .map((expense): EntryRow => ({
            name: expense.name,
            payer: names(group, splitMembers(expense.payers)),
            original: expenseTotal(expense),
            rate: rate(expense.cost),
            converted: group.expenseSharesSync(expense.id)?.total ?? zero,
        }));
    const transfers = group
        .transfersSync()
        .filter((transfer) => shown(transfer.cost.amount.currency))
        .map((transfer): EntryRow => ({
            name: `Transfer to ${name(group, transfer.to)}`,
            payer: name(group, transfer.from),
            original: transfer.cost.amount,
            rate: rate(transfer.cost),
            converted: group.transferAmountSync(transfer.id) ?? zero,
        }));
    printTable(entryColumns, [...expenses, ...transfers], 2);
}
