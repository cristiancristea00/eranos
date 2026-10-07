/**
 * The eranos library for the browser and Node, through WebAssembly: groups that share expenses, their members, entries,
 * totals, and settlements.
 *
 * Call {@link init} with the WebAssembly module first, once, before any other call:
 *
 * ```ts
 * import { Currency, Group, init } from 'eranos';
 *
 * await init(new URL('eranos/eranos_web.wasm', import.meta.url));
 * const group = Group.create('Flat', Currency.of('EUR'));
 * ```
 *
 * @packageDocumentation
 */

export { CalendarDate, TimeOfDay } from './calendar.js';
export { Email, Iban, Language, PhoneNumber, Username } from './contact.js';
export type { CallOptions } from './converting.js';
export { Currency } from './currency.js';
export {
    type Account,
    type AccountDetails,
    type Category,
    type Cost,
    type Expense,
    type ExpenseSplit,
    type Item,
    type Member,
    type MemberAmount,
    type MemberParts,
    type MemberPercentage,
    type Split,
    type Tip,
    type TipValue,
    type Transfer,
    TipDistribution,
    expenseMembers,
    expenseTipAmount,
    expenseTotal,
    splitMembers,
} from './entries.js';
export {
    type AccountFailure,
    type CalendarFailure,
    type CategoryFailure,
    type ContactFailure,
    type ExpenseFailure,
    type GroupFailure,
    type IbanReason,
    type IdFailure,
    type LanguageFailure,
    type MemberFailure,
    type MoneyFailure,
    type SettlementFailure,
    type TransferFailure,
    AccountError,
    CalendarError,
    CategoryError,
    ContactError,
    EranosError,
    ExpenseError,
    GroupError,
    IdError,
    InvalidJsonError,
    LanguageError,
    MemberError,
    MoneyError,
    SettlementError,
    TransferError,
} from './errors.js';
export { type CurrencyRate, Group, type GroupOptions } from './group.js';
export { GroupReader } from './group_reader.js';
export {
    type CategoryId,
    type ExpenseId,
    type GroupId,
    type MemberId,
    type TransferId,
    categoryId,
    expenseId,
    groupId,
    memberId,
    transferId,
} from './identifiers.js';
export { init } from './init.js';
export { Money } from './money.js';
export { ExchangeRate, Parts, Percentage } from './ratios.js';
export {
    type Balance,
    type CategoryTotal,
    type ExpenseShares,
    type MemberCategoryTotals,
    type MemberShare,
    type MemberSummary,
    type Payment,
    type Settlement,
    type Summary,
    SettlementStrategy,
    formatBalance,
} from './totals.js';
