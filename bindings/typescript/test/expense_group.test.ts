// The group, in both forms of every call, and its read handle. A call that answered another way than the group holds, a change
// that did not take effect, or a Promise that differed from its synchronous twin would show an app one group from an
// asynchronous function and another from a synchronous one.

import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
    CurrencyCode,
    ExchangeRate,
    ExpenseGroup,
    ExpenseGroupError,
    IdError,
    InvalidJsonError,
    Language,
    Money,
    SettlementStrategy,
    createCategory,
    createMember,
    createTransfer,
    expenseGroupId,
    memberId,
    parseExpenseGroupId,
} from '../src/index.js';
import { Flat, refusedIdentifiers } from './flat.js';

/** Whether `error` is the refusal of a call that found the group's lock held. */
function busy(error: unknown): boolean {
    return error instanceof ExpenseGroupError && 'busy' === error.failure.kind;
}

void describe('every call of the group, synchronous and asynchronous, and every read of its read handle', () => {
    /** A read that answered another way than the group holds would show an app entries, totals, or plans the group does not have. */
    void it('answers every synchronous read with what the group holds', () => {
        const flat = new Flat();
        const group = flat.group;
        assert.ok(0 < group.idSync().length);
        const categories = group.categoriesSync();
        assert.deepEqual(
            categories.map((category) => category.name),
            ['Uncategorised', 'Food'],
        );
        assert.deepEqual(group.categorySync(flat.food), categories[1]);
        assert.equal(group.currencySync(), flat.euro);
        const expenses = group.expensesSync();
        assert.deepEqual(
            expenses.map((expense) => expense.name),
            ['Dinner'],
        );
        assert.deepEqual(group.expenseSync(flat.dinner), expenses[0]);
        assert.equal(group.expenseSharesSync(flat.dinner)?.total.toString(), '30.00 EUR');
        const members = group.membersSync();
        assert.deepEqual(
            members.map((member) => member.firstName),
            ['Ana', 'Bob'],
        );
        assert.deepEqual(group.memberSync(flat.ana), members[0]);
        assert.equal(group.isArchivedSync(flat.ana), false);
        assert.equal(group.nameSync(), 'Flat');
        assert.equal(group.noteSync(), 'Rent');
        assert.equal(group.languageSync()?.toString(), 'pt-PT');
        assert.deepEqual(
            group.settlementSync(SettlementStrategy.GREEDY).payments.map((payment) => payment.amount.toString()),
            ['5.00 EUR'],
        );
        const transfers = group.transfersSync();
        assert.equal(transfers.length, 1);
        assert.deepEqual(group.transferSync(flat.repayment), transfers[0]);
        const json = group.saveSync();
        assert.equal(ExpenseGroup.loadSync(json).saveSync(), json);
        for (const refused of refusedIdentifiers) {
            assert.throws(
                () => group.memberSync(memberId(refused.text)),
                (error: unknown) => error instanceof IdError && refused.failure.kind === error.failure.kind && refused.message === error.message,
            );
        }
    });

    /**
     * A Promise that answered another way than its synchronous twin would show an app one group from an asynchronous
     * function and another from a synchronous one.
     */
    void it('answers every asynchronous read as its synchronous twin does', async () => {
        const flat = new Flat();
        const group = flat.group;
        assert.equal(await group.id(), group.idSync());
        assert.deepEqual(await group.categories(), group.categoriesSync());
        assert.deepEqual(await group.category(flat.food), group.categorySync(flat.food));
        assert.equal(await group.currency(), group.currencySync());
        assert.deepEqual(await group.expenses(), group.expensesSync());
        assert.deepEqual(await group.expense(flat.dinner), group.expenseSync(flat.dinner));
        assert.deepEqual(await group.expenseShares(flat.dinner), group.expenseSharesSync(flat.dinner));
        assert.deepEqual(await group.summary(), group.summarySync());
        assert.deepEqual(await group.members(), group.membersSync());
        assert.deepEqual(await group.member(flat.bob), group.memberSync(flat.bob));
        assert.equal(await group.isArchived(flat.bob), group.isArchivedSync(flat.bob));
        assert.equal(await group.name(), group.nameSync());
        assert.equal(await group.note(), group.noteSync());
        assert.deepEqual(await group.language(), group.languageSync());
        for (const strategy of Object.values(SettlementStrategy)) {
            assert.deepEqual(await group.settlement(strategy), group.settlementSync(strategy));
        }
        assert.deepEqual(await group.transfers(), group.transfersSync());
        assert.deepEqual(await group.transfer(flat.repayment), group.transferSync(flat.repayment));
        assert.deepEqual(await group.transferAmount(flat.repayment), group.transferAmountSync(flat.repayment));
        const json = await group.save();
        assert.equal(json, group.saveSync());
        assert.equal((await ExpenseGroup.load(json)).saveSync(), json);
    });

    /**
     * A lookup of an identifier the group does not hold that found an entry, or that read the member as archived or active,
     * would show an app an entry of another group.
     */
    void it('finds nothing for an identifier the group does not hold', async () => {
        const flat = new Flat();
        const stranger = new Flat();
        const group = flat.group;
        assert.equal(group.memberSync(stranger.ana), undefined);
        assert.equal(group.isArchivedSync(stranger.ana), undefined);
        assert.equal(group.categorySync(stranger.food), undefined);
        assert.equal(group.expenseSync(stranger.dinner), undefined);
        assert.equal(group.expenseSharesSync(stranger.dinner), undefined);
        assert.equal(group.transferSync(stranger.repayment), undefined);
        assert.equal(group.transferAmountSync(stranger.repayment), undefined);
        assert.equal(await group.member(stranger.ana), undefined);
        assert.equal(await group.isArchived(stranger.ana), undefined);
        assert.equal(await group.category(stranger.food), undefined);
        assert.equal(await group.expense(stranger.dinner), undefined);
        assert.equal(await group.expenseShares(stranger.dinner), undefined);
        assert.equal(await group.transfer(stranger.repayment), undefined);
        assert.equal(await group.transferAmount(stranger.repayment), undefined);
        {
            using reader = group.readSync();
            assert.equal(reader.member(stranger.ana), undefined);
            assert.equal(reader.isArchived(stranger.ana), undefined);
            assert.equal(reader.category(stranger.food), undefined);
            assert.equal(reader.expense(stranger.dinner), undefined);
            assert.equal(reader.expenseShares(stranger.dinner), undefined);
            assert.equal(reader.transfer(stranger.repayment), undefined);
            assert.equal(reader.transferAmount(stranger.repayment), undefined);
        }
        using plain = ExpenseGroup.create('Trip', flat.euro);
        assert.equal(plain.noteSync(), undefined);
        assert.equal(plain.languageSync(), undefined);
        using reader = plain.readSync();
        assert.equal(reader.language(), undefined);
    });

    /**
     * A synchronous change that did not take effect, or took another, a removed member that still read as archived or active,
     * or a note that kept the whitespace around it or stayed set when blank, would leave the group other than the app
     * changed it.
     */
    void it('takes every synchronous change into effect', () => {
        const flat = new Flat();
        const group = flat.group;
        const carol = group.addMemberSync(createMember({ firstName: 'Carol' }));
        const stored = group.memberSync(carol);
        assert.ok(undefined !== stored);
        assert.deepEqual(group.replaceMemberSync({ ...stored, lastName: 'Popa' }), stored);
        assert.equal(group.memberSync(carol)?.lastName, 'Popa');
        group.archiveMemberSync(carol);
        assert.equal(group.isArchivedSync(carol), true);
        group.unarchiveMemberSync(carol);
        assert.equal(group.isArchivedSync(carol), false);
        assert.equal(group.removeMemberSync(carol).lastName, 'Popa');
        assert.equal(group.isArchivedSync(carol), undefined);
        assert.throws(
            () => group.removeMemberSync(flat.ana),
            (error: unknown) =>
                error instanceof ExpenseGroupError &&
                'memberInUse' === error.failure.kind &&
                error.failure.member === flat.ana &&
                error.failure.expenses[0] === flat.dinner &&
                error.failure.transfers[0] === flat.repayment,
        );

        const travel = group.addCategorySync(createCategory({ name: 'Travel' }));
        assert.equal(group.replaceCategorySync({ id: travel, name: 'Trips' }).name, 'Travel');
        assert.equal(group.removeCategorySync(travel).name, 'Trips');

        const lunch = group.addExpenseSync(flat.expense('Lunch', flat.bob, [flat.ana, flat.bob]));
        assert.equal(group.replaceExpenseSync({ ...flat.expense('Brunch', flat.bob, [flat.ana]), id: lunch }).name, 'Lunch');
        assert.equal(group.removeExpenseSync(lunch).name, 'Brunch');

        const cash = group.addTransferSync(
            createTransfer({ sender: flat.ana, receiver: flat.bob, cost: { kind: 'local', amount: flat.euros(100n) }, date: flat.day }),
        );
        const storedCash = group.transferSync(cash);
        assert.ok(undefined !== storedCash);
        assert.equal(group.replaceTransferSync({ ...storedCash, note: 'Cash' }).note, undefined);
        assert.equal(group.removeTransferSync(cash).note, 'Cash');

        group.setNameSync('Home');
        group.clearNoteSync();
        group.clearLanguageSync();
        assert.equal(group.nameSync(), 'Home');
        assert.equal(group.noteSync(), undefined);
        assert.equal(group.languageSync(), undefined);
        group.setNoteSync(' Shared flat\n');
        group.setLanguageSync(Language.parse('ro'));
        assert.equal(group.noteSync(), 'Shared flat');
        assert.equal(group.languageSync()?.toString(), 'ro');
        group.setNoteSync(' ');
        assert.equal(group.noteSync(), undefined);
        const lei = CurrencyCode.of('RON');
        group.changeCurrencySync(lei, [{ currency: flat.euro, rate: ExchangeRate.parse('5') }]);
        assert.equal(group.currencySync(), lei);
        assert.ok(group.summarySync().totalSpent.equals(new Money(15_000n, lei)));
    });

    /**
     * An asynchronous change that took effect another way than its synchronous twin would change the group differently from
     * an asynchronous function, and a removed member that still read as archived or active would show one it does not hold.
     */
    void it('takes every asynchronous change into effect as its synchronous twin does', async () => {
        const flat = new Flat();
        const group = flat.group;
        const carol = await group.addMember(createMember({ firstName: 'Carol' }));
        const stored = group.memberSync(carol);
        assert.ok(undefined !== stored);
        assert.deepEqual(await group.replaceMember({ ...stored, lastName: 'Popa' }), stored);
        await group.archiveMember(carol);
        assert.equal(await group.isArchived(carol), true);
        await group.unarchiveMember(carol);
        assert.equal(await group.isArchived(carol), false);
        assert.equal((await group.removeMember(carol)).lastName, 'Popa');
        assert.equal(await group.isArchived(carol), undefined);

        const travel = await group.addCategory(createCategory({ name: 'Travel' }));
        assert.equal((await group.replaceCategory({ id: travel, name: 'Trips' })).name, 'Travel');
        assert.equal((await group.removeCategory(travel)).name, 'Trips');

        const lunch = await group.addExpense(flat.expense('Lunch', flat.bob, [flat.ana]));
        assert.equal((await group.replaceExpense({ ...flat.expense('Brunch', flat.bob, [flat.ana]), id: lunch })).name, 'Lunch');
        assert.equal((await group.removeExpense(lunch)).name, 'Brunch');

        const cash = await group.addTransfer(
            createTransfer({ sender: flat.ana, receiver: flat.bob, cost: { kind: 'local', amount: flat.euros(100n) }, date: flat.day }),
        );
        assert.equal(
            (
                await group.replaceTransfer({
                    id: cash,
                    sender: flat.ana,
                    receiver: flat.bob,
                    cost: { kind: 'local', amount: flat.euros(100n) },
                    date: flat.day,
                    note: 'Cash',
                })
            ).note,
            undefined,
        );
        assert.equal((await group.removeTransfer(cash)).note, 'Cash');

        await group.setName('Home');
        await group.setNote('Shared flat');
        await group.setLanguage(Language.parse('ro'));
        assert.equal(group.nameSync(), 'Home');
        assert.equal(group.noteSync(), 'Shared flat');
        await group.clearNote();
        await group.clearLanguage();
        assert.equal(group.noteSync(), undefined);
        assert.equal(group.languageSync(), undefined);
        await group.setNote(' Rent\n');
        assert.equal(group.noteSync(), 'Rent');
        await group.setNote(' ');
        assert.equal(group.noteSync(), undefined);
        const lei = CurrencyCode.of('RON');
        await group.changeCurrency(lei, [{ currency: flat.euro, rate: ExchangeRate.parse('5') }]);
        assert.equal(group.currencySync(), lei);
        await assert.rejects(group.removeMember(flat.ana), ExpenseGroupError);
    });

    /**
     * A new group that dropped what it starts with, the identifier the app gives it included, kept a blank note, took
     * an identifier the rules refuse, shared a generated identifier with another group, or a saved one that loaded from text
     * that is not the saved form, would start an app from another group.
     */
    void it('starts a group with what it is given and loads only its saved form', async () => {
        const euro = CurrencyCode.of('EUR');
        using trip = ExpenseGroup.createWithId(parseExpenseGroupId('trip-2026'), 'Trip', euro, { note: 'Alps' });
        assert.equal(trip.idSync(), 'trip-2026');
        assert.equal(trip.nameSync(), 'Trip');
        assert.equal(trip.noteSync(), 'Alps');
        using blank = ExpenseGroup.create('Trip', euro, { note: ' ' });
        assert.equal(blank.noteSync(), undefined);
        assert.equal(ExpenseGroup.loadSync(trip.saveSync()).idSync(), 'trip-2026');
        assert.throws(
            () => ExpenseGroup.createWithId(expenseGroupId(' trip'), 'Trip', euro),
            (error: unknown) => error instanceof IdError && 'surroundingWhitespace' === error.failure.kind,
        );
        using first = ExpenseGroup.create('Trip', euro);
        using second = ExpenseGroup.create('Trip', euro);
        assert.equal(parseExpenseGroupId(first.idSync()), first.idSync());
        assert.notEqual(first.idSync(), second.idSync());
        assert.throws(
            () => ExpenseGroup.create(' ', euro),
            (error: unknown) =>
                error instanceof ExpenseGroupError && 'emptyName' === error.failure.kind && 'the group name must not be empty' === error.message,
        );
        assert.throws(
            () => ExpenseGroup.loadSync('{'),
            (error: unknown) =>
                error instanceof InvalidJsonError &&
                1 === error.line &&
                1 === error.column &&
                'the saved group ends early, at line 1, column 1' === error.message,
        );
        await assert.rejects(ExpenseGroup.load('{'), InvalidJsonError);
    });

    /**
     * A read handle that answered another way than the group, kept reading after its release, or kept the lock would show a
     * report of a group that has changed since.
     */
    void it('reads through a read handle as the group does until it is released', async () => {
        const flat = new Flat();
        const group = flat.group;
        group.archiveMemberSync(flat.bob);
        const reader = group.readSync();
        assert.equal(reader.id(), group.idSync());
        assert.deepEqual(reader.categories(), group.categoriesSync());
        assert.deepEqual(reader.category(flat.food), group.categorySync(flat.food));
        assert.equal(reader.currency(), flat.euro);
        assert.deepEqual(reader.expenses(), group.expensesSync());
        assert.deepEqual(reader.expense(flat.dinner), group.expenseSync(flat.dinner));
        assert.deepEqual(reader.expenseShares(flat.dinner), group.expenseSharesSync(flat.dinner));
        assert.deepEqual(reader.summary(), group.summarySync());
        assert.deepEqual(reader.members(), group.membersSync());
        assert.deepEqual(reader.member(flat.ana), group.memberSync(flat.ana));
        assert.equal(reader.isArchived(flat.ana), false);
        assert.equal(reader.isArchived(flat.bob), true);
        assert.equal(reader.name(), 'Flat');
        assert.equal(reader.note(), 'Rent');
        assert.equal(reader.language()?.toString(), 'pt-PT');
        assert.deepEqual(reader.settlement(SettlementStrategy.GREEDY), group.settlementSync(SettlementStrategy.GREEDY));
        assert.deepEqual(reader.transfers(), group.transfersSync());
        assert.deepEqual(reader.transfer(flat.repayment), group.transferSync(flat.repayment));
        assert.ok(reader.transferAmount(flat.repayment)?.equals(flat.euros(1_000n)));
        assert.equal(reader.save(), group.saveSync());
        reader.release();
        reader.release();
        assert.throws(
            () => reader.name(),
            (error: unknown) => error instanceof TypeError && 'the handle was released' === error.message,
        );
        {
            using held = await group.read();
            assert.equal(held.name(), 'Flat');
        }
        group.setNameSync('Home');
        assert.equal(group.nameSync(), 'Home');
    });

    /**
     * A write handle that answered another way than the group, let another call in, or kept running after its release would
     * show an app a report and a change of two states of one group.
     */
    void it('reads and changes through a write handle as the group does until it is released', async () => {
        const flat = new Flat();
        const group = flat.group;
        group.archiveMemberSync(flat.bob);
        const id = group.idSync();
        const categories = group.categoriesSync();
        const expenses = group.expensesSync();
        const shares = group.expenseSharesSync(flat.dinner);
        const summary = group.summarySync();
        const members = group.membersSync();
        const settlement = group.settlementSync(SettlementStrategy.GREEDY);
        const transfers = group.transfersSync();
        const saved = group.saveSync();
        const writer = group.writeSync();
        assert.equal(writer.id(), id);
        assert.deepEqual(writer.categories(), categories);
        assert.deepEqual(writer.category(flat.food), categories.at(-1));
        assert.equal(writer.currency(), flat.euro);
        assert.deepEqual(writer.expenses(), expenses);
        assert.deepEqual(writer.expense(flat.dinner), expenses[0]);
        assert.deepEqual(writer.expenseShares(flat.dinner), shares);
        assert.deepEqual(writer.summary(), summary);
        assert.deepEqual(writer.members(), members);
        assert.deepEqual(writer.member(flat.ana), members[0]);
        assert.equal(writer.isArchived(flat.ana), false);
        assert.equal(writer.isArchived(flat.bob), true);
        assert.equal(writer.name(), 'Flat');
        assert.equal(writer.note(), 'Rent');
        assert.equal(writer.language()?.toString(), 'pt-PT');
        assert.deepEqual(writer.settlement(SettlementStrategy.GREEDY), settlement);
        assert.deepEqual(writer.transfers(), transfers);
        assert.deepEqual(writer.transfer(flat.repayment), transfers[0]);
        assert.ok(writer.transferAmount(flat.repayment)?.equals(flat.euros(1_000n)));
        assert.equal(writer.save(), saved);
        writer.unarchiveMember(flat.bob);
        writer.setName('Home');
        writer.setNote('Lease');
        writer.clearNote();
        writer.setLanguage(Language.parse('ro'));
        writer.clearLanguage();
        assert.equal(writer.name(), 'Home');
        assert.equal(writer.note(), undefined);
        assert.equal(writer.language(), undefined);
        const food = writer.category(flat.food);
        assert.ok(food);
        assert.deepEqual(writer.replaceCategory(food), food);
        const dinner = writer.expense(flat.dinner);
        assert.ok(dinner);
        assert.deepEqual(writer.replaceExpense(dinner), dinner);
        assert.deepEqual(writer.removeExpense(flat.dinner), dinner);
        assert.equal(writer.addExpense(dinner), flat.dinner);
        const repayment = writer.transfer(flat.repayment);
        assert.ok(repayment);
        assert.deepEqual(writer.replaceTransfer(repayment), repayment);
        assert.deepEqual(writer.removeTransfer(flat.repayment), repayment);
        assert.equal(writer.addTransfer(repayment), flat.repayment);
        const carol = writer.addMember(createMember({ firstName: 'Carol' }));
        const popa = writer.member(carol);
        assert.ok(popa);
        assert.deepEqual(writer.replaceMember(popa), popa);
        writer.archiveMember(carol);
        assert.equal(writer.isArchived(carol), true);
        writer.unarchiveMember(carol);
        assert.deepEqual(writer.removeMember(carol), popa);
        assert.deepEqual(writer.removeCategory(flat.food), food);
        assert.equal(writer.addCategory(food), flat.food);
        const lei = CurrencyCode.of('RON');
        writer.changeCurrency(lei, [{ currency: flat.euro, rate: ExchangeRate.parse('5') }]);
        assert.equal(writer.currency(), lei);
        writer.release();
        writer.release();
        const released = (error: unknown): boolean => error instanceof TypeError && 'the handle was released' === error.message;
        assert.throws(() => writer.name(), released);
        assert.throws(() => {
            writer.setName('Flat');
        }, released);
        assert.equal(group.nameSync(), 'Home');
        {
            using held = await group.write();
            held.setName('Flat');
        }
        assert.equal(group.nameSync(), 'Flat');
    });

    /**
     * A synchronous change while a read holds the lock would wait on the page's one thread forever; an asynchronous one that did
     * not resume on the release would never settle, and a read that overtook a waiting change could keep it waiting for good.
     */
    void it('refuses a synchronous change while a read holds the lock and lets an asynchronous one wait', async () => {
        const flat = new Flat();
        const group = flat.group;
        const reader = group.readSync();
        assert.throws(() => group.addMemberSync(createMember({ firstName: 'Carol' })), busy);
        group.readSync().release();
        const adding = group.addMember(createMember({ firstName: 'Carol' }));
        let settled = false;
        void adding.then(() => (settled = true));
        await new Promise((resolve) => setTimeout(resolve, 10));
        assert.equal(settled, false);
        assert.throws(() => group.readSync(), busy);
        assert.throws(() => group.nameSync(), busy);
        reader.release();
        const carol = await adding;
        assert.equal(settled, true);
        assert.equal(group.memberSync(carol)?.firstName, 'Carol');
    });

    /**
     * An aborted call that still ran would change the group after the app gave the change up, and one that rejected with
     * another reason would hide why.
     */
    void it('cancels an aborted call, leaves the group as it was, and rejects with the reason', async () => {
        const flat = new Flat();
        const group = flat.group;
        const reader = group.readSync();
        const controller = new AbortController();
        const reason = new Error('the user left');
        const adding = group.addMember(createMember({ firstName: 'Carol' }), { signal: controller.signal });
        await new Promise((resolve) => setTimeout(resolve, 10));
        controller.abort(reason);
        reader.release();
        await assert.rejects(adding, (error: unknown) => error === reason);
        await new Promise((resolve) => setTimeout(resolve, 10));
        assert.equal(group.membersSync().length, 2);
        assert.equal(await group.name({ signal: new AbortController().signal }), 'Flat');
    });

    /** A destroyed group that still answered would read memory the library freed. */
    void it('answers no more once destroyed', () => {
        const group = ExpenseGroup.create('Trip', CurrencyCode.of('EUR'));
        group.destroy();
        group.destroy();
        assert.throws(() => group.nameSync());
        const disposed = ExpenseGroup.create('Trip', CurrencyCode.of('EUR'));
        disposed[Symbol.dispose]();
        assert.throws(() => disposed.nameSync());
    });
});
