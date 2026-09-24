//! Money, currencies, exchange rates, costs, and the ratios splits are made of.
//!
//! Amounts are whole numbers of minor units (cents): arithmetic is exact, and
//! every split distributes its leftover minor units so parts always add up.

pub(crate) mod allocation;
mod amount;
mod cost;
mod currency;
mod rate;
mod ratio;

pub use amount::Money;
pub use cost::Cost;
pub use currency::Currency;
pub use rate::{CurrencyRate, ExchangeRate};
pub use ratio::{Parts, Percentage};
