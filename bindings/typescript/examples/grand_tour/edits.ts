// Sections 5 to 7 of the tour: entries in another currency, edits and refused changes, and members who come and go.

import assert from 'node:assert/strict';

import {
    type Balance,
    type Expense,
    type ExpenseGroup,
    type ExpenseId,
    CurrencyCode,
    ExchangeRate,
    ExpenseError,
    ExpenseGroupError,
    Money,
    Percentage,
    TimeOfDay,
    TransferError,
    createExpense,
    createTransfer,
    formatBalance,
    summaryMember,
} from '../../src/index.js';
import { type Columns, message, printLines, printTable, refusal, section, status } from '../report.js';
import { type Categories, type People, eur, name, october, printBalances, printEntries, refusalColumns, ron } from './tour.js';

/** An entry and what an edit did to it, as the table of edits prints it. */
interface EditRow {
    readonly entry: string;
    readonly edit: string;
}

const editColumns: Columns<EditRow> = [
    { header: 'Entry', text: (row) => row.entry },
    { header: 'Edit', text: (row) => row.edit },
];

/** A member's status and balance, as the table of statuses prints them. */
interface StatusRow {
    readonly name: string;
    readonly status: string;
    readonly balance: Balance;
}

const statusColumns: Columns<StatusRow> = [
    { header: 'Member', text: (row) => row.name },
    { header: 'Status', text: (row) => row.status },
    { header: 'Balance', text: (row) => formatBalance(row.balance) },
];

export function anotherCurrency(group: ExpenseGroup, people: People, categories: Categories): void {
    section('5. Another currency');
    const euro = CurrencyCode.of('EUR');
    const rate = ExchangeRate.parse('0.2');
    const parking = group.addExpenseSync(
        createExpense({
            name: 'Airport parking',
            cost: { kind: 'converted', amount: ron('250.00'), rate },
            payers: { kind: 'equal', members: [people.ana] },
            split: { kind: 'whole', split: { kind: 'equal', members: [people.ana, people.bob] } },
            date: october(6),
            category: categories.transport,
        }),
    );
    const cash = group.addTransferSync(
        createTransfer({
            sender: people.bob,
            receiver: people.ana,
            cost: { kind: 'converted', amount: ron('100.00'), rate },
            date: october(6),
            time: TimeOfDay.of(7, 40),
            note: 'Cash for the parking',
        }),
    );
    printLines(
        'Back at the Bucharest airport, Ana paid the parking in RON and Bob paid her back in',
        'cash. An entry in another currency carries the rate into the group currency, how',
        'many EUR one RON is worth, and is converted once, when it is entered:',
    );
    printEntries(group, (currency) => euro.code !== currency.code);
    const transfer = group.transferSync(cash);
    assert.ok(undefined !== transfer, 'the cash was recorded');
    const converted = ExchangeRate.parse('0.2011').convert(ron('250.00'), euro);
    printLines(
        `Bob's transfer, at ${transfer.time?.toString() ?? ''} ("${transfer.note ?? ''}"), counts as ${(group.transferAmountSync(cash) ?? Money.zero(euro)).toString()}.`,
        'A conversion rounds half away from zero: 250.00 RON at 0.2011 is 50.275 EUR,',
        `which becomes ${converted.toString()}.`,
    );

    assert.ok(eur('50.28').equals(converted));
    assert.ok(eur('50.00').equals(group.expenseSharesSync(parking)?.total ?? eur('0')));
    assert.ok(eur('20.00').equals(group.transferAmountSync(cash) ?? eur('0')));
    assert.equal(transfer.cost.kind, 'converted');
}

/** Replaces an expense and a transfer, and removes one of each; returns what changed. */
function replaceAndRemove(group: ExpenseGroup, people: People, categories: Categories, seafood: ExpenseId): EditRow[] {
    // The dinner's receipt said 99.00.
    const dinner = group.expenseSync(seafood);
    assert.ok(undefined !== dinner, 'the dinner was added');
    const previous = group.replaceExpenseSync({ ...dinner, cost: { kind: 'local', amount: eur('99.00') }, note: 'Cervejaria Ramiro' });
    // Bob entered the coffee twice.
    const coffee = (): Expense =>
        createExpense({
            name: 'Coffee',
            cost: { kind: 'local', amount: eur('6.00') },
            payers: { kind: 'equal', members: [people.bob] },
            split: { kind: 'whole', split: { kind: 'equal', members: [people.ana, people.bob, people.carol] } },
            date: october(5),
            category: categories.food,
        });
    group.addExpenseSync(coffee());
    const copy = group.addExpenseSync(coffee());
    const removed = group.removeExpenseSync(copy);
    // Dan's part of the apartment was 120.00, not 100.00.
    const first = group.addTransferSync(
        createTransfer({ sender: people.dan, receiver: people.ana, cost: { kind: 'local', amount: eur('100.00') }, date: october(5) }),
    );
    const stored = group.transferSync(first);
    assert.ok(undefined !== stored, 'the transfer was added');
    const replaced = group.replaceTransferSync({ ...stored, cost: { kind: 'local', amount: eur('120.00') }, note: 'Apartment' });
    // Carol sent 10.00 to the wrong member.
    const mistake = group.addTransferSync(
        createTransfer({ sender: people.carol, receiver: people.bob, cost: { kind: 'local', amount: eur('10.00') }, date: october(5) }),
    );
    const undone = group.removeTransferSync(mistake);

    assert.ok(eur('90.00').equals(previous.cost.amount));
    assert.equal(group.expenseSync(copy), undefined);
    assert.ok(eur('120.00').equals(group.transferAmountSync(first) ?? eur('0')));
    assert.equal(group.transferSync(mistake), undefined);
    const zero = Money.zero(CurrencyCode.of('EUR'));
    const corrected = group.expenseSync(seafood);
    const apartment = group.transferSync(first);
    return [
        {
            entry: previous.name,
            edit: `${previous.cost.amount.toString()} became ${(corrected?.cost.amount ?? zero).toString()}, at ${corrected?.note ?? ''}`,
        },
        { entry: removed.name, edit: `entered twice; the copy of ${removed.cost.amount.toString()} was removed` },
        {
            entry: `${name(group, people.dan)} to ${name(group, people.ana)}`,
            edit: `${replaced.cost.amount.toString()} became ${(apartment?.cost.amount ?? zero).toString()}, noted ${apartment?.note ?? ''}`,
        },
        {
            entry: `${name(group, people.carol)} to ${name(group, people.bob)}`,
            edit: `${undone.cost.amount.toString()} sent to the wrong member; removed`,
        },
    ];
}

export function edits(group: ExpenseGroup, people: People, categories: Categories, seafood: ExpenseId): void {
    section('6. Edits');
    const changes = replaceAndRemove(group, people, categories, seafood);
    printLines(
        'Entries are values: an edit starts from a copy made with to_builder, keeps the',
        'identifier, and replaces the entry; a removal takes it out with its totals.',
    );
    printTable(editColumns, changes, 2);
    printLines('The balances after the edits:');
    printBalances(group);

    const before = group.summarySync();
    const unbalanced = refusal(() =>
        group.addExpenseSync(
            createExpense({
                name: 'Taxi',
                cost: { kind: 'local', amount: eur('30.00') },
                payers: { kind: 'equal', members: [people.ana] },
                split: {
                    kind: 'whole',
                    split: {
                        kind: 'percentages',
                        shares: [
                            { member: people.ana, percentage: Percentage.parse('50') },
                            { member: people.bob, percentage: Percentage.parse('49.99') },
                        ],
                    },
                },
                date: october(6),
            }),
        ),
    );
    const toHerself = refusal(() =>
        group.addTransferSync(
            createTransfer({ sender: people.ana, receiver: people.ana, cost: { kind: 'local', amount: eur('5.00') }, date: october(6) }),
        ),
    );
    const unrated = refusal(() =>
        group.addExpenseSync(
            createExpense({
                name: 'Souvenirs',
                cost: { kind: 'local', amount: ron('40.00') },
                payers: { kind: 'equal', members: [people.ana] },
                split: { kind: 'whole', split: { kind: 'equal', members: [people.ana] } },
                date: october(6),
            }),
        ),
    );
    printLines('A change that does not fit is refused, and the group stays as it was:');
    printTable(
        refusalColumns,
        [
            { change: 'A taxi split 50% and 49.99%', error: message(unbalanced) },
            { change: 'Ana paying herself back', error: message(toHerself) },
            { change: 'Souvenirs in RON, without a rate', error: message(unrated) },
        ],
        2,
    );
    printLines('The balances again, unchanged:');
    printBalances(group);

    assert.ok(unbalanced instanceof ExpenseError);
    assert.deepEqual(unbalanced.failure, { kind: 'percentagesDoNotAddUp', totalBasisPoints: 9_999 });
    assert.ok(toHerself instanceof TransferError && 'sameMember' === toHerself.failure.kind);
    assert.ok(unrated instanceof ExpenseGroupError);
    assert.deepEqual(unrated.failure, { kind: 'missingExchangeRate', currency: CurrencyCode.of('RON') });
    assert.deepEqual(group.summarySync(), before);
}

export function comingsAndGoings(group: ExpenseGroup, people: People): void {
    section('7. Members come and go');
    const removed = group.removeMemberSync(people.filip);
    group.archiveMemberSync(people.dan);
    const farewell = refusal(() =>
        group.addExpenseSync(
            createExpense({
                name: 'Farewell dinner',
                cost: { kind: 'local', amount: eur('40.00') },
                payers: { kind: 'equal', members: [people.ana] },
                split: { kind: 'whole', split: { kind: 'equal', members: [people.ana, people.bob, people.carol, people.dan] } },
                date: october(6),
            }),
        ),
    );
    const inUse = refusal(() => group.removeMemberSync(people.dan));
    printLines(
        `${removed.firstName} was never part of an entry, so removing him leaves no trace. Dan flew home`,
        'early and was archived: his history and his balance stay, but a new expense',
        'cannot include him, and he cannot be removed while entries reference him.',
    );
    const summary = group.summarySync();
    printTable(
        statusColumns,
        group.membersSync().map((member) => {
            const found = summaryMember(summary, member.id);
            return { name: member.firstName, status: status(group, member.id), balance: found.balance };
        }),
        2,
    );
    printTable(
        refusalColumns,
        [
            { change: 'A farewell dinner with Dan', error: message(farewell) },
            { change: 'Removing Dan', error: message(inUse) },
        ],
        2,
    );
    group.unarchiveMemberSync(people.dan);
    printLines('When Dan joins the next trip, unarchiving him makes him active again.');

    assert.equal(removed.firstName, 'Filip');
    assert.equal(group.memberSync(people.filip), undefined);
    assert.ok(farewell instanceof ExpenseGroupError);
    assert.deepEqual(farewell.failure, { kind: 'archivedMember', member: people.dan });
    assert.ok(inUse instanceof ExpenseGroupError && 'memberInUse' === inUse.failure.kind);
    assert.equal(group.isArchivedSync(people.dan), false);
}
