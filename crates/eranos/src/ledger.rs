//! The group's totals and balances, kept current with every change.
//!
//! Every mutation of a group becomes a set of changes applied in two phases:
//! every new value is computed and range-checked first, and only then
//! written, so a change that would overflow leaves the totals untouched.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;

use crate::id::{CategoryId, MemberId};
use crate::money::{Currency, Money};

/// The group's running totals, in the group currency.
///
/// Updated in place by every change to the group's entries, so reading it
/// never recomputes anything.
///
/// # Examples
///
/// ```
/// use eranos::{Balance, Currency, Date, Expense, Group, Member, Money, Split};
///
/// let mut group = Group::builder(Currency::EUR, "Weekend").build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?);
/// let bob = group.add_member(Member::builder("Bob").build()?);
/// let (amount, day) = (Money::parse("30.00", Currency::EUR)?, Date::new(23, 9, 2026)?);
/// group.add_expense(Expense::builder(amount, day, "Groceries", ana, Split::equal([ana, bob])).build()?)?;
///
/// let summary = group.summary();
/// assert_eq!(summary.total_spent(), Money::parse("30.00", Currency::EUR)?);
/// assert_eq!(summary.member(ana).paid(), Money::parse("30.00", Currency::EUR)?);
/// assert_eq!(summary.member(ana).balance(), Balance::GetsBack(Money::parse("15.00", Currency::EUR)?));
/// assert_eq!(summary.member(bob).balance(), Balance::Owes(Money::parse("15.00", Currency::EUR)?));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    // In the three maps, rows whose values are all zero are removed, so two
    // summaries with the same totals compare equal however they were
    // reached.
    categories: BTreeMap<Option<CategoryId>, i64>,
    currency: Currency,
    member_categories: BTreeMap<(MemberId, Option<CategoryId>), i64>,
    members: BTreeMap<MemberId, Totals>,
    total_spent: i64,
}

impl Summary {
    const fn new(currency: Currency) -> Self {
        Self {
            categories: BTreeMap::new(),
            currency,
            member_categories: BTreeMap::new(),
            members: BTreeMap::new(),
            total_spent: 0,
        }
    }

    /// The spending in one category (`None` for uncategorised expenses).
    #[must_use]
    pub fn category_total(&self, category: Option<CategoryId>) -> Money {
        Money::from_minor_units(
            self.currency,
            self.categories.get(&category).copied().unwrap_or(0),
        )
    }

    /// The spending of every category with expenses, uncategorised first,
    /// then ordered by category.
    #[must_use]
    pub fn category_totals(
        &self,
    ) -> impl ExactSizeIterator<Item = (Option<CategoryId>, Money)> + '_ {
        self.categories
            .iter()
            .map(|(&category, &total)| (category, Money::from_minor_units(self.currency, total)))
    }

    /// One member's share of the spending in one category (`None` for
    /// uncategorised expenses).
    #[must_use]
    pub fn member_category_total(&self, member: MemberId, category: Option<CategoryId>) -> Money {
        let total = self
            .member_categories
            .get(&(member, category))
            .copied()
            .unwrap_or(0);
        Money::from_minor_units(self.currency, total)
    }

    /// One member's share of the spending in every category they have
    /// expenses in, uncategorised first, then ordered by category.
    pub fn member_category_totals(
        &self,
        member: MemberId,
    ) -> impl Iterator<Item = (Option<CategoryId>, Money)> + '_ {
        self.member_categories
            .range((member, None)..)
            .take_while(move |((row_member, _), _)| *row_member == member)
            .map(|(&(_, category), &total)| {
                (category, Money::from_minor_units(self.currency, total))
            })
    }

    /// The totals of every member with a non-zero total, ordered by member.
    #[must_use]
    pub fn members(&self) -> impl ExactSizeIterator<Item = (MemberId, MemberSummary)> + '_ {
        self.members.iter().map(|(&member, &totals)| {
            (
                member,
                MemberSummary {
                    currency: self.currency,
                    totals,
                },
            )
        })
    }

    /// The totals of one member; all zero for a member without entries.
    #[must_use]
    pub fn member(&self, member: MemberId) -> MemberSummary {
        let totals = self.members.get(&member).copied().unwrap_or_default();
        MemberSummary {
            currency: self.currency,
            totals,
        }
    }

    /// The sum of every expense's total (amount plus tip); transfers are not
    /// spending and do not count.
    #[must_use]
    pub const fn total_spent(&self) -> Money {
        Money::from_minor_units(self.currency, self.total_spent)
    }
}

/// One member's totals in the group currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberSummary {
    currency: Currency,
    totals: Totals,
}

impl MemberSummary {
    /// What the member paid towards expenses.
    #[must_use]
    pub const fn paid(&self) -> Money {
        Money::from_minor_units(self.currency, self.totals.paid)
    }

    /// The member's share of the expenses: what they consumed.
    #[must_use]
    pub const fn share(&self) -> Money {
        Money::from_minor_units(self.currency, self.totals.share)
    }

    /// What the member transferred to others.
    #[must_use]
    pub const fn sent(&self) -> Money {
        Money::from_minor_units(self.currency, self.totals.sent)
    }

    /// What others transferred to the member.
    #[must_use]
    pub const fn received(&self) -> Money {
        Money::from_minor_units(self.currency, self.totals.received)
    }

    /// `paid − share + sent − received`: positive when the group owes the
    /// member, negative when the member owes the group.
    #[must_use]
    pub const fn net(&self) -> Money {
        Money::from_minor_units(self.currency, self.totals.net)
    }

    /// The net amount as a [`Balance`].
    #[must_use]
    pub const fn balance(&self) -> Balance {
        let net = self.totals.net;
        match net.signum() {
            1 => Balance::GetsBack(Money::from_minor_units(self.currency, net)),
            // The ledger keeps every value above i64::MIN, so negating fits.
            -1 => Balance::Owes(Money::from_minor_units(self.currency, -net)),
            _ => Balance::Settled,
        }
    }
}

/// Where a member stands with the group.
///
/// The text form is `owes 12.00 EUR`, `gets back 12.00 EUR`, or `settled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Balance {
    /// The others owe this much to the member.
    GetsBack(Money),
    /// The member owes this much to the others.
    Owes(Money),
    /// The member owes nothing and is owed nothing.
    Settled,
}

impl fmt::Display for Balance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GetsBack(amount) => write!(formatter, "gets back {amount}"),
            Self::Owes(amount) => write!(formatter, "owes {amount}"),
            Self::Settled => formatter.write_str("settled"),
        }
    }
}

/// How one expense divides, in the group currency.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, Date, Expense, Group, Member, Money, Split};
///
/// let mut group = Group::builder(Currency::EUR, "Trip").build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?);
/// let bob = group.add_member(Member::builder("Bob").build()?);
/// let carol = group.add_member(Member::builder("Carol").build()?);
/// let (amount, day) = (Money::parse("10.00", Currency::EUR)?, Date::new(23, 9, 2026)?);
/// let everyone = Split::equal([ana, bob, carol]);
/// let taxi = group.add_expense(Expense::builder(amount, day, "Taxi", ana, everyone).build()?)?;
///
/// let shares = group.expense_shares(taxi).expect("the expense was just added");
/// let cents: Vec<i64> = shares.members().iter().map(|share| share.share().minor_units()).collect();
/// assert_eq!(cents.iter().sum::<i64>(), 1000); // 3.34 + 3.33 + 3.33 in some order
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpenseShares {
    /// What each participant owes each payer, for the direct settlement.
    pub(crate) debts: Vec<Debt>,
    /// Sorted by member.
    pub(crate) members: Vec<MemberShare>,
    pub(crate) tip: Money,
    pub(crate) total: Money,
}

impl ExpenseShares {
    /// What every involved member paid and consumed, ordered by member.
    #[must_use]
    pub fn members(&self) -> &[MemberShare] {
        &self.members
    }

    /// What one member paid and consumed; `None` when the expense does not
    /// involve them.
    pub fn member(&self, member: MemberId) -> Option<&MemberShare> {
        self.members
            .binary_search_by_key(&member, MemberShare::member)
            .ok()
            .and_then(|index| self.members.get(index))
    }

    /// The part of the total that is tip.
    #[must_use]
    pub const fn tip(&self) -> Money {
        self.tip
    }

    /// The expense's total (amount plus tip).
    #[must_use]
    pub const fn total(&self) -> Money {
        self.total
    }
}

/// What one member paid towards and consumed of one expense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberShare {
    pub(crate) member: MemberId,
    pub(crate) paid: Money,
    pub(crate) share: Money,
}

impl MemberShare {
    /// The member.
    #[must_use]
    pub const fn member(&self) -> MemberId {
        self.member
    }

    /// What the member paid.
    #[must_use]
    pub const fn paid(&self) -> Money {
        self.paid
    }

    /// What the member consumed, their tip portion included.
    #[must_use]
    pub const fn share(&self) -> Money {
        self.share
    }
}

/// Two members in a direction: `from` owes or pays `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct MemberPair {
    pub(crate) from: MemberId,
    pub(crate) to: MemberId,
}

impl MemberPair {
    /// The same two members the other way round.
    pub(crate) const fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

// Written out, so that it does not depend on the order of the fields: by
// `from`, then `to`.
impl Ord for MemberPair {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.from, self.to).cmp(&(other.from, other.to))
    }
}

impl PartialOrd for MemberPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// An amount one member owes another because of one entry: `members.from`
/// owes `members.to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Debt {
    pub(crate) amount: i64,
    pub(crate) members: MemberPair,
}

/// A value outside the ledger's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Overflow;

/// The totals of one member, in minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Totals {
    // Kept as its own accumulator: computing it from the other four could
    // overflow even when every total fits.
    net: i64,
    paid: i64,
    received: i64,
    sent: i64,
    share: i64,
}

impl Totals {
    fn is_zero(self) -> bool {
        self == Self::default()
    }
}

/// Changes to apply to a [`Ledger`] at once, in exact 128-bit arithmetic.
#[derive(Debug, Default)]
pub(crate) struct Delta {
    categories: BTreeMap<Option<CategoryId>, i128>,
    member_categories: BTreeMap<(MemberId, Option<CategoryId>), i128>,
    /// paid, share, sent, received, net
    members: BTreeMap<MemberId, [i128; 5]>,
    pairs: BTreeMap<MemberPair, i128>,
    total_spent: i128,
}

impl Delta {
    /// Adds (`sign` 1) or removes (`sign` −1) one expense.
    pub(crate) fn add_expense(
        &mut self,
        shares: &ExpenseShares,
        category: Option<CategoryId>,
        sign: i128,
    ) {
        let total = sign * i128::from(shares.total.minor_units());
        self.total_spent += total;
        *self.categories.entry(category).or_default() += total;
        for share in &shares.members {
            let paid = sign * i128::from(share.paid.minor_units());
            let consumed = sign * i128::from(share.share.minor_units());
            let row = self.members.entry(share.member).or_default();
            row[0] += paid;
            row[1] += consumed;
            row[4] += paid - consumed;
            *self
                .member_categories
                .entry((share.member, category))
                .or_default() += consumed;
        }
        for debt in &shares.debts {
            self.add_debt(debt.members, sign * i128::from(debt.amount));
        }
    }

    /// Adds (`sign` 1) or removes (`sign` −1) one transfer of `amount` minor
    /// units in the group currency, from `pair.from` to `pair.to`.
    pub(crate) fn add_transfer(&mut self, pair: MemberPair, amount: i64, sign: i128) {
        let amount = sign * i128::from(amount);
        let sender = self.members.entry(pair.from).or_default();
        sender[2] += amount;
        sender[4] += amount;
        let recipient = self.members.entry(pair.to).or_default();
        recipient[3] += amount;
        recipient[4] -= amount;
        // Paying someone back is, pairwise, the recipient now owing it back.
        self.add_debt(pair.reversed(), amount);
    }

    /// Records that `pair.from` owes `pair.to` another `amount`.
    fn add_debt(&mut self, pair: MemberPair, amount: i128) {
        // Pairs are keyed with the lower member as `from` and hold what
        // `from` owes `to`.
        if pair.from < pair.to {
            *self.pairs.entry(pair).or_default() += amount;
        } else {
            *self.pairs.entry(pair.reversed()).or_default() -= amount;
        }
    }
}

/// Everything the group derives from its entries: the [`Summary`] and the
/// netted debt between every pair of members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Ledger {
    /// Keyed with the lower member as `from`: what `from` owes `to`,
    /// negative when `to` owes `from`.
    pairs: BTreeMap<MemberPair, i64>,
    summary: Summary,
}

impl Ledger {
    pub(crate) const fn new(currency: Currency) -> Self {
        Self {
            pairs: BTreeMap::new(),
            summary: Summary::new(currency),
        }
    }

    pub(crate) const fn pairs(&self) -> &BTreeMap<MemberPair, i64> {
        &self.pairs
    }

    pub(crate) const fn summary(&self) -> &Summary {
        &self.summary
    }

    pub(crate) const fn currency(&self) -> Currency {
        self.summary.currency
    }

    /// Every non-zero net balance, ordered by member.
    pub(crate) fn balances(&self) -> Vec<(MemberId, i64)> {
        self.summary
            .members
            .iter()
            .filter(|(_, totals)| totals.net != 0)
            .map(|(&member, totals)| (member, totals.net))
            .collect()
    }

    /// Applies `delta` if every resulting value stays in range; otherwise
    /// changes nothing.
    pub(crate) fn apply(&mut self, delta: &Delta) -> Result<(), Overflow> {
        let summary = &self.summary;
        let total_spent = fit(i128::from(summary.total_spent) + delta.total_spent)?;
        let members = delta
            .members
            .iter()
            .map(|(&member, change)| {
                let old = summary.members.get(&member).copied().unwrap_or_default();
                let totals = Totals {
                    net: fit(i128::from(old.net) + change[4])?,
                    paid: fit(i128::from(old.paid) + change[0])?,
                    received: fit(i128::from(old.received) + change[3])?,
                    sent: fit(i128::from(old.sent) + change[2])?,
                    share: fit(i128::from(old.share) + change[1])?,
                };
                Ok((member, totals))
            })
            .collect::<Result<Vec<_>, Overflow>>()?;
        let categories = updated(&summary.categories, &delta.categories)?;
        let member_categories = updated(&summary.member_categories, &delta.member_categories)?;
        let pairs = updated(&self.pairs, &delta.pairs)?;

        self.summary.total_spent = total_spent;
        for (member, totals) in members {
            store(&mut self.summary.members, member, totals, Totals::is_zero);
        }
        for (category, total) in categories {
            store(&mut self.summary.categories, category, total, |total| {
                total == 0
            });
        }
        for (key, total) in member_categories {
            store(&mut self.summary.member_categories, key, total, |total| {
                total == 0
            });
        }
        for (key, amount) in pairs {
            store(&mut self.pairs, key, amount, |amount| amount == 0);
        }
        Ok(())
    }
}

/// The new values of the rows `changes` touches, range-checked.
fn updated<K: Ord + Copy>(
    current: &BTreeMap<K, i64>,
    changes: &BTreeMap<K, i128>,
) -> Result<Vec<(K, i64)>, Overflow> {
    changes
        .iter()
        .map(|(&key, &change)| {
            Ok((
                key,
                fit(i128::from(current.get(&key).copied().unwrap_or(0)) + change)?,
            ))
        })
        .collect()
}

/// Writes a row, or removes it when its value is zero.
fn store<K: Ord, V: Copy>(map: &mut BTreeMap<K, V>, key: K, value: V, is_zero: impl Fn(V) -> bool) {
    match map.entry(key) {
        Entry::Occupied(entry) if is_zero(value) => {
            entry.remove();
        }
        Entry::Occupied(mut entry) => {
            entry.insert(value);
        }
        Entry::Vacant(entry) => {
            if !is_zero(value) {
                entry.insert(value);
            }
        }
    }
}

/// Narrows to the ledger's range, ±`i64::MAX`: symmetric, so every value can
/// be negated.
fn fit(value: i128) -> Result<i64, Overflow> {
    i64::try_from(value)
        .ok()
        .filter(|&value| value != i64::MIN)
        .ok_or(Overflow)
}

#[cfg(test)]
mod tests;
