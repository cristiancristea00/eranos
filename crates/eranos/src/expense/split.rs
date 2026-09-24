//! Dividing an amount among members.

use std::collections::BTreeSet;

use crate::error::ExpenseError;
use crate::id::MemberId;
use crate::money::allocation::allocate;
use crate::money::{Money, Parts, Percentage};

/// How an amount is divided among members: who pays for an expense, who
/// shares its cost, or who shares one item.
///
/// Every method divides exactly: the shares add up to the amount, and the
/// minor units left over from rounding go to the members with the largest
/// remainders (for an equal split, to members in an order that rotates from
/// expense to expense). A member may appear only once, and every share must
/// be positive.
///
/// # Examples
///
/// ```
/// use eranos::{Money, Currency, Parts, Percentage, Split};
/// use eranos::id::MemberId;
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
///
/// let even = Split::equal([ana, bob]);
/// let couple = Split::parts([(ana, Parts::from_whole(2)?), (bob, Parts::ONE)]);
/// let fixed = Split::percentages([(ana, Percentage::parse("70")?), (bob, Percentage::parse("30")?)]);
/// let exact = Split::amounts([
///     (ana, Money::parse("12.50", Currency::EUR)?),
///     (bob, Money::parse("7.50", Currency::EUR)?),
/// ]);
/// assert_eq!(exact.members().collect::<Vec<_>>(), [ana, bob]);
/// # let _ = (even, couple, fixed);
/// # Ok::<(), eranos::error::ValueError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum Split {
    /// Every member pays an exact amount; the amounts add up to exactly the
    /// amount being split.
    Amounts(Vec<(MemberId, Money)>),
    /// Every member pays the same.
    Equal(Vec<MemberId>),
    /// Every member pays in proportion to their parts.
    Parts(Vec<(MemberId, Parts)>),
    /// Every member pays a percentage; the percentages add up to exactly
    /// 100 %.
    Percentages(Vec<(MemberId, Percentage)>),
}

impl Split {
    /// Divides equally among `members`.
    pub fn equal(members: impl IntoIterator<Item = MemberId>) -> Self {
        Self::Equal(members.into_iter().collect())
    }

    /// Divides by percentage; the percentages must add up to exactly 100 %.
    pub fn percentages(shares: impl IntoIterator<Item = (MemberId, Percentage)>) -> Self {
        Self::Percentages(shares.into_iter().collect())
    }

    /// Divides in proportion to each member's parts.
    pub fn parts(shares: impl IntoIterator<Item = (MemberId, Parts)>) -> Self {
        Self::Parts(shares.into_iter().collect())
    }

    /// Assigns each member an exact amount; the amounts must add up to
    /// exactly the amount being split.
    pub fn amounts(shares: impl IntoIterator<Item = (MemberId, Money)>) -> Self {
        Self::Amounts(shares.into_iter().collect())
    }

    /// The members, in the order they were listed.
    pub fn members(&self) -> impl Iterator<Item = MemberId> + '_ {
        let members: Box<dyn Iterator<Item = MemberId> + '_> = match self {
            Self::Amounts(shares) => Box::new(shares.iter().map(|&(member, _)| member)),
            Self::Equal(members) => Box::new(members.iter().copied()),
            Self::Parts(shares) => Box::new(shares.iter().map(|&(member, _)| member)),
            Self::Percentages(shares) => Box::new(shares.iter().map(|&(member, _)| member)),
        };
        members
    }

    /// The number of members.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Amounts(shares) => shares.len(),
            Self::Equal(members) => members.len(),
            Self::Parts(shares) => shares.len(),
            Self::Percentages(shares) => shares.len(),
        }
    }

    /// Whether the split lists no member.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks that the split can divide `total`: at least one member, no
    /// member twice, positive shares, and percentages or amounts that add up.
    pub(crate) fn validate(&self, total: Money) -> Result<(), ExpenseError> {
        if self.is_empty() {
            return Err(ExpenseError::EmptySplit);
        }
        let mut seen = BTreeSet::new();
        if let Some(member) = self.members().find(|&member| !seen.insert(member)) {
            return Err(ExpenseError::DuplicateMember { member });
        }
        match self {
            Self::Amounts(shares) => {
                for &(member, amount) in shares {
                    if amount.currency() != total.currency() {
                        return Err(ExpenseError::CurrencyMismatch {
                            expected: total.currency(),
                            found: amount.currency(),
                        });
                    }
                    if !amount.is_positive() {
                        return Err(ExpenseError::NonPositiveShare { member });
                    }
                }
                let sum: i128 = shares
                    .iter()
                    .map(|(_, amount)| i128::from(amount.minor_units()))
                    .sum();
                if sum == i128::from(total.minor_units()) {
                    Ok(())
                } else {
                    let actual = i64::try_from(sum).map_err(|_| ExpenseError::Overflow)?;
                    Err(ExpenseError::AmountsDoNotAddUp {
                        actual: Money::from_minor_units(total.currency(), actual),
                        expected: total,
                    })
                }
            }
            Self::Equal(_) | Self::Parts(_) => Ok(()),
            Self::Percentages(shares) => {
                if let Some(&(member, _)) = shares
                    .iter()
                    .find(|(_, percentage)| percentage.basis_points() == 0)
                {
                    return Err(ExpenseError::NonPositiveShare { member });
                }
                let total_basis_points = shares
                    .iter()
                    .map(|(_, percentage)| u64::from(percentage.basis_points()))
                    .sum();
                if total_basis_points == u64::from(Percentage::HUNDRED.basis_points()) {
                    Ok(())
                } else {
                    Err(ExpenseError::PercentagesDoNotAddUp { total_basis_points })
                }
            }
        }
    }

    /// Divides `total` minor units of a validated split; one share per
    /// member, sorted by member.
    ///
    /// Members are sorted before dividing, so the result does not depend on
    /// the order they were listed in.
    pub(crate) fn divide(&self, total: i64, rotation: u64) -> Vec<(MemberId, i64)> {
        let mut weighted: Vec<(MemberId, u64)> = match self {
            Self::Amounts(shares) => shares
                .iter()
                .map(|&(member, amount)| (member, amount.minor_units().unsigned_abs()))
                .collect(),
            Self::Equal(members) => members.iter().map(|&member| (member, 1)).collect(),
            Self::Parts(shares) => shares
                .iter()
                .map(|&(member, parts)| (member, u64::from(parts.hundredths())))
                .collect(),
            Self::Percentages(shares) => shares
                .iter()
                .map(|&(member, percentage)| (member, u64::from(percentage.basis_points())))
                .collect(),
        };
        weighted.sort_unstable_by_key(|&(member, _)| member);
        let weights: Vec<u64> = weighted.iter().map(|&(_, weight)| weight).collect();
        weighted
            .into_iter()
            .map(|(member, _)| member)
            .zip(allocate(total, &weights, rotation))
            .collect()
    }
}

#[cfg(test)]
mod tests;
