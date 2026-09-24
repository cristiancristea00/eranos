//! Group expense tracking: who paid what, who owes whom, and the fewest
//! payments that settle everyone up.
//!
//! A [`Group`] (a trip, a day out, a household) holds [`Member`]s, the
//! [`Expense`]s they share, and the [`Transfer`]s they make to pay each other
//! back. Every change updates the group's [`Summary`] in place: total
//! spending, what each member paid and consumed, their [`Balance`], and
//! totals per category and per member per category. A [`Settlement`] turns
//! the balances into payments with the [`SettlementStrategy`] the caller
//! chooses.
//!
//! # Quick start
//!
//! ```
//! use eranos::{
//!     Balance, Currency, CalendarDate, Expense, Group, Member, Money, Percentage, SettlementStrategy,
//!     Split, Tip,
//! };
//!
//! let mut group = Group::builder(Currency::EUR, "Lisbon").build()?;
//! let ana = group.add_member(Member::builder("Ana").build()?)?;
//! let bob = group.add_member(Member::builder("Bob").build()?)?;
//! let carol = group.add_member(Member::builder("Carol").build()?)?;
//!
//! // Ana paid dinner for three, plus a 10 % tip.
//! let (amount, day) = (Money::parse("90.00", Currency::EUR)?, CalendarDate::new(23, 9, 2026)?);
//! let dinner = Expense::builder(amount, day, "Dinner", ana, Split::equal([ana, bob, carol]))
//!     .tip(Tip::percentage(Percentage::parse("10")?))
//!     .build()?;
//! group.add_expense(dinner)?;
//!
//! let summary = group.summary();
//! assert_eq!(summary.total_spent(), Money::parse("99.00", Currency::EUR)?);
//! assert_eq!(
//!     summary.member(bob).balance(),
//!     Balance::Owes(Money::parse("33.00", Currency::EUR)?)
//! );
//!
//! // Bob and Carol each pay Ana back.
//! let settlement = group.settlement(SettlementStrategy::MinimumTransfers)?;
//! assert_eq!(settlement.len(), 2);
//! for payment in settlement.payments() {
//!     assert_eq!(payment.to(), ana);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Money is exact
//!
//! Amounts are whole numbers of minor units ([`Money`]), never floats. Every
//! division, whether a [`Split`] among members, a [`Tip`], an [`Item`] of an
//! itemised bill, or a conversion at an [`ExchangeRate`], hands out its
//! leftover minor units by the largest-remainder method, so the parts always
//! add up to the whole. Ties rotate with the expense's identifier: over many
//! expenses, the extra cent of an even split does not always land on the
//! same person, and a synced expense splits the same way on every device.
//!
//! # Consistency and caching
//!
//! A group checks every entry against its members, categories, and currency,
//! and a change that fails leaves it exactly as it was. The [`Summary`] is
//! kept current by every change in time proportional to the entry's size;
//! settlements are computed on first request and cached until a balance
//! changes.
//!
//! # Async calls and sharing
//!
//! The slow calls also come as futures that yield while they work, so an
//! app's other tasks keep running: [`Group::settlement_async`],
//! [`Group::change_currency_async`], and `SavedGroup::into_group_async`.
//! They need no particular executor, and
//! [`Cooperative::complete`](cooperative::Cooperative::complete) runs one on
//! the calling thread, which is what the synchronous calls do. A
//! [`SharedGroup`] shares a group between tasks and threads behind a
//! read-write lock; each of its calls is a future an executor can run on any
//! thread, and [`block_on`](cooperative::block_on) waits for one from
//! synchronous code.
//!
//! ```
//! use eranos::cooperative::block_on;
//! use eranos::{
//!     CalendarDate, Currency, Expense, Group, Member, Money, SettlementStrategy, SharedGroup,
//!     Split,
//! };
//!
//! let shared = SharedGroup::new(Group::builder(Currency::EUR, "Flat").build()?);
//! let ana = block_on(shared.add_member(Member::builder("Ana").build()?))?;
//! let bob = block_on(shared.add_member(Member::builder("Bob").build()?))?;
//!
//! // Ana pays the rent; a clone shares the group, as another task would.
//! let (amount, day) = (Money::parse("900.00", Currency::EUR)?, CalendarDate::new(1, 10, 2026)?);
//! let rent = Expense::builder(amount, day, "Rent", ana, Split::equal([ana, bob])).build()?;
//! let other = shared.clone();
//! block_on(other.add_expense(rent))?;
//!
//! let settlement = block_on(shared.settlement(SettlementStrategy::MinimumTransfers))?;
//! assert_eq!(settlement.payments()[0].from(), bob);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! - `serde`: `Serialize` and `Deserialize` for [`Group`] and every type it
//!   stores: entries, members, categories, identifiers, and the values in
//!   them; what a group derives, such as its [`Summary`] and settlements,
//!   does not serialise. A deserialised [`Group`] is rebuilt through the
//!   same checks as its `add_*` methods; a `SavedGroup` does the same in its
//!   `into_group`, which reports a refused file as a
//!   [`GroupError`](error::GroupError). Account details serialise in clear
//!   text, so store serialised groups in the platform's secure storage.
//! - `sequential-ids`: identifiers count up from one instead of reading the
//!   clock and the random source, so a program that builds the same entries
//!   in the same order prints the same identifiers on every run. It is for
//!   examples and tests: identifiers from two devices would collide.
//!
//! # Examples
//!
//! The `examples/` directory has runnable scenarios: `quickstart`,
//! `split_methods`, `restaurant_bill`, `multi_currency`, `settling_up`,
//! `members_and_accounts`, and, with `--features serde`, `persistence` and
//! `grand_tour`, which uses every feature. Run one with
//! `cargo run -p eranos --example settling_up`.
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
    pub use eranos_account::{AccountError, IbanError};
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
    pub use eranos_group::{GroupError, LanguageError};
    #[doc(inline)]
    pub use eranos_member::MemberError;
    #[doc(inline)]
    pub use eranos_money::MoneyError;
    #[doc(inline)]
    pub use eranos_settlement::SettlementError;
    #[doc(inline)]
    pub use eranos_transfer::TransferError;
}
pub mod expense {
    //! Expenses: what was bought, who paid, and how the cost is shared.

    #[doc(inline)]
    pub use eranos_expense::{
        Expense, ExpenseBuilder, ExpenseSplit, Item, ItemBuilder, MemberAmount, MemberParts,
        MemberPercentage, Split, Tip, TipDistribution, TipValue,
    };
}
pub mod group {
    //! Groups: the members, categories, expenses, and transfers of a trip, a
    //! household, or any shared spending, with their totals kept current.

    #[cfg(feature = "serde")]
    #[doc(inline)]
    pub use eranos_group::SavedGroup;
    #[doc(inline)]
    pub use eranos_group::{
        Group, GroupBuilder, GroupReadGuard, GroupWriteGuard, Language, SharedGroup,
    };
}
pub mod id {
    //! Identifiers for groups and their entries.
    //!
    //! Every identifier is a UUID v7: unique across devices, so entries
    //! created offline on two phones never collide, and ordered by creation
    //! time. With the `sequential-ids` feature, the identifiers of a process
    //! count up from one instead, so a program that builds the same entries
    //! in the same order gets the same identifiers on every run. The feature
    //! is for examples and tests: two devices would generate the same
    //! identifiers.

    #[doc(inline)]
    pub use eranos_core::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
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
    pub use eranos_account::{Account, AccountDetails, AccountKind, Iban, Username};
    #[doc(inline)]
    pub use eranos_contact::{Email, PhoneNumber};
    #[doc(inline)]
    pub use eranos_member::{Member, MemberBuilder};
}
pub mod money {
    //! Money, currencies, exchange rates, costs, and the ratios splits are
    //! made of.
    //!
    //! Amounts are whole numbers of minor units (cents): arithmetic is exact,
    //! and every split distributes its leftover minor units so parts always
    //! add up.

    #[doc(inline)]
    pub use eranos_money::{Cost, Currency, CurrencyRate, ExchangeRate, Money, Parts, Percentage};
}
pub mod settlement {
    //! Settling up: turning balances into payments between members.

    #[doc(inline)]
    pub use eranos_settlement::{Payment, Settlement, SettlementStrategy};
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
    Expense, ExpenseSplit, Item, MemberAmount, MemberParts, MemberPercentage, Split, Tip,
    TipDistribution, TipValue,
};
#[cfg(feature = "serde")]
#[doc(no_inline)]
pub use group::SavedGroup;
#[doc(no_inline)]
pub use group::{Group, Language, SharedGroup};
#[doc(no_inline)]
pub use id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
#[doc(no_inline)]
pub use ledger::{Balance, CategoryTotal, ExpenseShares, MemberShare, MemberSummary, Summary};
#[doc(no_inline)]
pub use member::{
    Account, AccountDetails, AccountKind, Email, Iban, Member, PhoneNumber, Username,
};
#[doc(no_inline)]
pub use money::{Cost, Currency, CurrencyRate, ExchangeRate, Money, Parts, Percentage};
#[doc(no_inline)]
pub use settlement::{Payment, Settlement, SettlementStrategy};
#[doc(no_inline)]
pub use transfer::Transfer;

/// Compiles the README's Rust code blocks as doctests, so its examples
/// cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeDoctests;
