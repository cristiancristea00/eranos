//! Receipt lines of an itemised expense.

use crate::error::{ExpenseError, Field};
use crate::expense::{Choice, Split};
use crate::id::MemberId;
use crate::money::Money;
use crate::text;

/// One line of an itemised expense: a unit price times a quantity, divided
/// among its own members by its own [`Split`].
///
/// The line totals of an expense's items must add up to exactly the
/// expense's amount.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Item, Money, Split};
/// use eranos::id::MemberId;
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let beers = Item::builder("Beer", Money::parse("4.50", Currency::EUR)?)
///     .quantity(3)
///     .shared_by([ana, bob])
///     .build()?;
/// assert_eq!(beers.line_total(), Money::parse("13.50", Currency::EUR)?);
///
/// let pizza = Item::builder("Pizza", Money::parse("12.00", Currency::EUR)?)
///     .split(Split::equal([ana]))
///     .build()?;
/// assert_eq!(pizza.quantity(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ItemRecord")
)]
pub struct Item {
    name: String,
    split: Split,
    unit_price: Money,
    quantity: u32,
}

impl Item {
    /// Starts an item with its name and unit price; the quantity defaults
    /// to 1.
    pub fn builder(name: impl Into<String>, unit_price: Money) -> ItemBuilder {
        ItemBuilder {
            name: name.into(),
            split: Choice::Unset,
            unit_price,
            quantity: 1,
        }
    }

    /// Starts a builder holding a copy of this item, to edit it.
    pub fn to_builder(&self) -> ItemBuilder {
        ItemBuilder {
            name: self.name.clone(),
            split: Choice::Inherited(self.split.clone()),
            unit_price: self.unit_price,
            quantity: self.quantity,
        }
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How the line total is divided.
    #[must_use]
    pub const fn split(&self) -> &Split {
        &self.split
    }

    /// The price of one unit.
    #[must_use]
    pub const fn unit_price(&self) -> Money {
        self.unit_price
    }

    /// The number of units, at least 1.
    #[must_use]
    pub const fn quantity(&self) -> u32 {
        self.quantity
    }

    /// The unit price times the quantity.
    #[must_use]
    pub fn line_total(&self) -> Money {
        // `build` rejected any item whose line total overflows.
        let total = self
            .unit_price
            .minor_units()
            .saturating_mul(i64::from(self.quantity));
        Money::from_minor_units(total, self.unit_price.currency())
    }
}

/// Builds an [`Item`]; created by [`Item::builder`] or [`Item::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ItemBuilder {
    name: String,
    split: Choice<Split>,
    unit_price: Money,
    quantity: u32,
}

impl ItemBuilder {
    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Divides the line total equally among `members`; the same as
    /// `split(Split::equal(members))`, and an error together with it.
    pub fn shared_by(mut self, members: impl IntoIterator<Item = MemberId>) -> Self {
        self.split.set_second(Split::equal(members));
        self
    }

    /// Divides the line total by `split`.
    pub fn split(mut self, split: Split) -> Self {
        self.split.set_first(split);
        self
    }

    /// Replaces the unit price.
    pub const fn unit_price(mut self, unit_price: Money) -> Self {
        self.unit_price = unit_price;
        self
    }

    /// Sets the number of units.
    pub const fn quantity(mut self, quantity: u32) -> Self {
        self.quantity = quantity;
        self
    }

    /// Validates the item and returns it.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::Value`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the unit price is not
    ///   positive, and [`ExpenseError::ZeroQuantity`] for a quantity of 0.
    /// - [`ExpenseError::MissingSplit`] when neither `split` nor `shared_by`
    ///   was called, and [`ExpenseError::ConflictingSplit`] when both were.
    /// - [`ExpenseError::Overflow`] when the line total does not fit.
    /// - Any error of an invalid [`Split`] for the line total.
    pub fn build(self) -> Result<Item, ExpenseError> {
        let name = text::required(self.name, Field::ItemName)?;
        if !self.unit_price.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        if self.quantity == 0 {
            return Err(ExpenseError::ZeroQuantity);
        }
        let split = self
            .split
            .resolve(ExpenseError::MissingSplit, ExpenseError::ConflictingSplit)?;
        let line_total = self
            .unit_price
            .minor_units()
            .checked_mul(i64::from(self.quantity))
            .ok_or(ExpenseError::Overflow)?;
        split.validate(Money::from_minor_units(
            line_total,
            self.unit_price.currency(),
        ))?;
        Ok(Item {
            name,
            split,
            unit_price: self.unit_price,
            quantity: self.quantity,
        })
    }
}

/// The serialised form of an [`Item`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ItemRecord {
    name: String,
    quantity: u32,
    split: Split,
    unit_price: Money,
}

#[cfg(feature = "serde")]
impl TryFrom<ItemRecord> for Item {
    type Error = ExpenseError;

    fn try_from(record: ItemRecord) -> Result<Self, Self::Error> {
        Self::builder(record.name, record.unit_price)
            .quantity(record.quantity)
            .split(record.split)
            .build()
    }
}

#[cfg(test)]
mod tests;
