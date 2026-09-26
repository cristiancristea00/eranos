// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.

import assert from 'node:assert/strict';

import { type GroupFailure, Group, GroupError } from '../../src/index.js';
import { message, printLines, printTable, refusal, section } from '../report.js';
import type { Categories, People } from './tour.js';

export function savingAndLoading(group: Group, people: People, categories: Categories): void {
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
    using loaded = Group.loadSync(saved);
    printLines(
        'Loading replays every entry through the same checks as adding it and rebuilds the',
        'totals: the loaded group equals the saved one. Reading a SavedGroup and loading it',
        'with into_group reports a refusal as a typed error instead of a message:',
    );
    const refusals = [
        ['"format": 3', saved.replace('"format": 2', '"format": 3')],
        ['a blank group name', saved.replace('"name": "Lisbon and Porto"', '"name": " "')],
        ['Lodging renamed FOOD', saved.replace('"name": "Lodging"', '"name": "FOOD"')],
    ].map(([change = '', text = '']) => {
        assert.notEqual(text, saved, `${change} changes the file`);
        const refused = refusal(() => {
            Group.loadSync(text).destroy();
        });
        assert.ok(refused instanceof GroupError, `${change} is refused`);
        return { change, refused };
    });
    printTable(
        ['Change to the file', 'Message'],
        refusals.map(({ change, refused }) => [change, message(refused)]),
        2,
    );

    assert.equal(loaded.saveSync(), saved);
    assert.deepEqual(loaded.summarySync(), group.summarySync());
    const failures: GroupFailure[] = refusals.map(({ refused }) => refused.failure);
    assert.deepEqual(failures, [
        { kind: 'unsupportedFormat', format: 3 },
        { kind: 'emptyName' },
        { kind: 'duplicateCategoryName', existing: categories.food },
    ]);
}
