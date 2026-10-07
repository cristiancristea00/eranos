/**
 * The entries of a group: its members, categories, expenses, and transfers, and the splits, costs, and tips they hold.
 * Every entry is a plain object; one to add has no `id`, and one the group holds has the `id` it gave.
 *
 * @packageDocumentation
 */

import { CalendarDate, TimeOfDay } from './calendar.js';
import { Email, Iban, PhoneNumber, Username } from './contact.js';
import { calling } from './converting.js';
import { type CategoryId, type ExpenseId, type MemberId, type TransferId, categoryId, expenseId, memberId, transferId } from './identifiers.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';
import { ExchangeRate, Parts, Percentage } from './ratios.js';

/** The exact amount one member pays of a split. */
export interface MemberAmount {
    /** The member. */
    readonly member: MemberId;
    /** The amount. */
    readonly amount: Money;
}

/** The parts one member pays in proportion to. */
export interface MemberParts {
    /** The member. */
    readonly member: MemberId;
    /** The member's parts. */
    readonly parts: Parts;
}

/** The percentage one member pays. */
export interface MemberPercentage {
    /** The member. */
    readonly member: MemberId;
    /** The member's percentage. */
    readonly percentage: Percentage;
}

/**
 * How an amount divides among members: equally, by parts, by percentages, or by exact amounts.
 *
 * The library divides every amount so that the shares add up to it exactly, and gives any minor unit left over by the
 * rounding to a member it picks from the expense's identifier.
 */
export type Split =
    | {
          /** Every member pays the same. */
          readonly kind: 'equal';
          /** The members who share the amount. */
          readonly members: readonly MemberId[];
      }
    | {
          /** Every member pays in proportion to their parts. */
          readonly kind: 'parts';
          /** Each member's parts. */
          readonly shares: readonly MemberParts[];
      }
    | {
          /** Every member pays a percentage; the percentages add up to 100 %. */
          readonly kind: 'percentages';
          /** Each member's percentage. */
          readonly shares: readonly MemberPercentage[];
      }
    | {
          /** Every member pays an exact amount; the amounts add up to what they divide. */
          readonly kind: 'amounts';
          /** Each member's amount. */
          readonly shares: readonly MemberAmount[];
      };

/**
 * The members `split` divides among, in the order it lists them.
 *
 * @param split - The split
 * @returns The members
 */
export function splitMembers(split: Split): readonly MemberId[] {
    switch (split.kind) {
        case 'equal':
            return split.members;
        case 'parts':
            return split.shares.map((share) => share.member);
        case 'percentages':
            return split.shares.map((share) => share.member);
        case 'amounts':
            return split.shares.map((share) => share.member);
    }
}

/**
 * The split the generated code holds as `split`.
 *
 * @internal
 */
export function convertedSplit(split: internal.Split): Split {
    switch (split.tag) {
        case internal.Split_Tags.Equal:
            return { kind: 'equal', members: split.inner.members.map(memberId) };
        case internal.Split_Tags.Parts:
            return {
                kind: 'parts',
                shares: split.inner.shares.map((share) => ({ member: memberId(share.member), parts: Parts.known(share.hundredths) })),
            };
        case internal.Split_Tags.Percentages:
            return {
                kind: 'percentages',
                shares: split.inner.shares.map((share) => ({ member: memberId(share.member), percentage: Percentage.known(share.basisPoints) })),
            };
        case internal.Split_Tags.Amounts:
            return {
                kind: 'amounts',
                shares: split.inner.shares.map((share) => ({ member: memberId(share.member), amount: Money.converted(share.amount) })),
            };
    }
}

/**
 * The split as the generated code holds it.
 *
 * @internal
 */
export function generatedSplit(split: Split): internal.Split {
    switch (split.kind) {
        case 'equal':
            return new internal.Split.Equal({ members: [...split.members] });
        case 'parts':
            return new internal.Split.Parts({
                shares: split.shares.map((share) => internal.MemberParts.create({ member: share.member, hundredths: share.parts.hundredths })),
            });
        case 'percentages':
            return new internal.Split.Percentages({
                shares: split.shares.map((share) =>
                    internal.MemberPercentage.create({ member: share.member, basisPoints: share.percentage.basisPoints }),
                ),
            });
        case 'amounts':
            return new internal.Split.Amounts({
                shares: split.shares.map((share) => internal.MemberAmount.create({ member: share.member, amount: share.amount.generated() })),
            });
    }
}

/**
 * What an entry cost: an amount in the group currency, or one in another currency with the rate that converts it into the
 * group currency.
 */
export type Cost =
    | {
          /** An amount in the group currency. */
          readonly kind: 'local';
          /** The amount. */
          readonly amount: Money;
      }
    | {
          /** An amount in another currency than the group's, converted at `rate`. */
          readonly kind: 'converted';
          /** The amount, in its own currency. */
          readonly amount: Money;
          /** The rate into the group currency. */
          readonly rate: ExchangeRate;
      };

/**
 * The cost the generated code holds as `cost`.
 *
 * @internal
 */
export function convertedCost(cost: internal.Cost): Cost {
    switch (cost.tag) {
        case internal.Cost_Tags.Local:
            return { kind: 'local', amount: Money.converted(cost.inner.amount) };
        case internal.Cost_Tags.Converted:
            return { kind: 'converted', amount: Money.converted(cost.inner.amount), rate: ExchangeRate.known(cost.inner.rate) };
    }
}

/**
 * The cost as the generated code holds it.
 *
 * @internal
 */
export function generatedCost(cost: Cost): internal.Cost {
    switch (cost.kind) {
        case 'local':
            return new internal.Cost.Local({ amount: cost.amount.generated() });
        case 'converted':
            return new internal.Cost.Converted({ amount: cost.amount.generated(), rate: cost.rate.toString() });
    }
}

/** A line of an itemised bill: `quantity` units of `name` at `unitPrice`, which `split` divides. */
export interface Item {
    /** What was bought. */
    readonly name: string;
    /** The price of one unit, in the expense's currency. */
    readonly unitPrice: Money;
    /** How the line's total divides. */
    readonly split: Split;
    /** How many units, from 1; one unless given. */
    readonly quantity?: number;
}

/** How an expense's cost is shared: as a whole, or line by line of an itemised bill. */
export type ExpenseSplit =
    | {
          /** The whole amount is divided by one split. */
          readonly kind: 'whole';
          /** How the amount divides. */
          readonly split: Split;
      }
    | {
          /** Every line is divided by its own split; the lines add up to the amount. */
          readonly kind: 'itemised';
          /** The lines of the bill. */
          readonly items: readonly Item[];
      };

/**
 * The expense split the generated code holds as `split`.
 *
 * @internal
 */
export function convertedExpenseSplit(split: internal.ExpenseSplit): ExpenseSplit {
    switch (split.tag) {
        case internal.ExpenseSplit_Tags.Whole:
            return { kind: 'whole', split: convertedSplit(split.inner.split) };
        case internal.ExpenseSplit_Tags.Itemised:
            return {
                kind: 'itemised',
                items: split.inner.items.map((item) => ({
                    name: item.name,
                    unitPrice: Money.converted(item.unitPrice),
                    split: convertedSplit(item.split),
                    quantity: item.quantity,
                })),
            };
    }
}

/**
 * The expense split as the generated code holds it.
 *
 * @internal
 */
export function generatedExpenseSplit(split: ExpenseSplit): internal.ExpenseSplit {
    switch (split.kind) {
        case 'whole':
            return new internal.ExpenseSplit.Whole({ split: generatedSplit(split.split) });
        case 'itemised':
            return new internal.ExpenseSplit.Itemised({
                items: split.items.map((item) =>
                    internal.Item.create({
                        name: item.name,
                        unitPrice: item.unitPrice.generated(),
                        split: generatedSplit(item.split),
                        quantity: item.quantity ?? 1,
                    }),
                ),
            });
    }
}

/** How a tip is shared among an expense's participants. */
export enum TipDistribution {
    /** In proportion to each participant's share of the amount. */
    PROPORTIONAL = 'PROPORTIONAL',
    /** The same for every participant. */
    EQUAL = 'EQUAL',
}

/** How a tip is expressed. */
export type TipValue =
    | {
          /** A percentage of the expense's amount. */
          readonly kind: 'percentage';
          /** The tip's share of the amount. */
          readonly percentage: Percentage;
      }
    | {
          /** A fixed amount in the expense's currency. */
          readonly kind: 'fixed';
          /** The tip. */
          readonly amount: Money;
      };

/** A tip on top of an expense's amount, which its payers cover. */
export interface Tip {
    /** How the tip is expressed. */
    readonly value: TipValue;
    /** How the tip is shared among the participants: in proportion to each share unless given. */
    readonly distribution?: TipDistribution;
}

/**
 * The tip the generated code holds as `tip`.
 *
 * @internal
 */
export function convertedTip(tip: internal.Tip): Tip {
    return { value: convertedTipValue(tip.value), distribution: convertedTipDistribution(tip.distribution) };
}

/** The tip's value the generated code holds as `value`. */
function convertedTipValue(value: internal.TipValue): TipValue {
    switch (value.tag) {
        case internal.TipValue_Tags.Percentage:
            return { kind: 'percentage', percentage: Percentage.known(value.inner.basisPoints) };
        case internal.TipValue_Tags.Fixed:
            return { kind: 'fixed', amount: Money.converted(value.inner.amount) };
    }
}

/** The distribution the generated code holds as `distribution`. */
function convertedTipDistribution(distribution: internal.TipDistribution): TipDistribution {
    switch (distribution) {
        case internal.TipDistribution.Proportional:
            return TipDistribution.PROPORTIONAL;
        case internal.TipDistribution.Equal:
            return TipDistribution.EQUAL;
    }
}

/**
 * The tip as the generated code holds it.
 *
 * @internal
 */
export function generatedTip(tip: Tip): internal.Tip {
    return internal.Tip.create({
        value: generatedTipValue(tip.value),
        distribution: generatedTipDistribution(tip.distribution ?? TipDistribution.PROPORTIONAL),
    });
}

/** The tip's value as the generated code holds it. */
function generatedTipValue(value: TipValue): internal.TipValue {
    switch (value.kind) {
        case 'percentage':
            return new internal.TipValue.Percentage({ basisPoints: value.percentage.basisPoints });
        case 'fixed':
            return new internal.TipValue.Fixed({ amount: value.amount.generated() });
    }
}

/** The distribution as the generated code holds it. */
function generatedTipDistribution(distribution: TipDistribution): internal.TipDistribution {
    switch (distribution) {
        case TipDistribution.PROPORTIONAL:
            return internal.TipDistribution.Proportional;
        case TipDistribution.EQUAL:
            return internal.TipDistribution.Equal;
    }
}

/**
 * Something bought for the group: what it cost, who paid, and how it divides.
 *
 * A new expense has no `id`; {@link ExpenseGroup.addExpense} gives it one. To change an expense, read it from the group, spread
 * it into a new object with the changes, and pass that to {@link ExpenseGroup.replaceExpense}.
 */
export interface Expense {
    /** The identifier: absent for an expense to add, present for one the group holds. */
    readonly id?: ExpenseId;
    /** What was bought. */
    readonly name: string;
    /** What it cost. */
    readonly cost: Cost;
    /** Who paid the total, amount and tip, and how much each. */
    readonly payers: Split;
    /** How the amount is shared. */
    readonly split: ExpenseSplit;
    /** The day it was bought. */
    readonly date: CalendarDate;
    /** The tip on top of the amount. */
    readonly tip?: Tip;
    /** The time of day it was bought. */
    readonly time?: TimeOfDay;
    /** The category; absent for the built-in one. */
    readonly category?: CategoryId;
    /** A longer description. */
    readonly description?: string;
}

/**
 * The expense the generated code holds as `expense`.
 *
 * @internal
 */
export function convertedExpense(expense: internal.Expense): Expense {
    return {
        ...(undefined === expense.id ? {} : { id: expenseId(expense.id) }),
        name: expense.name,
        cost: convertedCost(expense.cost),
        payers: convertedSplit(expense.payers),
        split: convertedExpenseSplit(expense.split),
        date: CalendarDate.converted(expense.date),
        ...(undefined === expense.tip ? {} : { tip: convertedTip(expense.tip) }),
        ...(undefined === expense.time ? {} : { time: TimeOfDay.converted(expense.time) }),
        ...(undefined === expense.category ? {} : { category: categoryId(expense.category) }),
        ...(undefined === expense.description ? {} : { description: expense.description }),
    };
}

/**
 * The expense as the generated code holds it.
 *
 * @internal
 */
export function generatedExpense(expense: Expense): internal.Expense {
    return internal.Expense.create({
        ...(undefined === expense.id ? {} : { id: expense.id }),
        name: expense.name,
        cost: generatedCost(expense.cost),
        payers: generatedSplit(expense.payers),
        split: generatedExpenseSplit(expense.split),
        date: expense.date.generated(),
        ...(undefined === expense.tip ? {} : { tip: generatedTip(expense.tip) }),
        ...(undefined === expense.time ? {} : { time: expense.time.generated() }),
        ...(undefined === expense.category ? {} : { category: expense.category }),
        ...(undefined === expense.description ? {} : { description: expense.description }),
    });
}

/**
 * The tip of `expense`, in its own currency.
 *
 * @param expense - The expense
 * @returns The tip
 * @throws {@link ExpenseError} with the failure `currencyMismatch` for a fixed tip in another currency, and `negativeTip`
 *   for a fixed tip below zero
 */
export function expenseTipAmount(expense: Expense): Money {
    return Money.converted(calling(() => internal.expenseTipAmount(generatedExpense(expense))));
}

/**
 * The total of `expense`, amount and tip, in its own currency: what its payers paid.
 *
 * @param expense - The expense
 * @returns The total
 * @throws {@link ExpenseError} as {@link expenseTipAmount} does, and {@link MoneyError} with the failure `overflow` for a
 *   total out of range
 */
export function expenseTotal(expense: Expense): Money {
    return Money.converted(calling(() => internal.expenseTotal(generatedExpense(expense))));
}

/**
 * Every member `expense` involves, as a payer or a participant, in identifier order.
 *
 * @param expense - The expense
 * @returns The members
 * @throws {@link IdError} for a member whose identifier is not one, and the failures of an item that does not build
 */
export function expenseMembers(expense: Expense): readonly MemberId[] {
    return calling(() => internal.expenseMembers(generatedExpense(expense))).map(memberId);
}

/** Where a member can be paid: a bank account or a payment-app username. */
export type AccountDetails =
    | {
          /** A bank account. */
          readonly kind: 'iban';
          /** The account's IBAN. */
          readonly iban: Iban;
      }
    | {
          /** A username in a payment app. */
          readonly kind: 'username';
          /** The username. */
          readonly username: Username;
      };

/** A place to send a member money, with a label: "Main account". */
export interface Account {
    /** The label. */
    readonly name: string;
    /** The IBAN or the username. */
    readonly details: AccountDetails;
}

/** The account details the generated code holds as `details`. */
function convertedAccountDetails(details: internal.AccountDetails): AccountDetails {
    switch (details.tag) {
        case internal.AccountDetails_Tags.Iban:
            return { kind: 'iban', iban: Iban.known(details.inner.iban) };
        case internal.AccountDetails_Tags.Username:
            return { kind: 'username', username: Username.known(details.inner.username) };
    }
}

/** The account details as the generated code holds them. */
function generatedAccountDetails(details: AccountDetails): internal.AccountDetails {
    switch (details.kind) {
        case 'iban':
            return new internal.AccountDetails.Iban({ iban: details.iban.toString() });
        case 'username':
            return new internal.AccountDetails.Username({ username: details.username.toString() });
    }
}

/**
 * A person in the group: a first name, and optionally more ways to reach and pay them.
 *
 * A new member has no `id`; {@link ExpenseGroup.addMember} gives them one.
 */
export interface Member {
    /** The identifier: absent for a member to add, present for one the group holds. */
    readonly id?: MemberId;
    /** The first name. */
    readonly firstName: string;
    /** The last name. */
    readonly lastName?: string;
    /** The email address. */
    readonly email?: Email;
    /** The phone number. */
    readonly phone?: PhoneNumber;
    /** Where the member can be paid, in the order they were given; none unless given. */
    readonly accounts?: readonly Account[];
}

/**
 * The member the generated code holds as `member`.
 *
 * @internal
 */
export function convertedMember(member: internal.Member): Member {
    return {
        ...(undefined === member.id ? {} : { id: memberId(member.id) }),
        firstName: member.firstName,
        ...(undefined === member.lastName ? {} : { lastName: member.lastName }),
        ...(undefined === member.email ? {} : { email: Email.known(member.email) }),
        ...(undefined === member.phone ? {} : { phone: PhoneNumber.known(member.phone) }),
        accounts: member.accounts.map((account) => ({ name: account.name, details: convertedAccountDetails(account.details) })),
    };
}

/**
 * The member as the generated code holds them.
 *
 * @internal
 */
export function generatedMember(member: Member): internal.Member {
    return internal.Member.create({
        ...(undefined === member.id ? {} : { id: member.id }),
        firstName: member.firstName,
        ...(undefined === member.lastName ? {} : { lastName: member.lastName }),
        ...(undefined === member.email ? {} : { email: member.email.toString() }),
        ...(undefined === member.phone ? {} : { phone: member.phone.e164 }),
        accounts: (member.accounts ?? []).map((account) =>
            internal.Account.create({ name: account.name, details: generatedAccountDetails(account.details) }),
        ),
    });
}

/**
 * A label the group gives expenses: "Food".
 *
 * A new category has no `id`; {@link ExpenseGroup.addCategory} gives it one. The built-in category, Uncategorised, holds every
 * expense without another.
 */
export interface Category {
    /** The identifier: absent for a category to add, present for one the group holds. */
    readonly id?: CategoryId;
    /** The name, which no other category of the group shares. */
    readonly name: string;
}

/**
 * The category the generated code holds as `category`.
 *
 * @internal
 */
export function convertedCategory(category: internal.Category): Category {
    return { ...(undefined === category.id ? {} : { id: categoryId(category.id) }), name: category.name };
}

/**
 * The category as the generated code holds it.
 *
 * @internal
 */
export function generatedCategory(category: Category): internal.Category {
    return internal.Category.create({ ...(undefined === category.id ? {} : { id: category.id }), name: category.name });
}

/**
 * Money one member paid another, such as a repayment.
 *
 * A new transfer has no `id`; {@link ExpenseGroup.addTransfer} gives it one.
 */
export interface Transfer {
    /** The identifier: absent for a transfer to add, present for one the group holds. */
    readonly id?: TransferId;
    /** The member who paid. */
    readonly from: MemberId;
    /** The member who was paid. */
    readonly to: MemberId;
    /** What was paid. */
    readonly cost: Cost;
    /** The day it was paid. */
    readonly date: CalendarDate;
    /** The time of day it was paid. */
    readonly time?: TimeOfDay;
    /** A note on the transfer. */
    readonly note?: string;
}

/**
 * The transfer the generated code holds as `transfer`.
 *
 * @internal
 */
export function convertedTransfer(transfer: internal.Transfer): Transfer {
    return {
        ...(undefined === transfer.id ? {} : { id: transferId(transfer.id) }),
        from: memberId(transfer.from),
        to: memberId(transfer.to),
        cost: convertedCost(transfer.cost),
        date: CalendarDate.converted(transfer.date),
        ...(undefined === transfer.time ? {} : { time: TimeOfDay.converted(transfer.time) }),
        ...(undefined === transfer.note ? {} : { note: transfer.note }),
    };
}

/**
 * The transfer as the generated code holds it.
 *
 * @internal
 */
export function generatedTransfer(transfer: Transfer): internal.Transfer {
    return internal.Transfer.create({
        ...(undefined === transfer.id ? {} : { id: transfer.id }),
        from: transfer.from,
        to: transfer.to,
        cost: generatedCost(transfer.cost),
        date: transfer.date.generated(),
        ...(undefined === transfer.time ? {} : { time: transfer.time.generated() }),
        ...(undefined === transfer.note ? {} : { note: transfer.note }),
    });
}
