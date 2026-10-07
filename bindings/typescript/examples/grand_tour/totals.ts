// Sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and settling up.

import assert from 'node:assert/strict';

import {
    type CategoryId,
    type CategoryTotal,
    type ExpenseGroup,
    type MemberId,
    type Summary,
    Currency,
    ExchangeRate,
    ExpenseGroupError,
    Money,
    SettlementStrategy,
} from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section, totals } from '../report.js';
import {
    type Categories,
    type People,
    builtInCategory,
    eur,
    name,
    october,
    paymentColumns,
    printBalances,
    printCategories,
    printEntries,
    ron,
    sum,
} from './tour.js';

/** A member's consumption in each category, as the table of consumption per category prints it. */
interface MemberCategoryRow {
    readonly name: string;
    readonly food: Money;
    readonly lodging: Money;
    readonly transport: Money;
    readonly uncategorised: Money;
}

const memberCategoryColumns: Columns<MemberCategoryRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Food', text: (row) => row.food.toString() },
    { header: 'Lodging', text: (row) => row.lodging.toString() },
    { header: 'Getting around', text: (row) => row.transport.toString() },
    { header: 'Uncategorised', text: (row) => row.uncategorised.toString() },
];

/** A strategy, the strategy whose plan it chose, and the number of payments of that plan, as the table of strategies prints them. */
interface StrategyRow {
    readonly strategy: string;
    readonly computedBy: string;
    readonly payments: number;
}

const strategyColumns: Columns<StrategyRow> = [
    { header: 'Strategy', text: (row) => row.strategy },
    { header: 'Computed by', text: (row) => row.computedBy },
    { header: 'Payments', text: (row) => String(row.payments) },
];

/** The totals per category of `member` in `summary`. */
function byCategory(summary: Summary, member: MemberId): readonly CategoryTotal[] {
    return summary.memberCategoryTotals.find((entry) => entry.member === member)?.totals ?? [];
}

/** The total of `category` among `categoryTotals`. */
function total(categoryTotals: readonly CategoryTotal[], category: CategoryId | undefined): Money | undefined {
    return categoryTotals.find((entry) => entry.category === category)?.total;
}

/** Whether `found` is `expected`. */
function is(expected: Money, found: Money | undefined): boolean {
    return undefined !== found && expected.equals(found);
}

export function summary(group: ExpenseGroup, people: People, categories: Categories): void {
    section('8. The summary');
    const summary = group.summarySync();
    printLines(
        `The group spent ${summary.totalSpent.toString()} in all. For each member, what they paid, what they`,
        'consumed, the transfers they sent and received, and what is left:',
    );
    printBalances(group);
    printLines('The spending per category:');
    printCategories(group);
    printLines("And each member's consumption per category:");
    const builtIn = builtInCategory(group);
    const zero = new Money(0n, group.currencySync());
    printTable(
        memberCategoryColumns,
        group.membersSync().flatMap((member) => {
            if (undefined === member.id) {
                return [];
            }
            const own = byCategory(summary, member.id);
            return [
                {
                    name: member.firstName,
                    food: total(own, categories.food) ?? zero,
                    lodging: total(own, categories.lodging) ?? zero,
                    transport: total(own, categories.transport) ?? zero,
                    uncategorised: total(own, builtIn) ?? zero,
                },
            ];
        }),
        1,
    );

    assert.ok(is(eur('1222.90'), summary.totalSpent));
    assert.ok(is(eur('890.00'), totals(people.ana, summary)?.paid));
    assert.ok(is(eur('140.00'), totals(people.ana, summary)?.received));
    assert.ok(is(eur('20.00'), totals(people.bob, summary)?.sent));
    assert.deepEqual(totals(people.dan, summary)?.balance, { kind: 'getsBack', amount: eur('12.55') });
    assert.deepEqual(totals(people.eva, summary)?.balance, { kind: 'owes', amount: eur('42.60') });
    assert.ok(is(eur('174.40'), total(summary.categoryTotals, categories.food)));
    assert.ok(is(eur('800.00'), total(summary.categoryTotals, categories.lodging)));
    assert.ok(is(eur('188.50'), total(summary.categoryTotals, categories.transport)));
    assert.ok(is(eur('60.00'), total(summary.categoryTotals, builtIn)));
    for (const found of summary.members) {
        assert.ok(
            is(
                sum(
                    zero.currency,
                    byCategory(summary, found.member).map((entry) => entry.total),
                ),
                found.share,
            ),
        );
    }
    assert.equal(
        sum(
            zero.currency,
            summary.members.map((found) => found.net),
        ).minorUnits,
        0n,
    );
}

export function removingACategory(group: ExpenseGroup, categories: Categories): void {
    section('9. Removing a category');
    const removed = group.removeCategorySync(categories.transport);
    const uncategorised = builtInCategory(group);
    assert.ok(undefined !== uncategorised, 'every group has the built-in category');
    const builtIn = refusal(() => group.removeCategorySync(uncategorised));
    printLines(
        `Removing ${removed.name} moves its expenses to Uncategorised and the category`,
        'totals with them; no balance changes. The built-in category cannot be removed:',
        `  error: ${message(builtIn)}`,
    );
    printCategories(group);

    assert.ok(builtIn instanceof ExpenseGroupError && 'builtInCategory' === builtIn.failure.kind);
    assert.equal(group.categorySync(categories.transport), undefined);
    assert.ok(is(eur('248.50'), total(group.summarySync().categoryTotals, uncategorised)));
    assert.ok(group.expensesSync().every((expense) => categories.transport !== expense.category));
}

export function newCurrency(group: ExpenseGroup, people: People): void {
    section('10. A new currency');
    const euro = Currency.of('EUR');
    const leu = Currency.of('RON');
    const missing = refusal(() => {
        group.changeCurrencySync(leu, []);
    });
    group.changeCurrencySync(leu, [{ currency: euro, rate: ExchangeRate.parse('5') }]);
    printLines(
        'Home again, the group moves its books to RON. The change needs a rate into RON',
        'for every other currency in use, here only EUR, or it is refused:',
        `  error: ${message(missing)}`,
        'Entries in EUR take the new rate, and the RON parking and cash become local:',
    );
    printEntries(group, () => true);
    printLines(
        'Each entry keeps its currency and converts at the new rate. At 5 every share',
        'converts exactly, so each balance is five times what it was. The trip cost',
        `${group.summarySync().totalSpent.toString()}:`,
    );
    printBalances(group);

    const summary = group.summarySync();
    assert.ok(missing instanceof ExpenseGroupError);
    assert.deepEqual(missing.failure, { kind: 'missingExchangeRate', currency: euro });
    assert.equal(group.currencySync().code, leu.code);
    assert.ok(is(ron('6114.50'), summary.totalSpent));
    assert.deepEqual(totals(people.dan, summary)?.balance, { kind: 'getsBack', amount: ron('62.75') });
    assert.deepEqual(totals(people.eva, summary)?.balance, { kind: 'owes', amount: ron('213.00') });
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

export function settlingUp(group: ExpenseGroup): void {
    section('11. Settling up');
    const counts = Object.values(SettlementStrategy).map((strategy): StrategyRow => {
        const settlement = group.settlementSync(strategy);
        // The group's open balances are well within the adaptive strategy's limit, so it settles them exactly.
        assert.equal(settlement.strategy, SettlementStrategy.ADAPTIVE === strategy ? SettlementStrategy.MINIMUM_TRANSFERS : strategy);
        return { strategy: strategyName(strategy), computedBy: strategyName(settlement.strategy), payments: settlement.payments.length };
    });
    printLines(
        'Each strategy turns the balances into payments: the direct one pays back every',
        "pair's own debts, greedy pays the largest amounts first, the minimum one finds",
        'the fewest payments, and the adaptive one finds the fewest while that is quick',
        "and greedy's beyond, naming the strategy whose plan it chose.",
    );
    printTable(strategyColumns, counts, 2);
    const plan = group.settlementSync(SettlementStrategy.MINIMUM_TRANSFERS);
    printLines('The group follows the minimum plan:');
    printTable(
        paymentColumns,
        plan.payments.map((payment) => ({ from: name(group, payment.from), to: name(group, payment.to), amount: payment.amount })),
        2,
    );
    for (const payment of plan.payments) {
        group.addTransferSync({ from: payment.from, to: payment.to, cost: { kind: 'local', amount: payment.amount }, date: october(7) });
    }
    printLines('Each payment, once made, is recorded as a transfer, and every balance is settled:');
    printBalances(group);

    assert.equal(group.settlementSync(SettlementStrategy.GREEDY).payments.length, 0);
    assert.equal(plan.payments.length, 4);
    assert.ok(group.summarySync().members.every((found) => 'settled' === found.balance.kind));
}
