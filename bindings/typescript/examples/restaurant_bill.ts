// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Expense,
    type ExpenseShares,
    type Item,
    type MemberId,
    CalendarDate,
    Currency,
    Group,
    Money,
    Parts,
    Percentage,
    TipDistribution,
    splitMembers,
} from '../src/index.js';
import { load, names, printLines, printTable, refusal, section, title } from './report.js';

/** The amount written `text`, in euros. */
function eur(text: string): Money {
    return Money.parse(text, Currency.of('EUR'));
}

/** The receipt: every line with the members who had it. */
function receipt(ana: MemberId, bob: MemberId, carol: MemberId): Item[] {
    return [
        { name: 'Ciorbă', split: { kind: 'equal', members: [ana] }, unitPrice: eur('8.00') },
        { name: 'Sarmale', split: { kind: 'equal', members: [bob, carol] }, unitPrice: eur('14.00'), quantity: 2 },
        {
            name: 'Papanași',
            split: {
                kind: 'parts',
                shares: [
                    { member: ana, parts: Parts.ofHundredths(100) },
                    { member: carol, parts: Parts.ofHundredths(200) },
                ],
            },
            unitPrice: eur('9.00'),
        },
        { name: 'Lemonade', split: { kind: 'equal', members: [ana, bob, carol] }, unitPrice: eur('3.00'), quantity: 3 },
    ];
}

/**
 * How `expense` would divide if it were added to `group`, computed on a copy loaded from the group's saved form, so the
 * group itself is unchanged.
 */
function preview(group: Group, expense: Expense): ExpenseShares | undefined {
    using copy = Group.loadSync(group.saveSync());
    return copy.expenseSharesSync(copy.addExpenseSync(expense));
}

/** What `member` consumed of the expense that `shares` divides. */
function consumed(shares: ExpenseShares | undefined, member: MemberId): Money | undefined {
    return shares?.members.find((share) => share.member === member)?.share;
}

await load();
using group = Group.create(Currency.of('EUR'), "Dinner at Caru' cu Bere");
const ana = group.addMemberSync({ firstName: 'Ana' });
const bob = group.addMemberSync({ firstName: 'Bob' });
const carol = group.addMemberSync({ firstName: 'Carol' });
const day = CalendarDate.of(23, 9, 2026);

title(`${group.nameSync()}: an itemised bill`);
printLines(
    'Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,',
    'every line of the receipt is split among the people who had it.',
);

section('1. The receipt');
printTable(
    ['Item', 'Shared by', 'Quantity', 'Unit price', 'Line total'],
    receipt(ana, bob, carol).map((item) => {
        const quantity = item.quantity ?? 1;
        const total = new Money(item.unitPrice.currency, item.unitPrice.minorUnits * BigInt(quantity));
        return [item.name, names(group, splitMembers(item.split)), String(quantity), item.unitPrice.toString(), total.toString()];
    }),
    2,
);
printLines(
    'The papanași are split by parts: Carol ate two of the three. The line totals add up',
    "to 54.00 EUR, and the expense's amount must match them exactly.",
);

section('2. Who ordered what, and the tip');
// The dinner, its tip of 10 % shared by `distribution`.
const bill = (distribution: TipDistribution): Expense => ({
    cost: { kind: 'local', amount: eur('54.00') },
    date: day,
    name: 'Dinner',
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'itemised', items: receipt(ana, bob, carol) },
    tip: { value: { kind: 'percentage', percentage: Percentage.parse('10') }, distribution },
});
const untipped: Expense = {
    cost: { kind: 'local', amount: eur('54.00') },
    date: day,
    name: 'Dinner',
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'itemised', items: receipt(ana, bob, carol) },
};
const ordered = preview(group, untipped);
const proportional = group.addExpenseSync(bill(TipDistribution.PROPORTIONAL));
const byConsumption = group.expenseSharesSync(proportional);
const even = preview(group, bill(TipDistribution.EQUAL));
printLines(
    `Ana paid the bill plus a 10 % tip (${byConsumption?.tip.toString() ?? '?'}). A tip can be shared in proportion to what`,
    'each person ordered, or equally; both are shown:',
);
printTable(
    ['Member', 'Ordered', 'Tip by consumption', 'Pays', 'Tip shared equally', 'Pays'],
    [ana, bob, carol].map((member) => {
        const subtotal = consumed(ordered, member);
        const proportionalTotal = consumed(byConsumption, member);
        const equalTotal = consumed(even, member);
        assert.ok(undefined !== subtotal && undefined !== proportionalTotal && undefined !== equalTotal, 'every member ordered');
        return [
            names(group, [member]),
            subtotal.toString(),
            proportionalTotal.minus(subtotal).toString(),
            proportionalTotal.toString(),
            equalTotal.minus(subtotal).toString(),
            equalTotal.toString(),
        ];
    }),
    1,
);
printLines('By consumption, Carol, who ordered the most, adds the largest part of the tip;', 'shared equally, everyone adds the same 1.80.');

section('3. A total that does not match is refused');
const mismatch = refusal(() =>
    group.addExpenseSync({
        cost: { kind: 'local', amount: eur('55.00') },
        date: day,
        name: 'Dinner',
        payers: { kind: 'equal', members: [ana] },
        split: { kind: 'itemised', items: receipt(ana, bob, carol) },
    }),
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
