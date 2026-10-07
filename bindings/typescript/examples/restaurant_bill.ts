// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Expense,
    type ExpenseShares,
    type ExpenseItemInput,
    type MemberId,
    CalendarDate,
    CurrencyCode,
    ExpenseGroup,
    Money,
    Parts,
    Percentage,
    TipDistribution,
    createExpense,
    createMember,
    splitMembers,
} from '../src/index.js';
import { type Columns, load, names, printLines, printTable, refusal, section, title } from './report.js';

/** A line of the receipt, as the table of the receipt prints it. */
interface LineRow {
    readonly name: string;
    readonly participants: string;
    readonly quantity: number;
    readonly unitPrice: Money;
    readonly lineTotal: Money;
}

const lineColumns: Columns<LineRow> = [
    { header: 'Item', text: (row) => row.name },
    { header: 'Shared by', text: (row) => row.participants },
    { header: 'Quantity', text: (row) => String(row.quantity) },
    { header: 'Unit price', text: (row) => row.unitPrice.toString() },
    { header: 'Line total', text: (row) => row.lineTotal.toString() },
];

/** What a member ordered and pays with each way of sharing the tip, as the table of tips prints it. */
interface TipRow {
    readonly name: string;
    readonly subtotal: Money;
    readonly proportionalTip: Money;
    readonly proportionalTotal: Money;
    readonly equalTip: Money;
    readonly equalTotal: Money;
}

const tipColumns: Columns<TipRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Ordered', text: (row) => row.subtotal.toString() },
    { header: 'Tip by consumption', text: (row) => row.proportionalTip.toString() },
    { header: 'Pays', text: (row) => row.proportionalTotal.toString() },
    { header: 'Tip shared equally', text: (row) => row.equalTip.toString() },
    { header: 'Pays', text: (row) => row.equalTotal.toString() },
];

/** The amount written `text`, in euros. */
function eur(text: string): Money {
    return Money.parse(text, CurrencyCode.of('EUR'));
}

/** The receipt: every line with the members who had it. */
function receipt(ana: MemberId, bob: MemberId, carol: MemberId): ExpenseItemInput[] {
    return [
        { name: 'Ciorbă', unitPrice: eur('8.00'), split: { kind: 'equal', members: [ana] } },
        { name: 'Sarmale', unitPrice: eur('14.00'), split: { kind: 'equal', members: [bob, carol] }, quantity: 2 },
        {
            name: 'Papanași',
            unitPrice: eur('9.00'),
            split: {
                kind: 'parts',
                shares: [
                    { member: ana, parts: Parts.ofHundredths(100) },
                    { member: carol, parts: Parts.ofHundredths(200) },
                ],
            },
        },
        { name: 'Lemonade', unitPrice: eur('3.00'), split: { kind: 'equal', members: [ana, bob, carol] }, quantity: 3 },
    ];
}

/**
 * How `expense` would divide if it were added to `group`, computed on a copy loaded from the group's saved form, so the
 * group itself is unchanged.
 */
function preview(group: ExpenseGroup, expense: Expense): ExpenseShares | undefined {
    using copy = ExpenseGroup.loadSync(group.saveSync());
    return copy.expenseSharesSync(copy.addExpenseSync(expense));
}

/** What `member` consumed of the expense that `shares` divides. */
function consumed(shares: ExpenseShares | undefined, member: MemberId): Money | undefined {
    return shares?.members.find((share) => share.member === member)?.share;
}

await load();
using group = ExpenseGroup.create("Dinner at Caru' cu Bere", CurrencyCode.of('EUR'));
const ana = group.addMemberSync(createMember({ firstName: 'Ana' }));
const bob = group.addMemberSync(createMember({ firstName: 'Bob' }));
const carol = group.addMemberSync(createMember({ firstName: 'Carol' }));
const day = CalendarDate.of(2026, 9, 23);

title(`${group.nameSync()}: an itemised bill`);
printLines(
    'Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,',
    'every line of the receipt is split among the people who had it.',
);

section('1. The receipt');
printTable(
    lineColumns,
    receipt(ana, bob, carol).map((item) => {
        const quantity = item.quantity ?? 1;
        return {
            name: item.name,
            participants: names(group, splitMembers(item.split)),
            quantity,
            unitPrice: item.unitPrice,
            lineTotal: new Money(item.unitPrice.minorUnits * BigInt(quantity), item.unitPrice.currency),
        };
    }),
    2,
);
printLines(
    'The papanași are split by parts: Carol ate two of the three. The line totals add up',
    "to 54.00 EUR, and the expense's amount must match them exactly.",
);

section('2. Who ordered what, and the tip');
// The dinner, its tip of 10 % shared by `distribution`.
const bill = (distribution: TipDistribution): Expense =>
    createExpense({
        name: 'Dinner',
        cost: { kind: 'local', amount: eur('54.00') },
        payers: { kind: 'equal', members: [ana] },
        split: { kind: 'itemised', items: receipt(ana, bob, carol) },
        date: day,
        tip: { value: { kind: 'percentage', percentage: Percentage.parse('10') }, distribution },
    });
const untipped = createExpense({
    name: 'Dinner',
    cost: { kind: 'local', amount: eur('54.00') },
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'itemised', items: receipt(ana, bob, carol) },
    date: day,
});
const ordered = preview(group, untipped);
const proportional = group.addExpenseSync(bill(TipDistribution.PROPORTIONAL));
const byConsumption = group.expenseSharesSync(proportional);
const even = preview(group, bill(TipDistribution.EQUAL));
printLines(
    `Ana paid the bill plus a 10 % tip (${byConsumption?.tip.toString() ?? '?'}). A tip can be shared in proportion to what`,
    'each person ordered, or equally; both are shown:',
);
printTable(
    tipColumns,
    [ana, bob, carol].map((member) => {
        const subtotal = consumed(ordered, member);
        const proportionalTotal = consumed(byConsumption, member);
        const equalTotal = consumed(even, member);
        assert.ok(undefined !== subtotal && undefined !== proportionalTotal && undefined !== equalTotal, 'every member ordered');
        return {
            name: names(group, [member]),
            subtotal,
            proportionalTip: proportionalTotal.minus(subtotal),
            proportionalTotal,
            equalTip: equalTotal.minus(subtotal),
            equalTotal,
        };
    }),
    1,
);
printLines('By consumption, Carol, who ordered the most, adds the largest part of the tip;', 'shared equally, everyone adds the same 1.80.');

section('3. A total that does not match is refused');
const mismatch = refusal(() =>
    group.addExpenseSync(
        createExpense({
            name: 'Dinner',
            cost: { kind: 'local', amount: eur('55.00') },
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'itemised', items: receipt(ana, bob, carol) },
            date: day,
        }),
    ),
);
assert.ok(undefined !== mismatch, 'the mismatch must be refused');
printLines('An amount of 55.00 EUR with these lines is refused when the expense is built:', `  error: ${mismatch.message}`);

// Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each;
// an equal one 1.80 to everyone.
assert.ok(eur('15.40').equals(consumed(byConsumption, ana) ?? eur('0')));
assert.ok(eur('18.70').equals(consumed(byConsumption, bob) ?? eur('0')));
assert.ok(eur('25.30').equals(consumed(byConsumption, carol) ?? eur('0')));
assert.ok(eur('24.80').equals(consumed(even, carol) ?? eur('0')));
assert.ok(eur('59.40').equals(even?.total ?? eur('0')));
