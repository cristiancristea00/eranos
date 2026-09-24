//! Expenses: what was bought, who paid, and how the cost is shared.

mod item;
mod split;
mod tip;

use std::collections::BTreeSet;
use std::mem;

pub use item::{Item, ItemBuilder};
pub use split::Split;
pub use tip::{Tip, TipDistribution, TipValue};

use crate::date::{Date, Time};
use crate::error::{ExpenseError, Field};
use crate::id::{CategoryId, MemberId};
use crate::money::{ExchangeRate, Money};
use crate::text;

/// Something bought for the group: its amount, who paid, and how its cost is
/// shared.
///
/// The amount is in the expense's own currency; an expense in another
/// currency than its group's needs an [`ExchangeRate`] into the group
/// currency, fixed when the expense is entered. A tip is added on top of the
/// amount, and the payers cover amount plus tip. Expenses are values: to
/// change one, edit a copy with [`to_builder`](Self::to_builder) and pass it
/// to [`Group::replace_expense`](crate::Group::replace_expense).
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Date, Expense, Money, Percentage, Split, Tip};
/// use eranos::id::MemberId;
///
/// let (ana, bob, carol) = (MemberId::generate(), MemberId::generate(), MemberId::generate());
/// let dinner = Expense::builder("Dinner", Money::parse("90.00", Currency::EUR)?, Date::new(2026, 9, 23)?)
///     .paid_by(ana)
///     .split(Split::equal([ana, bob, carol]))
///     .tip(Tip::percentage(Percentage::parse("10")?))
///     .build()?;
/// assert_eq!(dinner.tip_amount(), Money::parse("9.00", Currency::EUR)?);
/// assert_eq!(dinner.total(), Money::parse("99.00", Currency::EUR)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "ExpenseRecord")
)]
pub struct Expense {
    name: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    description: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    category: Option<CategoryId>,
    amount: Money,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    tip: Option<Tip>,
    // Derived from amount and tip by `build`, so never serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    total: Money,
    date: Date,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<Time>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    payers: Split,
    split: ExpenseSplit,
}

impl Expense {
    /// Starts an expense with its required name, amount, and date.
    pub fn builder(name: impl Into<String>, amount: Money, date: Date) -> ExpenseBuilder {
        ExpenseBuilder {
            name: name.into(),
            description: None,
            category: None,
            amount,
            tip: None,
            date,
            time: None,
            exchange_rate: None,
            payers: Choice::Unset,
            split: Choice::Unset,
        }
    }

    /// Starts a builder holding a copy of this expense, to edit it.
    ///
    /// Calling `split` or `items` on it replaces the split, and `paid_by` or
    /// `payers` the payers, whichever form they had.
    pub fn to_builder(&self) -> ExpenseBuilder {
        ExpenseBuilder {
            name: self.name.clone(),
            description: self.description.clone(),
            category: self.category,
            amount: self.amount,
            tip: self.tip,
            date: self.date,
            time: self.time,
            exchange_rate: self.exchange_rate,
            payers: Choice::Inherited(self.payers.clone()),
            split: Choice::Inherited(self.split.clone()),
        }
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The description, if any.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The category, if any.
    #[must_use]
    pub const fn category(&self) -> Option<CategoryId> {
        self.category
    }

    /// The amount before the tip, in the expense's currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The tip, if any.
    #[must_use]
    pub const fn tip(&self) -> Option<Tip> {
        self.tip
    }

    /// The tip as an amount in the expense's currency; zero without a tip.
    #[must_use]
    pub const fn tip_amount(&self) -> Money {
        // Both are in the expense currency and total = amount + tip.
        Money::from_minor_units(
            self.total.minor_units() - self.amount.minor_units(),
            self.amount.currency(),
        )
    }

    /// The amount plus the tip: what the payers paid.
    #[must_use]
    pub const fn total(&self) -> Money {
        self.total
    }

    /// The date of the expense.
    #[must_use]
    pub const fn date(&self) -> Date {
        self.date
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<Time> {
        self.time
    }

    /// The rate from the expense's currency into its group's currency, for a
    /// foreign-currency expense.
    #[must_use]
    pub const fn exchange_rate(&self) -> Option<ExchangeRate> {
        self.exchange_rate
    }

    /// Who paid, and how much each.
    #[must_use]
    pub const fn payers(&self) -> &Split {
        &self.payers
    }

    /// How the cost is shared.
    #[must_use]
    pub const fn split(&self) -> &ExpenseSplit {
        &self.split
    }

    /// Every member the expense involves, as a payer or as a participant.
    #[must_use]
    pub fn members(&self) -> BTreeSet<MemberId> {
        let mut members: BTreeSet<MemberId> = self.payers.members().collect();
        members.extend(self.split.participants());
        members
    }

    pub(crate) const fn set_exchange_rate(&mut self, exchange_rate: Option<ExchangeRate>) {
        self.exchange_rate = exchange_rate;
    }
}

/// How an expense's cost is shared: as a whole, or item by item.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum ExpenseSplit {
    /// The whole amount is divided by one [`Split`].
    Whole(Split),
    /// Every [`Item`] is divided by its own split.
    Itemised(Vec<Item>),
}

impl ExpenseSplit {
    /// Every member who shares the cost, once each.
    #[must_use]
    pub fn participants(&self) -> BTreeSet<MemberId> {
        match self {
            Self::Whole(split) => split.members().collect(),
            Self::Itemised(items) => items
                .iter()
                .flat_map(|item| item.split().members())
                .collect(),
        }
    }
}

/// Builds an [`Expense`]; created by [`Expense::builder`] or
/// [`Expense::to_builder`].
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseBuilder {
    name: String,
    description: Option<String>,
    category: Option<CategoryId>,
    amount: Money,
    tip: Option<Tip>,
    date: Date,
    time: Option<Time>,
    exchange_rate: Option<ExchangeRate>,
    payers: Choice<Split>,
    split: Choice<ExpenseSplit>,
}

impl ExpenseBuilder {
    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description; blank text clears it.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the category, or clears it with `None`.
    pub fn category(mut self, category: impl Into<Option<CategoryId>>) -> Self {
        self.category = category.into();
        self
    }

    /// Replaces the amount (before the tip).
    pub const fn amount(mut self, amount: Money) -> Self {
        self.amount = amount;
        self
    }

    /// Sets the tip, or clears it with `None`.
    pub fn tip(mut self, tip: impl Into<Option<Tip>>) -> Self {
        self.tip = tip.into();
        self
    }

    /// Replaces the date.
    pub const fn date(mut self, date: Date) -> Self {
        self.date = date;
        self
    }

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<Time>>) -> Self {
        self.time = time.into();
        self
    }

    /// Sets the rate from the expense's currency into the group currency, or
    /// clears it with `None`.
    pub fn exchange_rate(mut self, exchange_rate: impl Into<Option<ExchangeRate>>) -> Self {
        self.exchange_rate = exchange_rate.into();
        self
    }

    /// Records a single payer who paid the whole total; the same as
    /// `payers(Split::equal([member]))`, and an error together with it.
    pub fn paid_by(mut self, member: MemberId) -> Self {
        self.payers.set_first(Split::equal([member]));
        self
    }

    /// Records who paid the total (amount plus tip) and how much each.
    pub fn payers(mut self, payers: Split) -> Self {
        self.payers.set_second(payers);
        self
    }

    /// Divides the whole amount by `split`.
    pub fn split(mut self, split: Split) -> Self {
        self.split.set_first(ExpenseSplit::Whole(split));
        self
    }

    /// Divides the amount item by item; the line totals must add up to the
    /// amount.
    pub fn items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.split
            .set_second(ExpenseSplit::Itemised(items.into_iter().collect()));
        self
    }

    /// Validates the expense and returns it.
    ///
    /// Only the expense itself is checked here; whether its members,
    /// category, and exchange rate fit a group is checked when it is added
    /// to one.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::Value`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the amount is not positive.
    /// - [`ExpenseError::MissingPayers`] or [`ExpenseError::ConflictingPayers`]
    ///   when neither or both of `paid_by` and `payers` were called, and
    ///   [`ExpenseError::MissingSplit`] or [`ExpenseError::ConflictingSplit`]
    ///   for `split` and `items`.
    /// - [`ExpenseError::NoItems`] or [`ExpenseError::ItemsDoNotAddUp`] for
    ///   items that are missing or do not add up to the amount.
    /// - [`ExpenseError::CurrencyMismatch`] when an item, a fixed tip, or a
    ///   split amount is in another currency, and
    ///   [`ExpenseError::NegativeTip`] for a fixed tip below zero.
    /// - Any error of an invalid [`Split`]: the participants' split must
    ///   divide the amount, the payers' split the total.
    pub fn build(self) -> Result<Expense, ExpenseError> {
        let name = text::required(self.name, Field::ExpenseName)?;
        let amount = self.amount;
        if !amount.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        let payers = self
            .payers
            .resolve(ExpenseError::MissingPayers, ExpenseError::ConflictingPayers)?;
        let split = self
            .split
            .resolve(ExpenseError::MissingSplit, ExpenseError::ConflictingSplit)?;
        match &split {
            ExpenseSplit::Whole(split) => split.validate(amount)?,
            ExpenseSplit::Itemised(items) => validate_items(items, amount)?,
        }
        let tip_amount = match self.tip {
            Some(tip) => tip.amount_on(amount)?,
            None => Money::zero(amount.currency()),
        };
        let total = amount
            .checked_add(tip_amount)
            .map_err(|_| ExpenseError::Overflow)?;
        payers.validate(total)?;
        Ok(Expense {
            name,
            description: self.description.and_then(text::optional),
            category: self.category,
            amount,
            tip: self.tip,
            total,
            date: self.date,
            time: self.time,
            exchange_rate: self.exchange_rate,
            payers,
            split,
        })
    }
}

/// Checks that there is at least one item, every item is in the expense's
/// currency, and the line totals add up to the amount.
fn validate_items(items: &[Item], amount: Money) -> Result<(), ExpenseError> {
    if items.is_empty() {
        return Err(ExpenseError::NoItems);
    }
    let mut sum = 0_i128;
    for item in items {
        let line_total = item.line_total();
        if line_total.currency() != amount.currency() {
            return Err(ExpenseError::CurrencyMismatch {
                expected: amount.currency(),
                found: line_total.currency(),
            });
        }
        sum += i128::from(line_total.minor_units());
    }
    if sum == i128::from(amount.minor_units()) {
        return Ok(());
    }
    let actual = i64::try_from(sum).map_err(|_| ExpenseError::Overflow)?;
    Err(ExpenseError::ItemsDoNotAddUp {
        expected: amount,
        actual: Money::from_minor_units(actual, amount.currency()),
    })
}

/// A value two alternative builder setters compete for: repeating one
/// setter replaces its value, while using both is a conflict reported by
/// `build`, because a silent override would hide the caller's mistake.
#[derive(Debug, Clone)]
enum Choice<T> {
    Unset,
    /// Copied from an existing value by `to_builder`; either setter may
    /// replace it.
    Inherited(T),
    First(T),
    Second(T),
    Conflict,
}

impl<T> Choice<T> {
    fn set_first(&mut self, value: T) {
        *self = match mem::replace(self, Self::Conflict) {
            Self::Second(_) | Self::Conflict => Self::Conflict,
            Self::Unset | Self::Inherited(_) | Self::First(_) => Self::First(value),
        };
    }

    fn set_second(&mut self, value: T) {
        *self = match mem::replace(self, Self::Conflict) {
            Self::First(_) | Self::Conflict => Self::Conflict,
            Self::Unset | Self::Inherited(_) | Self::Second(_) => Self::Second(value),
        };
    }

    fn resolve(self, missing: ExpenseError, conflicting: ExpenseError) -> Result<T, ExpenseError> {
        match self {
            Self::Unset => Err(missing),
            Self::Conflict => Err(conflicting),
            Self::Inherited(value) | Self::First(value) | Self::Second(value) => Ok(value),
        }
    }
}

/// The serialised form of an [`Expense`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ExpenseRecord {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    category: Option<CategoryId>,
    amount: Money,
    #[serde(default)]
    tip: Option<Tip>,
    date: Date,
    #[serde(default)]
    time: Option<Time>,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    payers: Split,
    split: ExpenseSplit,
}

#[cfg(feature = "serde")]
impl TryFrom<ExpenseRecord> for Expense {
    type Error = ExpenseError;

    fn try_from(record: ExpenseRecord) -> Result<Self, Self::Error> {
        let builder = Self::builder(record.name, record.amount, record.date)
            .category(record.category)
            .tip(record.tip)
            .time(record.time)
            .exchange_rate(record.exchange_rate)
            .payers(record.payers);
        let builder = match record.split {
            ExpenseSplit::Whole(split) => builder.split(split),
            ExpenseSplit::Itemised(items) => builder.items(items),
        };
        match record.description {
            Some(description) => builder.description(description).build(),
            None => builder.build(),
        }
    }
}

#[cfg(test)]
mod tests;
