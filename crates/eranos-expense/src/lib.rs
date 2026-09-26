//! Expenses: what was bought, who paid, and how the cost is shared.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`resolve_expense`], which divides an expense into
//! group-currency shares for the ledger, is the group's, and the facade does
//! not re-export it; nor does it re-export the conversion traits
//! ([`FromExpenseError`], [`FromExpenseSplit`], [`FromSplit`],
//! [`FromTipDistribution`], and [`FromTipValue`]), through which a crate that
//! tells every variant of an enum apart converts it without the wildcard arm
//! a `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod error;
mod item;
mod resolution;
mod split;
mod tip;

use std::collections::BTreeSet;

pub use error::{ExpenseError, FromExpenseError};
pub use item::{Item, ItemBuilder};
pub use resolution::resolve_expense;
pub use split::{FromSplit, MemberAmount, MemberParts, MemberPercentage, Split};
pub use tip::{FromTipDistribution, FromTipValue, Tip, TipDistribution, TipValue};

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
/// several, and a whole split or items. An expense gets its identifier when
/// it is built, and keeps it through every edit. Expenses are values: to
/// change one, edit a copy with [`to_builder`](Self::to_builder) and pass it
/// to `Group::replace_expense`.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, CalendarDate, Expense, Money, Percentage, Split, Tip};
/// use eranos::id::MemberId;
///
/// let (ana, bob, carol) = (MemberId::generate(), MemberId::generate(), MemberId::generate());
/// let (amount, date) = (Money::parse("90.00", Currency::EUR)?, CalendarDate::new(23, 9, 2026)?);
/// let dinner = Expense::builder(amount, date, "Dinner", ana, Split::equal([ana, bob, carol]))
///     .tip(Tip::percentage(Percentage::parse("10")?))
///     .build()?;
/// assert_eq!(dinner.tip_amount(), Money::parse("9.00", Currency::EUR)?);
/// assert_eq!(dinner.total(), Money::parse("99.00", Currency::EUR)?);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Expense {
    id: ExpenseId,
    amount: Money,
    date: CalendarDate,
    name: String,
    payers: Split,
    split: ExpenseSplit,
    // Derived from amount and tip by `build`, so never serialised.
    #[cfg_attr(feature = "serde", serde(skip))]
    total: Money,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_uncategorised"))]
    category: CategoryId,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    description: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    exchange_rate: Option<ExchangeRate>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    time: Option<TimeOfDay>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_default_tip"))]
    tip: Tip,
}

impl Expense {
    /// Starts an expense that `payer` paid alone and whose amount `split`
    /// divides.
    ///
    /// A [`Money`] amount is a local cost, in the group currency;
    /// [`Money::at`] makes a converted one.
    pub fn builder(
        cost: impl Into<Cost>,
        date: CalendarDate,
        name: impl Into<String>,
        payer: MemberId,
        split: Split,
    ) -> ExpenseBuilder {
        starting(
            cost.into(),
            date,
            name.into(),
            Split::equal([payer]),
            ExpenseSplit::Whole(split),
        )
    }

    /// Starts an expense that `payer` paid alone and whose amount is divided
    /// item by item; the line totals must add up to the amount.
    pub fn itemised_builder(
        cost: impl Into<Cost>,
        date: CalendarDate,
        items: impl IntoIterator<Item = Item>,
        name: impl Into<String>,
        payer: MemberId,
    ) -> ExpenseBuilder {
        starting(
            cost.into(),
            date,
            name.into(),
            Split::equal([payer]),
            ExpenseSplit::Itemised(items.into_iter().collect()),
        )
    }

    /// Starts an expense whose total (the amount plus the tip) `payers`
    /// divides among the members who paid it, and whose amount `split`
    /// divides.
    pub fn builder_with_payers(
        cost: impl Into<Cost>,
        date: CalendarDate,
        name: impl Into<String>,
        payers: Split,
        split: Split,
    ) -> ExpenseBuilder {
        starting(
            cost.into(),
            date,
            name.into(),
            payers,
            ExpenseSplit::Whole(split),
        )
    }

    /// Starts an expense whose total `payers` divides among the members who
    /// paid it, and whose amount is divided item by item.
    pub fn itemised_builder_with_payers(
        cost: impl Into<Cost>,
        date: CalendarDate,
        items: impl IntoIterator<Item = Item>,
        name: impl Into<String>,
        payers: Split,
    ) -> ExpenseBuilder {
        starting(
            cost.into(),
            date,
            name.into(),
            payers,
            ExpenseSplit::Itemised(items.into_iter().collect()),
        )
    }

    /// Starts a builder holding a copy of this expense, to edit it; the copy
    /// keeps the identifier.
    ///
    /// Calling `split` or `items` on it replaces the split, and `paid_by` or
    /// `payers` the payers, whichever form they had.
    pub fn to_builder(&self) -> ExpenseBuilder {
        ExpenseBuilder {
            id: Some(self.id),
            cost: self.cost(),
            date: self.date,
            name: self.name.clone(),
            payers: self.payers.clone(),
            split: self.split.clone(),
            category: self.category,
            description: self.description.clone(),
            time: self.time,
            tip: self.tip,
        }
    }

    /// The identifier.
    #[must_use]
    pub const fn id(&self) -> ExpenseId {
        self.id
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

    /// The date of the expense.
    #[must_use]
    pub const fn date(&self) -> CalendarDate {
        self.date
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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

    /// The amount plus the tip: what the payers paid.
    #[must_use]
    pub const fn total(&self) -> Money {
        self.total
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
    /// use eranos::{CalendarDate, CategoryId, Currency, Expense, Money, Split};
    /// use eranos::id::MemberId;
    ///
    /// let (ana, food) = (MemberId::generate(), CategoryId::generate());
    /// let (amount, day) = (Money::parse("12.00", Currency::EUR)?, CalendarDate::new(23, 9, 2026)?);
    /// let lunch = Expense::builder(amount, day, "Lunch", ana, Split::equal([ana]))
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

    /// The description, if any.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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
    /// use eranos::{CalendarDate, Currency, ExchangeRate, Expense, Money, Split};
    /// use eranos::id::MemberId;
    ///
    /// let ana = MemberId::generate();
    /// let (amount, day) = (Money::parse("20.00", Currency::EUR)?, CalendarDate::new(23, 9, 2026)?);
    /// let museum = Expense::builder(amount, day, "Museum", ana, Split::equal([ana])).build()?;
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

    /// The tip: [`Tip::default`], no tip, unless another was set.
    #[must_use]
    pub const fn tip(&self) -> Tip {
        self.tip
    }

    /// The tip as an amount in the expense's currency; zero without a tip.
    #[must_use]
    pub const fn tip_amount(&self) -> Money {
        // Both are in the expense currency and total = amount + tip.
        Money::from_minor_units(
            self.amount.currency(),
            self.total.minor_units() - self.amount.minor_units(),
        )
    }

    /// Every member the expense involves, as a payer or as a participant.
    #[must_use]
    pub fn members(&self) -> BTreeSet<MemberId> {
        let mut members: BTreeSet<MemberId> = self.payers.members().collect();
        members.extend(self.split.participants());
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
    /// Every [`Item`] is divided by its own split.
    Itemised(Vec<Item>),
    /// The whole amount is divided by one [`Split`].
    Whole(Split),
}

impl ExpenseSplit {
    /// Every member who shares the cost, once each.
    #[must_use]
    pub fn participants(&self) -> BTreeSet<MemberId> {
        match self {
            Self::Itemised(items) => items
                .iter()
                .flat_map(|item| item.split().members())
                .collect(),
            Self::Whole(split) => split.members().collect(),
        }
    }
}

/// Builds a value from an [`ExpenseSplit`], one method per way of sharing a
/// cost.
pub trait FromExpenseSplit: Sized {
    /// The value for a cost shared item by item.
    fn itemised(items: &[Item]) -> Self;

    /// The value for a cost shared as a whole by `split`.
    fn whole(split: &Split) -> Self;

    /// The value for `expense_split`, from the method of its variant.
    #[must_use]
    fn from_expense_split(expense_split: &ExpenseSplit) -> Self {
        match expense_split {
            ExpenseSplit::Itemised(items) => Self::itemised(items),
            ExpenseSplit::Whole(split) => Self::whole(split),
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
    id: Option<ExpenseId>,
    cost: Cost,
    date: CalendarDate,
    name: String,
    payers: Split,
    split: ExpenseSplit,
    category: CategoryId,
    description: Option<String>,
    time: Option<TimeOfDay>,
    tip: Tip,
}

impl ExpenseBuilder {
    /// Uses `id` instead of a generated identifier, so a test can pin the
    /// tie rotation it seeds.
    #[cfg(test)]
    pub(crate) const fn with_id(mut self, id: ExpenseId) -> Self {
        self.id = Some(id);
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

    /// Replaces the date.
    pub const fn date(mut self, date: CalendarDate) -> Self {
        self.date = date;
        self
    }

    /// Replaces the name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
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
    pub fn items(mut self, items: impl IntoIterator<Item = Item>) -> Self {
        self.split = ExpenseSplit::Itemised(items.into_iter().collect());
        self
    }

    /// Replaces the split with `split`, which divides the whole amount.
    pub fn split(mut self, split: Split) -> Self {
        self.split = ExpenseSplit::Whole(split);
        self
    }

    /// Replaces the category; [`CategoryId::UNCATEGORISED`] clears it.
    pub const fn category(mut self, category: CategoryId) -> Self {
        self.category = category;
        self
    }

    /// Sets the description; blank text clears it.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the time of day, or clears it with `None`.
    pub fn time(mut self, time: impl Into<Option<TimeOfDay>>) -> Self {
        self.time = time.into();
        self
    }

    /// Replaces the tip; [`Tip::default`] removes it.
    pub const fn tip(mut self, tip: Tip) -> Self {
        self.tip = tip;
        self
    }

    /// Validates the expense and returns it, under a newly generated
    /// identifier unless the builder came from [`Expense::to_builder`].
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
            ExpenseSplit::Itemised(items) => validate_items(items, amount)?,
            ExpenseSplit::Whole(split) => split.validate(amount)?,
        }
        let tip_amount = self.tip.amount_on(amount)?;
        let total = amount
            .checked_add(tip_amount)
            .map_err(|_| ExpenseError::Overflow)?;
        payers.validate(total)?;
        Ok(Expense {
            id: self.id.unwrap_or_else(ExpenseId::generate),
            amount,
            date: self.date,
            name,
            payers,
            split,
            total,
            category: self.category,
            description: self.description.and_then(text::optional),
            exchange_rate: self.cost.conversion(),
            time: self.time,
            tip: self.tip,
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
        actual: Money::from_minor_units(amount.currency(), actual),
        expected: amount,
    })
}

/// A builder with the required fields set and every optional one at its
/// default; the constructors and the serialised form start here.
fn starting(
    cost: Cost,
    date: CalendarDate,
    name: String,
    payers: Split,
    split: ExpenseSplit,
) -> ExpenseBuilder {
    ExpenseBuilder {
        id: None,
        cost,
        date,
        name,
        payers,
        split,
        category: CategoryId::UNCATEGORISED,
        description: None,
        time: None,
        tip: Tip::default(),
    }
}

/// Whether `tip` is the default, which the serialised form leaves out.
#[cfg(feature = "serde")]
fn is_default_tip(tip: &Tip) -> bool {
    *tip == Tip::default()
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
    amount: Money,
    date: CalendarDate,
    name: String,
    payers: Split,
    split: ExpenseSplit,
    #[serde(default)]
    category: Option<CategoryId>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    exchange_rate: Option<ExchangeRate>,
    #[serde(default)]
    time: Option<TimeOfDay>,
    #[serde(default)]
    tip: Tip,
}

/// Builds the expense `record` describes, through the checks of the builder.
#[cfg(feature = "serde")]
fn from_record(record: ExpenseRecord) -> Result<Expense, ExpenseError> {
    let cost = Cost::from_parts(record.amount, record.exchange_rate);
    let builder = ExpenseBuilder {
        id: Some(record.id),
        ..starting(cost, record.date, record.name, record.payers, record.split)
    }
    .category(record.category.unwrap_or(CategoryId::UNCATEGORISED))
    .time(record.time)
    .tip(record.tip);
    match record.description {
        Some(description) => builder.description(description).build(),
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
