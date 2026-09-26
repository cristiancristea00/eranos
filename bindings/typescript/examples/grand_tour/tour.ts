// What the sections of the tour share: the members and categories they name, the amounts and dates of the trip, and
// the tables several sections print.

import {
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
import { names, printTable, refusal, totals } from '../report.js';

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
    const zero = new Money(group.currencySync(), 0n).toString();
    printTable(
        ['Member', 'Paid', 'Consumed', 'Sent', 'Received', 'Balance'],
        group.membersSync().map((member) => {
            // The summary lists the members with a total; every total of the others is zero.
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === found
                ? [member.firstName, zero, zero, zero, zero, formatBalance({ kind: 'settled' })]
                : [
                      member.firstName,
                      found.paid.toString(),
                      found.share.toString(),
                      found.sent.toString(),
                      found.received.toString(),
                      formatBalance(found.balance),
                  ];
        }),
        1,
    );
}

export function printCategories(group: Group): void {
    const expenses = group.expensesSync();
    const builtIn = builtInCategory(group);
    printTable(
        ['Category', 'Expenses', 'Total'],
        group
            .summarySync()
            .categoryTotals.map((total) => [
                group.categorySync(total.category)?.name ?? '?',
                String(expenses.filter((expense) => (expense.category ?? builtIn) === total.category).length),
                total.total.toString(),
            ]),
        1,
    );
}

/** Every expense and transfer in a currency `shown` accepts, as entered and in the group currency. */
export function printEntries(group: Group, shown: (currency: Currency) => boolean): void {
    const zero = new Money(group.currencySync(), 0n).toString();
    const expenses = group
        .expensesSync()
        .filter((expense) => shown(expense.cost.amount.currency))
        .map((expense) => [
            expense.name,
            names(group, splitMembers(expense.payers)),
            expenseTotal(expense).toString(),
            rate(expense.cost).toString(),
            (undefined === expense.id ? undefined : group.expenseSharesSync(expense.id))?.total.toString() ?? zero,
        ]);
    const transfers = group
        .transfersSync()
        .filter((transfer) => shown(transfer.cost.amount.currency))
        .map((transfer) => [
            `Transfer to ${name(group, transfer.to)}`,
            name(group, transfer.from),
            transfer.cost.amount.toString(),
            rate(transfer.cost).toString(),
            (undefined === transfer.id ? undefined : group.transferAmountSync(transfer.id))?.toString() ?? zero,
        ]);
    printTable(['Entry', 'Paid by', 'Entered as', 'Rate', 'In the group currency'], [...expenses, ...transfers], 2);
}
