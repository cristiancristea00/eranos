//! Receipt lines of an itemised expense.

use eranos_core::text;

use crate::error::ExpenseError;
use crate::expense::Split;
use crate::id::MemberId;
use crate::money::Money;

/// One line of an itemised expense: a unit price times a quantity, divided
/// among its own members by its own [`Split`].
///
/// The line totals of an expense's items must add up to exactly the
/// expense's amount.
///
/// # Examples
///
/// ```
/// use eranos::id::MemberId;
/// use eranos::{Currency, Item, MemberParts, Money, Parts, Split};
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let beers = Item::builder([ana, bob], "Beer", Money::parse("4.50", Currency::EUR)?)
///     .quantity(3)
///     .build()?;
/// assert_eq!(beers.line_total(), Money::parse("13.50", Currency::EUR)?);
///
/// let split = Split::parts([
///     MemberParts::new(ana, Parts::from_whole(2)?),
///     MemberParts::new(bob, Parts::from_whole(1)?),
/// ]);
/// let pizza = Item::builder_with_split("Pizza", split, Money::parse("12.00", Currency::EUR)?)
///     .build()?;
/// assert_eq!(pizza.quantity(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Item {
    name: String,
    split: Split,
    unit_price: Money,
    quantity: u32,
}

impl Item {
    /// Starts an item whose line total `members` share equally; the
    /// quantity defaults to 1.
    pub fn builder(
        members: impl IntoIterator<Item = MemberId>,
        name: impl Into<String>,
        unit_price: Money,
    ) -> ItemBuilder {
        Self::builder_with_split(name, Split::equal(members), unit_price)
    }

    /// Starts an item whose line total is divided by `split`; the quantity
    /// defaults to 1.
    pub fn builder_with_split(
        name: impl Into<String>,
        split: Split,
        unit_price: Money,
    ) -> ItemBuilder {
        ItemBuilder {
            name: name.into(),
            split,
            unit_price,
            quantity: 1,
        }
    }

    /// Starts a builder holding a copy of this item, to edit it.
    pub fn to_builder(&self) -> ItemBuilder {
        ItemBuilder {
            name: self.name.clone(),
            split: self.split.clone(),
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
        Money::from_minor_units(self.unit_price.currency(), total)
    }
}

/// Builds an [`Item`]; created by [`Item::builder`],
/// [`Item::builder_with_split`], or [`Item::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ItemBuilder {
    name: String,
    split: Split,
    unit_price: Money,
    quantity: u32,
}

impl ItemBuilder {
    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Replaces the split with an equal one among `members`; the same as
    /// `split(Split::equal(members))`.
    pub fn shared_by(mut self, members: impl IntoIterator<Item = MemberId>) -> Self {
        self.split = Split::equal(members);
        self
    }

    /// Replaces the split.
    pub fn split(mut self, split: Split) -> Self {
        self.split = split;
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
    /// - [`ExpenseError::EmptyItemName`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the unit price is not
    ///   positive, and [`ExpenseError::ZeroQuantity`] for a quantity of 0.
    /// - [`ExpenseError::Overflow`] when the line total does not fit.
    /// - Any error of an invalid [`Split`] for the line total.
    pub fn build(self) -> Result<Item, ExpenseError> {
        let name = text::required(self.name, ExpenseError::EmptyItemName)?;
        if !self.unit_price.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        if self.quantity == 0 {
            return Err(ExpenseError::ZeroQuantity);
        }
        let line_total = self
            .unit_price
            .minor_units()
            .checked_mul(i64::from(self.quantity))
            .ok_or(ExpenseError::Overflow)?;
        self.split.validate(Money::from_minor_units(
            self.unit_price.currency(),
            line_total,
        ))?;
        Ok(Item {
            name,
            split: self.split,
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

/// Builds the item `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: ItemRecord) -> Result<Item, ExpenseError> {
    Item::builder_with_split(record.name, record.split, record.unit_price)
        .quantity(record.quantity)
        .build()
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `Item`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Item {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        from_record(ItemRecord::deserialize(deserialiser)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
