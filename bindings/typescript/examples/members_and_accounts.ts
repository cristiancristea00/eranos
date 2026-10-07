// A flat's members with their contact and account details, what validation accepts of each, and a flatmate who moves
// out. The port of the Rust example `members_and_accounts`, which prints the same report.

import assert from 'node:assert/strict';

import {
    type PaymentAccount,
    type Balance,
    type MemberId,
    type Split,
    CalendarDate,
    CurrencyCode,
    Email,
    ExpenseGroupError,
    ExpenseGroup,
    Iban,
    Money,
    PhoneNumber,
    Username,
    createExpense,
    createMember,
    formatBalance,
} from '../src/index.js';
import { type Columns, load, printLines, printTable, refusal, section, totals } from './report.js';

/** A member with their contact and account details, as the table of members prints them. */
interface MemberRow {
    readonly name: string;
    readonly email: string;
    readonly phone: string;
    readonly accounts: string;
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

/** A member's status and totals, as the table of balances prints them. */
interface BalanceRow {
    readonly name: string;
    readonly status: string;
    readonly consumed: Money;
    readonly balance: Balance;
}

const balanceColumns: Columns<BalanceRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Status', text: (row) => row.status },
    { header: 'Consumed', text: (row) => row.consumed.toString() },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

/** The two flatmates: Ana, who gives every detail, and Bob, who moves out. */
interface Flatmates {
    readonly ana: MemberId;
    readonly bob: MemberId;
}

/** An account with its label and its details. */
function describe(account: PaymentAccount): string {
    const details = 'iban' === account.details.kind ? account.details.iban.toString() : account.details.username.toString();
    return `${account.name}: ${details}`;
}

/** `valid: ` and what `shown` writes of the value `parse` makes of `input`, or `refused: ` and the library's message. */
function validated<Value>(input: string, kind: string, parse: (text: string) => Value, shown: (value: Value) => string): CheckRow {
    let result = '';
    const refused = refusal(() => {
        result = `valid: ${shown(parse(input))}`;
    });
    return { input, kind, result: undefined === refused ? result : `refused: ${refused.message}` };
}

function printMembers(group: ExpenseGroup): void {
    printTable(
        memberColumns,
        group.membersSync().map((member) => ({
            name: [member.firstName, member.lastName].filter((name) => undefined !== name).join(' '),
            email: member.email?.toString() ?? '',
            phone: member.phone?.toString() ?? '',
            accounts: (member.accounts ?? []).map(describe).join('\n'),
        })),
        4,
    );
}

function printBalances(group: ExpenseGroup): void {
    const summary = group.summarySync();
    printTable(
        balanceColumns,
        group.membersSync().flatMap((member) => {
            const found = totals(member.id, summary);
            return undefined === found
                ? []
                : [
                      {
                          name: member.firstName,
                          status: group.isArchivedSync(member.id) ? 'archived' : 'active',
                          consumed: found.share,
                          balance: found.balance,
                      },
                  ];
        }),
        2,
    );
}

function members(group: ExpenseGroup): Flatmates {
    section('1. Members');
    const ana = group.addMemberSync(
        createMember({
            firstName: 'Ana',
            lastName: 'Popescu',
            email: Email.parse('ana@example.com'),
            phone: PhoneNumber.parse('+40 712 345 678'),
            accounts: [
                { name: 'ING', details: { kind: 'iban', iban: Iban.parse('ro49 aaaa 1b31 0075 9384 0000') } },
                { name: 'Revolut', details: { kind: 'username', username: Username.parse('@ana') } },
            ],
        }),
    );
    const bob = group.addMemberSync(createMember({ firstName: 'Bob' }));
    printMembers(group);
    printLines(
        'The phone number was stored in E.164 form, +40712345678, and the table shows it',
        'grouped for reading; the IBAN was stored in its electronic form, uppercase and',
        'without spaces, the way banking apps accept it.',
    );
    return { ana, bob };
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
    const iban = (input: string): CheckRow => validated(input, 'IBAN', (text) => Iban.parse(text), String);
    const email = (input: string): CheckRow => validated(input, 'email', (text) => Email.parse(text), String);
    const phone = (input: string): CheckRow =>
        validated(
            input,
            'phone',
            (text) => PhoneNumber.parse(text),
            (number) => number.e164,
        );
    printTable(
        checkColumns,
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

function movingOut(group: ExpenseGroup, flatmates: Flatmates): void {
    section('3. A flatmate moves out');
    const { ana, bob } = flatmates;
    const euro = CurrencyCode.of('EUR');
    const both: Split = { kind: 'equal', members: [ana, bob] };
    const rent = group.addExpenseSync(
        createExpense({
            name: 'Rent',
            cost: { kind: 'local', amount: Money.parse('900.00', euro) },
            payers: { kind: 'equal', members: [ana] },
            split: { kind: 'whole', split: both },
            date: CalendarDate.of(2026, 9, 1),
        }),
    );
    group.archiveMemberSync(bob);
    printLines(
        "Bob shared September's rent, then moved out. He is archived, not removed: his",
        'history and his balance stay, and he can still pay Ana back.',
    );
    printBalances(group);

    const refused = refusal(() =>
        group.addExpenseSync(
            createExpense({
                name: 'Utilities',
                cost: { kind: 'local', amount: Money.parse('60.00', euro) },
                payers: { kind: 'equal', members: [ana] },
                split: { kind: 'whole', split: both },
                date: CalendarDate.of(2026, 10, 1),
            }),
        ),
    );
    assert.ok(refused instanceof ExpenseGroupError, 'the archived member must be refused');
    printLines("October's utilities cannot include him:", `  error: ${refused.message}`);
    const removal = refusal(() => group.removeMemberSync(bob));
    assert.ok(removal instanceof ExpenseGroupError, 'the referenced member must stay');
    printLines('Nor can he be removed while the rent references him:', `  error: ${removal.message}`);

    assert.deepEqual(refused.failure, { kind: 'archivedMember', member: bob });
    assert.equal(removal.failure.kind, 'memberInUse');
    assert.deepEqual(totals(bob, group.summarySync())?.balance, { kind: 'owes', amount: Money.parse('450.00', euro) });
    assert.ok(undefined !== group.expenseSync(rent));
}

await load();
using group = ExpenseGroup.create('Flat 4B', CurrencyCode.of('EUR'));
printLines('Flat 4B: members and their accounts', '═══════════════════════════════════');
printLines(
    'Ana and Bob share a flat. A member needs only a first name; the last name,',
    'email, phone, and the accounts they can be paid to are optional.',
);
const flatmates = members(group);
validation();
movingOut(group, flatmates);
