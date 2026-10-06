// Sections 1 to 3 of the tour: the group and its details, the members with their contact details and accounts, and
// the categories.

import assert from 'node:assert/strict';

import { type Account, Currency, Email, Group, GroupError, Iban, Language, PhoneNumber, Username } from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section } from '../report.js';
import { type Categories, type People, cell, outcome, refusalColumns } from './tour.js';

/** A detail of the group, as the table of details prints it. */
interface DetailRow {
    readonly detail: string;
    readonly value: string;
}

const detailColumns: Columns<DetailRow> = [
    { header: 'Detail', text: (row) => row.detail },
    { header: 'Value', text: (row) => row.value },
];

/** A member with their contact and account details, as the table of members prints them. */
interface MemberRow {
    readonly accounts: string;
    readonly email: string;
    readonly name: string;
    readonly phone: string;
}

const memberColumns: Columns<MemberRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Email', text: (row) => row.email },
    { header: 'Phone', text: (row) => row.phone },
    { header: 'Accounts', text: (row) => row.accounts },
];

/** An input with what validation makes of it, as the table of checks prints it. */
interface CheckRow {
    readonly input: string;
    readonly kind: string;
    readonly result: string;
}

const checkColumns: Columns<CheckRow> = [
    { header: 'Input', text: (row) => row.input },
    { header: 'Checked as', text: (row) => row.kind },
    { header: 'Result', text: (row) => row.result },
];

export function theGroup(): Group {
    section('1. The group');
    const group = Group.create(Currency.of('EUR'), 'Lisbon', { description: 'A week by the Atlantic', language: Language.parse('pt-pt') });
    const blank = refusal(() => {
        group.setNameSync('   ');
    });
    group.setNameSync('Lisbon and Porto');
    group.setDescriptionSync('A week by the Atlantic, with a day in Porto');
    const language = group.languageSync()?.toString() ?? '';
    printTable(
        detailColumns,
        [
            { detail: 'Name', value: group.nameSync() },
            { detail: 'Currency', value: group.currencySync().toString() },
            { detail: 'Language', value: language },
            { detail: 'Description', value: group.descriptionSync() ?? '' },
        ],
        2,
    );
    printLines(
        'The group started as Lisbon and was renamed when a day in Porto joined the plan.',
        `Every total is kept in its currency, ${group.currencySync().toString()}. The language is a BCP 47 tag the apps`,
        `show the group in, stored in canonical form: pt-pt became ${language}. A blank name is`,
        'refused:',
        `  error: ${message(blank)}`,
    );

    assert.ok(blank instanceof GroupError && 'emptyName' === blank.failure.kind);
    assert.equal(group.nameSync(), 'Lisbon and Porto');
    assert.equal(language, 'pt-PT');
    return group;
}

/** An account with its label, its kind, and its details. */
function describe(account: Account): string {
    return 'iban' === account.details.kind
        ? `${account.name}, IBAN ${account.details.iban.toString()}`
        : `${account.name}, username ${account.details.username.toString()}`;
}

function printMembers(group: Group): void {
    printTable(
        memberColumns,
        group.membersSync().map((member) => ({
            accounts: cell((member.accounts ?? []).map(describe)),
            email: member.email?.toString() ?? '',
            name: [member.firstName, member.lastName].filter((part) => undefined !== part).join(' '),
            phone: member.phone?.toString() ?? '',
        })),
        4,
    );
}

function validation(group: Group): void {
    printLines(
        'IBANs are checked against the SWIFT registry and their mod-97 check digits,',
        "emails against RFC 5321 and 5322, and phone numbers against their country's",
        'numbering plan. Required text must not be blank:',
    );
    const iban = (input: string): CheckRow => ({
        input: `"${input}"`,
        kind: 'IBAN',
        result: outcome(
            () => Iban.parse(input),
            (parsed) => `${parsed.toString()}, country ${parsed.countryCode}`,
        ),
    });
    const phone = (input: string): CheckRow => ({
        input: `"${input}"`,
        kind: 'phone',
        result: outcome(
            () => PhoneNumber.parse(input),
            (parsed) => `${parsed.toString()}, stored as ${parsed.e164}`,
        ),
    });
    printTable(
        checkColumns,
        [
            iban('ro49 aaaa 1b31 0075 9384 0000'),
            iban('RO49 AAAA 1B31 0075 9384 0001'),
            { input: '"ana..popescu@example.com"', kind: 'email', result: outcome(() => Email.parse('ana..popescu@example.com'), String) },
            phone('+351 912 345 678'),
            phone('+40 612 345 678'),
            { input: '"   "', kind: 'username', result: outcome(() => Username.parse('   '), String) },
            { input: '"  "', kind: 'first name', result: outcome(() => group.addMemberSync({ firstName: '  ' }), String) },
        ],
        3,
    );
}

export function members(group: Group): People {
    section('2. Members');
    const ana = group.addMemberSync({
        firstName: 'Ana',
        accounts: [
            { details: { kind: 'iban', iban: Iban.parse('RO49 AAAA 1B31 0075 9384 0000') }, name: 'ING' },
            { details: { kind: 'username', username: Username.parse('@ana') }, name: 'Revolut' },
        ],
        email: Email.parse('ana@example.com'),
        lastName: 'Popescu',
        phone: PhoneNumber.parse('+40 712 345 678'),
    });
    const bob = group.addMemberSync({ firstName: 'Bob' });
    const carol = group.addMemberSync({ firstName: 'Carol' });
    const dan = group.addMemberSync({ firstName: 'Dan' });
    const eva = group.addMemberSync({ firstName: 'Eva' });
    const filip = group.addMemberSync({ firstName: 'Filip' });
    // An edit starts from a copy of the member, which keeps its identifier.
    const stored = group.memberSync(bob);
    assert.ok(undefined !== stored, 'Bob is a member');
    const before = group.replaceMemberSync({ ...stored, email: Email.parse('bob@example.com'), lastName: 'Ionescu' });
    printMembers(group);
    printLines(
        'Only a first name is required. Ana gave every detail; Bob added his last name and',
        'email later, through an edit that kept his identifier:',
        `  ${bob}`,
        "Eva is Carol's daughter, and Filip was added by mistake.",
    );
    validation(group);

    assert.equal(before.lastName, undefined);
    assert.equal(group.memberSync(bob)?.lastName, 'Ionescu');
    assert.equal(group.memberSync(ana)?.accounts?.length, 2);
    return { ana, bob, carol, dan, eva, filip };
}

export function addCategories(group: Group): Categories {
    section('3. Categories');
    const food = group.addCategorySync({ name: 'Food' });
    const lodging = group.addCategorySync({ name: 'Lodging' });
    const transport = group.addCategorySync({ name: 'Transport' });
    const homonym = refusal(() => group.addCategorySync({ name: 'FOOD' }));
    const stored = group.categorySync(transport);
    assert.ok(undefined !== stored, 'Transport was added');
    group.replaceCategorySync({ ...stored, name: 'Getting around' });
    const uncategorised = group.categoriesSync()[0];
    assert.ok(undefined !== uncategorised, 'every group has the built-in category');
    const builtIn = refusal(() => group.replaceCategorySync({ ...uncategorised, name: 'Other' }));
    printLines(
        'Every group has the built-in Uncategorised; this one adds Food, Lodging, and',
        'Transport, then renames Transport, which keeps its identifier. In identifier order:',
        `  ${group
            .categoriesSync()
            .map((category) => category.name)
            .join(', ')}`,
        'Names are compared caselessly, and the built-in category keeps its name:',
    );
    printTable(
        refusalColumns,
        [
            { change: 'Add a category named FOOD', error: message(homonym) },
            { change: 'Rename Uncategorised to Other', error: message(builtIn) },
        ],
        2,
    );

    assert.ok(homonym instanceof GroupError);
    assert.deepEqual(homonym.failure, { kind: 'duplicateCategoryName', existing: food });
    assert.ok(builtIn instanceof GroupError && 'builtInCategory' === builtIn.failure.kind);
    assert.equal(group.categorySync(transport)?.name, 'Getting around');
    return { food, lodging, transport };
}
