/**
 * The read handle of a group.
 *
 * @packageDocumentation
 */

import { Language } from './contact.js';
import { calling } from './converting.js';
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
} from './entries.js';
import { type CategoryId, type ExpenseId, type ExpenseGroupId, type MemberId, type TransferId, expenseGroupId } from './identifiers.js';
import type * as internal from './internal/eranos.js';
import { Money } from './money.js';
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
 * A group's reads under one shared lock, so that they see no change in between: changes wait until the handle is
 * released.
 *
 * A handle holds the lock until {@link ExpenseGroupReader.release}, which a `using` declaration calls at the end of its scope;
 * the reads never wait, since the handle holds the lock they need.
 */
export class ExpenseGroupReader implements Disposable {
    private constructor(
        /** The generated read handle this one forwards to. */
        private readonly handle: internal.ExpenseGroupReaderLike,
    ) {}

    /**
     * The read handle of `handle`.
     *
     * @internal
     */
    static wrapping(handle: internal.ExpenseGroupReaderLike): ExpenseGroupReader {
        return new ExpenseGroupReader(handle);
    }

    /** Releases the lock; every read after it throws a `TypeError`, and releasing a released handle does nothing. */
    release(): void {
        this.handle.release();
    }

    /** Releases the lock, as {@link ExpenseGroupReader.release} does, at the end of a `using` declaration's scope. */
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
     * The group's description, if it has one.
     *
     * @returns The description, if the group has one
     * @throws `TypeError` after {@link ExpenseGroupReader.release}
     */
    description(): string | undefined {
        return calling(() => this.handle.description());
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
}

/** `value` converted by `conversion`, or `undefined` for a value the group does not hold. */
function known<T, U>(value: T | undefined, conversion: (value: T) => U): U | undefined {
    return undefined === value ? undefined : conversion(value);
}
