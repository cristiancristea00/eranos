//! The binding core's records, enums, and error, stamped for `uniffi` from
//! the mirrors the core's items carry: each copy repeats its item field for
//! field, and `uniffi` derives its conversions from the copy while the core
//! keeps the type.

use eranos_bindings::{
    Account, AccountDetails, AccountFailure, Balance, CalendarDate, CalendarFailure, Category,
    CategoryFailure, CategoryTotal, ContactFailure, Cost, CurrencyDetails, CurrencyRate,
    EranosError, Expense, ExpenseFailure, ExpenseShares, ExpenseSplit, GroupFailure, IbanReason,
    IdFailure, Item, LanguageFailure, Member, MemberAmount, MemberCategoryTotals, MemberFailure,
    MemberParts, MemberPercentage, MemberShare, MemberSummary, Money, MoneyFailure, NewGroup,
    Payment, Settlement, SettlementFailure, SettlementStrategy, Split, Summary, TimeOfDay, Tip,
    TipDistribution, TipValue, Transfer, TransferFailure,
};

eranos_bindings::mirrors!(::eranos_glue_macros::uniffi_mirror);
