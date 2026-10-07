/**
 * The group: its details, members, categories, expenses, transfers, totals, and settlements.
 *
 * @packageDocumentation
 */

import { Language } from './contact.js';
import { type CallOptions, awaiting, calling } from './converting.js';
import { CurrencyCode } from './currency_code.js';
import {
    type Category,
    type Expense,
    type Member,
    type Transfer,
    convertedCategory,
    convertedExpense,
    convertedMember,
    convertedTransfer,
    generatedCategory,
    generatedExpense,
    generatedMember,
    generatedTransfer,
} from './entries.js';
import { ExpenseGroupReader } from './expense_group_reader.js';
import {
    type CategoryId,
    type ExpenseId,
    type ExpenseGroupId,
    type MemberId,
    type TransferId,
    categoryId,
    expenseId,
    groupId,
    memberId,
    transferId,
} from './identifiers.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';
import type { ExchangeRate } from './ratios.js';
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

/** The details a new group can start with besides its name and currency. */
export interface ExpenseGroupOptions {
    /** A longer description. */
    readonly description?: string;
    /** The language the apps show the group in. */
    readonly language?: Language;
}

/** The rate into a new group currency of an amount in `currency`. */
export interface CurrencyRate {
    /** The currency the rate converts from. */
    readonly currency: CurrencyCode;
    /** The rate. */
    readonly rate: ExchangeRate;
}

/**
 * A group of people who share expenses.
 *
 * Every call comes in two forms. The one named for the call returns a Promise and waits for the group's lock without
 * blocking, and aborting its signal cancels it, leaving the group as it was; its twin, whose name ends in `Sync`, returns
 * at once, and throws {@link ExpenseGroupError} with the failure `busy` while another call holds the lock, since a wait on the
 * page's one thread would never end:
 *
 * ```ts
 * const group = ExpenseGroup.create('Flat', CurrencyCode.of('EUR'));
 * const ana = group.addMemberSync({ firstName: 'Ana' });
 * const bob = await group.addMember({ firstName: 'Bob' });
 * ```
 *
 * Reads share the lock and run together; a change takes it alone, after the reads in progress. {@link ExpenseGroup.destroy}
 * frees the group, as a `using` declaration does; every call after it throws.
 */
export class ExpenseGroup implements Disposable {
    private constructor(
        /** The generated group this one forwards to. */
        private readonly handle: internal.ExpenseGroup,
    ) {}

    /**
     * A new group with no members and no entries, called `name`, whose totals are in `currency`.
     *
     * @param name - The name
     * @param currency - The group currency
     * @param options - The description and the language, if the group has them
     * @returns The group
     * @throws {@link ExpenseGroupError} with the failure `emptyName` for a blank name
     */
    static create(name: string, currency: CurrencyCode, options?: ExpenseGroupOptions): ExpenseGroup {
        const group = internal.NewExpenseGroup.create({
            name,
            currency: currency.code,
            ...(undefined === options?.description ? {} : { description: options.description }),
            ...(undefined === options?.language ? {} : { language: options.language.toString() }),
        });
        return new ExpenseGroup(calling(() => new internal.ExpenseGroup(group)));
    }

    /**
     * The group saved as `json`, loaded through the checks that adding its entries makes.
     *
     * Yields while it replays the entries; aborting `options.signal` cancels the load and rejects with the signal's reason.
     * Rejects as {@link ExpenseGroup.loadSync} throws.
     *
     * @param json - The saved group
     * @param options - The load's signal
     * @returns A promise of the group
     */
    static async load(json: string, options?: CallOptions): Promise<ExpenseGroup> {
        // The generated load names its object by the object's interface; the object is always the class.
        return new ExpenseGroup(
            (await awaiting((signal) => internal.loadExpenseGroupAsync(json, undefined, signal), options)) as internal.ExpenseGroup,
        );
    }

    /**
     * The group saved as `json`, loaded through the checks that adding its entries makes.
     *
     * @param json - The saved group
     * @returns The group
     * @throws {@link InvalidJsonError} for text that is not the saved form
     * @throws {@link ExpenseGroupError} with the failure `unsupportedFormat` for a format this release cannot read, and the
     *   failures of entries that do not fit together
     */
    static loadSync(json: string): ExpenseGroup {
        return new ExpenseGroup(calling(() => internal.ExpenseGroup.load(json)) as internal.ExpenseGroup);
    }

    /** Frees the group; every call after it throws, and freeing a freed group does nothing. */
    destroy(): void {
        this.handle.uniffiDestroy();
    }

    /** Frees the group, as {@link ExpenseGroup.destroy} does, at the end of a `using` declaration's scope. */
    [Symbol.dispose](): void {
        this.destroy();
    }

    /**
     * A read handle, which holds the shared lock until it is released.
     *
     * Waits for the lock without blocking; aborting `options.signal` cancels the wait.
     *
     * @param options - The call's signal
     * @returns A promise of the read handle
     */
    async read(options?: CallOptions): Promise<ExpenseGroupReader> {
        return ExpenseGroupReader.wrapping(await awaiting((signal) => this.handle.readAsync(undefined, signal), options));
    }

    /**
     * A read handle, which holds the shared lock until it is released.
     *
     * Does not wait for the lock: while a change holds it, it throws {@link ExpenseGroupError} with the failure `busy`.
     *
     * @returns The read handle
     * @throws {@link ExpenseGroupError} with the failure `busy` while a change holds the lock
     */
    readSync(): ExpenseGroupReader {
        return ExpenseGroupReader.wrapping(calling(() => this.handle.read()));
    }

    // The group's own details: its identifier, name, currency, description, language, and saved form.

    /**
     * The group's identifier.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.idSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the identifier
     */
    async id(options?: CallOptions): Promise<ExpenseGroupId> {
        return groupId(await awaiting((signal) => this.handle.idAsync(undefined, signal), options));
    }

    /**
     * The group's identifier.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.id} waits for it.
     *
     * @returns The identifier
     */
    idSync(): ExpenseGroupId {
        return groupId(calling(() => this.handle.id()));
    }

    /**
     * The group's name.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.nameSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the text
     */
    async name(options?: CallOptions): Promise<string> {
        return await awaiting((signal) => this.handle.nameAsync(undefined, signal), options);
    }

    /**
     * The group's name.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.name} waits for it.
     *
     * @returns The text
     */
    nameSync(): string {
        return calling(() => this.handle.name());
    }

    /**
     * Renames the group.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.setNameSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `emptyName` for a blank name.
     *
     * @param name - The new name
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async setName(name: string, options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.setNameAsync(name, undefined, signal), options);
    }

    /**
     * Renames the group.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.setName} waits for it.
     *
     * @param name - The new name
     * @throws {@link ExpenseGroupError} with the failure `emptyName` for a blank name
     */
    setNameSync(name: string): void {
        calling(() => {
            this.handle.setName(name);
        });
    }

    /**
     * The group currency, which every total is in.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.currencySync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the currency
     */
    async currency(options?: CallOptions): Promise<CurrencyCode> {
        return CurrencyCode.known(await awaiting((signal) => this.handle.currencyAsync(undefined, signal), options));
    }

    /**
     * The group currency, which every total is in.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.currency} waits for it.
     *
     * @returns The currency
     */
    currencySync(): CurrencyCode {
        return CurrencyCode.known(calling(() => this.handle.currency()));
    }

    /**
     * Moves the group to `currency`, converting every entry with the rate `rates` gives its currency.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.changeCurrencySync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `missingExchangeRate` for a currency in use without a rate,
     * `unexpectedExchangeRate` for a rate of the new group currency, and `duplicateExchangeRate` for a currency given two
     * rates.
     *
     * @param currency - The new group currency
     * @param rates - The rate into the new group currency of every other currency the entries use
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async changeCurrency(currency: CurrencyCode, rates: readonly CurrencyRate[], options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.changeCurrencyAsync(currency.code, generatedRates(rates), undefined, signal), options);
    }

    /**
     * Moves the group to `currency`, converting every entry with the rate `rates` gives its currency.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.changeCurrency} waits for it.
     *
     * @param currency - The new group currency
     * @param rates - The rate into the new group currency of every other currency the entries use
     * @throws {@link ExpenseGroupError} with the failure `missingExchangeRate` for a currency in use without a rate,
     *   `unexpectedExchangeRate` for a rate of the new group currency, and `duplicateExchangeRate` for a currency given two
     *   rates
     */
    changeCurrencySync(currency: CurrencyCode, rates: readonly CurrencyRate[]): void {
        calling(() => {
            this.handle.changeCurrency(currency.code, generatedRates(rates));
        });
    }

    /**
     * The group's description, if it has one.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.descriptionSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the description, if the group has one
     */
    async description(options?: CallOptions): Promise<string | undefined> {
        return await awaiting((signal) => this.handle.descriptionAsync(undefined, signal), options);
    }

    /**
     * The group's description, if it has one.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.description} waits for it.
     *
     * @returns The description, if the group has one
     */
    descriptionSync(): string | undefined {
        return calling(() => this.handle.description());
    }

    /**
     * Sets the group's description; blank text clears it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.setDescriptionSync} does not wait.
     *
     * @param description - The new description
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async setDescription(description: string, options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.setDescriptionAsync(description, undefined, signal), options);
    }

    /**
     * Sets the group's description; blank text clears it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.setDescription} waits for it.
     *
     * @param description - The new description
     */
    setDescriptionSync(description: string): void {
        calling(() => {
            this.handle.setDescription(description);
        });
    }

    /**
     * Clears the group's description.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.clearDescriptionSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async clearDescription(options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.setDescriptionAsync('', undefined, signal), options);
    }

    /**
     * Clears the group's description.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.clearDescription} waits for it.
     */
    clearDescriptionSync(): void {
        calling(() => {
            this.handle.setDescription('');
        });
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.languageSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the language, if the group has one
     */
    async language(options?: CallOptions): Promise<Language | undefined> {
        return known(await awaiting((signal) => this.handle.languageAsync(undefined, signal), options), (language) => Language.known(language));
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.language} waits for it.
     *
     * @returns The language, if the group has one
     */
    languageSync(): Language | undefined {
        return known(
            calling(() => this.handle.language()),
            (language) => Language.known(language),
        );
    }

    /**
     * Sets the language the apps show the group in.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.setLanguageSync} does not wait.
     *
     * @param language - The new language
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async setLanguage(language: Language, options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.setLanguageAsync(language.toString(), undefined, signal), options);
    }

    /**
     * Sets the language the apps show the group in.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.setLanguage} waits for it.
     *
     * @param language - The new language
     */
    setLanguageSync(language: Language): void {
        calling(() => {
            this.handle.setLanguage(language.toString());
        });
    }

    /**
     * Clears the language the apps show the group in.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.clearLanguageSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async clearLanguage(options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.setLanguageAsync(undefined, undefined, signal), options);
    }

    /**
     * Clears the language the apps show the group in.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.clearLanguage} waits for it.
     */
    clearLanguageSync(): void {
        calling(() => {
            this.handle.setLanguage(undefined);
        });
    }

    /**
     * The group in its saved form, as JSON, which {@link ExpenseGroup.load} reads back.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.saveSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the text
     */
    async save(options?: CallOptions): Promise<string> {
        return await awaiting((signal) => this.handle.saveAsync(undefined, signal), options);
    }

    /**
     * The group in its saved form, as JSON, which {@link ExpenseGroup.load} reads back.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.save} waits for it.
     *
     * @returns The text
     */
    saveSync(): string {
        return calling(() => this.handle.save());
    }

    // The group's members.

    /**
     * Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.membersSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the members
     */
    async members(options?: CallOptions): Promise<readonly Member[]> {
        return (await awaiting((signal) => this.handle.membersAsync(undefined, signal), options)).map(convertedMember);
    }

    /**
     * Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.members} waits for it.
     *
     * @returns The members
     */
    membersSync(): readonly Member[] {
        return calling(() => this.handle.members()).map(convertedMember);
    }

    /**
     * The member with the identifier `id`, if the group holds them.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.memberSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the member, if the group holds them
     */
    async member(id: MemberId, options?: CallOptions): Promise<Member | undefined> {
        return known(await awaiting((signal) => this.handle.memberAsync(id, undefined, signal), options), convertedMember);
    }

    /**
     * The member with the identifier `id`, if the group holds them.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.member} waits for it.
     *
     * @param id - The identifier
     * @returns The member, if the group holds them
     * @throws {@link IdError} for an identifier that is not one
     */
    memberSync(id: MemberId): Member | undefined {
        return known(
            calling(() => this.handle.member(id)),
            convertedMember,
        );
    }

    /**
     * Whether the member with the identifier `id` is archived; `false` for a member the group does not hold.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.isArchivedSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of whether the member is archived
     */
    async isArchived(id: MemberId, options?: CallOptions): Promise<boolean> {
        return await awaiting((signal) => this.handle.isArchivedAsync(id, undefined, signal), options);
    }

    /**
     * Whether the member with the identifier `id` is archived; `false` for a member the group does not hold.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.isArchived} waits for it.
     *
     * @param id - The identifier
     * @returns Whether the member is archived
     * @throws {@link IdError} for an identifier that is not one
     */
    isArchivedSync(id: MemberId): boolean {
        return calling(() => this.handle.isArchived(id));
    }

    /**
     * Adds `member`, who has no identifier yet, and returns the one the group gives them.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.addMemberSync} does not wait.
     *
     * Rejects with `TypeError` for a member with an identifier; {@link MemberError} with the failure `emptyFirstName` for a
     * blank first name; {@link PaymentAccountError} with the failure `emptyName` for an account with a blank name.
     *
     * @param member - The member
     * @param options - The call's signal
     * @returns A promise of the member's identifier
     */
    async addMember(member: Member, options?: CallOptions): Promise<MemberId> {
        return memberId(await awaiting((signal) => this.handle.addMemberAsync(generatedMember(member), undefined, signal), options));
    }

    /**
     * Adds `member`, who has no identifier yet, and returns the one the group gives them.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.addMember} waits for it.
     *
     * @param member - The member
     * @returns The member's identifier
     * @throws `TypeError` for a member with an identifier
     * @throws {@link MemberError} with the failure `emptyFirstName` for a blank first name
     * @throws {@link PaymentAccountError} with the failure `emptyName` for an account with a blank name
     */
    addMemberSync(member: Member): MemberId {
        return memberId(calling(() => this.handle.addMember(generatedMember(member))));
    }

    /**
     * Replaces the details of the member that `member` names by their identifier, and returns them as they were; an
     * archived member stays archived.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.replaceMemberSync} does not wait.
     *
     * Rejects with `TypeError` for a member without an identifier; {@link MemberError} as adding one does; {@link
     * PaymentAccountError} as adding one does; {@link ExpenseGroupError} with the failure `unknownMember`; {@link IdError} for an
     * identifier that is not one.
     *
     * @param member - The member
     * @param options - The call's signal
     * @returns A promise of the member as they were
     */
    async replaceMember(member: Member, options?: CallOptions): Promise<Member> {
        return convertedMember(await awaiting((signal) => this.handle.replaceMemberAsync(generatedMember(member), undefined, signal), options));
    }

    /**
     * Replaces the details of the member that `member` names by their identifier, and returns them as they were; an
     * archived member stays archived.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.replaceMember} waits for it.
     *
     * @param member - The member
     * @returns The member as they were
     * @throws `TypeError` for a member without an identifier
     * @throws {@link MemberError} as adding one does
     * @throws {@link PaymentAccountError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     */
    replaceMemberSync(member: Member): Member {
        return convertedMember(calling(() => this.handle.replaceMember(generatedMember(member))));
    }

    /**
     * Archives the member with the identifier `id`: they keep their history and balance, and join no new expense.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.archiveMemberSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownMember`; {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async archiveMember(id: MemberId, options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.archiveMemberAsync(id, undefined, signal), options);
    }

    /**
     * Archives the member with the identifier `id`: they keep their history and balance, and join no new expense.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.archiveMember} waits for it.
     *
     * @param id - The identifier
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     */
    archiveMemberSync(id: MemberId): void {
        calling(() => {
            this.handle.archiveMember(id);
        });
    }

    /**
     * Makes the archived member with the identifier `id` active again.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.unarchiveMemberSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownMember`; {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the call's end
     */
    async unarchiveMember(id: MemberId, options?: CallOptions): Promise<void> {
        await awaiting((signal) => this.handle.unarchiveMemberAsync(id, undefined, signal), options);
    }

    /**
     * Makes the archived member with the identifier `id` active again.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.unarchiveMember} waits for it.
     *
     * @param id - The identifier
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     */
    unarchiveMemberSync(id: MemberId): void {
        calling(() => {
            this.handle.unarchiveMember(id);
        });
    }

    /**
     * Removes the member with the identifier `id`, who appears in no entry, and returns them.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.removeMemberSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownMember`, and `memberInUse` with the entries that still name
     * them; archive such a member instead; {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the member as they were
     */
    async removeMember(id: MemberId, options?: CallOptions): Promise<Member> {
        return convertedMember(await awaiting((signal) => this.handle.removeMemberAsync(id, undefined, signal), options));
    }

    /**
     * Removes the member with the identifier `id`, who appears in no entry, and returns them.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.removeMember} waits for it.
     *
     * @param id - The identifier
     * @returns The member as they were
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`, and `memberInUse` with the entries that still name them;
     *   archive such a member instead
     * @throws {@link IdError} for an identifier that is not one
     */
    removeMemberSync(id: MemberId): Member {
        return convertedMember(calling(() => this.handle.removeMember(id)));
    }

    // The group's categories.

    /**
     * Every category: the built-in one first, then the others in identifier order, which is creation order for generated
     * identifiers.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.categoriesSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the categories
     */
    async categories(options?: CallOptions): Promise<readonly Category[]> {
        return (await awaiting((signal) => this.handle.categoriesAsync(undefined, signal), options)).map(convertedCategory);
    }

    /**
     * Every category: the built-in one first, then the others in identifier order, which is creation order for generated
     * identifiers.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.categories} waits for it.
     *
     * @returns The categories
     */
    categoriesSync(): readonly Category[] {
        return calling(() => this.handle.categories()).map(convertedCategory);
    }

    /**
     * The category with the identifier `id`, if the group holds it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.categorySync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the category, if the group holds it
     */
    async category(id: CategoryId, options?: CallOptions): Promise<Category | undefined> {
        return known(await awaiting((signal) => this.handle.categoryAsync(id, undefined, signal), options), convertedCategory);
    }

    /**
     * The category with the identifier `id`, if the group holds it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.category} waits for it.
     *
     * @param id - The identifier
     * @returns The category, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     */
    categorySync(id: CategoryId): Category | undefined {
        return known(
            calling(() => this.handle.category(id)),
            convertedCategory,
        );
    }

    /**
     * Adds `category`, which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.addCategorySync} does not wait.
     *
     * Rejects with `TypeError` for a category with an identifier; {@link CategoryError} with the failure `emptyName` for a
     * blank name; {@link ExpenseGroupError} with the failure `duplicateCategoryName` for a name another category has, compared
     * caselessly.
     *
     * @param category - The category
     * @param options - The call's signal
     * @returns A promise of the category's identifier
     */
    async addCategory(category: Category, options?: CallOptions): Promise<CategoryId> {
        return categoryId(await awaiting((signal) => this.handle.addCategoryAsync(generatedCategory(category), undefined, signal), options));
    }

    /**
     * Adds `category`, which has no identifier yet, and returns the one the group gives it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.addCategory} waits for it.
     *
     * @param category - The category
     * @returns The category's identifier
     * @throws `TypeError` for a category with an identifier
     * @throws {@link CategoryError} with the failure `emptyName` for a blank name
     * @throws {@link ExpenseGroupError} with the failure `duplicateCategoryName` for a name another category has, compared
     *   caselessly
     */
    addCategorySync(category: Category): CategoryId {
        return categoryId(calling(() => this.handle.addCategory(generatedCategory(category))));
    }

    /**
     * Renames the category that `category` names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.replaceCategorySync} does not wait.
     *
     * Rejects with `TypeError` for a category without an identifier; {@link CategoryError} with the failure `emptyName` for
     * a blank name; {@link ExpenseGroupError} with the failure `unknownCategory`, `builtInCategory`, or `duplicateCategoryName`;
     * {@link IdError} for an identifier that is not one.
     *
     * @param category - The category
     * @param options - The call's signal
     * @returns A promise of the category as it was
     */
    async replaceCategory(category: Category, options?: CallOptions): Promise<Category> {
        return convertedCategory(
            await awaiting((signal) => this.handle.replaceCategoryAsync(generatedCategory(category), undefined, signal), options),
        );
    }

    /**
     * Renames the category that `category` names by its identifier, and returns it as it was.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.replaceCategory} waits for it.
     *
     * @param category - The category
     * @returns The category as it was
     * @throws `TypeError` for a category without an identifier
     * @throws {@link CategoryError} with the failure `emptyName` for a blank name
     * @throws {@link ExpenseGroupError} with the failure `unknownCategory`, `builtInCategory`, or `duplicateCategoryName`
     * @throws {@link IdError} for an identifier that is not one
     */
    replaceCategorySync(category: Category): Category {
        return convertedCategory(calling(() => this.handle.replaceCategory(generatedCategory(category))));
    }

    /**
     * Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.removeCategorySync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownCategory` or `builtInCategory`; {@link IdError} for an
     * identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the category as it was
     */
    async removeCategory(id: CategoryId, options?: CallOptions): Promise<Category> {
        return convertedCategory(await awaiting((signal) => this.handle.removeCategoryAsync(id, undefined, signal), options));
    }

    /**
     * Removes the category with the identifier `id` and returns it; its expenses move to the built-in one.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.removeCategory} waits for it.
     *
     * @param id - The identifier
     * @returns The category as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownCategory` or `builtInCategory`
     * @throws {@link IdError} for an identifier that is not one
     */
    removeCategorySync(id: CategoryId): Category {
        return convertedCategory(calling(() => this.handle.removeCategory(id)));
    }

    // The group's expenses.

    /**
     * Every expense, in identifier order, which is creation order for generated identifiers.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.expensesSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the expenses
     */
    async expenses(options?: CallOptions): Promise<readonly Expense[]> {
        return (await awaiting((signal) => this.handle.expensesAsync(undefined, signal), options)).map(convertedExpense);
    }

    /**
     * Every expense, in identifier order, which is creation order for generated identifiers.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.expenses} waits for it.
     *
     * @returns The expenses
     */
    expensesSync(): readonly Expense[] {
        return calling(() => this.handle.expenses()).map(convertedExpense);
    }

    /**
     * The expense with the identifier `id`, if the group holds it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.expenseSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the expense, if the group holds it
     */
    async expense(id: ExpenseId, options?: CallOptions): Promise<Expense | undefined> {
        return known(await awaiting((signal) => this.handle.expenseAsync(id, undefined, signal), options), convertedExpense);
    }

    /**
     * The expense with the identifier `id`, if the group holds it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.expense} waits for it.
     *
     * @param id - The identifier
     * @returns The expense, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     */
    expenseSync(id: ExpenseId): Expense | undefined {
        return known(
            calling(() => this.handle.expense(id)),
            convertedExpense,
        );
    }

    /**
     * How the expense with the identifier `id` divides, in the group currency, if the group holds it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.expenseSharesSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the shares, if the group holds the expense
     */
    async expenseShares(id: ExpenseId, options?: CallOptions): Promise<ExpenseShares | undefined> {
        return known(await awaiting((signal) => this.handle.expenseSharesAsync(id, undefined, signal), options), convertedExpenseShares);
    }

    /**
     * How the expense with the identifier `id` divides, in the group currency, if the group holds it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.expenseShares} waits for it.
     *
     * @param id - The identifier
     * @returns The shares, if the group holds the expense
     * @throws {@link IdError} for an identifier that is not one
     */
    expenseSharesSync(id: ExpenseId): ExpenseShares | undefined {
        return known(
            calling(() => this.handle.expenseShares(id)),
            convertedExpenseShares,
        );
    }

    /**
     * Adds `expense`, which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.addExpenseSync} does not wait.
     *
     * Rejects with `TypeError` for an expense with an identifier; {@link ExpenseError} for an expense that does not build,
     * such as one whose amounts do not add up; {@link ExpenseGroupError} for one that does not fit the group, such as the failures
     * `unknownMember` and `archivedMember`; {@link IdError} for an identifier that is not one.
     *
     * @param expense - The expense
     * @param options - The call's signal
     * @returns A promise of the expense's identifier
     */
    async addExpense(expense: Expense, options?: CallOptions): Promise<ExpenseId> {
        return expenseId(await awaiting((signal) => this.handle.addExpenseAsync(generatedExpense(expense), undefined, signal), options));
    }

    /**
     * Adds `expense`, which has no identifier yet, and returns the one the group gives it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.addExpense} waits for it.
     *
     * @param expense - The expense
     * @returns The expense's identifier
     * @throws `TypeError` for an expense with an identifier
     * @throws {@link ExpenseError} for an expense that does not build, such as one whose amounts do not add up
     * @throws {@link ExpenseGroupError} for one that does not fit the group, such as the failures `unknownMember` and
     *   `archivedMember`
     * @throws {@link IdError} for an identifier that is not one
     */
    addExpenseSync(expense: Expense): ExpenseId {
        return expenseId(calling(() => this.handle.addExpense(generatedExpense(expense))));
    }

    /**
     * Replaces the expense that `expense` names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.replaceExpenseSync} does not wait.
     *
     * Rejects with `TypeError` for an expense without an identifier; {@link ExpenseError} as adding one does; {@link
     * ExpenseGroupError} with the failure `unknownExpense`, and as adding one does; {@link IdError} for an identifier that is not
     * one.
     *
     * @param expense - The expense
     * @param options - The call's signal
     * @returns A promise of the expense as it was
     */
    async replaceExpense(expense: Expense, options?: CallOptions): Promise<Expense> {
        return convertedExpense(await awaiting((signal) => this.handle.replaceExpenseAsync(generatedExpense(expense), undefined, signal), options));
    }

    /**
     * Replaces the expense that `expense` names by its identifier, and returns it as it was.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.replaceExpense} waits for it.
     *
     * @param expense - The expense
     * @returns The expense as it was
     * @throws `TypeError` for an expense without an identifier
     * @throws {@link ExpenseError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownExpense`, and as adding one does
     * @throws {@link IdError} for an identifier that is not one
     */
    replaceExpenseSync(expense: Expense): Expense {
        return convertedExpense(calling(() => this.handle.replaceExpense(generatedExpense(expense))));
    }

    /**
     * Removes the expense with the identifier `id` and returns it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.removeExpenseSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownExpense`; {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the expense as it was
     */
    async removeExpense(id: ExpenseId, options?: CallOptions): Promise<Expense> {
        return convertedExpense(await awaiting((signal) => this.handle.removeExpenseAsync(id, undefined, signal), options));
    }

    /**
     * Removes the expense with the identifier `id` and returns it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.removeExpense} waits for it.
     *
     * @param id - The identifier
     * @returns The expense as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownExpense`
     * @throws {@link IdError} for an identifier that is not one
     */
    removeExpenseSync(id: ExpenseId): Expense {
        return convertedExpense(calling(() => this.handle.removeExpense(id)));
    }

    // The group's transfers.

    /**
     * Every transfer, in identifier order, which is creation order for generated identifiers.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.transfersSync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the transfers
     */
    async transfers(options?: CallOptions): Promise<readonly Transfer[]> {
        return (await awaiting((signal) => this.handle.transfersAsync(undefined, signal), options)).map(convertedTransfer);
    }

    /**
     * Every transfer, in identifier order, which is creation order for generated identifiers.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.transfers} waits for it.
     *
     * @returns The transfers
     */
    transfersSync(): readonly Transfer[] {
        return calling(() => this.handle.transfers()).map(convertedTransfer);
    }

    /**
     * The transfer with the identifier `id`, if the group holds it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.transferSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the transfer, if the group holds it
     */
    async transfer(id: TransferId, options?: CallOptions): Promise<Transfer | undefined> {
        return known(await awaiting((signal) => this.handle.transferAsync(id, undefined, signal), options), convertedTransfer);
    }

    /**
     * The transfer with the identifier `id`, if the group holds it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.transfer} waits for it.
     *
     * @param id - The identifier
     * @returns The transfer, if the group holds it
     * @throws {@link IdError} for an identifier that is not one
     */
    transferSync(id: TransferId): Transfer | undefined {
        return known(
            calling(() => this.handle.transfer(id)),
            convertedTransfer,
        );
    }

    /**
     * The amount in the group currency of the transfer with the identifier `id`, if the group holds it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.transferAmountSync} does not wait.
     *
     * Rejects with {@link IdError} for an identifier that is not one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the amount, if the group holds the transfer
     */
    async transferAmount(id: TransferId, options?: CallOptions): Promise<Money | undefined> {
        return known(await awaiting((signal) => this.handle.transferAmountAsync(id, undefined, signal), options), (amount) =>
            Money.converted(amount),
        );
    }

    /**
     * The amount in the group currency of the transfer with the identifier `id`, if the group holds it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.transferAmount} waits for it.
     *
     * @param id - The identifier
     * @returns The amount, if the group holds the transfer
     * @throws {@link IdError} for an identifier that is not one
     */
    transferAmountSync(id: TransferId): Money | undefined {
        return known(
            calling(() => this.handle.transferAmount(id)),
            (amount) => Money.converted(amount),
        );
    }

    /**
     * Adds `transfer`, which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.addTransferSync} does not wait.
     *
     * Rejects with `TypeError` for a transfer with an identifier; {@link TransferError} for a transfer that does not build,
     * such as the failure `sameMember`; {@link ExpenseGroupError} with the failure `unknownMember`; {@link IdError} for an
     * identifier that is not one.
     *
     * @param transfer - The transfer
     * @param options - The call's signal
     * @returns A promise of the transfer's identifier
     */
    async addTransfer(transfer: Transfer, options?: CallOptions): Promise<TransferId> {
        return transferId(await awaiting((signal) => this.handle.addTransferAsync(generatedTransfer(transfer), undefined, signal), options));
    }

    /**
     * Adds `transfer`, which has no identifier yet, and returns the one the group gives it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.addTransfer} waits for it.
     *
     * @param transfer - The transfer
     * @returns The transfer's identifier
     * @throws `TypeError` for a transfer with an identifier
     * @throws {@link TransferError} for a transfer that does not build, such as the failure `sameMember`
     * @throws {@link ExpenseGroupError} with the failure `unknownMember`
     * @throws {@link IdError} for an identifier that is not one
     */
    addTransferSync(transfer: Transfer): TransferId {
        return transferId(calling(() => this.handle.addTransfer(generatedTransfer(transfer))));
    }

    /**
     * Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.replaceTransferSync} does not wait.
     *
     * Rejects with `TypeError` for a transfer without an identifier; {@link TransferError} as adding one does; {@link
     * ExpenseGroupError} with the failure `unknownTransfer`, and as adding one does; {@link IdError} for an identifier that is not
     * one.
     *
     * @param transfer - The transfer
     * @param options - The call's signal
     * @returns A promise of the transfer as it was
     */
    async replaceTransfer(transfer: Transfer, options?: CallOptions): Promise<Transfer> {
        return convertedTransfer(
            await awaiting((signal) => this.handle.replaceTransferAsync(generatedTransfer(transfer), undefined, signal), options),
        );
    }

    /**
     * Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.replaceTransfer} waits for it.
     *
     * @param transfer - The transfer
     * @returns The transfer as it was
     * @throws `TypeError` for a transfer without an identifier
     * @throws {@link TransferError} as adding one does
     * @throws {@link ExpenseGroupError} with the failure `unknownTransfer`, and as adding one does
     * @throws {@link IdError} for an identifier that is not one
     */
    replaceTransferSync(transfer: Transfer): Transfer {
        return convertedTransfer(calling(() => this.handle.replaceTransfer(generatedTransfer(transfer))));
    }

    /**
     * Removes the transfer with the identifier `id` and returns it.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.removeTransferSync} does not wait.
     *
     * Rejects with {@link ExpenseGroupError} with the failure `unknownTransfer`; {@link IdError} for an identifier that is not
     * one.
     *
     * @param id - The identifier
     * @param options - The call's signal
     * @returns A promise of the transfer as it was
     */
    async removeTransfer(id: TransferId, options?: CallOptions): Promise<Transfer> {
        return convertedTransfer(await awaiting((signal) => this.handle.removeTransferAsync(id, undefined, signal), options));
    }

    /**
     * Removes the transfer with the identifier `id` and returns it.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.removeTransfer} waits for it.
     *
     * @param id - The identifier
     * @returns The transfer as it was
     * @throws {@link ExpenseGroupError} with the failure `unknownTransfer`
     * @throws {@link IdError} for an identifier that is not one
     */
    removeTransferSync(id: TransferId): Transfer {
        return convertedTransfer(calling(() => this.handle.removeTransfer(id)));
    }

    // The group's totals and its settlements.

    /**
     * The group's totals.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.summarySync} does not wait.
     *
     * @param options - The call's signal
     * @returns A promise of the totals
     */
    async summary(options?: CallOptions): Promise<Summary> {
        return convertedSummary(await awaiting((signal) => this.handle.summaryAsync(undefined, signal), options));
    }

    /**
     * The group's totals.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.summary} waits for it.
     *
     * @returns The totals
     */
    summarySync(): Summary {
        return convertedSummary(calling(() => this.handle.summary()));
    }

    /**
     * The payments that settle every balance by `strategy`.
     *
     * Waits for the group's lock without blocking; aborting `options.signal` cancels the call, which leaves the group as it
     * was, and rejects with the signal's reason. {@link ExpenseGroup.settlementSync} does not wait.
     *
     * Rejects with {@link SettlementError} with the failure `outOfMemory` for an exact plan of more open balances than its
     * table fits.
     *
     * @param strategy - The strategy
     * @param options - The call's signal
     * @returns A promise of the payments
     */
    async settlement(strategy: SettlementStrategy, options?: CallOptions): Promise<SettlementPlan> {
        return convertedSettlementPlan(
            await awaiting((signal) => this.handle.settlementAsync(generatedStrategy(strategy), undefined, signal), options),
        );
    }

    /**
     * The payments that settle every balance by `strategy`.
     *
     * Does not wait for the group's lock: while another call holds it, it throws {@link ExpenseGroupError} with the failure
     * `busy`, since a wait on the page's one thread would never end. {@link ExpenseGroup.settlement} waits for it.
     *
     * @param strategy - The strategy
     * @returns The payments
     * @throws {@link SettlementError} with the failure `outOfMemory` for an exact plan of more open balances than its table
     *   fits
     */
    settlementSync(strategy: SettlementStrategy): SettlementPlan {
        return convertedSettlementPlan(calling(() => this.handle.settlement(generatedStrategy(strategy))));
    }
}

/** The rates as the generated code takes them. */
function generatedRates(rates: readonly CurrencyRate[]): internal.CurrencyRate[] {
    return rates.map((rate) => internal.CurrencyRate.create({ currency: rate.currency.code, rate: rate.rate.toString() }));
}

/** `value` converted by `conversion`, or `undefined` for a value the group does not hold. */
function known<T, U>(value: T | undefined, conversion: (value: T) => U): U | undefined {
    return undefined === value ? undefined : conversion(value);
}
