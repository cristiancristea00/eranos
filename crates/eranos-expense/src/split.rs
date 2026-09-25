//! Dividing an amount among members.

use std::collections::BTreeSet;

use eranos_core::MemberId;
use eranos_money::allocation::allocate;
use eranos_money::{Money, Parts, Percentage};

use crate::error::ExpenseError;

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
/// use eranos::id::MemberId;
/// use eranos::{
///     Currency, MemberAmount, MemberParts, MemberPercentage, Money, Parts, Percentage, Split,
/// };
///
/// let (ana, bob) = (MemberId::generate(), MemberId::generate());
///
/// let even = Split::equal([ana, bob]);
/// let couple = Split::parts([
///     MemberParts::new(ana, Parts::from_whole(2)?),
///     MemberParts::new(bob, Parts::ONE),
/// ]);
/// let fixed = Split::percentages([
///     MemberPercentage::new(ana, Percentage::parse("70")?),
///     MemberPercentage::new(bob, Percentage::parse("30")?),
/// ]);
/// let exact = Split::amounts([
///     MemberAmount::new(Money::parse("12.50", Currency::EUR)?, ana),
///     MemberAmount::new(Money::parse("7.50", Currency::EUR)?, bob),
/// ]);
/// assert_eq!(exact.members().collect::<Vec<_>>(), [ana, bob]);
/// # let _ = (even, couple, fixed);
/// # Ok::<(), eranos::error::MoneyError>(())
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
    Amounts(Vec<MemberAmount>),
    /// Every member pays the same.
    Equal(Vec<MemberId>),
    /// Every member pays in proportion to their parts.
    Parts(Vec<MemberParts>),
    /// Every member pays a percentage; the percentages add up to exactly
    /// 100 %.
    Percentages(Vec<MemberPercentage>),
}

impl Split {
    /// Divides equally among `members`.
    pub fn equal(members: impl IntoIterator<Item = MemberId>) -> Self {
        Self::Equal(members.into_iter().collect())
    }

    /// Divides by percentage; the percentages must add up to exactly 100 %.
    pub fn percentages(shares: impl IntoIterator<Item = MemberPercentage>) -> Self {
        Self::Percentages(shares.into_iter().collect())
    }

    /// Divides in proportion to each member's parts.
    pub fn parts(shares: impl IntoIterator<Item = MemberParts>) -> Self {
        Self::Parts(shares.into_iter().collect())
    }

    /// Assigns each member an exact amount; the amounts must add up to
    /// exactly the amount being split.
    pub fn amounts(shares: impl IntoIterator<Item = MemberAmount>) -> Self {
        Self::Amounts(shares.into_iter().collect())
    }

    /// The members, in the order they were listed.
    pub fn members(&self) -> impl Iterator<Item = MemberId> + '_ {
        let members: Box<dyn Iterator<Item = MemberId> + '_> = match self {
            Self::Amounts(shares) => Box::new(shares.iter().map(|share| share.member)),
            Self::Equal(members) => Box::new(members.iter().copied()),
            Self::Parts(shares) => Box::new(shares.iter().map(|share| share.member)),
            Self::Percentages(shares) => Box::new(shares.iter().map(|share| share.member)),
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
                for &MemberAmount { amount, member } in shares {
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
                    .map(|share| i128::from(share.amount.minor_units()))
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
                if let Some(share) = shares
                    .iter()
                    .find(|share| share.percentage.basis_points() == 0)
                {
                    return Err(ExpenseError::NonPositiveShare {
                        member: share.member,
                    });
                }
                let total_basis_points = shares
                    .iter()
                    .map(|share| u64::from(share.percentage.basis_points()))
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
                .map(|share| (share.member, share.amount.minor_units().unsigned_abs()))
                .collect(),
            Self::Equal(members) => members.iter().map(|&member| (member, 1)).collect(),
            Self::Parts(shares) => shares
                .iter()
                .map(|share| (share.member, u64::from(share.parts.hundredths())))
                .collect(),
            Self::Percentages(shares) => shares
                .iter()
                .map(|share| (share.member, u64::from(share.percentage.basis_points())))
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

/// A member's exact amount in a [`Split::Amounts`].
///
/// Serialised as the two-element array `[member, amount]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberAmount {
    amount: Money,
    member: MemberId,
}

impl MemberAmount {
    /// The share of `member`: exactly `amount`.
    #[must_use]
    pub const fn new(amount: Money, member: MemberId) -> Self {
        Self { amount, member }
    }

    /// The amount the member pays.
    #[must_use]
    pub const fn amount(self) -> Money {
        self.amount
    }

    /// The member.
    #[must_use]
    pub const fn member(self) -> MemberId {
        self.member
    }
}

/// A member's parts in a [`Split::Parts`].
///
/// Serialised as the two-element array `[member, hundredths]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberParts {
    member: MemberId,
    parts: Parts,
}

impl MemberParts {
    /// The share of `member`: `parts` of the whole.
    #[must_use]
    pub const fn new(member: MemberId, parts: Parts) -> Self {
        Self { member, parts }
    }

    /// The member.
    #[must_use]
    pub const fn member(self) -> MemberId {
        self.member
    }

    /// The member's parts.
    #[must_use]
    pub const fn parts(self) -> Parts {
        self.parts
    }
}

/// A member's percentage in a [`Split::Percentages`].
///
/// Serialised as the two-element array `[member, basis points]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemberPercentage {
    member: MemberId,
    percentage: Percentage,
}

impl MemberPercentage {
    /// The share of `member`: `percentage` of the whole.
    #[must_use]
    pub const fn new(member: MemberId, percentage: Percentage) -> Self {
        Self { member, percentage }
    }

    /// The member.
    #[must_use]
    pub const fn member(self) -> MemberId {
        self.member
    }

    /// The member's percentage.
    #[must_use]
    pub const fn percentage(self) -> Percentage {
        self.percentage
    }
}

/// The shares serialise as the `[member, value]` arrays of format 2, which a
/// derived implementation would write as objects.
#[cfg(feature = "serde")]
mod serde_impls {
    use eranos_core::MemberId;
    use eranos_money::{Money, Parts, Percentage};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{MemberAmount, MemberParts, MemberPercentage};

    impl Serialize for MemberAmount {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            (self.member, self.amount).serialize(serialiser)
        }
    }

    impl<'de> Deserialize<'de> for MemberAmount {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            let (member, amount) = <(MemberId, Money)>::deserialize(deserialiser)?;
            Ok(Self::new(amount, member))
        }
    }

    impl Serialize for MemberParts {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            (self.member, self.parts).serialize(serialiser)
        }
    }

    impl<'de> Deserialize<'de> for MemberParts {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            let (member, parts) = <(MemberId, Parts)>::deserialize(deserialiser)?;
            Ok(Self::new(member, parts))
        }
    }

    impl Serialize for MemberPercentage {
        fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
            (self.member, self.percentage).serialize(serialiser)
        }
    }

    impl<'de> Deserialize<'de> for MemberPercentage {
        fn deserialize<D: Deserializer<'de>>(deserialiser: D) -> Result<Self, D::Error> {
            let (member, percentage) = <(MemberId, Percentage)>::deserialize(deserialiser)?;
            Ok(Self::new(member, percentage))
        }
    }
}

/// Builds a value from a [`Split`], one method per way of dividing.
pub trait FromSplit: Sized {
    /// The value for an exact amount per member.
    fn amounts(shares: &[MemberAmount]) -> Self;

    /// The value for an equal division among `members`.
    fn equal(members: &[MemberId]) -> Self;

    /// The value for a number of parts per member.
    fn parts(shares: &[MemberParts]) -> Self;

    /// The value for a percentage per member.
    fn percentages(shares: &[MemberPercentage]) -> Self;

    /// The value for `split`, from the method of its variant.
    #[must_use]
    fn from_split(split: &Split) -> Self {
        match split {
            Split::Amounts(shares) => Self::amounts(shares),
            Split::Equal(members) => Self::equal(members),
            Split::Parts(shares) => Self::parts(shares),
            Split::Percentages(shares) => Self::percentages(shares),
        }
    }
}

#[cfg(test)]
mod tests;
