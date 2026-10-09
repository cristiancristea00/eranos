#![doc = include_str!("../../../README.md")]
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

pub mod calendar {
    //! Calendar dates and times of day, without time zones.
    //!
    //! The library never reads the clock: every date comes from the caller,
    //! which keeps the core deterministic and usable on wasm, where the
    //! system clock is unavailable.

    #[doc(inline)]
    pub use eranos_calendar::{CalendarDate, TimeOfDay};
}
pub mod category {
    //! Expense categories, defined per group.

    #[doc(inline)]
    pub use eranos_category::Category;
}
pub mod cooperative {
    //! Driving the library's asynchronous calls.
    //!
    //! A long computation, such as the exact settlement or a currency change,
    //! is a [`Cooperative`] future: it pends only by yielding, so it runs to
    //! completion on the calling thread with [`Cooperative::complete`], which
    //! the synchronous calls use, or in any executor, where each yield lets
    //! other tasks run. [`block_on`] drives a future that waits for another
    //! thread, such as one for a shared group's lock.

    #[doc(inline)]
    pub use eranos_core::cooperative::Cooperative;
    #[cfg(not(target_family = "wasm"))]
    #[doc(inline)]
    pub use eranos_core::cooperative::block_on;
}
pub mod error {
    //! Error types, one per module: every function reports the failure of
    //! its own module, and no error wraps another.
    //!
    //! No error echoes the text it rejected: inputs include account numbers
    //! and contact details, and error messages end up in logs.

    #[doc(inline)]
    pub use eranos_calendar::CalendarError;
    #[doc(inline)]
    pub use eranos_category::CategoryError;
    #[doc(inline)]
    pub use eranos_contact::ContactError;
    #[doc(inline)]
    pub use eranos_core::IdError;
    #[doc(inline)]
    pub use eranos_expense::ExpenseError;
    #[doc(inline)]
    pub use eranos_expense_group::{ExpenseGroupError, LanguageError};
    #[doc(inline)]
    pub use eranos_member::MemberError;
    #[doc(inline)]
    pub use eranos_money::MoneyError;
    #[doc(inline)]
    pub use eranos_payment_account::{IbanError, PaymentAccountError};
    #[doc(inline)]
    pub use eranos_settlement::SettlementError;
    #[doc(inline)]
    pub use eranos_transfer::TransferError;
}
pub mod expense {
    //! Expenses: what was bought, who paid, and how the cost is shared.

    #[doc(inline)]
    pub use eranos_expense::{
        Discount, DiscountScope, DiscountValue, Expense, ExpenseBuilder, ExpenseItem,
        ExpenseItemBuilder, ExpenseSplit, MemberAmount, MemberParts, MemberPercentage, Split,
        TipDistribution, TipRule, TipValue,
    };
}
pub mod expense_group {
    //! Groups: the members, categories, expenses, and transfers of a trip, a
    //! household, or any shared spending, with their totals kept current.

    #[cfg(feature = "serde")]
    #[doc(inline)]
    pub use eranos_expense_group::SavedExpenseGroup;
    #[doc(inline)]
    pub use eranos_expense_group::{
        ExpenseGroup, ExpenseGroupBuilder, ExpenseGroupReadGuard, ExpenseGroupWriteGuard, Language,
        SharedExpenseGroup,
    };
}
pub mod id {
    //! Identifiers for groups and their entries.
    //!
    //! An identifier is text, stored as it is given: at most 64 bytes of
    //! UTF-8, not empty, without control characters, and without whitespace
    //! at either end. Two identifiers are equal when their bytes are, so
    //! case matters, and identifiers order by their bytes. A group and every
    //! entry have an identifier from the moment their constructor is
    //! called: the constructors with the plain name, such as
    //! [`Member::builder`](crate::Member::builder) and
    //! [`Category::new`](crate::Category::new), generate one, and their
    //! twins, such as [`Member::builder_with_id`](crate::Member::builder_with_id)
    //! and [`Category::with_id`](crate::Category::with_id), take the app's
    //! own, such as the key of its store. A generated identifier is a UUID v7
    //! written as lowercase hyphenated text: unique across devices, so
    //! entries created offline on two phones never collide, and in creation
    //! order. A saved group loads under the identifiers it saved, through the
    //! twins, so loading generates none. With the
    //! `sequential-ids` feature, the identifiers of a process count up from
    //! one instead, so a program that generates its identifiers in the same
    //! order gets the same ones on every run. The feature is for examples and
    //! tests: two devices would generate the same identifiers.

    #[doc(inline)]
    pub use eranos_core::{CategoryId, ExpenseGroupId, ExpenseId, MemberId, TransferId};
}
pub mod ledger {
    //! The group's totals and balances, kept current with every change.

    #[doc(inline)]
    pub use eranos_ledger::{
        Balance, CategoryTotal, ExpenseShares, MemberShare, MemberSummary, Summary,
    };
}
pub mod member {
    //! The people in a group, their contact details, and where they can be
    //! paid.

    #[doc(inline)]
    pub use eranos_contact::{Email, PhoneNumber};
    #[doc(inline)]
    pub use eranos_member::{Member, MemberBuilder};
    #[doc(inline)]
    pub use eranos_payment_account::{
        Iban, PaymentAccount, PaymentAccountDetails, PaymentAccountKind, Username,
    };
}
pub mod money {
    //! Money, currencies, exchange rates, costs, the ratios splits are made
    //! of, and the quantities of a bill's items.
    //!
    //! Amounts are whole numbers of minor units (cents): arithmetic is exact,
    //! and every split distributes its leftover minor units so parts always
    //! add up.

    #[doc(inline)]
    pub use eranos_money::{
        Cost, CurrencyCode, CurrencyRate, ExchangeRate, Money, Parts, Percentage, Quantity,
    };
}
pub mod settlement {
    //! Settling up: turning balances into payments between members.

    #[doc(inline)]
    pub use eranos_settlement::{Payment, SettlementPlan, SettlementStrategy};
}
pub mod transfer {
    //! Transfers: money one member paid another, such as a repayment.

    #[doc(inline)]
    pub use eranos_transfer::{Transfer, TransferBuilder};
}

#[doc(no_inline)]
pub use calendar::{CalendarDate, TimeOfDay};
#[doc(no_inline)]
pub use category::Category;
#[doc(no_inline)]
pub use expense::{
    Discount, DiscountScope, DiscountValue, Expense, ExpenseItem, ExpenseSplit, MemberAmount,
    MemberParts, MemberPercentage, Split, TipDistribution, TipRule, TipValue,
};
#[cfg(feature = "serde")]
#[doc(no_inline)]
pub use expense_group::SavedExpenseGroup;
#[doc(no_inline)]
pub use expense_group::{ExpenseGroup, Language, SharedExpenseGroup};
#[doc(no_inline)]
pub use id::{CategoryId, ExpenseGroupId, ExpenseId, MemberId, TransferId};
#[doc(no_inline)]
pub use ledger::{Balance, CategoryTotal, ExpenseShares, MemberShare, MemberSummary, Summary};
#[doc(no_inline)]
pub use member::{
    Email, Iban, Member, PaymentAccount, PaymentAccountDetails, PaymentAccountKind, PhoneNumber,
    Username,
};
#[doc(no_inline)]
pub use money::{
    Cost, CurrencyCode, CurrencyRate, ExchangeRate, Money, Parts, Percentage, Quantity,
};
#[doc(no_inline)]
pub use settlement::{Payment, SettlementPlan, SettlementStrategy};
#[doc(no_inline)]
pub use transfer::Transfer;
