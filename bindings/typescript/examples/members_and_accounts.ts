// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type Account,
    type MemberId,
    type Split,
    CalendarDate,
    Currency,
    Email,
    GroupError,
    Group,
    Iban,
    Money,
    PhoneNumber,
    Username,
    formatBalance,
} from '../src/index.js';
import { load, printLines, printTable, refusal, section, totals } from './report.js';

/** An account with its label and its details. */
function describe(account: Account): string {
    const details = 'iban' === account.details.kind ? account.details.iban.toString() : account.details.username.toString();
    return `${account.name}: ${details}`;
}

/** `valid: ` and what `shown` writes of the value `parse` makes of `input`, or `refused: ` and the library's message. */
function validated<Value>(input: string, kind: string, parse: (text: string) => Value, shown: (value: Value) => string): string[] {
    let result = '';
    const refused = refusal(() => {
        result = `valid: ${shown(parse(input))}`;
    });
    return [input, kind, undefined === refused ? result : `refused: ${refused.message}`];
}

function printMembers(group: Group): void {
    printTable(
        ['Member', 'Email', 'Phone', 'Accounts'],
        group
            .membersSync()
            .map((member) => [
                [member.firstName, member.lastName].filter((name) => undefined !== name).join(' '),
                member.email?.toString() ?? '',
                member.phone?.toString() ?? '',
                (member.accounts ?? []).map(describe).join('\n'),
            ]),
        4,
    );
}

function printBalances(group: Group): void {
    const summary = group.summarySync();
    printTable(
        ['Member', 'Status', 'Consumed', 'Balance'],
        group.membersSync().flatMap((member) => {
            const found = undefined === member.id ? undefined : totals(member.id, summary);
            return undefined === member.id || undefined === found
                ? []
                : [[member.firstName, group.isArchivedSync(member.id) ? 'archived' : 'active', found.share.toString(), formatBalance(found.balance)]];
        }),
        2,
    );
}

function members(group: Group): [MemberId, MemberId] {
    section('1. Members');
    const ana = group.addMemberSync({
        firstName: 'Ana',
        accounts: [
            { details: { kind: 'iban', iban: Iban.parse('ro49 aaaa 1b31 0075 9384 0000') }, name: 'ING' },
            { details: { kind: 'username', username: Username.parse('@ana') }, name: 'Revolut' },
        ],
        email: Email.parse('ana@example.com'),
        lastName: 'Popescu',
        phone: PhoneNumber.parse('+40 712 345 678'),
    });
    const bob = group.addMemberSync({ firstName: 'Bob' });
    printMembers(group);
    printLines(
        'The phone number was stored in E.164 form, +40712345678, and the table shows it',
        'grouped for reading; the IBAN was stored in its electronic form, uppercase and',
        'without spaces, the way banking apps accept it.',
    );
    return [ana, bob];
}

function validation(): void {
    section('2. What validation accepts');
    printLines(
        'IBANs are checked against the SWIFT registry: the country, its length, which',
        'positions hold digits and which letters, and the mod-97 check digits, which catch',
        'any single mistyped character. Emails are checked against the rules of RFC 5321',
        'and 5322, which refuse the doubled dots and stray commas of hurried typing, and',
        "phone numbers against their country's numbering plan, stored in E.164 form.",
    );
    const iban = (input: string): string[] => validated(input, 'IBAN', (text) => Iban.parse(text), String);
    const email = (input: string): string[] => validated(input, 'email', (text) => Email.parse(text), String);
    const phone = (input: string): string[] =>
        validated(
            input,
            'phone',
            (text) => PhoneNumber.parse(text),
            (number) => number.e164,
        );
    printTable(
        ['Input', 'Checked as', 'Result'],
        [
            iban('ro49 aaaa 1b31 0075 9384 0000'),
            iban('RO49 AAAA 1B31 0075 9384 0001'),
            iban('RO49 AAAA 1B31 0075'),
            iban('DE46 37O4 0044 0532 0130 00'),
            email('ana@example.com'),
            email('ana.example.com'),
            email('ana..popescu@example.com'),
            phone('+40 (712) 345-678'),
            phone('0712 345 678'),
            phone('+40 612 345 678'),
            phone('+44 (0)20 7946 0958'),
        ],
        3,
    );
}

function movingOut(group: Group, ana: MemberId, bob: MemberId): void {
    section('3. A flatmate moves out');
    const euro = Currency.of('EUR');
    const both: Split = { kind: 'equal', members: [ana, bob] };
    const rent = group.addExpenseSync({
        cost: { kind: 'local', amount: Money.parse('900.00', euro) },
        date: CalendarDate.of(1, 9, 2026),
        name: 'Rent',
        payers: { kind: 'equal', members: [ana] },
        split: { kind: 'whole', split: both },
    });
    group.archiveMemberSync(bob);
    printLines(
        "Bob shared September's rent, then moved out. He is archived, not removed: his",
        'history and his balance stay, and he can still pay Ana back.',
    );
    printBalances(group);

    const refused = refusal(() =>
        group.addExpenseSync({
            cost: { kind: 'local', amount: Money.parse('60.00', euro) },
            date: CalendarDate.of(1, 10, 2026),
            name: 'Utilities',
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'whole', split: both },
        }),
    );
    assert.ok(refused instanceof GroupError, 'the archived member must be refused');
    printLines("October's utilities cannot include him:", `  error: ${refused.message}`);
    const removal = refusal(() => group.removeMemberSync(bob));
    assert.ok(removal instanceof GroupError, 'the referenced member must stay');
    printLines('Nor can he be removed while the rent references him:', `  error: ${removal.message}`);

    assert.deepEqual(refused.failure, { kind: 'archivedMember', member: bob });
    assert.equal(removal.failure.kind, 'memberInUse');
    assert.deepEqual(totals(bob, group.summarySync())?.balance, { kind: 'owes', amount: Money.parse('450.00', euro) });
    assert.ok(undefined !== group.expenseSync(rent));
}

await load();
using group = Group.create(Currency.of('EUR'), 'Flat 4B');
printLines('Flat 4B: members and their accounts', '═══════════════════════════════════');
printLines(
    'Ana and Bob share a flat. A member needs only a first name; the last name,',
    'email, phone, and the accounts they can be paid to are optional.',
);
const [ana, bob] = members(group);
validation();
movingOut(group, ana, bob);
