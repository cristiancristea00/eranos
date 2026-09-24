//! Expenses: what was bought, who paid, and how the cost is shared.

mod item;
mod split;
mod tip;

use std::collections::BTreeSet;

pub use item::{Item, ItemBuilder};
pub use split::Split;
pub use tip::{Tip, TipDistribution, TipValue};

use crate::date::{Date, Time};
use crate::error::{ExpenseError, Field};
use crate::id::{CategoryId, ExpenseId, MemberId};
use crate::money::{Cost, ExchangeRate, Money};
use crate::text;

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
/// to [`Group::replace_expense`](crate::Group::replace_expense).
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Date, Expense, Money, Percentage, Split, Tip};
/// use eranos::id::MemberId;
///
/// let (ana, bob, carol) = (MemberId::generate(), MemberId::generate(), MemberId::generate());
/// let (amount, date) = (Money::parse("90.00", Currency::EUR)?, Date::new(23, 9, 2026)?);
/// let dinner = Expense::builder(amount, date, "Dinner", ana, Split::equal([ana, bob, carol]))
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
    id: ExpenseId,
    amount: Money,
    date: Date,
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
    time: Option<Time>,
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
        date: Date,
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
        date: Date,
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
        date: Date,
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
        date: Date,
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
    pub const fn date(&self) -> Date {
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

    pub(crate) const fn set_category(&mut self, category: CategoryId) {
        self.category = category;
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

    pub(crate) const fn set_exchange_rate(&mut self, exchange_rate: Option<ExchangeRate>) {
        self.exchange_rate = exchange_rate;
    }

    /// The time of day, if recorded.
    #[must_use]
    pub const fn time(&self) -> Option<Time> {
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
    date: Date,
    name: String,
    payers: Split,
    split: ExpenseSplit,
    category: CategoryId,
    description: Option<String>,
    time: Option<Time>,
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
    pub const fn date(mut self, date: Date) -> Self {
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
    pub fn time(mut self, time: impl Into<Option<Time>>) -> Self {
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
    /// - [`ExpenseError::Value`] when the name is blank.
    /// - [`ExpenseError::NonPositiveAmount`] when the amount is not positive.
    /// - [`ExpenseError::NoItems`] or [`ExpenseError::ItemsDoNotAddUp`] for
    ///   items that are missing or do not add up to the amount.
    /// - [`ExpenseError::CurrencyMismatch`] when an item, a fixed tip, or a
    ///   split amount is in another currency, and
    ///   [`ExpenseError::NegativeTip`] for a fixed tip below zero.
    /// - Any error of an invalid [`Split`]: the participants' split must
    ///   divide the amount, the payers' split the total.
    pub fn build(self) -> Result<Expense, ExpenseError> {
        let name = text::required(self.name, Field::ExpenseName)?;
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
    date: Date,
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
    *category == CategoryId::UNCATEGORISED
}

/// The serialised form of an [`Expense`], validated on the way in.
#[cfg(feature = "serde")]
#[derive(serde::Deserialize)]
struct ExpenseRecord {
    id: ExpenseId,
    amount: Money,
    date: Date,
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
    time: Option<Time>,
    #[serde(default)]
    tip: Tip,
}

#[cfg(feature = "serde")]
impl TryFrom<ExpenseRecord> for Expense {
    type Error = ExpenseError;

    fn try_from(record: ExpenseRecord) -> Result<Self, Self::Error> {
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
}

#[cfg(test)]
mod tests;
