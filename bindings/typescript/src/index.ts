/**
 * The eranos library for the browser and Node, through WebAssembly: groups that share expenses, their members, entries,
 * totals, and settlements.
 *
 * Call {@link init} with the WebAssembly module first, once, before any other call:
 *
 * ```ts
 * import { CurrencyCode, ExpenseGroup, init } from 'eranos';
 *
 * await init(new URL('eranos/eranos_web.wasm', import.meta.url));
 * const group = ExpenseGroup.create('Flat', CurrencyCode.of('EUR'));
 * ```
 *
 * @packageDocumentation
 */

export { CalendarDate, TimeOfDay } from './calendar.js';
export { Email, Iban, Language, PhoneNumber, Username } from './contact.js';
export type { CallOptions } from './converting.js';
export { CurrencyCode } from './currency_code.js';
export {
    type PaymentAccount,
    type PaymentAccountDetails,
    type Category,
    type Cost,
    type Discount,
    type DiscountInput,
    type DiscountValue,
    type Expense,
    type ExpenseInput,
    type ExpenseSplit,
    type ExpenseSplitInput,
    type ExpenseItem,
    type ExpenseItemInput,
    type Member,
    type MemberInput,
    type MemberAmount,
    type MemberParts,
    type MemberPercentage,
    type Split,
    type TipRule,
    type TipRuleInput,
    type TipValue,
    type Transfer,
    DiscountScope,
    NO_TIP,
    TipDistribution,
    createCategory,
    createExpense,
    createMember,
    createTransfer,
    expenseItemDiscountAmount,
    expenseItemLineTotal,
    expenseItemLineTotalBeforeDiscount,
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
    MismatchedField,
    MismatchedRecordError,
    MoneyError,
    SettlementError,
    TransferError,
} from './errors.js';
export { ExpenseGroup, type ExpenseGroupOptions } from './expense_group.js';
export { ExpenseGroupReader } from './expense_group_reader.js';
export { ExpenseGroupWriter } from './expense_group_writer.js';
export {
    type CategoryId,
    type ExpenseId,
    type ExpenseGroupId,
    type MemberId,
    type TransferId,
    UNCATEGORISED,
    categoryId,
    expenseId,
    expenseGroupId,
    generateCategoryId,
    generateExpenseGroupId,
    generateExpenseId,
    generateMemberId,
    generateTransferId,
    memberId,
    parseCategoryId,
    parseExpenseId,
    parseExpenseGroupId,
    parseMemberId,
    parseTransferId,
    transferId,
} from './identifiers.js';
export { init } from './init.js';
export { Money } from './money.js';
export { type CurrencyRate, ExchangeRate, Parts, Percentage, Quantity } from './ratios.js';
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
