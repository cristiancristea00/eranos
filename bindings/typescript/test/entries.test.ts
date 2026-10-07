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

/** The two expenses of {@link expenses}. */
interface Expenses {
    readonly dinner: Expense;
    readonly taxi: Expense;
}

/**
 * A taxi in lei, itemised (a fare of two units and a toll of one, left implicit), paid by percentages, with a fixed tip
 * shared equally, and a dinner in euros, split equally, paid by amounts, with a percentage tip: between them, every kind of
 * cost, split, and tip.
 */
function expenses(flat: Flat): Expenses {
    const lei = Currency.of('RON');
    const taxi: Expense = {
        name: 'Taxi',
        cost: { kind: 'converted', amount: new Money(5_000n, lei), rate: ExchangeRate.parse('0.2') },
        payers: { kind: 'percentages', shares: [{ member: flat.ana, percentage: Percentage.ofBasisPoints(10_000) }] },
        split: {
            kind: 'itemised',
            items: [
                {
                    name: 'Fare',
                    unitPrice: new Money(2_000n, lei),
                    split: {
                        kind: 'parts',
                        shares: [
                            { member: flat.ana, parts: Parts.ofHundredths(100) },
                            { member: flat.bob, parts: Parts.ofHundredths(50) },
                        ],
                    },
                    quantity: 2,
                },
                { name: 'Toll', unitPrice: new Money(1_000n, lei), split: { kind: 'equal', members: [flat.bob] } },
            ],
        },
        date: flat.day,
        tip: { value: { kind: 'fixed', amount: new Money(500n, lei) }, distribution: TipDistribution.EQUAL },
        time: TimeOfDay.of(9, 5),
        category: flat.food,
        description: 'To the airport',
    };
    const dinner: Expense = {
        name: 'Dinner',
        cost: { kind: 'local', amount: flat.euros(3_000n) },
        payers: { kind: 'amounts', shares: [{ member: flat.ana, amount: flat.euros(3_300n) }] },
        split: { kind: 'whole', split: { kind: 'equal', members: [flat.ana, flat.bob] } },
        date: flat.day,
        tip: { value: { kind: 'percentage', percentage: Percentage.ofBasisPoints(1_000) } },
    };
    return { dinner, taxi };
}

void describe('the expenses, members, categories, and transfers', () => {
    /**
     * An expense that converted with a field changed or dropped, or that the group stored another way, would divide another
     * expense than the app built.
     */
    void it('converts and stores every kind of expense whole', () => {
        const flat = new Flat();
        const { dinner, taxi } = expenses(flat);
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
            assert.equal(stored.name, expense.name);
            assert.deepEqual(stored.cost, expense.cost);
            assert.deepEqual(stored.payers, expense.payers);
            assert.deepEqual(stored.date, expense.date);
            assert.deepEqual(stored.time, expense.time);
            assert.equal(stored.category, expense.category);
            assert.equal(stored.description, expense.description);
        }
    });

    /** The tip, total, and members an app shows for an expense would differ from those the group records when it adds it. */
    void it("tells an expense's tip, total, and members", () => {
        const flat = new Flat();
        const lei = Currency.of('RON');
        const { dinner, taxi } = expenses(flat);
        assert.ok(expenseTipAmount(taxi).equals(new Money(500n, lei)));
        assert.ok(expenseTotal(taxi).equals(new Money(5_500n, lei)));
        assert.deepEqual(expenseMembers(taxi), [flat.ana, flat.bob]);
        assert.ok(expenseTipAmount(dinner).equals(flat.euros(300n)));
        assert.throws(() => expenseMembers({ ...taxi, payers: { kind: 'equal', members: [memberId('nobody')] } }), IdError);
        assert.throws(() => expenseTipAmount({ ...taxi, tip: { value: { kind: 'fixed', amount: flat.euros(1n) } } }), ExpenseError);
        assert.throws(() => expenseTotal({ ...taxi, tip: { value: { kind: 'fixed', amount: new Money(-1n, lei) } } }), ExpenseError);
    });

    /** A split that listed other members than it divides among would show an app another set of participants. */
    void it('lists the members of every split', () => {
        const flat = new Flat();
        const splits: readonly Split[] = [
            { kind: 'equal', members: [flat.bob, flat.ana] },
            { kind: 'parts', shares: [{ member: flat.bob, parts: Parts.ofHundredths(1) }] },
            { kind: 'percentages', shares: [{ member: flat.ana, percentage: Percentage.ofBasisPoints(1) }] },
            { kind: 'amounts', shares: [{ member: flat.bob, amount: flat.euros(1n) }] },
        ];
        assert.deepEqual(splits.map(splitMembers), [[flat.bob, flat.ana], [flat.bob], [flat.ana], [flat.bob]]);
    });

    /**
     * A member or a transfer that converted with a detail changed or dropped, or that the group stored another way, would
     * pay or reach another person.
     */
    void it('converts and stores members, categories, and transfers whole', () => {
        const flat = new Flat();
        const carol: Member = {
            firstName: 'Carol',
            lastName: 'Popa',
            email: Email.parse('carol@example.com'),
            phone: PhoneNumber.parse('+40712345678'),
            accounts: [
                { name: 'Main account', details: { kind: 'iban', iban: Iban.parse('RO49AAAA1B31007593840000') } },
                { name: 'Payment app', details: { kind: 'username', username: Username.parse('carol.pays') } },
            ],
        };
        assert.deepEqual(convertedMember(generatedMember(carol)), carol);
        assert.deepEqual(convertedMember(generatedMember({ firstName: 'Dan' })), { firstName: 'Dan', accounts: [] });
        const id = flat.group.addMemberSync(carol);
        const stored = flat.group.memberSync(id);
        assert.deepEqual(stored, { ...carol, id });

        const lei = Currency.of('RON');
        const cash = {
            from: flat.bob,
            to: flat.ana,
            cost: { kind: 'converted', amount: new Money(1_000n, lei), rate: ExchangeRate.parse('0.2') },
            date: flat.day,
            time: TimeOfDay.of(10, 0),
            note: 'Cash',
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
        assert.deepEqual(CalendarDate.of(2026, 1, 1).toString(), '2026-01-01');
    });
});
