//! Money, currencies, exchange rates, costs, and the ratios splits are made of.
//!
//! Amounts are whole numbers of minor units (cents): arithmetic is exact, and
//! every split distributes its leftover minor units so parts always add up,
//! by the rules of the `allocation` module.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use.
#![doc(test(attr(deny(warnings))))]

pub mod allocation;
mod amount;
mod cost;
mod currency;
mod error;
mod rate;
mod ratio;

pub use amount::Money;
pub use cost::Cost;
pub use currency::Currency;
pub use error::MoneyError;
pub use rate::{CurrencyRate, ExchangeRate};
pub use ratio::{Parts, Percentage};
