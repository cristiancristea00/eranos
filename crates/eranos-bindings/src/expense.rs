//! Expenses, how they divide, their items, and their tips, as the bindings
//! carry them.

use eranos::error::{ExpenseError, MoneyError};
use eranos::{CategoryId, ExpenseId, MemberId};
use eranos::{expense as library, money};
use eranos_expense::{FromExpenseSplit, FromSplit, FromTipDistribution, FromTipValue};
use eranos_money::FromCost;

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::money::{Cost, Money};
use crate::{EranosError, identifier};

/// An exact amount one member pays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberAmount {
    /// The amount.
    pub amount: Money,
    /// The member, as their identifier.
    pub member: String,
}

/// The parts one member pays in proportion to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberParts {
    /// The parts, in hundredths: 50 for half a part.
    pub hundredths: i32,
    /// The member, as their identifier.
    pub member: String,
}

/// The percentage one member pays.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberPercentage {
    /// The percentage, in basis points: 2 500 for 25 %.
    pub basis_points: i32,
    /// The member, as their identifier.
    pub member: String,
}

/// How an amount divides among members.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Split {
    /// Every member pays an exact amount; the amounts add up to what they
    /// divide.
    Amounts {
        /// The amount of every member.
        shares: Vec<MemberAmount>,
    },
    /// Every member pays the same.
    Equal {
        /// The members, as identifiers.
        members: Vec<String>,
    },
    /// Every member pays in proportion to their parts.
    Parts {
        /// The parts of every member.
        shares: Vec<MemberParts>,
    },
    /// Every member pays a percentage; the percentages add up to 100 %.
    Percentages {
        /// The percentage of every member.
        shares: Vec<MemberPercentage>,
    },
}

impl Split {
    /// The library's split.
    ///
    /// # Errors
    ///
    /// [`IdFailure::InvalidId`](crate::IdFailure::InvalidId) for a member that
    /// is not an identifier,
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for an amount in an unknown currency,
    /// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for
    /// parts that are not positive, and
    /// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
    /// for a negative percentage.
    pub(crate) fn to_library(&self) -> Result<library::Split, EranosError> {
        Ok(match self {
            Self::Amounts { shares } => library::Split::amounts(
                shares
                    .iter()
                    .map(|share| {
                        Ok(library::MemberAmount::new(
                            share.amount.to_library()?,
                            identifier(&share.member)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EranosError>>()?,
            ),
            Self::Equal { members } => library::Split::equal(
                members
                    .iter()
                    .map(|member| identifier(member))
                    .collect::<Result<Vec<MemberId>, EranosError>>()?,
            ),
            Self::Parts { shares } => library::Split::parts(
                shares
                    .iter()
                    .map(|share| {
                        let hundredths = u32::try_from(share.hundredths)
                            .map_err(|_| MoneyError::InvalidParts)?;
                        Ok(library::MemberParts::new(
                            identifier(&share.member)?,
                            money::Parts::from_hundredths(hundredths)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EranosError>>()?,
            ),
            Self::Percentages { shares } => library::Split::percentages(
                shares
                    .iter()
                    .map(|share| {
                        let basis_points = u32::try_from(share.basis_points)
                            .map_err(|_| MoneyError::InvalidPercentage)?;
                        Ok(library::MemberPercentage::new(
                            identifier(&share.member)?,
                            money::Percentage::from_basis_points(basis_points),
                        ))
                    })
                    .collect::<Result<Vec<_>, EranosError>>()?,
            ),
        })
    }

    /// The members of `split`, in the order it lists them.
    pub(crate) fn members_of(split: &library::Split) -> Vec<String> {
        split.members().map(|member| member.to_string()).collect()
    }
}

impl FromSplit for Split {
    fn amounts(shares: &[library::MemberAmount]) -> Self {
        Self::Amounts {
            shares: shares
                .iter()
                .map(|share| MemberAmount {
                    amount: Money::from_library(share.amount()),
                    member: share.member().to_string(),
                })
                .collect(),
        }
    }

    fn equal(members: &[MemberId]) -> Self {
        Self::Equal {
            members: members.iter().map(ToString::to_string).collect(),
        }
    }

    fn parts(shares: &[library::MemberParts]) -> Self {
        Self::Parts {
            shares: shares
                .iter()
                .map(|share| MemberParts {
                    hundredths: count(share.parts().hundredths()),
                    member: share.member().to_string(),
                })
                .collect(),
        }
    }

    fn percentages(shares: &[library::MemberPercentage]) -> Self {
        Self::Percentages {
            shares: shares
                .iter()
                .map(|share| MemberPercentage {
                    basis_points: count(share.percentage().basis_points()),
                    member: share.member().to_string(),
                })
                .collect(),
        }
    }
}

/// A line of an itemised bill.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    /// What was bought.
    pub name: String,
    /// How the line total divides.
    pub split: Split,
    /// The price of one unit, in the expense's currency.
    pub unit_price: Money,
    /// How many units, from 1.
    pub quantity: i32,
}

impl Item {
    /// The record of `item`.
    pub(crate) fn from_library(item: &library::Item) -> Self {
        Self {
            name: item.name().to_owned(),
            split: Split::from_split(item.split()),
            unit_price: Money::from_library(item.unit_price()),
            quantity: count(item.quantity()),
        }
    }

    /// The library's item.
    ///
    /// # Errors
    ///
    /// The errors of [`Split::to_library`],
    /// [`ExpenseFailure::EmptyItemName`](crate::ExpenseFailure::EmptyItemName)
    /// for a blank name, and
    /// [`ExpenseFailure::ZeroQuantity`](crate::ExpenseFailure::ZeroQuantity)
    /// for a quantity below 1.
    pub(crate) fn to_library(&self) -> Result<library::Item, EranosError> {
        let quantity = u32::try_from(self.quantity).map_err(|_| ExpenseError::ZeroQuantity)?;
        Ok(library::Item::builder_with_split(
            self.name.clone(),
            self.split.to_library()?,
            self.unit_price.to_library()?,
        )
        .quantity(quantity)
        .build()?)
    }
}

/// How an expense's cost is shared: as a whole, or item by item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpenseSplit {
    /// Every item is divided by its own split.
    Itemised {
        /// The lines of the bill.
        items: Vec<Item>,
    },
    /// The whole amount is divided by one split.
    Whole {
        /// How the amount divides.
        split: Split,
    },
}

impl FromExpenseSplit for ExpenseSplit {
    fn itemised(items: &[library::Item]) -> Self {
        Self::Itemised {
            items: items.iter().map(Item::from_library).collect(),
        }
    }

    fn whole(split: &library::Split) -> Self {
        Self::Whole {
            split: Split::from_split(split),
        }
    }
}

/// How a tip is expressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipValue {
    /// A fixed amount in the expense's currency.
    Fixed {
        /// The amount.
        amount: Money,
    },
    /// A percentage of the expense's amount.
    Percentage {
        /// The percentage, in basis points: 1 000 for 10 %.
        basis_points: i32,
    },
}

impl FromTipValue for TipValue {
    fn fixed(amount: money::Money) -> Self {
        Self::Fixed {
            amount: Money::from_library(amount),
        }
    }

    fn percentage(percentage: money::Percentage) -> Self {
        Self::Percentage {
            basis_points: count(percentage.basis_points()),
        }
    }
}

/// How a tip is shared among the participants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipDistribution {
    /// The same for every participant.
    Equal,
    /// In proportion to each participant's share of the amount.
    Proportional,
}

impl TipDistribution {
    /// The library's distribution.
    pub(crate) const fn to_library(self) -> library::TipDistribution {
        match self {
            Self::Equal => library::TipDistribution::Equal,
            Self::Proportional => library::TipDistribution::Proportional,
        }
    }
}

impl FromTipDistribution for TipDistribution {
    fn equal() -> Self {
        Self::Equal
    }

    fn proportional() -> Self {
        Self::Proportional
    }
}

/// A tip on top of an expense, covered by its payers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tip {
    /// How the tip is expressed.
    pub value: TipValue,
    /// How the tip is shared.
    pub distribution: TipDistribution,
}

impl Tip {
    /// The record of `tip`.
    pub(crate) fn from_library(tip: library::Tip) -> Self {
        Self {
            value: TipValue::from_tip_value(tip.value()),
            distribution: TipDistribution::from_tip_distribution(tip.distribution()),
        }
    }

    /// The library's tip.
    ///
    /// # Errors
    ///
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for a fixed tip in an unknown currency, and
    /// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
    /// for a negative percentage.
    pub(crate) fn to_library(&self) -> Result<library::Tip, EranosError> {
        let tip = match &self.value {
            TipValue::Fixed { amount } => library::Tip::fixed(amount.to_library()?),
            TipValue::Percentage { basis_points } => {
                let basis_points =
                    u32::try_from(*basis_points).map_err(|_| MoneyError::InvalidPercentage)?;
                library::Tip::percentage(money::Percentage::from_basis_points(basis_points))
            }
        };
        Ok(tip.distributed(self.distribution.to_library()))
    }
}

/// Something bought for the group.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expense {
    /// The identifier: absent for an expense to add, present for one the
    /// group holds or one to replace.
    pub id: Option<String>,
    /// What it cost.
    pub cost: Cost,
    /// The day it was bought.
    pub date: CalendarDate,
    /// What was bought.
    pub name: String,
    /// Who paid the total, amount plus tip, and how much each.
    pub payers: Split,
    /// How the amount is shared.
    pub split: ExpenseSplit,
    /// The category, as its identifier; absent for the built-in one.
    pub category: Option<String>,
    /// A longer description.
    pub description: Option<String>,
    /// The time of day it was bought.
    pub time: Option<TimeOfDay>,
    /// The tip on top of the amount.
    pub tip: Option<Tip>,
}

impl Expense {
    /// The record of `expense`.
    pub(crate) fn from_library(expense: &library::Expense) -> Self {
        let category = expense.category();
        let tip = expense.tip();
        Self {
            id: Some(expense.id().to_string()),
            cost: Cost::from_cost(expense.cost()),
            date: CalendarDate::from_library(expense.date()),
            name: expense.name().to_owned(),
            payers: Split::from_split(expense.payers()),
            split: ExpenseSplit::from_expense_split(expense.split()),
            category: (CategoryId::UNCATEGORISED != category).then(|| category.to_string()),
            description: expense.description().map(str::to_owned),
            time: expense.time().map(TimeOfDay::from_library),
            tip: (tip != library::Tip::default()).then(|| Tip::from_library(tip)),
        }
    }

    /// The library's expense, for a record to add: it gets a new identifier.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierGiven`] when the record has an identifier,
    /// the errors of the fields' conversions, and those of
    /// [`ExpenseBuilder::build`](library::ExpenseBuilder::build).
    pub(crate) fn to_new(&self) -> Result<library::Expense, EranosError> {
        if self.id.is_some() {
            return Err(EranosError::identifier_given());
        }
        let (cost, date, payers) = (
            self.cost.to_library()?,
            self.date.to_library()?,
            self.payers.to_library()?,
        );
        let builder = match &self.split {
            ExpenseSplit::Itemised { items } => library::Expense::itemised_builder_with_payers(
                cost,
                date,
                items_of(items)?,
                self.name.clone(),
                payers,
            ),
            ExpenseSplit::Whole { split } => library::Expense::builder_with_payers(
                cost,
                date,
                self.name.clone(),
                payers,
                split.to_library()?,
            ),
        };
        Ok(self.with_optional_fields(builder)?.build()?)
    }

    /// The identifier of a record to replace.
    ///
    /// # Errors
    ///
    /// [`EranosError::IdentifierMissing`] when the record has none, and
    /// [`IdFailure::InvalidId`](crate::IdFailure::InvalidId) when it is not an
    /// identifier.
    pub(crate) fn id_to_replace(&self) -> Result<ExpenseId, EranosError> {
        identifier(
            self.id
                .as_deref()
                .ok_or_else(EranosError::identifier_missing)?,
        )
    }

    /// The library's expense that replaces `existing`, keeping its
    /// identifier, with every field of the record.
    ///
    /// # Errors
    ///
    /// The errors of the fields' conversions and those of
    /// [`ExpenseBuilder::build`](library::ExpenseBuilder::build).
    pub(crate) fn replacing(
        &self,
        existing: &library::Expense,
    ) -> Result<library::Expense, EranosError> {
        let builder = existing
            .to_builder()
            .cost(self.cost.to_library()?)
            .date(self.date.to_library()?)
            .name(self.name.clone())
            .payers(self.payers.to_library()?);
        let builder = match &self.split {
            ExpenseSplit::Itemised { items } => builder.items(items_of(items)?),
            ExpenseSplit::Whole { split } => builder.split(split.to_library()?),
        };
        Ok(self.with_optional_fields(builder)?.build()?)
    }

    /// `builder` with the record's category, description, time, and tip,
    /// each cleared when the record has none.
    fn with_optional_fields(
        &self,
        builder: library::ExpenseBuilder,
    ) -> Result<library::ExpenseBuilder, EranosError> {
        let category = match &self.category {
            Some(category) => identifier(category)?,
            None => CategoryId::UNCATEGORISED,
        };
        let tip = match &self.tip {
            Some(tip) => tip.to_library()?,
            None => library::Tip::default(),
        };
        let time = self.time.map(TimeOfDay::to_library).transpose()?;
        Ok(builder
            .category(category)
            .description(self.description.clone().unwrap_or_default())
            .time(time)
            .tip(tip))
    }
}

/// The library's items of `items`.
fn items_of(items: &[Item]) -> Result<Vec<library::Item>, EranosError> {
    items.iter().map(Item::to_library).collect()
}

/// A count the library holds as an unsigned integer, as the bindings carry
/// it; every count here is far below `i32::MAX`, so the fallback never
/// applies.
fn count(number: u32) -> i32 {
    i32::try_from(number).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests;
