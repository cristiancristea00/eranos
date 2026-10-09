/**
 * The write handle of a group.
 *
 * @packageDocumentation
 */

import { Language } from './contact.js';
import { calling } from './converting.js';
import { CurrencyCode } from './currency_code.js';
import {
    type Category,
    type Expense,
    type ExpenseInput,
    type Member,
    type MemberInput,
    type Transfer,
    completedExpense,
    completedMember,
    convertedCategory,
    convertedExpense,
    convertedMember,
    convertedTransfer,
    generatedCategory,
    generatedExpense,
    generatedMember,
    generatedTransfer,
} from './entries.js';
import {
    type CategoryId,
    type ExpenseId,
    type ExpenseGroupId,
    type MemberId,
    type TransferId,
    categoryId,
    expenseId,
    expenseGroupId,
    memberId,
    transferId,
} from './identifiers.js';
import type * as internal from './internal/eranos.js';
import { Money } from './money.js';
import { type CurrencyRate, generatedRates } from './ratios.js';
import {
    type ExpenseShares,
    type SettlementPlan,
    type SettlementStrategy,
    type Summary,
    convertedExpenseShares,
    convertedSettlementPlan,
    convertedSummary,
    generatedStrategy,
} from './totals.js';

/**
 * A group's reads and changes under one exclusive lock, so that no other call runs in between: every other call waits until
 * the handle is released.
 *
 * A handle holds the lock until {@link ExpenseGroupWriter.release}, which a `using` declaration calls at the end of its scope;
 * the calls never wait, since the handle holds the lock they need, and a synchronous call of the group while a handle is open
 * fails as busy, since the page's one thread cannot wait for the handle.
 */
export class ExpenseGroupWriter implements Disposable {
    private constructor(
        /** The generated write handle this one forwards to. */
        private readonly handle: internal.ExpenseGroupWriterLike,
    ) {}

    /**
     * The write handle of `handle`.
     *
     * @internal
     */
    static wrapping(handle: internal.ExpenseGroupWriterLike): ExpenseGroupWriter {
        return new ExpenseGroupWriter(handle);
    }

    /** Releases the lock; every call after it throws a `TypeError`, and releasing a released handle does nothing. */
    release(): void {
        this.handle.release();
    }

    /** Releases the lock, as {@link ExpenseGroupWriter.release} does, at the end of a `using` declaration's scope. */
    [Symbol.dispose](): void {
        this.release();
    }

    /**
     * The group's identifier.
     *
     * @returns The identifier
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    id(): ExpenseGroupId {
        return expenseGroupId(calling(() => this.handle.id()));
    }

    /**
     * The group's name.
     *
     * @returns The text
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    name(): string {
        return calling(() => this.handle.name());
    }

    /**
     * The group currency, which every total is in.
     *
     * @returns The currency
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    currency(): CurrencyCode {
        return CurrencyCode.converted(calling(() => this.handle.currency()));
    }

    /**
     * The group's note, if it has one.
     *
     * @returns The note, if the group has one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    note(): string | undefined {
        return calling(() => this.handle.note());
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * @returns The language, if the group has one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    language(): Language | undefined {
        return known(
            calling(() => this.handle.language()),
            (language) => Language.known(language),
        );
    }

    /**
     * The group in its saved form, as JSON, which {@link ExpenseGroup.load} reads back.
     *
     * @returns The text
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    save(): string {
        return calling(() => this.handle.save());
    }

    /**
     * Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
     *
     * @returns The members
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    members(): readonly Member[] {
        return calling(() => this.handle.members()).map(convertedMember);
    }

    /**
     * The member with the identifier `id`, if the group holds them.
     *
     * @param id - The identifier
     * @returns The member, if the group holds them
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    member(id: MemberId): Member | undefined {
        return known(
            calling(() => this.handle.member(id)),
            convertedMember,
        );
    }

    /**
     * Whether the member with the identifier `id` is archived; `undefined` for a member the group does not hold.
     *
     * @param id - The identifier
     * @returns Whether the member is archived, if the group holds them
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    isArchived(id: MemberId): boolean | undefined {
        return calling(() => this.handle.isArchived(id));
    }

    /**
     * Every category: the built-in one first, then the others in identifier order, which is creation order for generated
     * identifiers.
     *
     * @returns The categories
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    categories(): readonly Category[] {
        return calling(() => this.handle.categories()).map(convertedCategory);
    }

    /**
     * The category with the identifier `id`, if the group holds it.
     *
     * @param id - The identifier
     * @returns The category, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    category(id: CategoryId): Category | undefined {
        return known(
            calling(() => this.handle.category(id)),
            convertedCategory,
        );
    }

    /**
     * Every expense, in identifier order, which is creation order for generated identifiers.
     *
     * @returns The expenses
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    expenses(): readonly Expense[] {
        return calling(() => this.handle.expenses()).map(convertedExpense);
    }

    /**
     * The expense with the identifier `id`, if the group holds it.
     *
     * @param id - The identifier
     * @returns The expense, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    expense(id: ExpenseId): Expense | undefined {
        return known(
            calling(() => this.handle.expense(id)),
            convertedExpense,
        );
    }

    /**
     * How the expense with the identifier `id` divides, in the group currency, if the group holds it.
     *
     * @param id - The identifier
     * @returns The shares, if the group holds the expense
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    expenseShares(id: ExpenseId): ExpenseShares | undefined {
        return known(
            calling(() => this.handle.expenseShares(id)),
            convertedExpenseShares,
        );
    }

    /**
     * Every transfer, in identifier order, which is creation order for generated identifiers.
     *
     * @returns The transfers
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    transfers(): readonly Transfer[] {
        return calling(() => this.handle.transfers()).map(convertedTransfer);
    }

    /**
     * The transfer with the identifier `id`, if the group holds it.
     *
     * @param id - The identifier
     * @returns The transfer, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    transfer(id: TransferId): Transfer | undefined {
        return known(
            calling(() => this.handle.transfer(id)),
            convertedTransfer,
        );
    }

    /**
     * The amount in the group currency of the transfer with the identifier `id`, if the group holds it.
     *
     * @param id - The identifier
     * @returns The amount, if the group holds the transfer
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    transferAmount(id: TransferId): Money | undefined {
        return known(
            calling(() => this.handle.transferAmount(id)),
            (amount) => Money.converted(amount),
        );
    }

    /**
     * The group's totals.
     *
     * @returns The totals
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    summary(): Summary {
        return convertedSummary(calling(() => this.handle.summary()));
    }

    /**
     * The payments that settle every balance by `strategy`.
     *
     * @param strategy - The strategy
     * @returns The payments
     * @throws {@link SettlementError} with the failure `outOfMemory` for an exact plan of more open balances than its table
     *   fits
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    settlement(strategy: SettlementStrategy): SettlementPlan {
        return convertedSettlementPlan(calling(() => this.handle.settlement(generatedStrategy(strategy))));
    }

    /**
     * Renames the group.
     *
     * @param name - The new name
     * @throws {@link ExpenseGroupError} with the failure `emptyName` for a blank name
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    setName(name: string): void {
        calling(() => {
            this.handle.setName(name);
        });
    }

    /**
     * Moves the group to `currency`, converting every entry with the rate `rates` gives its currency.
     *
     * @param currency - The new group currency
     * @param rates - The rate into the new group currency of every other currency the entries use
     * @throws {@link ExpenseGroupError} with the failure `missingExchangeRate` for a currency in use without a rate,
     *   `unexpectedExchangeRate` for a rate of the new group currency, and `duplicateExchangeRate` for a currency given two
     *   rates
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    changeCurrency(currency: CurrencyCode, rates: readonly CurrencyRate[]): void {
        calling(() => {
            this.handle.changeCurrency(currency.code, generatedRates(rates));
        });
    }

    /**
     * Sets the group's note, trimmed; blank text clears it, as {@link ExpenseGroupWriter.clearNote} does.
     *
     * @param note - The new note
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    setNote(note: string): void {
        calling(() => {
            this.handle.setNote(note);
        });
    }

    /**
     * Clears the group's note, which {@link ExpenseGroupWriter.setNote} sets.
     *
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    clearNote(): void {
        calling(() => {
            this.handle.setNote(undefined);
        });
    }

    /**
     * Sets the language the apps show the group in.
     *
     * @param language - The new language
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    setLanguage(language: Language): void {
        calling(() => {
            this.handle.setLanguage(language.toString());
        });
    }

    /**
     * Clears the language the apps show the group in.
     *
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    clearLanguage(): void {
        calling(() => {
            this.handle.setLanguage(undefined);
        });
    }

    /**
     * Adds `member` under their identifier, with no accounts unless `member` gives some, and returns the identifier.
     *
     * @param member - The member
     * @returns The member's identifier
     * @throws {@link MemberError} with the failure `emptyFirstName` for a blank first name
     * @throws {@link PaymentAccountError} with the failure `emptyName` for an account with a blank name
     * @throws {@link ExpenseGroupError} with the failure `duplicateMember` for an identifier another member has
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    addMember(member: MemberInput): MemberId {
        return memberId(calling(() => this.handle.addMember(generatedMember(completedMember(member)))));
    }

    /**
     * Replaces the details of the member that `member` names by their identifier, and returns them as they were; an
     * archived member stays archived. Accounts that `member` leaves out are none, not those the group holds.
     *
     * @param member - The member
     * @returns The member as they were
     * @throws {@link MemberError} as adding one does
     * @throws {@link PaymentAccountError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    replaceMember(member: MemberInput): Member {
        return convertedMember(calling(() => this.handle.replaceMember(generatedMember(completedMember(member)))));
    }

    /**
     * Archives the member with the identifier `id`: they keep their history and balance, and join no new expense.
     *
     * @param id - The identifier
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    archiveMember(id: MemberId): void {
        calling(() => {
            this.handle.archiveMember(id);
        });
    }

    /**
     * Makes the archived member with the identifier `id` active again.
     *
     * @param id - The identifier
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    unarchiveMember(id: MemberId): void {
        calling(() => {
            this.handle.unarchiveMember(id);
        });
    }

    /**
     * Removes the member with the identifier `id`, who appears in no entry, and returns them.
     *
     * @param id - The identifier
     * @returns The member as they were
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`, and `memberInUse` with the entries that still name them;
     *   archive such a member instead
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    removeMember(id: MemberId): Member {
        return convertedMember(calling(() => this.handle.removeMember(id)));
    }

    /**
     * Adds `category` under its identifier and returns the identifier.
     *
     * @param category - The category
     * @returns The category's identifier
     * @throws {@link CategoryError} with the failure `emptyName` for a blank name
     * @throws {@link ExpenseGroupError} with the failure `duplicateCategory` for an identifier another category has, and
     *   `duplicateCategoryName` for a name another category has, compared caselessly
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    addCategory(category: Category): CategoryId {
        return categoryId(calling(() => this.handle.addCategory(generatedCategory(category))));
    }

    /**
     * Renames the category that `category` names by its identifier, and returns it as it was.
     *
     * @param category - The category
     * @returns The category as it was
     * @throws {@link CategoryError} with the failure `emptyName` for a blank name
     * @throws {@link ExpenseGroupError} with the failure `unknownCategory`, `builtInCategory`, or `duplicateCategoryName`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    replaceCategory(category: Category): Category {
        return convertedCategory(calling(() => this.handle.replaceCategory(generatedCategory(category))));
    }

    /**
     * Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
     *
     * @param id - The identifier
     * @returns The category as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownCategory` or `builtInCategory`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    removeCategory(id: CategoryId): Category {
        return convertedCategory(calling(() => this.handle.removeCategory(id)));
    }

    /**
     * Adds `expense` under its identifier, every field it leaves out at its default, and returns the identifier.
     *
     * @param expense - The expense
     * @returns The expense's identifier
     * @throws {@link ExpenseError} for an expense that does not build, such as one whose amounts do not add up
     * @throws {@link ExpenseGroupError} for one that does not fit the group, such as the failures `duplicateExpense` for an
     *   identifier another expense has, `unknownMember`, and `archivedMember`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    addExpense(expense: ExpenseInput): ExpenseId {
        return expenseId(calling(() => this.handle.addExpense(generatedExpense(completedExpense(expense)))));
    }

    /**
     * Replaces the expense that `expense` names by its identifier, and returns it as it was. A field `expense` leaves out
     * takes its default, as in a new expense, not the value the group holds.
     *
     * @param expense - The expense
     * @returns The expense as it was
     * @throws {@link ExpenseError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownExpense`, and as adding one does
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    replaceExpense(expense: ExpenseInput): Expense {
        return convertedExpense(calling(() => this.handle.replaceExpense(generatedExpense(completedExpense(expense)))));
    }

    /**
     * Removes the expense with the identifier `id` and returns it.
     *
     * @param id - The identifier
     * @returns The expense as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownExpense`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    removeExpense(id: ExpenseId): Expense {
        return convertedExpense(calling(() => this.handle.removeExpense(id)));
    }

    /**
     * Adds `transfer` under its identifier and returns the identifier.
     *
     * @param transfer - The transfer
     * @returns The transfer's identifier
     * @throws {@link TransferError} for a transfer that does not build, such as the failure `sameMember`
     * @throws {@link ExpenseGroupError} with the failure `duplicateTransfer` for an identifier another transfer has, and
     *   `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    addTransfer(transfer: Transfer): TransferId {
        return transferId(calling(() => this.handle.addTransfer(generatedTransfer(transfer))));
    }

    /**
     * Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
     *
     * @param transfer - The transfer
     * @returns The transfer as it was
     * @throws {@link TransferError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownTransfer`, and as adding one does
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    replaceTransfer(transfer: Transfer): Transfer {
        return convertedTransfer(calling(() => this.handle.replaceTransfer(generatedTransfer(transfer))));
    }

    /**
     * Removes the transfer with the identifier `id` and returns it.
     *
     * @param id - The identifier
     * @returns The transfer as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownTransfer`
     * @throws {@link IdError} for an identifier that is not one
     * @throws `TypeError` after {@link ExpenseGroupWriter.release}
     */
    removeTransfer(id: TransferId): Transfer {
        return convertedTransfer(calling(() => this.handle.removeTransfer(id)));
    }
}

/** `value` converted by `conversion`, or `undefined` for a value the group does not hold. */
function known<T, U>(value: T | undefined, conversion: (value: T) => U): U | undefined {
    return undefined === value ? undefined : conversion(value);
}
