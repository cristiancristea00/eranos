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
        assert.deepEqual(summary.categoryTotals, [{ category: flat.food, total: flat.euros(3_000n) }]);
        const [ana, bob] = summary.members;
        assert.ok(undefined !== ana && undefined !== bob);
        assert.equal(ana.member, flat.ana);
        assert.equal(bob.member, flat.bob);
        assert.equal(ana.paid.toString(), '30.00 EUR');
        assert.equal(ana.share.toString(), '15.00 EUR');
        assert.equal(ana.sent.toString(), '0.00 EUR');
        assert.equal(ana.received.toString(), '10.00 EUR');
        assert.equal(ana.net.toString(), '5.00 EUR');
        assert.equal(formatBalance(ana.balance), 'gets back 5.00 EUR');
        assert.equal(formatBalance(bob.balance), 'owes 5.00 EUR');
        assert.deepEqual(summary.memberCategoryTotals, [
            { member: flat.ana, totals: [{ category: flat.food, total: flat.euros(1_500n) }] },
            { member: flat.bob, totals: [{ category: flat.food, total: flat.euros(1_500n) }] },
        ]);
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
        assert.equal(shares.total.toString(), '30.00 EUR');
        assert.equal(shares.tip.toString(), '0.00 EUR');
        assert.deepEqual(shares.members, [
            { member: flat.ana, paid: flat.euros(3_000n), share: flat.euros(1_500n) },
            { member: flat.bob, paid: flat.euros(0n), share: flat.euros(1_500n) },
        ]);
        for (const strategy of Object.values(SettlementStrategy)) {
            const settlement = flat.group.settlementSync(strategy);
            assert.equal(settlement.strategy, strategy);
            assert.deepEqual(settlement.payments, [{ amount: flat.euros(500n), from: flat.bob, to: flat.ana }]);
        }
    });
});
