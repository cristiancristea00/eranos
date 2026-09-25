/**
 * The totals of a group, the shares of one expense, and the settlements, all in the group currency.
 *
 * @packageDocumentation
 */

import { type CategoryId, type MemberId, categoryId, memberId } from './identifiers.js';
import * as internal from './internal/eranos.js';
import { Money } from './money.js';

/** What the expenses of a category add up to, in the group currency. */
export interface CategoryTotal {
    /** The category. */
    readonly category: CategoryId;
    /** The total. */
    readonly total: Money;
}

/** Where a member stands with the group; {@link formatBalance} writes it as the library does, "owes 12.00 EUR". */
export type Balance =
    | {
          /** The others owe the member `amount`. */
          readonly kind: 'getsBack';
          /** What the others owe the member. */
          readonly amount: Money;
      }
    | {
          /** The member owes the others `amount`. */
          readonly kind: 'owes';
          /** What the member owes the others. */
          readonly amount: Money;
      }
    | {
          /** The member owes nothing and is owed nothing. */
          readonly kind: 'settled';
      };

/** A member's totals, in the group currency. */
export interface MemberSummary {
    /** Where the member stands. */
    readonly balance: Balance;
    /** The member. */
    readonly member: MemberId;
    /** What the member is owed, less what they owe: positive when they get money back. */
    readonly net: Money;
    /** What the member paid for expenses. */
    readonly paid: Money;
    /** What the member received in transfers. */
    readonly received: Money;
    /** What the member sent in transfers. */
    readonly sent: Money;
    /** The member's share of the expenses: what they consumed. */
    readonly share: Money;
}

/** What one member's shares add up to in each category. */
export interface MemberCategoryTotals {
    /** The member. */
    readonly member: MemberId;
    /** The member's total in every category they have a share in. */
    readonly totals: readonly CategoryTotal[];
}

/** The group's totals: overall, per member, per category, and per member per category, all in the group currency. */
export interface Summary {
    /** The total of every category. */
    readonly categoryTotals: readonly CategoryTotal[];
    /** The totals per category of every member. */
    readonly memberCategoryTotals: readonly MemberCategoryTotals[];
    /** The totals of every member. */
    readonly members: readonly MemberSummary[];
    /** What the group spent. */
    readonly totalSpent: Money;
}

/** What one member paid of an expense and what their share of it is. */
export interface MemberShare {
    /** The member. */
    readonly member: MemberId;
    /** What the member paid, tip included. */
    readonly paid: Money;
    /** The member's share, tip included. */
    readonly share: Money;
}

/** How one expense divides, in the group currency. */
export interface ExpenseShares {
    /** What every member paid and consumed. */
    readonly members: readonly MemberShare[];
    /** The tip. */
    readonly tip: Money;
    /** The total: the amount and the tip. */
    readonly total: Money;
}

/** How balances turn into payments. */
export enum SettlementStrategy {
    /** Everyone pays back the people who paid for them. */
    DIRECT = 'DIRECT',
    /** The largest debtor pays the largest creditor, repeatedly. */
    GREEDY = 'GREEDY',
    /** The fewest payments possible. */
    MINIMUM_TRANSFERS = 'MINIMUM_TRANSFERS',
}

/** One payment of a settlement: one member pays another an amount. */
export interface Payment {
    /** The amount, in the group currency. */
    readonly amount: Money;
    /** The member who pays. */
    readonly from: MemberId;
    /** The member who is paid. */
    readonly to: MemberId;
}

/** The payments that settle every balance. */
export interface Settlement {
    /** The payments, ordered by payer and then by recipient. */
    readonly payments: readonly Payment[];
    /** The strategy that produced them. */
    readonly strategy: SettlementStrategy;
}

/**
 * `balance` as the library writes it: "gets back 12.00 EUR", "owes 12.00 EUR", or "settled".
 *
 * @param balance - The balance
 * @returns The text
 */
export function formatBalance(balance: Balance): string {
    const generated = generatedBalance(balance);
    // Only an amount can hold a currency the library cannot format, and a settled balance holds none.
    if ('settled' === balance.kind) {
        return internal.balanceFormat(generated);
    }
    try {
        return internal.balanceFormat(generated);
    } catch {
        return `${balance.kind} ${balance.amount.toString()}`;
    }
}

/** The balance as the generated code holds it. */
function generatedBalance(balance: Balance): internal.Balance {
    switch (balance.kind) {
        case 'getsBack':
            return new internal.Balance.GetsBack({ amount: balance.amount.generated() });
        case 'owes':
            return new internal.Balance.Owes({ amount: balance.amount.generated() });
        case 'settled':
            return new internal.Balance.Settled();
    }
}

/** The balance the generated code holds as `balance`. */
function convertedBalance(balance: internal.Balance): Balance {
    switch (balance.tag) {
        case internal.Balance_Tags.GetsBack:
            return { kind: 'getsBack', amount: Money.converted(balance.inner.amount) };
        case internal.Balance_Tags.Owes:
            return { kind: 'owes', amount: Money.converted(balance.inner.amount) };
        case internal.Balance_Tags.Settled:
            return { kind: 'settled' };
    }
}

/** The category's total the generated code holds as `total`. */
function convertedCategoryTotal(total: internal.CategoryTotal): CategoryTotal {
    return { category: categoryId(total.category), total: Money.converted(total.total) };
}

/**
 * The totals the generated code holds as `summary`.
 *
 * @internal
 */
export function convertedSummary(summary: internal.Summary): Summary {
    return {
        categoryTotals: summary.categoryTotals.map(convertedCategoryTotal),
        memberCategoryTotals: summary.memberCategoryTotals.map((totals) => ({
            member: memberId(totals.member),
            totals: totals.totals.map(convertedCategoryTotal),
        })),
        members: summary.members.map((member) => ({
            balance: convertedBalance(member.balance),
            member: memberId(member.member),
            net: Money.converted(member.net),
            paid: Money.converted(member.paid),
            received: Money.converted(member.received),
            sent: Money.converted(member.sent),
            share: Money.converted(member.share),
        })),
        totalSpent: Money.converted(summary.totalSpent),
    };
}

/**
 * The expense's shares the generated code holds as `shares`.
 *
 * @internal
 */
export function convertedExpenseShares(shares: internal.ExpenseShares): ExpenseShares {
    return {
        members: shares.members.map((share) => ({
            member: memberId(share.member),
            paid: Money.converted(share.paid),
            share: Money.converted(share.share),
        })),
        tip: Money.converted(shares.tip),
        total: Money.converted(shares.total),
    };
}

/**
 * The strategy as the generated code holds it.
 *
 * @internal
 */
export function generatedStrategy(strategy: SettlementStrategy): internal.SettlementStrategy {
    switch (strategy) {
        case SettlementStrategy.DIRECT:
            return internal.SettlementStrategy.Direct;
        case SettlementStrategy.GREEDY:
            return internal.SettlementStrategy.Greedy;
        case SettlementStrategy.MINIMUM_TRANSFERS:
            return internal.SettlementStrategy.MinimumTransfers;
    }
}

/** The strategy the generated code holds as `strategy`. */
function convertedStrategy(strategy: internal.SettlementStrategy): SettlementStrategy {
    switch (strategy) {
        case internal.SettlementStrategy.Direct:
            return SettlementStrategy.DIRECT;
        case internal.SettlementStrategy.Greedy:
            return SettlementStrategy.GREEDY;
        case internal.SettlementStrategy.MinimumTransfers:
            return SettlementStrategy.MINIMUM_TRANSFERS;
    }
}

/**
 * The settlement the generated code holds as `settlement`.
 *
 * @internal
 */
export function convertedSettlement(settlement: internal.Settlement): Settlement {
    return {
        payments: settlement.payments.map((payment) => ({
            amount: Money.converted(payment.amount),
            from: memberId(payment.from),
            to: memberId(payment.to),
        })),
        strategy: convertedStrategy(settlement.strategy),
    };
}
