// Sections 8 to 11 of the tour: the summary, removing a category, a new group currency, and settling up.

import assert from 'node:assert/strict';

import {
    type CategoryId,
    type CategoryTotal,
    type ExpenseGroup,
    type MemberId,
    type Summary,
    CurrencyCode,
    ExchangeRate,
    ExpenseGroupError,
    Money,
    SettlementStrategy,
    UNCATEGORISED,
    createTransferFromPayment,
    summaryCategoryTotal,
    summaryMember,
    summaryMemberCategoryTotal,
} from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section } from '../report.js';
import { type Categories, type People, eur, name, october, paymentColumns, printBalances, printCategories, printEntries, ron, sum } from './tour.js';

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
    const zero = Money.zero(group.currencySync());
    printTable(
        memberCategoryColumns,
        group.membersSync().map((member) => {
            const total = (category: CategoryId): Money => summaryMemberCategoryTotal(summary, member.id, category);
            return {
                name: member.firstName,
                food: total(categories.food),
                lodging: total(categories.lodging),
                transport: total(categories.transport),
                uncategorised: total(UNCATEGORISED),
            };
        }),
        1,
    );

    assert.ok(is(eur('1222.90'), summary.totalSpent));
    assert.ok(is(eur('890.00'), summaryMember(summary, people.ana).paid));
    assert.ok(is(eur('140.00'), summaryMember(summary, people.ana).received));
    assert.ok(is(eur('20.00'), summaryMember(summary, people.bob).sent));
    assert.deepEqual(summaryMember(summary, people.dan).balance, { kind: 'getsBack', amount: eur('12.55') });
    assert.deepEqual(summaryMember(summary, people.eva).balance, { kind: 'owes', amount: eur('42.60') });
    assert.ok(is(eur('174.40'), summaryCategoryTotal(summary, categories.food)));
    assert.ok(is(eur('800.00'), summaryCategoryTotal(summary, categories.lodging)));
    assert.ok(is(eur('188.50'), summaryCategoryTotal(summary, categories.transport)));
    assert.ok(is(eur('60.00'), summaryCategoryTotal(summary, UNCATEGORISED)));
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
    const builtIn = refusal(() => group.removeCategorySync(UNCATEGORISED));
    printLines(
        `Removing ${removed.name} moves its expenses to Uncategorised and the category`,
        'totals with them; no balance changes. The built-in category cannot be removed:',
        `  error: ${message(builtIn)}`,
    );
    printCategories(group);

    assert.ok(builtIn instanceof ExpenseGroupError && 'builtInCategory' === builtIn.failure.kind);
    assert.equal(group.categorySync(categories.transport), undefined);
    assert.ok(is(eur('248.50'), summaryCategoryTotal(group.summarySync(), UNCATEGORISED)));
    assert.ok(group.expensesSync().every((expense) => categories.transport !== expense.category));
}

export function newCurrency(group: ExpenseGroup, people: People): void {
    section('10. A new currency');
    const euro = CurrencyCode.of('EUR');
    const leu = CurrencyCode.of('RON');
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
    assert.deepEqual(summaryMember(summary, people.dan).balance, { kind: 'getsBack', amount: ron('62.75') });
    assert.deepEqual(summaryMember(summary, people.eva).balance, { kind: 'owes', amount: ron('213.00') });
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
        plan.payments.map((payment) => ({ sender: name(group, payment.sender), receiver: name(group, payment.receiver), amount: payment.amount })),
        2,
    );
    for (const payment of plan.payments) {
        group.addTransferSync(createTransferFromPayment(payment, october(7)));
    }
    printLines('Each payment, once made, is recorded as a transfer, and every balance is settled:');
    printBalances(group);

    assert.equal(group.settlementSync(SettlementStrategy.GREEDY).payments.length, 0);
    assert.equal(plan.payments.length, 4);
    assert.ok(group.summarySync().members.every((found) => 'settled' === found.balance.kind));
}
