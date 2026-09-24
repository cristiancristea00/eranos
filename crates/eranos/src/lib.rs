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
pub mod category;
pub mod error;
pub mod expense;
pub mod group;
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
pub mod ledger;
pub mod member;
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
pub mod settlement;
pub mod transfer;

#[doc(no_inline)]
pub use calendar::{CalendarDate, TimeOfDay};
pub use category::Category;
pub use expense::{
    Expense, ExpenseSplit, Item, MemberAmount, MemberParts, MemberPercentage, Split, Tip,
    TipDistribution, TipValue,
};
#[cfg(feature = "serde")]
pub use group::SavedGroup;
pub use group::{Group, Language};
#[doc(no_inline)]
pub use id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
pub use ledger::{Balance, CategoryTotal, ExpenseShares, MemberShare, MemberSummary, Summary};
#[doc(no_inline)]
pub use member::{
    Account, AccountDetails, AccountKind, Email, Iban, Member, PhoneNumber, Username,
};
#[doc(no_inline)]
pub use money::{Cost, Currency, CurrencyRate, ExchangeRate, Money, Parts, Percentage};
pub use settlement::{Payment, Settlement, SettlementStrategy};
pub use transfer::Transfer;

/// Compiles the README's Rust code blocks as doctests, so its examples
/// cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeDoctests;
