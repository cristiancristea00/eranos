//! Expenses, how they divide, their items, and their tips, as the bindings
//! carry them.

use eranos::error::ExpenseError;
use eranos::{CategoryId, MemberId};
use eranos::{expense as library, money};
use eranos_expense::{FromExpenseSplit, FromSplit, FromTipDistribution, FromTipValue};
use eranos_money::FromCost;

use crate::calendar::{CalendarDate, TimeOfDay};
use crate::money::{Cost, Money, parts, percentage};
use crate::{EranosError, identifier};

/// An exact amount one member pays.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberAmount {
    /// The member, as their identifier.
    pub member: String,
    /// The amount.
    pub amount: Money,
}

/// The parts one member pays in proportion to.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberParts {
    /// The member, as their identifier.
    pub member: String,
    /// The parts, in hundredths: 50 for half a part.
    pub hundredths: i32,
}

/// The percentage one member pays.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberPercentage {
    /// The member, as their identifier.
    pub member: String,
    /// The percentage, in basis points: 2 500 for 25 %.
    pub basis_points: i32,
}

/// How an amount divides among members.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Split {
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
    /// Every member pays an exact amount; the amounts add up to what they
    /// divide.
    Amounts {
        /// The amount of every member.
        shares: Vec<MemberAmount>,
    },
}

impl Split {
    /// The library's split.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for a member that is not an identifier,
    /// [`MoneyFailure::UnknownCurrency`](crate::MoneyFailure::UnknownCurrency)
    /// for an amount in an unknown currency,
    /// [`MoneyFailure::InvalidParts`](crate::MoneyFailure::InvalidParts) for
    /// parts that are not positive, and
    /// [`MoneyFailure::InvalidPercentage`](crate::MoneyFailure::InvalidPercentage)
    /// for a negative percentage.
    pub(crate) fn to_library(&self) -> Result<library::Split, EranosError> {
        Ok(match self {
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
                        Ok(library::MemberParts::new(
                            identifier(&share.member)?,
                            parts(share.hundredths)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EranosError>>()?,
            ),
            Self::Percentages { shares } => library::Split::percentages(
                shares
                    .iter()
                    .map(|share| {
                        Ok(library::MemberPercentage::new(
                            identifier(&share.member)?,
                            percentage(share.basis_points)?,
                        ))
                    })
                    .collect::<Result<Vec<_>, EranosError>>()?,
            ),
            Self::Amounts { shares } => library::Split::amounts(
                shares
                    .iter()
                    .map(|share| {
                        Ok(library::MemberAmount::new(
                            identifier(&share.member)?,
                            share.amount.to_library()?,
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
                    member: share.member().to_string(),
                    hundredths: count(share.parts().hundredths()),
                })
                .collect(),
        }
    }

    fn percentages(shares: &[library::MemberPercentage]) -> Self {
        Self::Percentages {
            shares: shares
                .iter()
                .map(|share| MemberPercentage {
                    member: share.member().to_string(),
                    basis_points: count(share.percentage().basis_points()),
                })
                .collect(),
        }
    }

    fn amounts(shares: &[library::MemberAmount]) -> Self {
        Self::Amounts {
            shares: shares
                .iter()
                .map(|share| MemberAmount {
                    member: share.member().to_string(),
                    amount: Money::from_library(share.amount()),
                })
                .collect(),
        }
    }
}

/// A line of an itemised bill.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpenseItem {
    /// What was bought.
    pub name: String,
    /// The price of one unit, in the expense's currency.
    pub unit_price: Money,
    /// How the line total divides.
    pub split: Split,
    /// How many units, from 1.
    pub quantity: i32,
}

impl ExpenseItem {
    /// The record of `item`.
    pub(crate) fn from_library(item: &library::ExpenseItem) -> Self {
        Self {
            name: item.name().to_owned(),
            unit_price: Money::from_library(item.unit_price()),
            split: Split::from_split(item.split()),
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
    pub(crate) fn to_library(&self) -> Result<library::ExpenseItem, EranosError> {
        let quantity = u32::try_from(self.quantity).map_err(|_| ExpenseError::ZeroQuantity)?;
        Ok(library::ExpenseItem::builder_with_split(
            self.name.clone(),
            self.unit_price.to_library()?,
            self.split.to_library()?,
        )
        .quantity(quantity)
        .build()?)
    }
}

/// How an expense's cost is shared: as a whole, or item by item.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpenseSplit {
    /// The whole amount is divided by one split.
    Whole {
        /// How the amount divides.
        split: Split,
    },
    /// Every item is divided by its own split.
    Itemised {
        /// The lines of the bill.
        items: Vec<ExpenseItem>,
    },
}

impl FromExpenseSplit for ExpenseSplit {
    fn whole(split: &library::Split) -> Self {
        Self::Whole {
            split: Split::from_split(split),
        }
    }

    fn itemised(items: &[library::ExpenseItem]) -> Self {
        Self::Itemised {
            items: items.iter().map(ExpenseItem::from_library).collect(),
        }
    }
}

/// How a tip is expressed.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipValue {
    /// A percentage of the expense's amount.
    Percentage {
        /// The percentage, in basis points: 1 000 for 10 %.
        basis_points: i32,
    },
    /// A fixed amount in the expense's currency.
    Fixed {
        /// The amount.
        amount: Money,
    },
}

impl FromTipValue for TipValue {
    fn percentage(percentage: money::Percentage) -> Self {
        Self::Percentage {
            basis_points: count(percentage.basis_points()),
        }
    }

    fn fixed(amount: money::Money) -> Self {
        Self::Fixed {
            amount: Money::from_library(amount),
        }
    }
}

/// How a tip is shared among the participants.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipDistribution {
    /// In proportion to each participant's share of the amount.
    Proportional,
    /// The same for every participant.
    Equal,
}

impl TipDistribution {
    /// The library's distribution.
    pub(crate) const fn to_library(self) -> library::TipDistribution {
        match self {
            Self::Proportional => library::TipDistribution::Proportional,
            Self::Equal => library::TipDistribution::Equal,
        }
    }
}

impl FromTipDistribution for TipDistribution {
    fn proportional() -> Self {
        Self::Proportional
    }

    fn equal() -> Self {
        Self::Equal
    }
}

/// A tip on top of an expense, covered by its payers.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TipRule {
    /// How the tip is expressed.
    pub value: TipValue,
    /// How the tip is shared.
    pub distribution: TipDistribution,
}

impl TipRule {
    /// The record of `tip`.
    pub(crate) fn from_library(tip: library::TipRule) -> Self {
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
    pub(crate) fn to_library(&self) -> Result<library::TipRule, EranosError> {
        let tip = match &self.value {
            TipValue::Percentage { basis_points } => {
                library::TipRule::percentage(percentage(*basis_points)?)
            }
            TipValue::Fixed { amount } => library::TipRule::fixed(amount.to_library()?),
        };
        Ok(tip.distributed(self.distribution.to_library()))
    }
}

/// Something bought for the group.
#[eranos_glue_macros::mirrored]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expense {
    /// The identifier, which the app gives: its own, or one
    /// [`identifier_generate`](crate::identifier_generate) returns.
    pub id: String,
    /// What was bought.
    pub name: String,
    /// What it cost.
    pub cost: Cost,
    /// Who paid the total, amount plus tip, and how much each.
    pub payers: Split,
    /// How the amount is shared.
    pub split: ExpenseSplit,
    /// The day it was bought.
    pub date: CalendarDate,
    /// The tip on top of the amount.
    pub tip: Option<TipRule>,
    /// The time of day it was bought.
    pub time: Option<TimeOfDay>,
    /// The category, as its identifier; absent for the built-in one.
    pub category: Option<String>,
    /// A longer description.
    pub description: Option<String>,
}

impl Expense {
    /// The record of `expense`.
    pub(crate) fn from_library(expense: &library::Expense) -> Self {
        let category = expense.category();
        let tip = expense.tip();
        Self {
            id: expense.id().to_string(),
            name: expense.name().to_owned(),
            cost: Cost::from_cost(expense.cost()),
            payers: Split::from_split(expense.payers()),
            split: ExpenseSplit::from_expense_split(expense.split()),
            date: CalendarDate::from_library(expense.date()),
            tip: (tip != library::TipRule::default()).then(|| TipRule::from_library(tip)),
            time: expense.time().map(TimeOfDay::from_library),
            category: (CategoryId::UNCATEGORISED != category).then(|| category.to_string()),
            description: expense.description().map(str::to_owned),
        }
    }

    /// The library's expense, for a record to add, under the record's
    /// identifier.
    ///
    /// # Errors
    ///
    /// An [`EranosError::Id`] for an identifier the rules refuse, the errors
    /// of the fields' conversions, and those of
    /// [`ExpenseBuilder::build`](library::ExpenseBuilder::build).
    pub(crate) fn to_new(&self) -> Result<library::Expense, EranosError> {
        let id = identifier(&self.id)?;
        let (cost, payers, date) = (
            self.cost.to_library()?,
            self.payers.to_library()?,
            self.date.to_library()?,
        );
        let builder = match &self.split {
            ExpenseSplit::Whole { split } => library::Expense::builder_with_payers_and_id(
                id,
                self.name.clone(),
                cost,
                payers,
                split.to_library()?,
                date,
            ),
            ExpenseSplit::Itemised { items } => {
                library::Expense::itemised_builder_with_payers_and_id(
                    id,
                    self.name.clone(),
                    cost,
                    payers,
                    items_of(items)?,
                    date,
                )
            }
        };
        Ok(self.with_optional_fields(builder)?.build()?)
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
            .name(self.name.clone())
            .cost(self.cost.to_library()?)
            .payers(self.payers.to_library()?)
            .date(self.date.to_library()?);
        let builder = match &self.split {
            ExpenseSplit::Whole { split } => builder.split(split.to_library()?),
            ExpenseSplit::Itemised { items } => builder.items(items_of(items)?),
        };
        Ok(self.with_optional_fields(builder)?.build()?)
    }

    /// `builder` with the record's tip, time, category, and description,
    /// each cleared when the record has none.
    fn with_optional_fields(
        &self,
        builder: library::ExpenseBuilder,
    ) -> Result<library::ExpenseBuilder, EranosError> {
        let tip = match &self.tip {
            Some(tip) => tip.to_library()?,
            None => library::TipRule::default(),
        };
        let time = self.time.map(TimeOfDay::to_library).transpose()?;
        let category = match &self.category {
            Some(category) => identifier(category)?,
            None => CategoryId::UNCATEGORISED,
        };
        Ok(builder
            .tip(tip)
            .time(time)
            .category(category)
            .description(self.description.clone().unwrap_or_default()))
    }
}

/// The library's items of `items`.
fn items_of(items: &[ExpenseItem]) -> Result<Vec<library::ExpenseItem>, EranosError> {
    items.iter().map(ExpenseItem::to_library).collect()
}

/// A count the library holds as an unsigned integer, as the bindings carry
/// it; every count here is far below `i32::MAX`, so the fallback never
/// applies.
fn count(number: u32) -> i32 {
    i32::try_from(number).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests;
