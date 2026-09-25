// The entries. An entry that converted to the generated code and back with a field changed or dropped, or read back from the
// group another way than it was added, would store another expense, member, or transfer than the app built.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
    convertedCategory,
    convertedExpense,
    convertedMember,
    convertedTransfer,
    generatedCategory,
    generatedExpense,
    generatedMember,
    generatedTransfer,
} from '../src/entries.js';
import {
    type Expense,
    type Member,
    type Split,
    CalendarDate,
    Currency,
    Email,
    ExchangeRate,
    ExpenseError,
    Iban,
    IdError,
    Money,
    Parts,
    Percentage,
    PhoneNumber,
    TimeOfDay,
    TipDistribution,
    Username,
    expenseMembers,
    expenseTipAmount,
    expenseTotal,
    memberId,
    splitMembers,
} from '../src/index.js';
import { Flat } from './flat.js';

/**
 * A taxi in lei, itemised (a fare of two units and a toll of one, left implicit), paid by percentages, with a fixed tip
 * shared equally, and a dinner in euros, split equally, paid by amounts, with a percentage tip: between them, every kind of
 * cost, split, and tip.
 */
function expenses(flat: Flat): readonly [Expense, Expense] {
    const lei = Currency.of('RON');
    const taxi: Expense = {
        cost: { kind: 'converted', amount: new Money(lei, 5_000n), rate: ExchangeRate.parse('0.2') },
        date: flat.day,
        name: 'Taxi',
        payers: { kind: 'percentages', shares: [{ member: flat.ana, percentage: Percentage.ofBasisPoints(10_000) }] },
        split: {
            kind: 'itemised',
            items: [
                {
                    name: 'Fare',
                    split: {
                        kind: 'parts',
                        shares: [
                            { member: flat.ana, parts: Parts.ofHundredths(100) },
                            { member: flat.bob, parts: Parts.ofHundredths(50) },
                        ],
                    },
                    unitPrice: new Money(lei, 2_000n),
                    quantity: 2,
                },
                { name: 'Toll', split: { kind: 'equal', members: [flat.bob] }, unitPrice: new Money(lei, 1_000n) },
            ],
        },
        category: flat.food,
        description: 'To the airport',
        time: TimeOfDay.of(9, 5),
        tip: { value: { kind: 'fixed', amount: new Money(lei, 500n) }, distribution: TipDistribution.EQUAL },
    };
    const dinner: Expense = {
        cost: { kind: 'local', amount: flat.euros(3_000n) },
        date: flat.day,
        name: 'Dinner',
        payers: { kind: 'amounts', shares: [{ amount: flat.euros(3_300n), member: flat.ana }] },
        split: { kind: 'whole', split: { kind: 'equal', members: [flat.ana, flat.bob] } },
        tip: { value: { kind: 'percentage', percentage: Percentage.ofBasisPoints(1_000) } },
    };
    return [taxi, dinner];
}

void describe('the expenses, members, categories, and transfers', () => {
    /**
     * An expense that converted with a field changed or dropped, or that the group stored another way, would divide another
     * expense than the app built.
     */
    void it('converts and stores every kind of expense whole', () => {
        const flat = new Flat();
        const [taxi, dinner] = expenses(flat);
        assert.ok('itemised' === taxi.split.kind);
        const [fare, toll] = taxi.split.items;
        assert.ok(undefined !== fare && undefined !== toll);
        assert.deepEqual(convertedExpense(generatedExpense(taxi)), { ...taxi, split: { kind: 'itemised', items: [fare, { ...toll, quantity: 1 }] } });
        assert.deepEqual(convertedExpense(generatedExpense(dinner)), {
            ...dinner,
            tip: { ...dinner.tip, distribution: TipDistribution.PROPORTIONAL },
        });
        for (const expense of [taxi, dinner]) {
            const id = flat.group.addExpenseSync(expense);
            const stored = flat.group.expenseSync(id);
            assert.ok(undefined !== stored);
            assert.equal(stored.id, id);
            assert.deepEqual([stored.cost, stored.date, stored.name, stored.payers], [expense.cost, expense.date, expense.name, expense.payers]);
            assert.deepEqual([stored.category, stored.description, stored.time], [expense.category, expense.description, expense.time]);
        }
    });

    /** The tip, total, and members an app shows for an expense would differ from those the group records when it adds it. */
    void it("tells an expense's tip, total, and members", () => {
        const flat = new Flat();
        const lei = Currency.of('RON');
        const [taxi, dinner] = expenses(flat);
        assert.ok(expenseTipAmount(taxi).equals(new Money(lei, 500n)));
        assert.ok(expenseTotal(taxi).equals(new Money(lei, 5_500n)));
        assert.deepEqual(expenseMembers(taxi), [flat.ana, flat.bob]);
        assert.ok(expenseTipAmount(dinner).equals(flat.euros(300n)));
        assert.throws(() => expenseMembers({ ...taxi, payers: { kind: 'equal', members: [memberId('nobody')] } }), IdError);
        assert.throws(() => expenseTipAmount({ ...taxi, tip: { value: { kind: 'fixed', amount: flat.euros(1n) } } }), ExpenseError);
        assert.throws(() => expenseTotal({ ...taxi, tip: { value: { kind: 'fixed', amount: new Money(lei, -1n) } } }), ExpenseError);
    });

    /** A split that listed other members than it divides among would show an app another set of participants. */
    void it('lists the members of every split', () => {
        const flat = new Flat();
        const splits: readonly Split[] = [
            { kind: 'amounts', shares: [{ amount: flat.euros(1n), member: flat.bob }] },
            { kind: 'equal', members: [flat.bob, flat.ana] },
            { kind: 'parts', shares: [{ member: flat.bob, parts: Parts.ofHundredths(1) }] },
            { kind: 'percentages', shares: [{ member: flat.ana, percentage: Percentage.ofBasisPoints(1) }] },
        ];
        assert.deepEqual(splits.map(splitMembers), [[flat.bob], [flat.bob, flat.ana], [flat.bob], [flat.ana]]);
    });

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group stored another way, would
     * pay or reach another person.
     */
    void it('converts and stores members, categories, and transfers whole', () => {
        const flat = new Flat();
        const carol: Member = {
            firstName: 'Carol',
            accounts: [
                { details: { kind: 'iban', iban: Iban.parse('RO49AAAA1B31007593840000') }, name: 'Main account' },
                { details: { kind: 'username', username: Username.parse('carol.pays') }, name: 'Payment app' },
            ],
            email: Email.parse('carol@example.com'),
            lastName: 'Popa',
            phone: PhoneNumber.parse('+40712345678'),
        };
        assert.deepEqual(convertedMember(generatedMember(carol)), carol);
        assert.deepEqual(convertedMember(generatedMember({ firstName: 'Dan' })), { firstName: 'Dan', accounts: [] });
        const id = flat.group.addMemberSync(carol);
        const stored = flat.group.memberSync(id);
        assert.deepEqual(stored, { ...carol, id });

        const lei = Currency.of('RON');
        const cash = {
            cost: { kind: 'converted', amount: new Money(lei, 1_000n), rate: ExchangeRate.parse('0.2') },
            date: flat.day,
            from: flat.bob,
            to: flat.ana,
            note: 'Cash',
            time: TimeOfDay.of(10, 0),
        } as const;
        assert.deepEqual(convertedTransfer(generatedTransfer(cash)), cash);
        const payment = flat.group.addTransferSync(cash);
        assert.deepEqual(flat.group.transferSync(payment), { ...cash, id: payment });
        assert.ok(flat.group.transferAmountSync(payment)?.equals(flat.euros(200n)));

        assert.deepEqual(convertedCategory(generatedCategory({ name: 'Travel' })), { name: 'Travel' });
        const trip = flat.group.addCategorySync({ name: 'Travel' });
        assert.deepEqual(flat.group.categorySync(trip), { id: trip, name: 'Travel' });

        assert.throws(
            () => flat.group.addMemberSync({ ...carol, id }),
            (error: unknown) => error instanceof TypeError && 'an entry to add must not carry an identifier' === error.message,
        );
        assert.throws(
            () => flat.group.replaceMemberSync(carol),
            (error: unknown) => error instanceof TypeError && 'an entry to replace must carry its identifier' === error.message,
        );
        assert.deepEqual(CalendarDate.of(1, 1, 2026).toString(), '2026-01-01');
    });
});
