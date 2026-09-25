// The totals. A summary, a balance, a share, or a settlement that converted with an amount changed or dropped would show an app
// other totals than the group holds; one that printed on its own would show it other words than the library's.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import { SettlementStrategy, formatBalance } from '../src/index.js';
import { Flat } from './flat.js';

void describe('the summary, the balances, the shares of an expense, and the settlements', () => {
    /** A summary that converted with a total changed or dropped would show an app other totals than the group holds. */
    void it("holds the group's totals", () => {
        const flat = new Flat();
        const summary = flat.group.summarySync();
        assert.ok(summary.totalSpent.equals(flat.euros(3_000n)));
        assert.deepEqual(
            summary.categoryTotals.map((total) => [total.category, total.total.toString()]),
            [[flat.food, '30.00 EUR']],
        );
        const [ana, bob] = summary.members;
        assert.ok(undefined !== ana && undefined !== bob);
        assert.deepEqual([ana.member, bob.member], [flat.ana, flat.bob]);
        assert.deepEqual([ana.paid, ana.share, ana.sent, ana.received, ana.net].map(String), [
            '30.00 EUR',
            '15.00 EUR',
            '0.00 EUR',
            '10.00 EUR',
            '5.00 EUR',
        ]);
        assert.deepEqual([formatBalance(ana.balance), formatBalance(bob.balance)], ['gets back 5.00 EUR', 'owes 5.00 EUR']);
        assert.deepEqual(
            summary.memberCategoryTotals.map((totals) => [totals.member, totals.totals.map((total) => total.total.toString())]),
            [
                [flat.ana, ['15.00 EUR']],
                [flat.bob, ['15.00 EUR']],
            ],
        );
    });

    /** A balance that printed on its own would tell a member they owe what they are owed. */
    void it('prints a balance as the library writes it', () => {
        const flat = new Flat();
        assert.equal(formatBalance({ kind: 'getsBack', amount: flat.euros(1_200n) }), 'gets back 12.00 EUR');
        assert.equal(formatBalance({ kind: 'owes', amount: flat.euros(1_200n) }), 'owes 12.00 EUR');
        assert.equal(formatBalance({ kind: 'settled' }), 'settled');
        const group = flat.group;
        group.addTransferSync({ cost: { kind: 'local', amount: flat.euros(500n) }, date: flat.day, from: flat.bob, to: flat.ana });
        assert.deepEqual(
            group.summarySync().members.map((member) => formatBalance(member.balance)),
            ['settled', 'settled'],
        );
    });

    /** An expense's shares or a settlement that converted with an amount changed would tell members to pay what they do not owe. */
    void it("holds an expense's shares and the settlements' payments", () => {
        const flat = new Flat();
        const shares = flat.group.expenseSharesSync(flat.dinner);
        assert.ok(undefined !== shares);
        assert.deepEqual([shares.total.toString(), shares.tip.toString()], ['30.00 EUR', '0.00 EUR']);
        assert.deepEqual(
            shares.members.map((share) => [share.member, share.paid.toString(), share.share.toString()]),
            [
                [flat.ana, '30.00 EUR', '15.00 EUR'],
                [flat.bob, '0.00 EUR', '15.00 EUR'],
            ],
        );
        for (const strategy of Object.values(SettlementStrategy)) {
            const settlement = flat.group.settlementSync(strategy);
            assert.equal(settlement.strategy, strategy);
            assert.deepEqual(
                settlement.payments.map((payment) => [payment.from, payment.to, payment.amount.toString()]),
                [[flat.bob, flat.ana, '5.00 EUR']],
            );
        }
    });
});
