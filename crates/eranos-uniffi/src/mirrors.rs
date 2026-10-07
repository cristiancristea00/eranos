//! The binding core's records, enums, and error, stamped for `uniffi` from
//! the mirrors the core's items carry: each copy repeats its item field for
//! field, and `uniffi` derives its conversions from the copy while the core
//! keeps the type.

use eranos_bindings::{
    Balance, CalendarDate, CalendarFailure, Category, CategoryFailure, CategoryTotal,
    ContactFailure, Cost, CurrencyDetails, CurrencyRate, EranosError, Expense, ExpenseFailure,
    ExpenseGroupFailure, ExpenseShares, ExpenseSplit, IbanReason, IdFailure, Item, LanguageFailure,
    Member, MemberAmount, MemberCategoryTotals, MemberFailure, MemberParts, MemberPercentage,
    MemberShare, MemberSummary, Money, MoneyFailure, NewExpenseGroup, Payment, PaymentAccount,
    PaymentAccountDetails, PaymentAccountFailure, SettlementFailure, SettlementPlan,
    SettlementStrategy, Split, Summary, TimeOfDay, Tip, TipDistribution, TipValue, Transfer,
    TransferFailure,
};

eranos_bindings::mirrors!(::eranos_glue_macros::uniffi_mirror);
