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
    type Discount,
    type DiscountInput,
    type Expense,
    type ExpenseInput,
    type ExpenseItem,
    type ExpenseItemInput,
    type ExpenseSplitInput,
    type Member,
    type MemberInput,
    type Split,
    type TipRule,
    type TipRuleInput,
    CalendarDate,
    CurrencyCode,
    DiscountScope,
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
    Quantity,
    TimeOfDay,
    TipDistribution,
    UNCATEGORISED,
    Username,
    createCategory,
    createExpense,
    createMember,
    createTransfer,
    expenseItemDiscountAmount,
    expenseItemLineTotal,
    expenseItemLineTotalBeforeDiscount,
    expenseMembers,
    expenseTipAmount,
    expenseTotal,
    generateExpenseId,
    generateMemberId,
    generateTransferId,
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
                    quantity: Quantity.ofWhole(2n),
                },
                { name: 'Toll', unitPrice: new Money(1_000n, lei), split: { kind: 'equal', members: [flat.bob] } },
            ],
        },
        date: flat.day,
        tip: { value: { kind: 'fixed', amount: new Money(500n, lei) }, distribution: TipDistribution.EQUAL },
        time: TimeOfDay.of(9, 5),
        category: flat.food,
        note: 'To the airport',
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
        assert.deepEqual(convertedExpense(generatedExpense(taxi)), taxi);
        assert.deepEqual(convertedExpense(generatedExpense(dinner)), dinner);
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
            assert.equal(stored.note, expense.note);
            assert.deepEqual(stored, expense);
        }
    });

    /**
     * A record that reached the library without a field it left out, or with another value than the default its type
     * documents, would store a line of another quantity, a tip shared another way, an expense in another category, or a
     * member with accounts the app never gave; a replace that kept the value the group holds instead would leave an expense
     * in a category the app moved it out of.
     */
    void it('fills every field a record leaves out with its default on the way in', async () => {
        const flat = new Flat();
        const group = flat.group;
        const { dinner, taxi } = expenses(flat);
        assert.ok('itemised' === taxi.split.kind);
        assert.equal(taxi.split.items[1]?.quantity, Quantity.ONE);
        assert.equal(dinner.tip.distribution, TipDistribution.PROPORTIONAL);
        assert.equal(dinner.category, UNCATEGORISED);

        const zoe = parseMemberId('zoe');
        const account = { name: 'Main account', details: { kind: 'username', username: Username.parse('zoe.pays') } } as const;
        assert.equal(await group.addMember({ id: zoe, firstName: 'Zoe', accounts: [account] }), zoe);
        assert.deepEqual(group.replaceMemberSync({ id: zoe, firstName: 'Zoë' }), { id: zoe, firstName: 'Zoe', accounts: [account] });
        assert.deepEqual(group.memberSync(zoe), { id: zoe, firstName: 'Zoë', accounts: [] });

        const toll = { name: 'Toll', unitPrice: flat.euros(1_000n), split: { kind: 'equal', members: [flat.bob] } } as const;
        const bare: ExpenseInput = {
            id: parseExpenseId('toll'),
            name: 'Toll',
            cost: { kind: 'local', amount: flat.euros(1_000n) },
            payers: { kind: 'equal', members: [flat.ana] },
            split: { kind: 'itemised', items: [toll] },
            date: flat.day,
        };
        const tip = { value: { kind: 'percentage', percentage: Percentage.ofBasisPoints(1_000) } } as const;
        assert.equal(await group.addExpense({ ...bare, tip }), bare.id);
        const filled = { ...bare, split: { kind: 'itemised', items: [{ ...toll, quantity: Quantity.ONE }] }, tip: NO_TIP, category: UNCATEGORISED };
        const tipped = { ...filled, tip: { ...tip, distribution: TipDistribution.PROPORTIONAL } };
        assert.deepEqual(group.expenseSync(bare.id), tipped);
        assert.deepEqual(await group.replaceExpense({ ...bare, category: flat.food }), tipped);
        assert.equal(group.expenseSync(bare.id)?.category, flat.food);
        assert.deepEqual(group.replaceExpenseSync(bare), { ...filled, category: flat.food });
        assert.deepEqual(group.expenseSync(bare.id), filled);
    });

    /**
     * An output type that an add or a replace refused would make an app copy a record it read back field by field to pass
     * it again; an input type that compiled without the identifier or another field the library has no default for would let
     * a record reach the library without it, and an output type that compiled without a field the library always holds
     * would make an app check for a value that is always there.
     */
    void it('takes every record it returns, and types every field without a default as required', async () => {
        const flat = new Flat();
        const group = flat.group;
        const { taxi } = expenses(flat);
        group.addExpenseSync(taxi);
        const member = group.memberSync(flat.ana);
        const expense = group.expenseSync(taxi.id);
        assert.ok(undefined !== member && 'itemised' === expense?.split.kind);
        // Every output type is assignable to its input type.
        const memberInput: MemberInput = member;
        const items: readonly ExpenseItemInput[] = expense.split.items;
        const split: ExpenseSplitInput = { kind: 'itemised', items };
        const tip: TipRuleInput = expense.tip;
        const expenseInput: ExpenseInput = { ...expense, split, tip };
        assert.deepEqual(await group.replaceMember(memberInput), member);
        assert.deepEqual(group.replaceExpenseSync(expenseInput), expense);
        assert.deepEqual(group.expenseSync(taxi.id), expense);

        const cost = { kind: 'local', amount: flat.euros(1_000n) } as const;
        const payers = { kind: 'equal', members: [flat.ana] } as const;
        const whole = { kind: 'whole', split: payers } as const;
        const fields = { name: 'Toll', cost, payers, split: whole, date: flat.day } as const;
        const value = { kind: 'percentage', percentage: Percentage.ofBasisPoints(0) } as const;
        // Records that leave out a field their type requires, and so must not compile; the test never makes these calls.
        const uncompiled = [
            // @ts-expect-error: a member to add names their identifier.
            () => group.addMemberSync({ firstName: 'Zoe' }),
            // @ts-expect-error: an expense to add names its identifier.
            () => group.addExpenseSync(fields),
            // @ts-expect-error: an expense names its date.
            () => createExpense({ name: 'Toll', cost, payers, split: whole }),
            // @ts-expect-error: a line of a bill names its unit price.
            () => createExpense({ ...fields, split: { kind: 'itemised', items: [{ name: 'Toll', split: payers }] } }),
            // @ts-expect-error: a tip names its value.
            () => createExpense({ ...fields, tip: { distribution: TipDistribution.EQUAL } }),
            // @ts-expect-error: a member read back has their accounts.
            (): Member => ({ id: flat.ana, firstName: 'Ana' }),
            // @ts-expect-error: an expense read back has its tip and its category.
            (): Expense => ({ ...fields, id: taxi.id }),
            // @ts-expect-error: a line read back has its quantity.
            (): ExpenseItem => ({ name: 'Toll', unitPrice: flat.euros(1_000n), split: payers }),
            // @ts-expect-error: a tip read back has its distribution.
            (): TipRule => ({ value }),
            // @ts-expect-error: a discount names its value.
            (): DiscountInput => ({ scope: DiscountScope.UNIT }),
            // @ts-expect-error: a discount read back has its scope.
            (): Discount => ({ value }),
        ];
        assert.equal(uncompiled.length, 11);
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

    /**
     * A line total that an app computed another way than the library's, multiplying a double or rounding another way, would
     * show a bill whose lines do not add up to the amount the library checks them against; a fractional quantity or a free
     * line that the group stored another way, or refused, would divide another bill than the app built.
     */
    void it("tells an item's line total as the library does, and stores a fractional quantity whole", () => {
        const flat = new Flat();
        const sharing = { kind: 'equal', members: [flat.ana, flat.bob] } as const;
        const cheese = { name: 'Cheese', unitPrice: flat.euros(4_200n), split: sharing, quantity: Quantity.parse('0.35') };
        // 0.335 at 1.00 EUR is 33.5 cents, which rounds away from zero.
        const olives = { name: 'Olives', unitPrice: flat.euros(100n), split: sharing, quantity: Quantity.parse('0.335') };
        // A ten-thousandth of 1.00 EUR is a hundredth of a cent, which rounds to zero: a free line.
        const free = { ...olives, name: 'Bread', quantity: Quantity.ofTenThousandths(1n) };
        assert.ok(expenseItemLineTotal(cheese).equals(flat.euros(1_470n)));
        assert.ok(expenseItemLineTotal(olives).equals(flat.euros(34n)));
        assert.ok(expenseItemLineTotal(free).equals(flat.euros(0n)));
        assert.ok(expenseItemLineTotal({ ...olives, quantity: Quantity.ofWhole(2n) }).equals(flat.euros(200n)));
        assert.ok(expenseItemLineTotal({ name: 'Water', unitPrice: flat.euros(150n), split: sharing }).equals(flat.euros(150n)));

        const refusals = [
            {
                item: { ...free, split: { kind: 'amounts', shares: [{ member: flat.ana, amount: flat.euros(1n) }] } } as const,
                failure: { kind: 'amountsDoNotAddUp', expected: flat.euros(0n), actual: flat.euros(1n) },
            },
            { item: { ...olives, unitPrice: flat.euros(9_223_372_036_854_775_807n), quantity: Quantity.ofWhole(2n) }, failure: { kind: 'overflow' } },
            { item: { ...olives, name: ' ' }, failure: { kind: 'emptyItemName' } },
        ];
        for (const refusal of refusals) {
            assert.throws(
                () => expenseItemLineTotal(refusal.item),
                (error: unknown) => {
                    assert.ok(error instanceof ExpenseError);
                    assert.deepEqual(error.failure, refusal.failure);
                    return true;
                },
            );
        }

        const deli = createExpense({
            name: 'Deli',
            cost: { kind: 'local', amount: flat.euros(1_504n) },
            payers: { kind: 'equal', members: [flat.ana] },
            split: { kind: 'itemised', items: [cheese, olives, free] },
            date: flat.day,
        });
        assert.deepEqual(convertedExpense(generatedExpense(deli)), deli);
        assert.deepEqual(flat.group.expenseSync(flat.group.addExpenseSync(deli)), deli);
    });

    /**
     * A discount that an app took off a line itself, rounding the line where the library rounds each unit or the other way,
     * would show a bill whose lines do not add up to the amount the library checks them against; a whole price taken off
     * that the library refused would leave an app no way to record a line on the house.
     */
    void it("takes an item's discount off the line or off each unit as the library does", () => {
        const flat = new Flat();
        const sharing = { kind: 'equal', members: [flat.ana, flat.bob] } as const;
        const half = { kind: 'percentage', percentage: Percentage.parse('50') } as const;
        // Three units at 3.33 EUR are 9.99 EUR. Half of each unit is 1.665 EUR, which rounds to 1.67 EUR, so each unit costs
        // 1.66 EUR; half the line is 4.995 EUR, which rounds to 5.00 EUR.
        const juice = { name: 'Juice', unitPrice: flat.euros(333n), split: sharing, quantity: Quantity.ofWhole(3n) };
        const offEachJuice = { ...juice, discount: { value: half, scope: DiscountScope.UNIT } };
        const offTheJuice = { ...juice, discount: { value: half } };
        assert.ok(expenseItemLineTotalBeforeDiscount(offEachJuice).equals(flat.euros(999n)));
        assert.ok(expenseItemDiscountAmount(offEachJuice).equals(flat.euros(501n)));
        assert.ok(expenseItemLineTotal(offEachJuice).equals(flat.euros(498n)));
        assert.ok(expenseItemLineTotalBeforeDiscount(offTheJuice).equals(flat.euros(999n)));
        assert.ok(expenseItemDiscountAmount(offTheJuice).equals(flat.euros(500n)));
        assert.ok(expenseItemLineTotal(offTheJuice).equals(flat.euros(499n)));

        const euro = { kind: 'fixed', amount: flat.euros(100n) } as const;
        const lemonade = { name: 'Lemonade', unitPrice: flat.euros(300n), split: sharing, quantity: Quantity.ofWhole(3n) };
        const offEachLemonade = { ...lemonade, discount: { value: euro, scope: DiscountScope.UNIT } };
        const offTheLemonade = { ...lemonade, discount: { value: euro, scope: DiscountScope.LINE } };
        assert.ok(expenseItemDiscountAmount(offEachLemonade).equals(flat.euros(300n)));
        assert.ok(expenseItemLineTotal(offEachLemonade).equals(flat.euros(600n)));
        assert.ok(expenseItemDiscountAmount(offTheLemonade).equals(flat.euros(100n)));
        assert.ok(expenseItemLineTotal(offTheLemonade).equals(flat.euros(800n)));
        assert.ok(expenseItemLineTotalBeforeDiscount(lemonade).equals(flat.euros(900n)));
        assert.ok(expenseItemDiscountAmount(lemonade).equals(flat.euros(0n)));
        assert.ok(expenseItemLineTotal(lemonade).equals(flat.euros(900n)));

        // The whole price taken off, by 100 % or by a fixed amount equal to it, leaves a free line.
        const onTheHouse = { name: 'Papanași', unitPrice: flat.euros(900n), split: { kind: 'equal', members: [flat.bob] } } as const;
        const free = { ...onTheHouse, discount: { value: { kind: 'percentage', percentage: Percentage.parse('100') } } } as const;
        const eachFree = { ...lemonade, discount: { value: { kind: 'fixed', amount: flat.euros(300n) }, scope: DiscountScope.UNIT } } as const;
        assert.ok(expenseItemDiscountAmount(free).equals(flat.euros(900n)));
        assert.ok(expenseItemLineTotal(free).equals(flat.euros(0n)));
        assert.ok(expenseItemLineTotal(eachFree).equals(flat.euros(0n)));
        assert.ok(expenseItemLineTotal({ ...onTheHouse, discount: { value: { kind: 'fixed', amount: flat.euros(900n) } } }).equals(flat.euros(0n)));

        // A free line divides to zeros: Bob's share is his half of the juice alone.
        const round = createExpense({
            name: 'Round',
            cost: { kind: 'local', amount: flat.euros(1_298n) },
            payers: { kind: 'equal', members: [flat.ana] },
            split: { kind: 'itemised', items: [offEachJuice, { ...offTheLemonade, split: { kind: 'equal', members: [flat.ana] } }, free] },
            date: flat.day,
        });
        const shares = flat.group.expenseSharesSync(flat.group.addExpenseSync(round));
        assert.ok(undefined !== shares);
        assert.ok(shares.members.find((share) => flat.bob === share.member)?.share.equals(flat.euros(249n)));
        assert.ok(shares.members.find((share) => flat.ana === share.member)?.share.equals(flat.euros(1_049n)));
    });

    /**
     * A discount the layer let through that the library refuses, or one it refused under another failure, would leave an app
     * a line whose total it cannot show, or tell the user another reason than the library's.
     */
    void it('refuses a discount the line cannot take', () => {
        const flat = new Flat();
        const papanasi = { name: 'Papanași', unitPrice: flat.euros(900n), split: { kind: 'equal', members: [flat.ana] } } as const;
        const lei = CurrencyCode.of('RON');
        const refusals = [
            { discount: { value: { kind: 'fixed', amount: flat.euros(-1n) } }, failure: { kind: 'negativeDiscount' } },
            {
                discount: { value: { kind: 'percentage', percentage: Percentage.ofBasisPoints(10_001) } },
                failure: { kind: 'discountAboveHundredPercent' },
            },
            { discount: { value: { kind: 'fixed', amount: flat.euros(901n) } }, failure: { kind: 'discountAbovePrice' } },
            // 10.00 EUR off a line of two units at 9.00 EUR would fit its 18.00 EUR, but off each unit it is above the price.
            {
                discount: { value: { kind: 'fixed', amount: flat.euros(1_000n) }, scope: DiscountScope.UNIT },
                failure: { kind: 'discountAbovePrice' },
                quantity: Quantity.ofWhole(2n),
            },
            {
                discount: { value: { kind: 'fixed', amount: new Money(100n, lei) } },
                failure: { kind: 'currencyMismatch', expected: flat.euro, found: lei },
            },
        ] as const;
        for (const refusal of refusals) {
            const item = { ...papanasi, quantity: 'quantity' in refusal ? refusal.quantity : Quantity.ONE, discount: refusal.discount };
            for (const told of [expenseItemLineTotalBeforeDiscount, expenseItemDiscountAmount, expenseItemLineTotal]) {
                assert.throws(
                    () => told(item),
                    (error: unknown) => {
                        assert.ok(error instanceof ExpenseError);
                        assert.deepEqual(error.failure, refusal.failure);
                        return true;
                    },
                );
            }
        }
        assert.throws(
            () => expenseItemLineTotal({ ...papanasi, discount: { value: { kind: 'fixed', amount: flat.euros(1_000n) } } }),
            (error: unknown) => error instanceof ExpenseError && 'the discount is larger than the price it reduces' === error.message,
        );
    });

    /**
     * A discount that converted with its value or its scope changed or dropped, that the group stored another way, or that
     * read back without the scope the layer gives one left out would take another amount off the line than the app set.
     */
    void it('converts and stores every kind of discount whole, off the line unless it says otherwise', () => {
        const flat = new Flat();
        const sharing = { kind: 'equal', members: [flat.ana, flat.bob] } as const;
        const tenth = { kind: 'percentage', percentage: Percentage.parse('10') } as const;
        const euro = { kind: 'fixed', amount: flat.euros(100n) } as const;
        const items: readonly ExpenseItemInput[] = [
            { name: 'Telemea', unitPrice: flat.euros(4_200n), split: sharing, quantity: Quantity.fromNumber(0.35), discount: { value: tenth } },
            {
                name: 'Lemonade',
                unitPrice: flat.euros(300n),
                split: sharing,
                quantity: Quantity.ofWhole(3n),
                discount: { value: euro, scope: DiscountScope.UNIT },
            },
            { name: 'Wine', unitPrice: flat.euros(2_000n), split: sharing, discount: { value: tenth, scope: DiscountScope.UNIT } },
            { name: 'Bread', unitPrice: flat.euros(500n), split: sharing, discount: { value: euro, scope: DiscountScope.LINE } },
            { name: 'Water', unitPrice: flat.euros(150n), split: sharing },
        ];
        // 13.23 + 6.00 + 18.00 + 4.00 + 1.50.
        const round = createExpense({
            name: 'Round',
            cost: { kind: 'local', amount: flat.euros(4_273n) },
            payers: { kind: 'equal', members: [flat.ana] },
            split: { kind: 'itemised', items },
            date: flat.day,
        });
        assert.ok('itemised' === round.split.kind);
        const discounts: ReadonlyArray<Discount | undefined> = round.split.items.map((item) => item.discount);
        assert.deepEqual(discounts, [
            { value: tenth, scope: DiscountScope.LINE },
            { value: euro, scope: DiscountScope.UNIT },
            { value: tenth, scope: DiscountScope.UNIT },
            { value: euro, scope: DiscountScope.LINE },
            undefined,
        ]);
        assert.deepEqual(Object.keys(round.split.items[4] ?? {}), ['name', 'unitPrice', 'split', 'quantity']);
        assert.deepEqual(convertedExpense(generatedExpense(round)), round);
        assert.deepEqual(flat.group.expenseSync(flat.group.addExpenseSync(round)), round);

        // A record read back is passed again as it is, and a replacement without a line's discount clears it.
        const stored = flat.group.expenseSync(round.id);
        assert.ok('itemised' === stored?.split.kind);
        const discount: DiscountInput | undefined = stored.split.items[0]?.discount;
        assert.deepEqual(discount, { value: tenth, scope: DiscountScope.LINE });
        assert.deepEqual(flat.group.replaceExpenseSync(stored), round);
        const [telemea, ...rest] = stored.split.items;
        assert.ok(undefined !== telemea);
        const undiscounted = { name: telemea.name, unitPrice: telemea.unitPrice, split: telemea.split, quantity: telemea.quantity };
        const fullPrice = {
            ...stored,
            cost: { kind: 'local', amount: flat.euros(4_420n) },
            split: { kind: 'itemised', items: [undiscounted, ...rest] },
        } as const;
        flat.group.replaceExpenseSync(fullPrice);
        const replaced = flat.group.expenseSync(round.id);
        assert.ok('itemised' === replaced?.split.kind);
        assert.equal(replaced.split.items[0]?.discount, undefined);
        assert.deepEqual(replaced, fullPrice);
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
        assert.deepEqual(dan, { id: dan.id, firstName: 'Dan', accounts: [] });
        assert.deepEqual(convertedMember(generatedMember(dan)), dan);
        assert.equal(flat.group.addMemberSync(carol), carol.id);
        assert.deepEqual(flat.group.memberSync(carol.id), carol);

        const lei = CurrencyCode.of('RON');
        const cash = createTransfer({
            sender: flat.bob,
            receiver: flat.ana,
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
     * A last name, a note, or a note that read back as empty text when it was left out or blank, that a replace
     * without it kept, or that kept the whitespace around it, would show an app a detail it never gave.
     */
    void it('reads back a left-out or blank last name, note, or note as absent, and the rest trimmed', () => {
        const flat = new Flat();
        const group = flat.group;

        const carol = createMember({ firstName: 'Carol' });
        group.addMemberSync({ ...carol, lastName: ' Popa\n' });
        assert.deepEqual(group.memberSync(carol.id), { ...carol, lastName: 'Popa' });
        for (const member of [carol, { ...carol, lastName: ' ' }]) {
            group.replaceMemberSync({ ...carol, lastName: 'Popa' });
            group.replaceMemberSync(member);
            assert.deepEqual(group.memberSync(carol.id), carol);
            const added = { ...member, id: generateMemberId() };
            group.addMemberSync(added);
            assert.deepEqual(group.memberSync(added.id), { ...carol, id: added.id });
        }

        const lunch = flat.expense('Lunch', flat.ana, [flat.ana, flat.bob]);
        group.addExpenseSync({ ...lunch, note: ' At the market\n' });
        assert.deepEqual(group.expenseSync(lunch.id), { ...lunch, note: 'At the market' });
        for (const expense of [lunch, { ...lunch, note: ' ' }]) {
            group.replaceExpenseSync({ ...lunch, note: 'At the market' });
            group.replaceExpenseSync(expense);
            assert.deepEqual(group.expenseSync(lunch.id), lunch);
            const added = { ...expense, id: generateExpenseId() };
            group.addExpenseSync(added);
            assert.deepEqual(group.expenseSync(added.id), { ...lunch, id: added.id });
        }

        const cash = createTransfer({ sender: flat.bob, receiver: flat.ana, cost: { kind: 'local', amount: flat.euros(500n) }, date: flat.day });
        group.addTransferSync({ ...cash, note: ' Cash\n' });
        assert.deepEqual(group.transferSync(cash.id), { ...cash, note: 'Cash' });
        for (const transfer of [cash, { ...cash, note: ' ' }]) {
            group.replaceTransferSync({ ...cash, note: 'Cash' });
            group.replaceTransferSync(transfer);
            assert.deepEqual(group.transferSync(cash.id), cash);
            const added = { ...transfer, id: generateTransferId() };
            group.addTransferSync(added);
            assert.deepEqual(group.transferSync(added.id), { ...cash, id: added.id });
        }
    });

    /**
     * A create function that returned an identifier an earlier call had returned, or kept one its fields carry, would make a
     * copy of an entry that the group refuses as a duplicate of the original; one that listed the fields in another order
     * than a record read back, its identifier elsewhere than first, would make the two print and serialise differently.
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
        assert.deepEqual(Object.keys(memberCopy), Object.keys(member));
        assert.deepEqual(Object.keys(categoryCopy), Object.keys(category));
        assert.deepEqual(Object.keys(expenseCopy), Object.keys(expense));
        assert.deepEqual(Object.keys(transferCopy), Object.keys(transfer));
        assert.ok([memberCopy, categoryCopy, expenseCopy, transferCopy].every((created) => 'id' === Object.keys(created)[0]));
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
        const cash = createTransfer({ sender: flat.ana, receiver: flat.bob, cost: { kind: 'local', amount: flat.euros(100n) }, date: flat.day });
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
        const cash = {
            id: repayment,
            sender: flat.ana,
            receiver: carol,
            cost: { kind: 'local', amount: flat.euros(1_500n) },
            date: flat.day,
        } as const;
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
