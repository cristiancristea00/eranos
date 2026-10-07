// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

import assert from 'node:assert/strict';

import { type MemberId, type Summary, CalendarDate, Currency, Group, Money, Percentage, SettlementStrategy, formatBalance } from '../src/index.js';
import { type Columns, load, printLines, printTable, refusal, section, title, totals } from './report.js';

/** A member's balance before saving and after loading, as the table of balances prints it. */
interface ComparisonRow {
    readonly name: string;
    readonly before: string;
    readonly after: string;
}

const comparisonColumns: Columns<ComparisonRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Balance before saving', text: (row) => row.before },
    { header: 'Balance after loading', text: (row) => row.after },
];

/** The balance of `member` in `summary`, as the library writes it. */
function balance(member: MemberId, summary: Summary): string {
    const found = totals(member, summary);
    return undefined === found ? '?' : formatBalance(found.balance);
}

await load();
const euro = Currency.of('EUR');
const day = CalendarDate.of(2026, 9, 23);
using group = Group.create('Road trip', euro);
const ana = group.addMemberSync({ firstName: 'Ana' });
const bob = group.addMemberSync({ firstName: 'Bob' });
group.addExpenseSync({
    name: 'Fuel',
    cost: { kind: 'local', amount: Money.parse('70.00', euro) },
    payers: { kind: 'equal', members: [ana] },
    split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob] } },
    date: day,
});
group.addExpenseSync({
    name: 'Lunch',
    cost: { kind: 'local', amount: Money.parse('30.00', euro) },
    payers: { kind: 'equal', members: [bob] },
    split: { kind: 'whole', split: { kind: 'equal', members: [ana, bob] } },
    date: day,
    tip: { value: { kind: 'percentage', percentage: Percentage.parse('10') } },
});

title('Road trip: saving and loading a group');
printLines(
    'With the `serde` feature a group serialises, and so does every type it stores.',
    'It saves only what was entered: its details, members, categories, expenses,',
    'and transfers. The totals, balances, and settlements are recomputed when it is',
    'loaded.',
);

section('1. The saved form');
// Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an encrypted
// database).
const saved = group.saveSync();
printLines(saved);
printLines(
    'Amounts are minor units with their currency, dates are YYYY-MM-DD, and the',
    '`format` number lets a future release refuse a file it cannot read instead of',
    'dropping fields.',
);

section('2. Loading it back');
using loaded = Group.loadSync(saved);
printLines(
    'Loading replays every entry through the same checks as adding it, then rebuilds',
    'the totals. The loaded group equals the saved one, and so do its balances:',
);
const before = group.summarySync();
const after = loaded.summarySync();
printTable(
    comparisonColumns,
    [ana, bob].map((member) => ({
        name: group.memberSync(member)?.firstName ?? '?',
        before: balance(member, before),
        after: balance(member, after),
    })),
    1,
);
const plan = loaded.settlementSync(SettlementStrategy.GREEDY);
printLines(`Payments needed to settle the loaded group: ${String(plan.payments.length)}.`);

section('3. A corrupted file is refused');
printLines(
    "Here Ana's identifier was edited in her member entry, so the fuel's payer is an",
    'identifier no member has. Reading the file as a SavedGroup and loading it with',
    'into_group replays every entry through the same checks as adding it, and refuses',
    'the file instead of producing balances that do not add up:',
);
// Ana's own entry is the first to name her: the members come before the expenses in the saved form, so the fuel
// she paid is left naming an identifier no member has.
const corrupted = saved.replace(ana, '0199a000-0000-7000-8000-00000000ffff');
const refused = refusal(() => {
    Group.loadSync(corrupted).destroy();
});
assert.ok(undefined !== refused, 'the file must be refused');
printLines(`  error: ${refused.message}`);

assert.equal(loaded.saveSync(), saved);
assert.deepEqual(after, before);
assert.equal(plan.payments.length, 1);
