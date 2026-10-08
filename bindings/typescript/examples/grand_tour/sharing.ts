// Section 14 of the tour: the group shared between tasks, a settlement computed with the asynchronous calls, and a
// change that waits for a report's read and is cancelled.

import assert from 'node:assert/strict';

import { type ExpenseGroup, CurrencyCode, ExchangeRate, SettlementStrategy, createExpense } from '../../src/index.js';
import { printLines, printTable, section } from '../report.js';
import { type People, october, paymentColumns, ron } from './tour.js';

export async function sharing(group: ExpenseGroup, people: People): Promise<void> {
    section('14. Sharing the group');
    // Another handle on the same group, as another task holds: a group is shared as it is.
    const other = group;
    const everyone = [people.ana, people.bob, people.carol, people.dan, people.eva];
    await other.addExpense(
        createExpense({
            name: 'Airport coffee',
            cost: { kind: 'local', amount: ron('75.00') },
            payers: { kind: 'equal', members: [people.bob] },
            split: { kind: 'whole', split: { kind: 'equal', members: everyone } },
            date: october(8),
        }),
    );
    const plan = await group.settlement(SettlementStrategy.MINIMUM_TRANSFERS);
    printLines(
        'An app shares the group between its tasks and threads through a',
        "SharedExpenseGroup, whose calls are futures any executor runs; here the library's",
        'block_on runs each one on this thread. Bob adds the airport coffee through another',
        'handle, and the plan that settles it comes through the first:',
    );
    {
        using reader = await group.read();
        printTable(
            paymentColumns,
            plan.payments.map((payment) => ({
                sender: reader.member(payment.sender)?.firstName ?? '?',
                receiver: reader.member(payment.receiver)?.firstName ?? '?',
                amount: payment.amount,
            })),
            2,
        );
    }

    const before = await group.summary();
    const rates = [{ currency: CurrencyCode.of('RON'), rate: ExchangeRate.parse('0.2') }];
    // A report reads the group, holding the shared lock meanwhile, so the change waits for it; a new read would wait
    // behind the change, so the report reads through its own handle.
    const report = await group.read();
    const cancelling = new AbortController();
    const change = group.changeCurrency(CurrencyCode.of('EUR'), rates, { signal: cancelling.signal });
    const reported = report.summary().totalSpent;
    // Aborting the signal cancels the waiting change, which leaves the group as it was.
    cancelling.abort();
    report.release();
    const cancelled = await change.then(
        () => false,
        () => true,
    );
    const after = await group.summary();
    const currency = await group.currency();
    printLines(
        'While a report reads the group, a change of its currency to EUR waits for the',
        'report. The app drops the waiting change, which cancels it: the group still',
        `keeps its books in ${currency.toString()}, and the trip cost ${after.totalSpent.toString()}, as the report read.`,
    );
    printLines(`With the other handle gone, the app takes the group back: ${String((await group.expenses()).length)} expenses.`);

    assert.equal(plan.payments.length, 4);
    for (const payment of plan.payments) {
        assert.ok(people.bob === payment.receiver && ron('15.00').equals(payment.amount));
    }
    assert.ok(cancelled);
    assert.deepEqual(after, before);
    assert.ok(reported.equals(after.totalSpent));
    assert.equal(currency.code, 'RON');
}
