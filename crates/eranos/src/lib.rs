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
//!     Balance, Currency, Date, Expense, Group, Member, Money, Percentage, SettlementStrategy,
//!     Split, Tip,
//! };
//!
//! let mut group = Group::builder(Currency::EUR, "Lisbon").build()?;
//! let ana = group.add_member(Member::builder("Ana").build()?);
//! let bob = group.add_member(Member::builder("Bob").build()?);
//! let carol = group.add_member(Member::builder("Carol").build()?);
//!
//! // Ana paid dinner for three, plus a 10 % tip.
//! let amount = Money::parse("90.00", Currency::EUR)?;
//! let dinner = Expense::builder(amount, Date::new(23, 9, 2026)?, "Dinner")
//!     .paid_by(ana)
//!     .split(Split::equal([ana, bob, carol]))
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
//! - `serde`: `Serialize` and `Deserialize` for every domain type. A
//!   deserialised [`Group`] is rebuilt through the same checks as its insert
//!   methods. Account details serialise in clear text, so store serialised
//!   groups in the platform's secure storage.
//!
//! # Examples
//!
//! The `examples/` directory has runnable scenarios: `quickstart`,
//! `split_methods`, `restaurant_bill`, `multi_currency`, `settling_up`,
//! `members_and_accounts`, and `persistence` (with `--features serde`). Run
//! one with `cargo run -p eranos --example settling_up`.
#![doc(test(attr(deny(warnings))))]

pub mod category;
pub mod date;
pub mod error;
pub mod expense;
pub mod group;
pub mod id;
pub mod ledger;
pub mod member;
pub mod money;
pub mod settlement;
pub mod transfer;

mod resolution;
mod secret;
#[cfg(feature = "serde")]
mod serde_text;
mod text;

pub use category::Category;
pub use date::{Date, Time};
pub use expense::{Expense, ExpenseSplit, Item, Split, Tip, TipDistribution, TipValue};
pub use group::{Group, Language};
pub use id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
pub use ledger::{Balance, ExpenseShares, MemberShare, MemberSummary, Summary};
pub use member::{
    Account, AccountDetails, AccountKind, Email, Iban, Member, PhoneNumber, Username,
};
pub use money::{Cost, Currency, ExchangeRate, Money, Parts, Percentage};
pub use settlement::{Payment, Settlement, SettlementStrategy};
pub use transfer::Transfer;

/// Compiles the README's Rust code blocks as doctests, so its examples
/// cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../../../README.md")]
struct ReadmeDoctests;
