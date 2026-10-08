//! Receipt lines of an itemised expense.

use eranos_core::{MemberId, text};
use eranos_money::allocation::apply_ten_thousandths;
use eranos_money::{Money, Quantity};

use crate::Split;
use crate::error::ExpenseError;

/// One line of an itemised expense: a unit price times a quantity, divided
/// among its own members by its own [`Split`].
///
/// The quantity is a whole number of units or a decimal with four places,
/// such as 0.452 kg, and the line total is the unit price times the
/// quantity, rounded half away from zero to the currency's minor unit, so a
/// whole quantity gives the exact product. A line whose product is below
/// half a minor unit is free: its total is zero, and it divides to zeros.
/// The line totals of an expense's items must add up to exactly the
/// expense's amount.
///
/// # Examples
///
/// ```
/// use eranos::id::MemberId;
/// use eranos::{CurrencyCode, ExpenseItem, MemberParts, Money, Parts, Quantity, Split};
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let beers = ExpenseItem::builder("Beer", Money::parse("4.50", CurrencyCode::EUR)?, [ana, bob])
///     .quantity(Quantity::from_whole(3)?)
///     .build()?;
/// assert_eq!(beers.line_total(), Money::parse("13.50", CurrencyCode::EUR)?);
///
/// // 0.35 kg at 42.00 EUR a kilogram.
/// let cheese = ExpenseItem::builder("Cheese", Money::parse("42.00", CurrencyCode::EUR)?, [ana])
///     .quantity(Quantity::parse("0.35")?)
///     .build()?;
/// assert_eq!(cheese.line_total(), Money::parse("14.70", CurrencyCode::EUR)?);
///
/// let split = Split::parts([
///     MemberParts::new(ana, Parts::from_whole(2)?),
///     MemberParts::new(bob, Parts::from_whole(1)?),
/// ]);
/// let pizza = ExpenseItem::builder_with_split("Pizza", Money::parse("12.00", CurrencyCode::EUR)?, split)
///     .build()?;
/// assert_eq!(pizza.quantity(), Quantity::ONE);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ExpenseItem {
    name: String,
    unit_price: Money,
    split: Split,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_one"))]
    quantity: Quantity,
    // Derived from the unit price and the quantity by `build`, so never
    // serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    line_total: Money,
}

impl ExpenseItem {
    /// Starts an item whose line total `members` share equally; the
    /// quantity defaults to [`Quantity::ONE`].
    pub fn builder(
        name: impl Into<String>,
        unit_price: Money,
        members: impl IntoIterator<Item = MemberId>,
    ) -> ExpenseItemBuilder {
        Self::builder_with_split(name, unit_price, Split::equal(members))
    }

    /// Starts an item whose line total is divided by `split`; the quantity
    /// defaults to [`Quantity::ONE`].
    pub fn builder_with_split(
        name: impl Into<String>,
        unit_price: Money,
        split: Split,
    ) -> ExpenseItemBuilder {
        ExpenseItemBuilder {
            name: name.into(),
            unit_price,
            split,
            quantity: Quantity::ONE,
        }
    }

    /// Starts a builder holding a copy of this item, to edit it.
    pub fn to_builder(&self) -> ExpenseItemBuilder {
        ExpenseItemBuilder {
            name: self.name.clone(),
            unit_price: self.unit_price,
            split: self.split.clone(),
            quantity: self.quantity,
        }
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The price of one unit.
    #[must_use]
    pub const fn unit_price(&self) -> Money {
        self.unit_price
    }

    /// How the line total is divided.
    #[must_use]
    pub const fn split(&self) -> &Split {
        &self.split
    }

    /// The number of units: positive, with at most four decimal places.
    #[must_use]
    pub const fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// The unit price times the quantity, rounded half away from zero to the
    /// currency's minor unit, in the unit price's currency: zero for a free
    /// line, positive otherwise.
    #[must_use]
    pub const fn line_total(&self) -> Money {
        self.line_total
    }
}

/// Builds an [`ExpenseItem`]; created by [`ExpenseItem::builder`],
/// [`ExpenseItem::builder_with_split`], or [`ExpenseItem::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseItemBuilder {
    name: String,
    unit_price: Money,
    split: Split,
    quantity: Quantity,
}

impl ExpenseItemBuilder {
    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Replaces the unit price.
    pub const fn unit_price(mut self, unit_price: Money) -> Self {
        self.unit_price = unit_price;
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

    /// Replaces the number of units.
    pub const fn quantity(mut self, quantity: Quantity) -> Self {
        self.quantity = quantity;
        self
    }

    /// Validates the item and returns it.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::EmptyItemName`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the unit price is not
    ///   positive.
    /// - [`ExpenseError::Overflow`] when the line total does not fit.
    /// - Any error of an invalid [`Split`] for the line total, among them
    ///   [`ExpenseError::AmountsDoNotAddUp`] for a split of amounts on a
    ///   free line, whose total of zero no positive amounts add up to.
    pub fn build(self) -> Result<ExpenseItem, ExpenseError> {
        let name = text::required(self.name, ExpenseError::EmptyItemName)?;
        if !self.unit_price.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        let minor_units = apply_ten_thousandths(
            self.unit_price.minor_units(),
            self.quantity.ten_thousandths(),
        )
        .ok_or(ExpenseError::Overflow)?;
        // A positive price times a positive quantity is never negative, but
        // less than half a minor unit rounds to zero: a free line, which its
        // split divides to zeros.
        let line_total = Money::from_minor_units(minor_units, self.unit_price.currency());
        self.split.validate(line_total)?;
        Ok(ExpenseItem {
            name,
            unit_price: self.unit_price,
            split: self.split,
            quantity: self.quantity,
            line_total,
        })
    }
}

/// Whether `quantity` is [`Quantity::ONE`], which the serialised form leaves
/// out.
#[cfg(feature = "serde")]
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde's `skip_serializing_if` passes the field by reference; the lint fires where a reference is as wide as a quantity, on a 64-bit target"
    )
)]
fn is_one(quantity: &Quantity) -> bool {
    Quantity::ONE == *quantity
}

/// The serialised form of an [`ExpenseItem`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ExpenseItemRecord {
    name: String,
    unit_price: Money,
    split: Split,
    #[serde(default)]
    quantity: Quantity,
}

/// Builds the item `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: ExpenseItemRecord) -> Result<ExpenseItem, ExpenseError> {
    ExpenseItem::builder_with_split(record.name, record.unit_price, record.split)
        .quantity(record.quantity)
        .build()
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `ExpenseItem`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ExpenseItem {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        from_record(ExpenseItemRecord::deserialize(deserialiser)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
