//! Money, currencies, exchange rates, costs, the ratios splits are made of,
//! and the quantities of a bill's items.
//!
//! Amounts are whole numbers of minor units (cents): arithmetic is exact, and
//! every split distributes its leftover minor units so parts always add up,
//! by the rules of the `allocation` module.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromCost`] and
//! [`FromMoneyError`], through which a crate that tells every variant apart
//! converts a cost or a failure without the wildcard arm a `match` outside
//! this crate would need, nor [`format_minor_units`], [`format_basis_points`],
//! [`format_hundredths`], and [`format_ten_thousandths`], with which the
//! binding core writes an amount from the currency details its record
//! carries, and a percentage, parts, and a quantity from the numbers their
//! records hold.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

pub mod allocation;
mod amount;
mod cost;
mod currency_code;
mod decimal;
mod error;
mod quantity;
mod rate;
mod ratio;

pub use amount::{Money, format_minor_units};
pub use cost::{Cost, FromCost};
pub use currency_code::CurrencyCode;
pub use error::{FromMoneyError, MoneyError};
pub use quantity::{Quantity, format_ten_thousandths};
pub use rate::{CurrencyRate, ExchangeRate};
pub use ratio::{Parts, Percentage, format_basis_points, format_hundredths};
