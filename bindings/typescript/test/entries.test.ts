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
    CurrencyCode,
    Email,
    ExchangeRate,
    ExpenseError,
    ExpenseGroupError,
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
    parseCategoryId,
    parseExpenseId,
    parseMemberId,
    parseTransferId,
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
    const lei = CurrencyCode.of('RON');
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
        const lei = CurrencyCode.of('RON');
        const { dinner, taxi } = expenses(flat);
        assert.ok(expenseTipAmount(taxi).equals(new Money(500n, lei)));
        assert.ok(expenseTotal(taxi).equals(new Money(5_500n, lei)));
        assert.deepEqual(expenseMembers(taxi), [flat.ana, flat.bob]);
        assert.ok(expenseTipAmount(dinner).equals(flat.euros(300n)));
        assert.throws(() => expenseMembers({ ...taxi, payers: { kind: 'equal', members: [memberId(' ana')] } }), IdError);
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

        const lei = CurrencyCode.of('RON');
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
            () => flat.group.replaceMemberSync(carol),
            (error: unknown) => error instanceof TypeError && 'an entry to replace must carry its identifier' === error.message,
        );
        assert.deepEqual(CalendarDate.of(2026, 1, 1).toString(), '2026-01-01');
    });

    /**
     * An entry stored under another identifier than the one the app gives it, a second entry under an identifier the group
     * holds, or an identifier the rules refuse would break the link between the app's own store and the group.
     */
    void it("adds every kind of entry under the app's identifier and refuses one the group holds", async () => {
        const flat = new Flat();
        const group = flat.group;
        const carol = parseMemberId('carol');
        assert.equal(group.addMemberSync({ id: carol, firstName: 'Carol' }), carol);
        assert.deepEqual(group.memberSync(carol), { id: carol, firstName: 'Carol', accounts: [] });
        // Identifiers compare byte for byte: two spellings of é that Unicode holds equivalent are two identifiers.
        const composed = parseMemberId('\u00e9');
        const decomposed = parseMemberId('e\u0301');
        assert.equal(await group.addMember({ id: composed, firstName: 'Élise' }), composed);
        assert.equal(await group.addMember({ id: decomposed, firstName: 'Emil' }), decomposed);
        assert.equal(group.memberSync(composed)?.firstName, 'Élise');
        assert.equal(group.memberSync(decomposed)?.firstName, 'Emil');

        const travel = parseCategoryId('travel');
        assert.equal(group.addCategorySync({ id: travel, name: 'Travel' }), travel);
        assert.deepEqual(group.categorySync(travel), { id: travel, name: 'Travel' });

        const flight = parseExpenseId('-flight');
        assert.equal(group.addExpenseSync({ ...flat.expense('Flight', carol, [carol, flat.ana]), id: flight, category: travel }), flight);
        assert.equal(group.expenseSync(flight)?.id, flight);
        assert.equal(group.expenseSync(flight)?.category, travel);

        const repayment = parseTransferId('zz');
        const cash = { id: repayment, from: flat.ana, to: carol, cost: { kind: 'local', amount: flat.euros(1_500n) }, date: flat.day } as const;
        assert.equal(group.addTransferSync(cash), repayment);
        assert.deepEqual(group.transferSync(repayment), cash);

        const duplicates = [
            { add: () => group.addMemberSync({ id: carol, firstName: 'Caroline' }), failure: { kind: 'duplicateMember', member: carol } },
            { add: () => group.addCategorySync({ id: travel, name: 'Trips' }), failure: { kind: 'duplicateCategory', category: travel } },
            {
                add: () => group.addExpenseSync({ ...flat.expense('Taxi', carol, [carol]), id: flight }),
                failure: { kind: 'duplicateExpense', expense: flight },
            },
            { add: () => group.addTransferSync({ ...cash, note: 'Again' }), failure: { kind: 'duplicateTransfer', transfer: repayment } },
        ];
        for (const duplicate of duplicates) {
            assert.throws(duplicate.add, (error: unknown) => {
                assert.ok(error instanceof ExpenseGroupError);
                assert.deepEqual(error.failure, duplicate.failure);
                return true;
            });
        }
        assert.equal(group.memberSync(carol)?.firstName, 'Carol');
        assert.equal(group.categorySync(travel)?.name, 'Travel');
        assert.equal(group.expenseSync(flight)?.name, 'Flight');
        assert.equal(group.transferSync(repayment)?.note, undefined);
        assert.throws(
            () => group.addMemberSync({ id: memberId(' x'), firstName: 'X' }),
            (error: unknown) =>
                error instanceof IdError &&
                'surroundingWhitespace' === error.failure.kind &&
                'an identifier cannot start or end with whitespace' === error.message,
        );
    });
});
