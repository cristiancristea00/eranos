//! Receipt lines of an itemised expense.

use eranos_core::{MemberId, text};
use eranos_money::{Money, Quantity};

use crate::Split;
use crate::discount::{Discount, LineTotals};
use crate::error::ExpenseError;

/// One line of an itemised expense: a unit price times a quantity, less an
/// optional [`Discount`], divided among its own members by its own
/// [`Split`].
///
/// The quantity is a whole number of units or a decimal with four places,
/// such as 0.452 kg, and the line total before the discount is the unit
/// price times the quantity, rounded half away from zero to the currency's
/// minor unit, so a whole quantity gives the exact product. A discount,
/// taken off the line or off each unit, reduces it to the line total. A
/// line whose total is zero is free: its product is below half a minor
/// unit, or a discount takes its whole price, and it divides to zeros. The
/// line totals of an expense's items must add up to exactly the expense's
/// amount.
///
/// # Examples
///
/// ```
/// use eranos::id::MemberId;
/// use eranos::{CurrencyCode, Discount, ExpenseItem, MemberParts, Money, Parts, Percentage, Quantity, Split};
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
/// let beers = ExpenseItem::builder("Beer", Money::parse("4.50", CurrencyCode::EUR)?, [ana, bob])
///     .quantity(Quantity::from_whole(3)?)
///     .build()?;
/// assert_eq!(beers.line_total(), Money::parse("13.50", CurrencyCode::EUR)?);
///
/// // 0.35 kg at 42.00 EUR a kilogram, with 10 % off.
/// let cheese = ExpenseItem::builder("Cheese", Money::parse("42.00", CurrencyCode::EUR)?, [ana])
///     .quantity(Quantity::parse("0.35")?)
///     .discount(Discount::percentage(Percentage::parse("10")?))
///     .build()?;
/// assert_eq!(cheese.line_total_before_discount(), Money::parse("14.70", CurrencyCode::EUR)?);
/// assert_eq!(cheese.discount_amount(), Money::parse("1.47", CurrencyCode::EUR)?);
/// assert_eq!(cheese.line_total(), Money::parse("13.23", CurrencyCode::EUR)?);
///
/// let split = Split::parts([
///     MemberParts::new(ana, Parts::from_whole(2)?),
///     MemberParts::new(bob, Parts::from_whole(1)?),
/// ]);
/// let pizza = ExpenseItem::builder_with_split("Pizza", Money::parse("12.00", CurrencyCode::EUR)?, split)
///     .build()?;
/// assert_eq!(pizza.quantity(), Quantity::ONE);
/// assert_eq!(pizza.discount(), None);
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
    line_total_before_discount: Money,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    discount: Option<Discount>,
    // Derived from the total before the discount and the discount by
    // `build`, so never serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    line_total: Money,
}

impl ExpenseItem {
    /// Starts an item whose line total `members` share equally; the
    /// quantity defaults to [`Quantity::ONE`], and the item has no discount.
    pub fn builder(
        name: impl Into<String>,
        unit_price: Money,
        members: impl IntoIterator<Item = MemberId>,
    ) -> ExpenseItemBuilder {
        Self::builder_with_split(name, unit_price, Split::equal(members))
    }

    /// Starts an item whose line total is divided by `split`; the quantity
    /// defaults to [`Quantity::ONE`], and the item has no discount.
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
            discount: None,
        }
    }

    /// Starts a builder holding a copy of this item, to edit it.
    pub fn to_builder(&self) -> ExpenseItemBuilder {
        ExpenseItemBuilder {
            name: self.name.clone(),
            unit_price: self.unit_price,
            split: self.split.clone(),
            quantity: self.quantity,
            discount: self.discount,
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
    /// currency's minor unit, in the unit price's currency: the line total
    /// before the discount, zero when the product is below half a minor
    /// unit.
    #[must_use]
    pub const fn line_total_before_discount(&self) -> Money {
        self.line_total_before_discount
    }

    /// The discount, if any.
    #[must_use]
    pub const fn discount(&self) -> Option<Discount> {
        self.discount
    }

    /// What the discount takes off the line, in the unit price's currency:
    /// the line total before the discount minus the line total, zero
    /// without a discount.
    #[must_use]
    pub const fn discount_amount(&self) -> Money {
        // Both are in the unit price's currency, and 0 ≤ line total ≤ the
        // total before the discount.
        Money::from_minor_units(
            self.line_total_before_discount.minor_units() - self.line_total.minor_units(),
            self.line_total.currency(),
        )
    }

    /// The line total before the discount, less the discount, in the unit
    /// price's currency: zero for a free line, positive otherwise.
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
    discount: Option<Discount>,
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

    /// Sets the discount, or clears it with `None`.
    pub fn discount(mut self, discount: impl Into<Option<Discount>>) -> Self {
        self.discount = discount.into();
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
    /// - [`ExpenseError::CurrencyMismatch`] when a fixed discount is in
    ///   another currency than the unit price,
    ///   [`ExpenseError::NegativeDiscount`] when it is below zero, and
    ///   [`ExpenseError::DiscountAbovePrice`] when it is larger than what it
    ///   reduces: the line total before the discount, or the unit price for
    ///   a discount off each unit.
    /// - [`ExpenseError::DiscountAboveHundredPercent`] for a percentage
    ///   discount above 100 %.
    /// - Any error of an invalid [`Split`] for the line total, among them
    ///   [`ExpenseError::AmountsDoNotAddUp`] for a split of amounts on a
    ///   free line, whose total of zero no positive amounts add up to.
    pub fn build(self) -> Result<ExpenseItem, ExpenseError> {
        let name = text::required(self.name, ExpenseError::EmptyItemName)?;
        if !self.unit_price.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        // A positive price times a positive quantity is never negative, but
        // less than half a minor unit rounds to zero, and a discount may
        // take the whole price: a free line, which its split divides to
        // zeros.
        let totals = LineTotals::of(self.unit_price, self.quantity, self.discount)?;
        self.split.validate(totals.line_total)?;
        Ok(ExpenseItem {
            name,
            unit_price: self.unit_price,
            split: self.split,
            quantity: self.quantity,
            line_total_before_discount: totals.before_discount,
            discount: self.discount,
            line_total: totals.line_total,
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
    #[serde(default)]
    discount: Option<Discount>,
}

/// Builds the item `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: ExpenseItemRecord) -> Result<ExpenseItem, ExpenseError> {
    ExpenseItem::builder_with_split(record.name, record.unit_price, record.split)
        .quantity(record.quantity)
        .discount(record.discount)
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
