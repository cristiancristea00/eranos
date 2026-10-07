// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.

import assert from 'node:assert/strict';

import { type ExpenseGroupFailure, ExpenseGroup, ExpenseGroupError } from '../../src/index.js';
import { type Columns, printLines, printTable, refusal, section } from '../report.js';
import type { Categories, People } from './tour.js';

/** A change made to a saved group, and the text it leaves. */
interface Tampering {
    readonly change: string;
    readonly text: string;
}

/** A change to the saved group and the refusal of its loading, as the table of refusals prints them. */
interface LoadRow {
    readonly change: string;
    readonly message: ExpenseGroupError;
}

const loadColumns: Columns<LoadRow> = [
    { header: 'Change to the file', text: (row) => row.change },
    { header: 'Message', text: (row) => row.message.message },
];

export function savingAndLoading(group: ExpenseGroup, people: People, categories: Categories): void {
    section('12. Saving and loading');
    const saved = group.saveSync();
    printLines(
        `The group saves to ${String(new TextEncoder().encode(saved).length)} bytes of JSON: its details and entries, never the totals.`,
        "Ana's accounts are saved in clear text, as the library holds them:",
    );
    for (const account of group.memberSync(people.ana)?.accounts ?? []) {
        const details = 'iban' === account.details.kind ? account.details.iban.toString() : account.details.username.toString();
        assert.ok(saved.includes(details), `${details} is saved as it is`);
        printLines(`  ${account.name}: ${details}`);
    }
    using loaded = ExpenseGroup.loadSync(saved);
    printLines(
        'Loading replays every entry through the same checks as adding it and rebuilds the',
        'totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and',
        'loading it with into_expense_group reports a refusal as a typed error, not as text:',
    );
    const tampered: Tampering[] = [
        { change: '"format": 3', text: saved.replace('"format": 2', '"format": 3') },
        { change: 'a blank group name', text: saved.replace('"name": "Lisbon and Porto"', '"name": " "') },
        { change: 'Lodging renamed FOOD', text: saved.replace('"name": "Lodging"', '"name": "FOOD"') },
    ];
    const rows = tampered.map((tampering): LoadRow => {
        assert.notEqual(tampering.text, saved, `${tampering.change} changes the file`);
        const refused = refusal(() => {
            ExpenseGroup.loadSync(tampering.text).destroy();
        });
        assert.ok(refused instanceof ExpenseGroupError, `${tampering.change} is refused`);
        return { change: tampering.change, message: refused };
    });
    printTable(loadColumns, rows, 2);

    assert.equal(loaded.saveSync(), saved);
    assert.deepEqual(loaded.summarySync(), group.summarySync());
    const failures: ExpenseGroupFailure[] = rows.map((row) => row.message.failure);
    assert.deepEqual(failures, [
        { kind: 'unsupportedFormat', format: 3 },
        { kind: 'emptyName' },
        { kind: 'duplicateCategoryName', existing: categories.food },
    ]);
}
