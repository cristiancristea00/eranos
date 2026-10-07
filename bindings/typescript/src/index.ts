/**
 * The eranos library for the browser and Node, through WebAssembly: groups that share expenses, their members, entries,
 * totals, and settlements.
 *
 * Call {@link init} with the WebAssembly module first, once, before any other call:
 *
 * ```ts
 * import { Currency, ExpenseGroup, init } from 'eranos';
 *
 * await init(new URL('eranos/eranos_web.wasm', import.meta.url));
 * const group = ExpenseGroup.create('Flat', Currency.of('EUR'));
 * ```
 *
 * @packageDocumentation
 */

export { CalendarDate, TimeOfDay } from './calendar.js';
export { Email, Iban, Language, PhoneNumber, Username } from './contact.js';
export type { CallOptions } from './converting.js';
export { Currency } from './currency.js';
export {
    type PaymentAccount,
    type PaymentAccountDetails,
    type Category,
    type Cost,
    type Expense,
    type ExpenseSplit,
    type ExpenseItem,
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
    type PaymentAccountFailure,
    type CalendarFailure,
    type CategoryFailure,
    type ContactFailure,
    type ExpenseFailure,
    type ExpenseGroupFailure,
    type IbanReason,
    type IdFailure,
    type LanguageFailure,
    type MemberFailure,
    type MoneyFailure,
    type SettlementFailure,
    type TransferFailure,
    PaymentAccountError,
    CalendarError,
    CategoryError,
    ContactError,
    EranosError,
    ExpenseError,
    ExpenseGroupError,
    IdError,
    InvalidJsonError,
    LanguageError,
    MemberError,
    MoneyError,
    SettlementError,
    TransferError,
} from './errors.js';
export { type CurrencyRate, ExpenseGroup, type ExpenseGroupOptions } from './expense_group.js';
export { ExpenseGroupReader } from './expense_group_reader.js';
export {
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
    type SettlementPlan,
    type Summary,
    SettlementStrategy,
    formatBalance,
} from './totals.js';
