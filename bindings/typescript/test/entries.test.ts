// The entries. An entry that converted to the generated code and back with a field changed or dropped, or read back from the
// group another way than it was added, would store another expense, member, or transfer than the app built.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
    convertedCategory,
    convertedExpense,
    convertedMember,
    convertedTipRule,
    convertedTransfer,
    generatedCategory,
    generatedExpense,
    generatedMember,
    generatedTipRule,
    generatedTransfer,
} from '../src/entries.js';
import {
    type Expense,
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
    NO_TIP,
    Parts,
    Percentage,
    PhoneNumber,
    TimeOfDay,
    TipDistribution,
    UNCATEGORISED,
    Username,
    createCategory,
    createExpense,
    createMember,
    createTransfer,
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
import * as internal from '../src/internal/eranos.js';
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
    const taxi = createExpense({
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
    });
    const dinner = createExpense({
        name: 'Dinner',
        cost: { kind: 'local', amount: flat.euros(3_000n) },
        payers: { kind: 'amounts', shares: [{ member: flat.ana, amount: flat.euros(3_300n) }] },
        split: { kind: 'whole', split: { kind: 'equal', members: [flat.ana, flat.bob] } },
        date: flat.day,
        tip: { value: { kind: 'percentage', percentage: Percentage.ofBasisPoints(1_000) } },
    });
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
            assert.equal(id, expense.id);
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

    /**
     * An expense without a tip or a category that read back with either left out, or with another value than the library's
     * no tip and built-in category, would show an app a tip or a category it never set; constants that named another rule or
     * another category than the library's would make an app's check for either miss.
     */
    void it("reads back an expense without a tip or a category under the library's no tip and built-in category", () => {
        const flat = new Flat();
        assert.equal(UNCATEGORISED, internal.categoryIdUncategorised());
        assert.equal(flat.group.categoriesSync()[0]?.id, UNCATEGORISED);
        assert.deepEqual(NO_TIP, convertedTipRule(internal.tipRuleNone()));
        assert.deepEqual(generatedTipRule(NO_TIP), internal.tipRuleNone());
        const taxi = flat.expense('Taxi', flat.ana, [flat.ana, flat.bob]);
        assert.equal(taxi.tip, NO_TIP);
        assert.equal(taxi.category, UNCATEGORISED);
        const stored = flat.group.expenseSync(flat.group.addExpenseSync(taxi));
        assert.ok(undefined !== stored);
        assert.deepEqual(stored.tip, NO_TIP);
        assert.equal(stored.category, UNCATEGORISED);
        assert.deepEqual(stored, convertedExpense(generatedExpense(taxi)));
        // Plain JavaScript can pass `undefined` where the types leave a field out.
        const untyped = { ...taxi, tip: undefined, category: undefined } as unknown as Parameters<typeof createExpense>[0];
        const filled = createExpense(untyped);
        assert.deepEqual(filled, { ...taxi, id: filled.id });
    });

    /**
     * A tip of 0 % set on purpose that read back as another rule than no tip when it is shared in proportion, or as no tip
     * when it is shared equally, would show an app another tip than the one it set.
     */
    void it('reads back a tip of 0 % as the rule it is', () => {
        const flat = new Flat();
        const zero = { kind: 'percentage', percentage: Percentage.ofBasisPoints(0) } as const;
        const proportional = { ...flat.expense('Water', flat.ana, [flat.ana]), tip: { value: zero, distribution: TipDistribution.PROPORTIONAL } };
        assert.deepEqual(flat.group.expenseSync(flat.group.addExpenseSync(proportional))?.tip, NO_TIP);
        const equal = { ...flat.expense('Bread', flat.ana, [flat.ana]), tip: { value: zero, distribution: TipDistribution.EQUAL } };
        const stored = flat.group.expenseSync(flat.group.addExpenseSync(equal))?.tip;
        assert.deepEqual(stored, equal.tip);
        assert.notDeepEqual(stored, NO_TIP);
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
        const carol = createMember({
            firstName: 'Carol',
            lastName: 'Popa',
            email: Email.parse('carol@example.com'),
            phone: PhoneNumber.parse('+40712345678'),
            accounts: [
                { name: 'Main account', details: { kind: 'iban', iban: Iban.parse('RO49AAAA1B31007593840000') } },
                { name: 'Payment app', details: { kind: 'username', username: Username.parse('carol.pays') } },
            ],
        });
        assert.deepEqual(convertedMember(generatedMember(carol)), carol);
        const dan = createMember({ firstName: 'Dan' });
        assert.deepEqual(convertedMember(generatedMember(dan)), { ...dan, accounts: [] });
        assert.equal(flat.group.addMemberSync(carol), carol.id);
        assert.deepEqual(flat.group.memberSync(carol.id), carol);

        const lei = CurrencyCode.of('RON');
        const cash = createTransfer({
            from: flat.bob,
            to: flat.ana,
            cost: { kind: 'converted', amount: new Money(1_000n, lei), rate: ExchangeRate.parse('0.2') },
            date: flat.day,
            time: TimeOfDay.of(10, 0),
            note: 'Cash',
        });
        assert.deepEqual(convertedTransfer(generatedTransfer(cash)), cash);
        assert.equal(flat.group.addTransferSync(cash), cash.id);
        assert.deepEqual(flat.group.transferSync(cash.id), cash);
        assert.ok(flat.group.transferAmountSync(cash.id)?.equals(flat.euros(200n)));

        const travel = createCategory({ name: 'Travel' });
        assert.deepEqual(convertedCategory(generatedCategory(travel)), travel);
        assert.equal(flat.group.addCategorySync(travel), travel.id);
        assert.deepEqual(flat.group.categorySync(travel.id), travel);
        assert.deepEqual(CalendarDate.of(2026, 1, 1).toString(), '2026-01-01');
    });

    /**
     * A create function that returned an identifier an earlier call had returned, or kept one its fields carry, would make a
     * copy of an entry that the group refuses as a duplicate of the original.
     */
    void it('creates every kind of entry under an identifier generated at each call', () => {
        const flat = new Flat();
        const group = flat.group;
        const member = group.memberSync(flat.ana);
        const category = group.categorySync(flat.food);
        const expense = group.expenseSync(flat.dinner);
        const transfer = group.transferSync(flat.repayment);
        assert.ok(undefined !== member && undefined !== category && undefined !== expense && undefined !== transfer);

        const memberCopy = createMember({ ...member });
        assert.notEqual(memberCopy.id, member.id);
        assert.deepEqual(memberCopy, { ...member, id: memberCopy.id });
        assert.equal(group.addMemberSync(memberCopy), memberCopy.id);
        const categoryCopy = createCategory({ ...category, name: 'Groceries' });
        assert.notEqual(categoryCopy.id, category.id);
        assert.equal(group.addCategorySync(categoryCopy), categoryCopy.id);
        const expenseCopy = createExpense({ ...expense });
        assert.notEqual(expenseCopy.id, expense.id);
        assert.deepEqual(expenseCopy, { ...expense, id: expenseCopy.id });
        assert.equal(group.addExpenseSync(expenseCopy), expenseCopy.id);
        const transferCopy = createTransfer({ ...transfer });
        assert.notEqual(transferCopy.id, transfer.id);
        assert.deepEqual(transferCopy, { ...transfer, id: transferCopy.id });
        assert.equal(group.addTransferSync(transferCopy), transferCopy.id);
        assert.equal(parseMemberId(memberCopy.id), memberCopy.id);
        assert.equal(parseCategoryId(categoryCopy.id), categoryCopy.id);
        assert.equal(parseExpenseId(expenseCopy.id), expenseCopy.id);
        assert.equal(parseTransferId(transferCopy.id), transferCopy.id);
    });

    /**
     * A replacement of an entry the group does not hold that the group added, or that changed another entry, would store an
     * entry the app never added.
     */
    void it('refuses to replace an entry the group does not hold', async () => {
        const flat = new Flat();
        const group = flat.group;
        const zoe = createMember({ firstName: 'Zoe' });
        const trips = createCategory({ name: 'Trips' });
        const taxi = flat.expense('Taxi', flat.ana, [flat.ana]);
        const cash = createTransfer({ from: flat.ana, to: flat.bob, cost: { kind: 'local', amount: flat.euros(100n) }, date: flat.day });
        const unknowns = [
            { replace: () => group.replaceMemberSync(zoe), failure: { kind: 'unknownMember', member: zoe.id } },
            { replace: () => group.replaceCategorySync(trips), failure: { kind: 'unknownCategory', category: trips.id } },
            { replace: () => group.replaceExpenseSync(taxi), failure: { kind: 'unknownExpense', expense: taxi.id } },
            { replace: () => group.replaceTransferSync(cash), failure: { kind: 'unknownTransfer', transfer: cash.id } },
        ];
        for (const unknown of unknowns) {
            assert.throws(unknown.replace, (error: unknown) => {
                assert.ok(error instanceof ExpenseGroupError);
                assert.deepEqual(error.failure, unknown.failure);
                return true;
            });
        }
        await assert.rejects(
            group.replaceMember(zoe),
            (error: unknown) => error instanceof ExpenseGroupError && 'unknownMember' === error.failure.kind,
        );
        assert.equal(group.memberSync(zoe.id), undefined);
        assert.equal(group.categorySync(trips.id), undefined);
        assert.equal(group.expenseSync(taxi.id), undefined);
        assert.equal(group.transferSync(cash.id), undefined);
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
