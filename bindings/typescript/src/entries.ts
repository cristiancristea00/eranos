/**
 * The entries of a group: its members, categories, expenses, and transfers, and the splits, costs, and tips they hold.
 * Every entry is a plain object with its `id` from the moment it is made: a `{ id, … }` literal takes the app's own, and
 * {@link createMember} and its siblings take the other fields and generate one.
 *
 * A record with a field the library gives a default has two types: the one the group returns under the record's name,
 * {@link Member}, with every field it always holds required, and the one an add or a replace takes, {@link MemberInput},
 * which may leave those fields out. Every record the group returns is also one it takes.
 *
 * @packageDocumentation
 */

import { CalendarDate, TimeOfDay } from './calendar.js';
import { Email, Iban, PhoneNumber, Username } from './contact.js';
import { calling } from './converting.js';
import {
    type CategoryId,
    type ExpenseId,
    type MemberId,
    type TransferId,
    UNCATEGORISED,
    categoryId,
    expenseId,
    generateCategoryId,
    generateExpenseId,
    generateMemberId,
    generateTransferId,
    memberId,
    transferId,
} from './identifiers.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';
import { ExchangeRate, Parts, Percentage, Quantity } from './ratios.js';

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

/**
 * A line of an itemised bill, as the group holds it: `quantity` units of `name` at `unitPrice`, which `split` divides. Its
 * line total, which {@link expenseItemLineTotal} tells, is the unit price times the quantity, rounded half away from zero to
 * the currency's minor unit.
 */
export interface ExpenseItem {
    /** What was bought. */
    readonly name: string;
    /** The price of one unit, in the expense's currency. */
    readonly unitPrice: Money;
    /** How the line's total divides. */
    readonly split: Split;
    /** How many units, to a ten-thousandth: 2 bottles, or 0.452 kg. */
    readonly quantity: Quantity;
}

/**
 * A line of an itemised bill in an expense to add or replace; it may leave out its quantity, which is then
 * {@link Quantity.ONE}. Every {@link ExpenseItem} is one.
 */
export interface ExpenseItemInput {
    /** What was bought. */
    readonly name: string;
    /** The price of one unit, in the expense's currency. */
    readonly unitPrice: Money;
    /** How the line's total divides. */
    readonly split: Split;
    /** How many units, to a ten-thousandth: 2 bottles, or 0.452 kg; {@link Quantity.ONE} unless given. */
    readonly quantity?: Quantity;
}

/** The item that `item` describes, with a quantity of one unless it gives another. */
function completedExpenseItem(item: ExpenseItemInput): ExpenseItem {
    return { name: item.name, unitPrice: item.unitPrice, split: item.split, quantity: item.quantity ?? Quantity.ONE };
}

/** The item the generated code holds as `item`. */
function convertedExpenseItem(item: internal.ExpenseItem): ExpenseItem {
    return {
        name: item.name,
        unitPrice: Money.converted(item.unitPrice),
        split: convertedSplit(item.split),
        quantity: Quantity.known(item.quantity),
    };
}

/** The item as the generated code holds it. */
function generatedExpenseItem(item: ExpenseItem): internal.ExpenseItem {
    return internal.ExpenseItem.create({
        name: item.name,
        unitPrice: item.unitPrice.generated(),
        split: generatedSplit(item.split),
        quantity: item.quantity.tenThousandths,
    });
}

/**
 * The line total of `item`, in its unit price's currency: the unit price times the quantity, rounded half away from zero to
 * the currency's minor unit, as the library adds the lines up against the expense's amount. A line whose product is below
 * half a minor unit is free: its total is zero.
 *
 * @param item - The line
 * @returns The line total: 14.70 EUR for 0.35 units at 42.00 EUR, and 0.34 EUR for 0.335 units at 1.00 EUR
 * @throws {@link IdError} for a member whose identifier is not one, {@link MoneyError} with the failure `unknownCurrency`
 *   for a unit price whose currency details are not the library's, and {@link ExpenseError} for a line the library refuses:
 *   `emptyItemName` for a blank name, `nonPositiveAmount` for a unit price that is not positive, the failures of a split
 *   that does not divide the line, among them `amountsDoNotAddUp` for amounts that do not add up to the line total, a free
 *   line's included, and `overflow` for a line total out of range
 */
export function expenseItemLineTotal(item: ExpenseItemInput): Money {
    return Money.converted(calling(() => internal.expenseItemLineTotal(generatedExpenseItem(completedExpenseItem(item)))));
}

/** How an expense's cost is shared, as the group holds it: as a whole, or line by line of an itemised bill. */
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
          readonly items: readonly ExpenseItem[];
      };

/**
 * How an expense to add or replace shares its cost, whose lines may leave out their quantity. Every {@link ExpenseSplit} is
 * one.
 */
export type ExpenseSplitInput =
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
          readonly items: readonly ExpenseItemInput[];
      };

/** The split that `split` describes, every line of it with its quantity. */
function completedExpenseSplit(split: ExpenseSplitInput): ExpenseSplit {
    switch (split.kind) {
        case 'whole':
            return { kind: 'whole', split: split.split };
        case 'itemised':
            return { kind: 'itemised', items: split.items.map(completedExpenseItem) };
    }
}

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
            return { kind: 'itemised', items: split.inner.items.map(convertedExpenseItem) };
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
            return new internal.ExpenseSplit.Itemised({ items: split.items.map(generatedExpenseItem) });
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

/** A tip on top of an expense's amount, which its payers cover, as the group holds it. */
export interface TipRule {
    /** How the tip is expressed. */
    readonly value: TipValue;
    /** How the tip is shared among the participants. */
    readonly distribution: TipDistribution;
}

/**
 * A tip to set on an expense; it may leave out how the tip is shared, which is then in proportion to each share. Every
 * {@link TipRule} is one.
 */
export interface TipRuleInput {
    /** How the tip is expressed. */
    readonly value: TipValue;
    /** How the tip is shared among the participants: in proportion to each share unless given. */
    readonly distribution?: TipDistribution;
}

/** The tip that `tip` describes, shared in proportion to each share unless it says otherwise. */
function completedTipRule(tip: TipRuleInput): TipRule {
    return { value: tip.value, distribution: tip.distribution ?? TipDistribution.PROPORTIONAL };
}

/** No tip: 0 % of the amount, shared in proportion to each share, which an expense has until it is given another tip. */
// The literal, since a module constant is evaluated before `init` loads the WebAssembly module; a test holds it to the
// library's.
export const NO_TIP: TipRule = { value: { kind: 'percentage', percentage: Percentage.known(0) }, distribution: TipDistribution.PROPORTIONAL };

/**
 * The tip the generated code holds as `tip`.
 *
 * @internal
 */
export function convertedTipRule(tip: internal.TipRule): TipRule {
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
export function generatedTipRule(tip: TipRule): internal.TipRule {
    return internal.TipRule.create({
        value: generatedTipValue(tip.value),
        distribution: generatedTipDistribution(tip.distribution),
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
 * Something bought for the group, as the group holds it: what it cost, who paid, and how it divides.
 *
 * To change an expense, read it from the group, spread it into a new object with the changes, and pass that to
 * {@link ExpenseGroup.replaceExpense}.
 */
export interface Expense {
    /** The identifier, which the app gives: its own, or one that {@link generateExpenseId} makes. */
    readonly id: ExpenseId;
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
    /** The tip on top of the amount: {@link NO_TIP} for an expense without one. */
    readonly tip: TipRule;
    /** The time of day it was bought. */
    readonly time?: TimeOfDay;
    /** The category: the built-in one, {@link UNCATEGORISED}, until the expense is given another. */
    readonly category: CategoryId;
    /** A longer description. */
    readonly description?: string;
}

/**
 * An expense to add or replace: what it cost, who paid, and how it divides, with its identifier from the moment it is made.
 * A `{ id, … }` literal takes the app's own identifier, and {@link createExpense} generates one.
 *
 * It may leave out its tip, its category, how a tip is shared, and the quantity of each line of an itemised bill, which
 * take their defaults: no tip, the built-in category, in proportion to each share, and one unit. Every {@link Expense} is
 * one.
 */
export interface ExpenseInput {
    /** The identifier, which the app gives: its own, or one that {@link generateExpenseId} makes. */
    readonly id: ExpenseId;
    /** What was bought. */
    readonly name: string;
    /** What it cost. */
    readonly cost: Cost;
    /** Who paid the total, amount and tip, and how much each. */
    readonly payers: Split;
    /** How the amount is shared. */
    readonly split: ExpenseSplitInput;
    /** The day it was bought. */
    readonly date: CalendarDate;
    /** The tip on top of the amount; {@link NO_TIP} unless given. */
    readonly tip?: TipRuleInput;
    /** The time of day it was bought. */
    readonly time?: TimeOfDay;
    /** The category; the built-in one, {@link UNCATEGORISED}, unless given. */
    readonly category?: CategoryId;
    /** A longer description. */
    readonly description?: string;
}

/**
 * The expense that `expense` describes, every field it leaves out at its default.
 *
 * @internal
 */
export function completedExpense(expense: ExpenseInput): Expense {
    return {
        id: expense.id,
        name: expense.name,
        cost: expense.cost,
        payers: expense.payers,
        split: completedExpenseSplit(expense.split),
        date: expense.date,
        tip: undefined === expense.tip ? NO_TIP : completedTipRule(expense.tip),
        ...(undefined === expense.time ? {} : { time: expense.time }),
        category: expense.category ?? UNCATEGORISED,
        ...(undefined === expense.description ? {} : { description: expense.description }),
    };
}

/**
 * The expense that `fields` describe, under an identifier this call generates, every field they leave out at its default;
 * a `{ id, … }` literal takes the app's own identifier instead.
 *
 * @param fields - Every field of the expense but its identifier
 * @returns The expense
 */
export function createExpense(fields: Omit<ExpenseInput, 'id'>): Expense {
    // The generated identifier is set after the fields, so that an `id` they carry at run time, which `Omit` lets through,
    // does not replace it.
    return completedExpense({ ...fields, id: generateExpenseId() });
}

/**
 * The expense the generated code holds as `expense`.
 *
 * @internal
 */
export function convertedExpense(expense: internal.Expense): Expense {
    return {
        id: expenseId(expense.id),
        name: expense.name,
        cost: convertedCost(expense.cost),
        payers: convertedSplit(expense.payers),
        split: convertedExpenseSplit(expense.split),
        date: CalendarDate.converted(expense.date),
        tip: convertedTipRule(expense.tip),
        ...(undefined === expense.time ? {} : { time: TimeOfDay.converted(expense.time) }),
        category: categoryId(expense.category),
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
        id: expense.id,
        name: expense.name,
        cost: generatedCost(expense.cost),
        payers: generatedSplit(expense.payers),
        split: generatedExpenseSplit(expense.split),
        date: expense.date.generated(),
        tip: generatedTipRule(expense.tip),
        ...(undefined === expense.time ? {} : { time: expense.time.generated() }),
        category: expense.category,
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
export function expenseTipAmount(expense: ExpenseInput): Money {
    return Money.converted(calling(() => internal.expenseTipAmount(generatedExpense(completedExpense(expense)))));
}

/**
 * The total of `expense`, amount and tip, in its own currency: what its payers paid.
 *
 * @param expense - The expense
 * @returns The total
 * @throws {@link ExpenseError} as {@link expenseTipAmount} does, and {@link MoneyError} with the failure `overflow` for a
 *   total out of range
 */
export function expenseTotal(expense: ExpenseInput): Money {
    return Money.converted(calling(() => internal.expenseTotal(generatedExpense(completedExpense(expense)))));
}

/**
 * Every member `expense` involves, as a payer or a participant, in identifier order.
 *
 * @param expense - The expense
 * @returns The members
 * @throws {@link IdError} for a member whose identifier is not one, and the failures of an item that does not build
 */
export function expenseMembers(expense: ExpenseInput): readonly MemberId[] {
    return calling(() => internal.expenseMembers(generatedExpense(completedExpense(expense)))).map(memberId);
}

/** Where a member can be paid: a bank account or a payment-app username. */
export type PaymentAccountDetails =
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
export interface PaymentAccount {
    /** The label. */
    readonly name: string;
    /** The IBAN or the username. */
    readonly details: PaymentAccountDetails;
}

/** The account details the generated code holds as `details`. */
function convertedPaymentAccountDetails(details: internal.PaymentAccountDetails): PaymentAccountDetails {
    switch (details.tag) {
        case internal.PaymentAccountDetails_Tags.Iban:
            return { kind: 'iban', iban: Iban.converted(details.inner) };
        case internal.PaymentAccountDetails_Tags.Username:
            return { kind: 'username', username: Username.known(details.inner.username) };
    }
}

/** The account details as the generated code holds them. */
function generatedPaymentAccountDetails(details: PaymentAccountDetails): internal.PaymentAccountDetails {
    switch (details.kind) {
        case 'iban':
            return new internal.PaymentAccountDetails.Iban(details.iban.generated());
        case 'username':
            return new internal.PaymentAccountDetails.Username({ username: details.username.toString() });
    }
}

/** A person in the group, as the group holds them: a first name, and optionally more ways to reach and pay them. */
export interface Member {
    /** The identifier, which the app gives: its own, or one that {@link generateMemberId} makes. */
    readonly id: MemberId;
    /** The first name. */
    readonly firstName: string;
    /** The last name. */
    readonly lastName?: string;
    /** The email address. */
    readonly email?: Email;
    /** The phone number. */
    readonly phone?: PhoneNumber;
    /** Where the member can be paid, in the order they were given; empty for a member without an account. */
    readonly accounts: readonly PaymentAccount[];
}

/**
 * A person to add to the group or whose details to replace: a first name, and optionally more ways to reach and pay them,
 * with their identifier from the moment they are made. A `{ id, … }` literal takes the app's own identifier, and
 * {@link createMember} generates one.
 *
 * It may leave out the accounts, which are then none. Every {@link Member} is one.
 */
export interface MemberInput {
    /** The identifier, which the app gives: its own, or one that {@link generateMemberId} makes. */
    readonly id: MemberId;
    /** The first name. */
    readonly firstName: string;
    /** The last name. */
    readonly lastName?: string;
    /** The email address. */
    readonly email?: Email;
    /** The phone number. */
    readonly phone?: PhoneNumber;
    /** Where the member can be paid, in the order they are given; none unless given. */
    readonly accounts?: readonly PaymentAccount[];
}

/**
 * The member that `member` describes, with no accounts unless it gives some.
 *
 * @internal
 */
export function completedMember(member: MemberInput): Member {
    return {
        id: member.id,
        firstName: member.firstName,
        ...(undefined === member.lastName ? {} : { lastName: member.lastName }),
        ...(undefined === member.email ? {} : { email: member.email }),
        ...(undefined === member.phone ? {} : { phone: member.phone }),
        accounts: member.accounts ?? [],
    };
}

/**
 * The member that `fields` describe, under an identifier this call generates, with no accounts unless they give some; a
 * `{ id, … }` literal takes the app's own identifier instead.
 *
 * @param fields - Every field of the member but their identifier
 * @returns The member
 */
export function createMember(fields: Omit<MemberInput, 'id'>): Member {
    // The generated identifier is set after the fields, so that an `id` they carry at run time, which `Omit` lets through,
    // does not replace it.
    return completedMember({ ...fields, id: generateMemberId() });
}

/**
 * The member the generated code holds as `member`.
 *
 * @internal
 */
export function convertedMember(member: internal.Member): Member {
    return {
        id: memberId(member.id),
        firstName: member.firstName,
        ...(undefined === member.lastName ? {} : { lastName: member.lastName }),
        ...(undefined === member.email ? {} : { email: Email.known(member.email) }),
        ...(undefined === member.phone ? {} : { phone: PhoneNumber.converted(member.phone) }),
        accounts: member.accounts.map((account) => ({ name: account.name, details: convertedPaymentAccountDetails(account.details) })),
    };
}

/**
 * The member as the generated code holds them.
 *
 * @internal
 */
export function generatedMember(member: Member): internal.Member {
    return internal.Member.create({
        id: member.id,
        firstName: member.firstName,
        ...(undefined === member.lastName ? {} : { lastName: member.lastName }),
        ...(undefined === member.email ? {} : { email: member.email.toString() }),
        ...(undefined === member.phone ? {} : { phone: member.phone.generated() }),
        accounts: member.accounts.map((account) =>
            internal.PaymentAccount.create({ name: account.name, details: generatedPaymentAccountDetails(account.details) }),
        ),
    });
}

/**
 * A label the group gives expenses: "Food".
 *
 * A new category has its identifier from the moment it is made: a `{ id, … }` literal takes the app's own, and
 * {@link createCategory} generates one. The built-in category, Uncategorised, holds every expense without another.
 */
export interface Category {
    /** The identifier, which the app gives: its own, or one that {@link generateCategoryId} makes. */
    readonly id: CategoryId;
    /** The name, which no other category of the group shares. */
    readonly name: string;
}

/**
 * The category that `fields` describe, under an identifier this call generates; a `{ id, … }` literal takes the app's own
 * instead.
 *
 * @param fields - Every field of the category but its identifier
 * @returns The category
 */
export function createCategory(fields: Omit<Category, 'id'>): Category {
    return { id: generateCategoryId(), name: fields.name };
}

/**
 * The category the generated code holds as `category`.
 *
 * @internal
 */
export function convertedCategory(category: internal.Category): Category {
    return { id: categoryId(category.id), name: category.name };
}

/**
 * The category as the generated code holds it.
 *
 * @internal
 */
export function generatedCategory(category: Category): internal.Category {
    return internal.Category.create({ id: category.id, name: category.name });
}

/**
 * Money one member paid another, such as a repayment.
 *
 * A new transfer has its identifier from the moment it is made: a `{ id, … }` literal takes the app's own, and
 * {@link createTransfer} generates one.
 */
export interface Transfer {
    /** The identifier, which the app gives: its own, or one that {@link generateTransferId} makes. */
    readonly id: TransferId;
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
 * The transfer that `fields` describe, under an identifier this call generates; a `{ id, … }` literal takes the app's own
 * instead.
 *
 * @param fields - Every field of the transfer but its identifier
 * @returns The transfer
 */
export function createTransfer(fields: Omit<Transfer, 'id'>): Transfer {
    return {
        id: generateTransferId(),
        from: fields.from,
        to: fields.to,
        cost: fields.cost,
        date: fields.date,
        ...(undefined === fields.time ? {} : { time: fields.time }),
        ...(undefined === fields.note ? {} : { note: fields.note }),
    };
}

/**
 * The transfer the generated code holds as `transfer`.
 *
 * @internal
 */
export function convertedTransfer(transfer: internal.Transfer): Transfer {
    return {
        id: transferId(transfer.id),
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
        id: transfer.id,
        from: transfer.from,
        to: transfer.to,
        cost: generatedCost(transfer.cost),
        date: transfer.date.generated(),
        ...(undefined === transfer.time ? {} : { time: transfer.time.generated() }),
        ...(undefined === transfer.note ? {} : { note: transfer.note }),
    });
}
