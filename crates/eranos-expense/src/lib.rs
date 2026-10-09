//! Expenses: what was bought, who paid, and how the cost is shared.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`resolve_expense`] and [`resolve_expense_at`], which
//! divide an expense into group-currency shares for the ledger, are the
//! group's, and the facade does not re-export them; nor does it re-export
//! the conversion traits
//! ([`FromDiscountScope`], [`FromDiscountValue`], [`FromExpenseError`],
//! [`FromExpenseSplit`], [`FromSplit`], [`FromTipDistribution`], and
//! [`FromTipValue`]), through which a crate that
//! tells every variant of an enum apart converts it without the wildcard arm
//! a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod discount;
mod error;
mod expense_item;
mod resolution;
mod split;
mod tip_rule;

use std::collections::BTreeSet;

pub use discount::{Discount, DiscountScope, DiscountValue, FromDiscountScope, FromDiscountValue};
pub use error::{ExpenseError, FromExpenseError};
pub use expense_item::{ExpenseItem, ExpenseItemBuilder};
pub use resolution::{resolve_expense, resolve_expense_at};
pub use split::{FromSplit, MemberAmount, MemberParts, MemberPercentage, Split};
pub use tip_rule::{FromTipDistribution, FromTipValue, TipDistribution, TipRule, TipValue};

use eranos_calendar::{CalendarDate, TimeOfDay};
use eranos_core::{CategoryId, ExpenseId, MemberId, text};
use eranos_money::{Cost, ExchangeRate, Money};

/// Something bought for the group: its amount, who paid, and how its cost is
/// shared.
///
/// The [`Cost`] is the amount in the expense's own currency and, for an
/// expense in another currency than its group's, the [`ExchangeRate`] into
/// the group currency, fixed when the expense is entered. A tip is added on
/// top of the amount, and the payers cover amount plus tip. Four
/// constructors name the payers and the split up front: one payer or
/// several, and a whole split or items. Each comes in two forms: one
/// generates the identifier when it is called, and its twin, whose name
/// ends in `with_id`, takes it. An expense keeps its identifier
/// through every edit. Expenses are values: to change one, edit a copy with
/// [`to_builder`](Self::to_builder) and pass it to
/// `ExpenseGroup::replace_expense`.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, CalendarDate, Expense, Money, Percentage, Split, TipRule};
/// use eranos::id::MemberId;
///
/// let (ana, bob, carol) = (MemberId::generate(), MemberId::generate(), MemberId::generate());
/// let (amount, date) = (Money::parse("90.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
/// let dinner = Expense::builder("Dinner", amount, ana, Split::equal([ana, bob, carol]), date)
///     .tip(TipRule::percentage(Percentage::parse("10")?))
///     .build()?;
/// assert_eq!(dinner.tip_amount(), Money::parse("9.00", CurrencyCode::EUR)?);
/// assert_eq!(dinner.total(), Money::parse("99.00", CurrencyCode::EUR)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Expense {
    id: ExpenseId,
    name: String,
    amount: Money,
    // Derived from amount and tip by `build`, so never serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    total: Money,
    payers: Split,
    split: ExpenseSplit,
    date: CalendarDate,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_no_tip"))]
    tip: TipRule,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<TimeOfDay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_uncategorised"))]
    category: CategoryId,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    note: Option<String>,
}

impl Expense {
    /// Starts an expense of `cost`, which `payers` paid and `split` shares,
    /// on `date`, under an identifier generated when it is called;
    /// [`builder_with_id`](Self::builder_with_id) takes one instead.
    ///
    /// `payers` is one member, who paid alone, or a [`Split`] of the total,
    /// amount and tip, among several, and `split` a [`Split`] of the whole
    /// amount or the [`ExpenseItem`]s it is divided by, whose line totals
    /// must add up to it. A [`Money`] amount is a local cost, in the group
    /// currency; [`Money::at`] makes a converted one.
    pub fn builder(
        name: impl Into<String>,
        cost: impl Into<Cost>,
        payers: impl Into<Split>,
        split: impl Into<ExpenseSplit>,
        date: CalendarDate,
    ) -> ExpenseBuilder {
        Self::builder_with_id(ExpenseId::generate(), name, cost, payers, split, date)
    }

    /// Starts an expense as [`builder`](Self::builder) does, under the
    /// identifier `id`, such as the key an app's own store gives it;
    /// `builder` generates one instead. The identifier seeds the tie
    /// rotation, so it decides who gets a leftover cent.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, CurrencyCode, Expense, ExpenseId, MemberId, Money, Split};
    ///
    /// let (ana, bob) = (MemberId::generate(), MemberId::generate());
    /// let (amount, date) = (Money::parse("30.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
    /// let id = ExpenseId::new("dinner-1")?;
    /// let dinner = Expense::builder_with_id(id, "Dinner", amount, ana, Split::equal([ana, bob]), date).build()?;
    /// assert_eq!(dinner.id().as_str(), "dinner-1");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder_with_id(
        id: ExpenseId,
        name: impl Into<String>,
        cost: impl Into<Cost>,
        payers: impl Into<Split>,
        split: impl Into<ExpenseSplit>,
        date: CalendarDate,
    ) -> ExpenseBuilder {
        starting(
            id,
            name.into(),
            cost.into(),
            payers.into(),
            split.into(),
            date,
        )
    }

    /// Starts a builder holding a copy of this expense, to edit it; the copy
    /// keeps the identifier.
    ///
    /// Calling `split` or `items` on it replaces the split, and `paid_by` or
    /// `payers` the payers, whichever form they had.
    pub fn to_builder(&self) -> ExpenseBuilder {
        ExpenseBuilder {
            id: self.id,
            name: self.name.clone(),
            cost: self.cost(),
            payers: self.payers.clone(),
            split: self.split.clone(),
            date: self.date,
            tip: self.tip,
            time: self.time,
            category: self.category,
            note: self.note.clone(),
        }
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> ExpenseId {
        self.id
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The amount before the tip, in the expense's currency.
    #[must_use]
    pub const fn amount(&self) -> Money {
        self.amount
    }

    /// The cost before the tip: the amount and, for an expense in another
    /// currency than its group's, the rate into the group currency.
    #[must_use]
    pub const fn cost(&self) -> Cost {
        Cost::from_parts(self.amount, self.exchange_rate)
    }

    /// The amount plus the tip: what the payers paid.
    #[must_use]
    pub const fn total(&self) -> Money {
        self.total
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

    /// The date of the expense.
    #[must_use]
    pub const fn date(&self) -> CalendarDate {
        self.date
    }

    /// The tip: [`TipRule::NONE`], no tip, unless another was set.
    #[must_use]
    pub const fn tip(&self) -> TipRule {
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

    /// The rate from the expense's currency into its group's currency:
    /// [`ExchangeRate::ONE`] for a local cost.
    #[must_use]
    pub const fn exchange_rate(&self) -> ExchangeRate {
        self.cost().exchange_rate()
    }

    /// The same expense at `exchange_rate` into its group's currency, or as a
    /// local cost with `None`, identifier included: what a group does to its
    /// expenses when it changes currency.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, CurrencyCode, ExchangeRate, Expense, Money, Split};
    /// use eranos::id::MemberId;
    ///
    /// let ana = MemberId::generate();
    /// let (amount, day) = (Money::parse("20.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
    /// let museum = Expense::builder("Museum", amount, ana, Split::equal([ana]), day).build()?;
    /// let rate = ExchangeRate::parse("4.97")?;
    /// assert_eq!(museum.with_exchange_rate(Some(rate)).cost(), amount.at(rate));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_exchange_rate(mut self, exchange_rate: Option<ExchangeRate>) -> Self {
        self.exchange_rate = exchange_rate;
        self
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<TimeOfDay> {
        self.time
    }

    /// The category: [`CategoryId::UNCATEGORISED`] unless another was set.
    #[must_use]
    pub const fn category(&self) -> CategoryId {
        self.category
    }

    /// The same expense in the category `category`, identifier included:
    /// what a group does to the expenses of a category it removes.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CalendarDate, CategoryId, CurrencyCode, Expense, Money, Split};
    /// use eranos::id::MemberId;
    ///
    /// let (ana, food) = (MemberId::generate(), CategoryId::generate());
    /// let (amount, day) = (Money::parse("12.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
    /// let lunch = Expense::builder("Lunch", amount, ana, Split::equal([ana]), day)
    ///     .category(food)
    ///     .build()?;
    /// let moved = lunch.clone().with_category(CategoryId::UNCATEGORISED);
    /// assert_eq!((moved.id(), moved.category()), (lunch.id(), CategoryId::UNCATEGORISED));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_category(mut self, category: CategoryId) -> Self {
        self.category = category;
        self
    }

    /// The note, if any.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// Every member the expense involves, as a payer or as a participant.
    #[must_use]
    pub fn members(&self) -> BTreeSet<MemberId> {
        let mut members: BTreeSet<MemberId> = self.payers.members().collect();
        match &self.split {
            ExpenseSplit::Whole(split) => members.extend(split.members()),
            ExpenseSplit::Itemised(items) => {
                members.extend(items.iter().flat_map(|item| item.split().members()));
            }
        }
        members
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
    /// Every [`ExpenseItem`] is divided by its own split.
    Itemised(Vec<ExpenseItem>),
}

impl From<Split> for ExpenseSplit {
    /// The whole amount divided by `split`.
    fn from(split: Split) -> Self {
        Self::Whole(split)
    }
}

impl From<Vec<ExpenseItem>> for ExpenseSplit {
    /// The amount divided item by item.
    fn from(items: Vec<ExpenseItem>) -> Self {
        Self::Itemised(items)
    }
}

impl<const N: usize> From<[ExpenseItem; N]> for ExpenseSplit {
    /// The amount divided item by item.
    fn from(items: [ExpenseItem; N]) -> Self {
        Self::Itemised(items.into())
    }
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

/// Builds a value from an [`ExpenseSplit`], one method per way of sharing a
/// cost.
pub trait FromExpenseSplit: Sized {
    /// The value for a cost shared as a whole by `split`.
    fn whole(split: &Split) -> Self;

    /// The value for a cost shared item by item.
    fn itemised(items: &[ExpenseItem]) -> Self;

    /// The value for `expense_split`, from the method of its variant.
    #[must_use]
    fn from_expense_split(expense_split: &ExpenseSplit) -> Self {
        match expense_split {
            ExpenseSplit::Whole(split) => Self::whole(split),
            ExpenseSplit::Itemised(items) => Self::itemised(items),
        }
    }
}

/// Builds an [`Expense`]; created by [`Expense::builder`], one of its
/// variants, or [`Expense::to_builder`].
///
/// Every setter replaces what the constructor or an earlier call set: the
/// last call wins.
#[derive(Debug, Clone)]
#[must_use = "a builder does nothing until `build` is called"]
pub struct ExpenseBuilder {
    id: ExpenseId,
    name: String,
    cost: Cost,
    payers: Split,
    split: ExpenseSplit,
    date: CalendarDate,
    tip: TipRule,
    time: Option<TimeOfDay>,
    category: CategoryId,
    note: Option<String>,
}

impl ExpenseBuilder {
    /// Replaces the identifier, which seeds the tie rotation, so it decides
    /// who gets a leftover cent.
    pub const fn id(mut self, id: ExpenseId) -> Self {
        self.id = id;
        self
    }

    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Replaces the cost (before the tip).
    ///
    /// A [`Money`] amount makes the cost local; to change the amount of a
    /// converted cost and keep its rate, pass
    /// `expense.cost().with_amount(amount)`.
    pub fn cost(mut self, cost: impl Into<Cost>) -> Self {
        self.cost = cost.into();
        self
    }

    /// Replaces the payers with `member` alone, who paid the whole total;
    /// the same as `payers(Split::equal([member]))`.
    pub fn paid_by(mut self, member: MemberId) -> Self {
        self.payers = Split::equal([member]);
        self
    }

    /// Replaces who paid the total (amount plus tip) and how much each.
    pub fn payers(mut self, payers: Split) -> Self {
        self.payers = payers;
        self
    }

    /// Replaces the split with one item by item; the line totals must add up
    /// to the amount.
    pub fn items(mut self, items: impl IntoIterator<Item = ExpenseItem>) -> Self {
        self.split = ExpenseSplit::Itemised(items.into_iter().collect());
        self
    }

    /// Replaces the split with `split`, which divides the whole amount.
    pub fn split(mut self, split: Split) -> Self {
        self.split = ExpenseSplit::Whole(split);
        self
    }

    /// Replaces the date.
    pub const fn date(mut self, date: CalendarDate) -> Self {
        self.date = date;
        self
    }

    /// Replaces the tip; [`TipRule::NONE`] removes it.
    pub const fn tip(mut self, tip: TipRule) -> Self {
        self.tip = tip;
        self
    }

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<TimeOfDay>>) -> Self {
        self.time = time.into();
        self
    }

    /// Replaces the category; [`CategoryId::UNCATEGORISED`] clears it.
    pub const fn category(mut self, category: CategoryId) -> Self {
        self.category = category;
        self
    }

    /// Sets the note, trimmed; blank text leaves it absent, as
    /// [`clear_note`](Self::clear_note) does.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Clears the note, which [`note`](Self::note)
    /// sets.
    pub fn clear_note(mut self) -> Self {
        self.note = None;
        self
    }

    /// Validates the expense and returns it, under the identifier the
    /// builder holds.
    ///
    /// Only the expense itself is checked here; whether its members,
    /// category, and cost fit a group is checked when it is added to one.
    ///
    /// # Errors
    ///
    /// - [`ExpenseError::EmptyName`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the amount is not positive.
    /// - [`ExpenseError::NoItems`] or [`ExpenseError::ItemsDoNotAddUp`] for
    ///   items that are missing or do not add up to the amount.
    /// - [`ExpenseError::CurrencyMismatch`] when an item, a fixed tip, or a
    ///   split amount is in another currency, and
    ///   [`ExpenseError::NegativeTip`] for a fixed tip below zero.
    /// - Any error of an invalid [`Split`]: the participants' split must
    ///   divide the amount, the payers' split the total.
    pub fn build(self) -> Result<Expense, ExpenseError> {
        let name = text::required(self.name, ExpenseError::EmptyName)?;
        let amount = self.cost.amount();
        if !amount.is_positive() {
            return Err(ExpenseError::NonPositiveAmount);
        }
        let (payers, split) = (self.payers, self.split);
        match &split {
            ExpenseSplit::Whole(split) => split.validate(amount)?,
            ExpenseSplit::Itemised(items) => validate_items(items, amount)?,
        }
        let tip_amount = self.tip.amount_on(amount)?;
        let total = amount
            .checked_add(tip_amount)
            .map_err(|_| ExpenseError::Overflow)?;
        payers.validate(total)?;
        Ok(Expense {
            id: self.id,
            name,
            amount,
            total,
            payers,
            split,
            date: self.date,
            tip: self.tip,
            exchange_rate: self.cost.conversion(),
            time: self.time,
            category: self.category,
            note: self.note.and_then(text::optional),
        })
    }
}

/// Checks that there is at least one item, every item is in the expense's
/// currency, and the line totals add up to the amount.
fn validate_items(items: &[ExpenseItem], amount: Money) -> Result<(), ExpenseError> {
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

/// A builder with the required fields set and every optional one at its
/// default; the constructors and the serialised form start here.
const fn starting(
    id: ExpenseId,
    name: String,
    cost: Cost,
    payers: Split,
    split: ExpenseSplit,
    date: CalendarDate,
) -> ExpenseBuilder {
    ExpenseBuilder {
        id,
        name,
        cost,
        payers,
        split,
        date,
        tip: TipRule::NONE,
        time: None,
        category: CategoryId::UNCATEGORISED,
        note: None,
    }
}

/// Whether `tip` is [`TipRule::NONE`], which the serialised form leaves out.
#[cfg(feature = "serde")]
fn is_no_tip(tip: &TipRule) -> bool {
    TipRule::NONE == *tip
}

/// Whether `category` is the built-in one, which the serialised form leaves
/// out.
#[cfg(feature = "serde")]
fn is_uncategorised(category: &CategoryId) -> bool {
    CategoryId::UNCATEGORISED == *category
}

/// The serialised form of an [`Expense`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ExpenseRecord {
    id: ExpenseId,
    name: String,
    amount: Money,
    payers: Split,
    split: ExpenseSplit,
    date: CalendarDate,
    #[serde(default)]
    tip: TipRule,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    #[serde(default)]
    time: Option<TimeOfDay>,
    #[serde(default)]
    category: Option<CategoryId>,
    #[serde(default)]
    note: Option<String>,
}

/// Builds the expense `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: ExpenseRecord) -> Result<Expense, ExpenseError> {
    let cost = Cost::from_parts(record.amount, record.exchange_rate);
    let builder = starting(
        record.id,
        record.name,
        cost,
        record.payers,
        record.split,
        record.date,
    )
    .tip(record.tip)
    .time(record.time)
    .category(record.category.unwrap_or(CategoryId::UNCATEGORISED));
    match record.note {
        Some(note) => builder.note(note).build(),
        None => builder.build(),
    }
}

// Written out rather than derived with `serde(try_from)`: its `TryFrom` impl
// from the private record would appear on the facade's page for `Expense`.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Expense {
    fn deserialize<D: serde::Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
        from_record(ExpenseRecord::deserialize(deserialiser)?).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests;
