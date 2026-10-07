//! The group's totals and balances, kept current with every change.
//!
//! Every mutation of a group becomes a set of changes applied in two phases:
//! every new value is computed and range-checked first, and only then
//! written, so a change that would overflow leaves the totals untouched.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. [`Ledger`], [`Delta`], and the pair types are the
//! machinery the group writes through, and the facade does not re-export
//! them; nor does it re-export [`FromBalance`], through which a crate that
//! tells every standing apart converts a balance without the wildcard arm a
//! `match` outside this crate would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::fmt;

use eranos_core::{CategoryId, MemberId};
use eranos_money::{Currency, Money, MoneyError};

/// The group's running totals, in the group currency.
///
/// Updated in place by every change to the group's entries, so reading it
/// never recomputes anything.
///
/// # Examples
///
/// ```
/// use eranos::{Balance, Currency, CalendarDate, Expense, Group, Member, Money, Split};
///
/// let mut group = Group::builder("Weekend", Currency::EUR).build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?)?;
/// let bob = group.add_member(Member::builder("Bob").build()?)?;
/// let (amount, day) = (Money::parse("30.00", Currency::EUR)?, CalendarDate::new(2026, 9, 23)?);
/// group.add_expense(Expense::builder("Groceries", amount, ana, Split::equal([ana, bob]), day).build()?)?;
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
    currency: Currency,
    total_spent: i64,
    // In the three maps, rows whose values are all zero are removed, so two
    // summaries with the same totals compare equal however they were
    // reached.
    members: BTreeMap<MemberId, Totals>,
    categories: BTreeMap<CategoryId, i64>,
    member_categories: BTreeMap<MemberCategory, i64>,
}

impl Summary {
    const fn new(currency: Currency) -> Self {
        Self {
            currency,
            total_spent: 0,
            members: BTreeMap::new(),
            categories: BTreeMap::new(),
            member_categories: BTreeMap::new(),
        }
    }

    /// The sum of every expense's total (amount plus tip); transfers are not
    /// spending and do not count.
    #[must_use]
    pub const fn total_spent(&self) -> Money {
        Money::from_minor_units(self.total_spent, self.currency)
    }

    /// The totals of every member with a non-zero total, ordered by member;
    /// each names its member.
    pub fn members(&self) -> impl ExactSizeIterator<Item = MemberSummary> + '_ {
        self.members.iter().map(|(&member, &totals)| MemberSummary {
            member,
            totals,
            currency: self.currency,
        })
    }

    /// The totals of one member; all zero for a member without entries.
    #[must_use]
    pub fn member(&self, member: MemberId) -> MemberSummary {
        let totals = self.members.get(&member).copied().unwrap_or_default();
        MemberSummary {
            member,
            totals,
            currency: self.currency,
        }
    }

    /// The spending of every category with expenses, ordered by category,
    /// so Uncategorised comes first.
    pub fn category_totals(&self) -> impl ExactSizeIterator<Item = CategoryTotal> + '_ {
        self.categories
            .iter()
            .map(|(&category, &total)| CategoryTotal {
                category,
                total: Money::from_minor_units(total, self.currency),
            })
    }

    /// The spending in one category.
    #[must_use]
    pub fn category_total(&self, category: CategoryId) -> Money {
        Money::from_minor_units(
            self.categories.get(&category).copied().unwrap_or(0),
            self.currency,
        )
    }

    /// One member's share of the spending in every category they have
    /// expenses in, ordered by category, so Uncategorised comes first.
    pub fn member_category_totals(
        &self,
        member: MemberId,
    ) -> impl Iterator<Item = CategoryTotal> + '_ {
        // The nil UUID of Uncategorised is the smallest identifier, so the
        // member's rows start here.
        let first = MemberCategory {
            member,
            category: CategoryId::UNCATEGORISED,
        };
        self.member_categories
            .range(first..)
            .take_while(move |&(key, _)| key.member == member)
            .map(|(key, &total)| CategoryTotal {
                category: key.category,
                total: Money::from_minor_units(total, self.currency),
            })
    }

    /// One member's share of the spending in one category.
    #[must_use]
    pub fn member_category_total(&self, member: MemberId, category: CategoryId) -> Money {
        let total = self
            .member_categories
            .get(&MemberCategory { member, category })
            .copied()
            .unwrap_or(0);
        Money::from_minor_units(total, self.currency)
    }
}

/// One member's totals in the group currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberSummary {
    member: MemberId,
    totals: Totals,
    currency: Currency,
}

impl MemberSummary {
    /// The member the totals belong to.
    #[must_use]
    pub const fn member(&self) -> MemberId {
        self.member
    }

    /// What the member paid towards expenses.
    #[must_use]
    pub const fn paid(&self) -> Money {
        Money::from_minor_units(self.totals.paid, self.currency)
    }

    /// The member's share of the expenses: what they consumed.
    #[must_use]
    pub const fn share(&self) -> Money {
        Money::from_minor_units(self.totals.share, self.currency)
    }

    /// What the member transferred to others.
    #[must_use]
    pub const fn sent(&self) -> Money {
        Money::from_minor_units(self.totals.sent, self.currency)
    }

    /// What others transferred to the member.
    #[must_use]
    pub const fn received(&self) -> Money {
        Money::from_minor_units(self.totals.received, self.currency)
    }

    /// `paid − share + sent − received`: positive when the group owes the
    /// member, negative when the member owes the group.
    #[must_use]
    pub const fn net(&self) -> Money {
        Money::from_minor_units(self.totals.net, self.currency)
    }

    /// The net amount as a [`Balance`].
    #[must_use]
    pub const fn balance(&self) -> Balance {
        let net = self.totals.net;
        match net.signum() {
            // The ledger keeps every value above i64::MIN, so negating fits.
            -1 => Balance::Owes(Money::from_minor_units(-net, self.currency)),
            1 => Balance::GetsBack(Money::from_minor_units(net, self.currency)),
            _ => Balance::Settled,
        }
    }
}

/// The spending in one category, of the whole group or of one member, in
/// the group currency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CategoryTotal {
    category: CategoryId,
    total: Money,
}

impl CategoryTotal {
    /// The category.
    #[must_use]
    pub const fn category(self) -> CategoryId {
        self.category
    }

    /// The spending in it.
    #[must_use]
    pub const fn total(self) -> Money {
        self.total
    }
}

/// Where a member stands with the group.
///
/// The text form is `owes 12.00 EUR`, `settled`, or `gets back 12.00 EUR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Balance {
    /// The member owes this much to the others.
    Owes(Money),
    /// The member owes nothing and is owed nothing.
    Settled,
    /// The others owe this much to the member.
    GetsBack(Money),
}

impl fmt::Display for Balance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owes(amount) => write!(formatter, "owes {amount}"),
            Self::Settled => formatter.write_str("settled"),
            Self::GetsBack(amount) => write!(formatter, "gets back {amount}"),
        }
    }
}

/// Builds a value from a [`Balance`], one method per standing.
pub trait FromBalance: Sized {
    /// The value for a member who owes the others `amount`.
    fn owes(amount: Money) -> Self;

    /// The value for a member who owes nothing and is owed nothing.
    fn settled() -> Self;

    /// The value for a member the others owe `amount`.
    fn gets_back(amount: Money) -> Self;

    /// The value for `balance`, from the method of its variant.
    #[must_use]
    fn from_balance(balance: Balance) -> Self {
        match balance {
            Balance::Owes(amount) => Self::owes(amount),
            Balance::Settled => Self::settled(),
            Balance::GetsBack(amount) => Self::gets_back(amount),
        }
    }
}

/// How one expense divides, in the group currency.
///
/// # Examples
///
/// ```
/// use eranos::{Currency, CalendarDate, Expense, Group, Member, Money, Split};
///
/// let mut group = Group::builder("Trip", Currency::EUR).build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?)?;
/// let bob = group.add_member(Member::builder("Bob").build()?)?;
/// let carol = group.add_member(Member::builder("Carol").build()?)?;
/// let (amount, day) = (Money::parse("10.00", Currency::EUR)?, CalendarDate::new(2026, 9, 23)?);
/// let everyone = Split::equal([ana, bob, carol]);
/// let taxi = group.add_expense(Expense::builder("Taxi", amount, ana, everyone, day).build()?)?;
///
/// let shares = group.expense_shares(taxi).expect("the expense was just added");
/// let cents: Vec<i64> = shares.members().iter().map(|share| share.share().minor_units()).collect();
/// assert_eq!(cents.iter().sum::<i64>(), 1000); // 3.34 + 3.33 + 3.33 in some order
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpenseShares {
    /// Sorted by member.
    pub(crate) members: Vec<MemberShare>,
    pub(crate) total: Money,
    pub(crate) tip: Money,
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

    /// The expense's total (amount plus tip).
    #[must_use]
    pub const fn total(&self) -> Money {
        self.total
    }

    /// The part of the total that is tip.
    #[must_use]
    pub const fn tip(&self) -> Money {
        self.tip
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
    /// Returns what `member` paid towards and consumed of one expense.
    #[must_use]
    pub const fn new(member: MemberId, paid: Money, share: Money) -> Self {
        Self {
            member,
            paid,
            share,
        }
    }

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
// Derived: by `from`, then `to`. The derived comparisons are `#[inline]`,
// which the crates above need: they sort and search by pair, and without the
// attribute each comparison is a call into this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberPair {
    /// The member who owes or pays.
    pub from: MemberId,
    /// The member who is owed or paid.
    pub to: MemberId,
}

impl MemberPair {
    /// The same two members the other way round.
    #[must_use]
    pub const fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

/// An amount one member owes another because of one entry: `members.from`
/// owes `members.to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Debt {
    /// Who owes whom.
    pub members: MemberPair,
    /// The amount, in minor units of the group currency.
    pub amount: i64,
}

/// An expense divided in the group currency: the shares callers read, and
/// what each participant owes each payer, which only the direct settlement
/// reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedExpense {
    /// What every involved member paid and consumed.
    pub shares: ExpenseShares,
    /// What each participant owes each payer, for the direct settlement.
    pub debts: Vec<Debt>,
}

impl ResolvedExpense {
    /// Returns the expense with these debts and, in `members`, sorted by
    /// member, what each involved member paid and consumed.
    #[must_use]
    pub const fn new(
        members: Vec<MemberShare>,
        total: Money,
        tip: Money,
        debts: Vec<Debt>,
    ) -> Self {
        Self {
            shares: ExpenseShares {
                members,
                total,
                tip,
            },
            debts,
        }
    }
}

/// The totals of one member, in minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Totals {
    paid: i64,
    share: i64,
    sent: i64,
    received: i64,
    // Kept as its own accumulator: computing it from the other four could
    // overflow even when every total fits.
    net: i64,
}

impl Totals {
    fn is_zero(self) -> bool {
        self == Self::default()
    }
}

/// The changes to one member's totals, in minor units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TotalsChange {
    paid: i128,
    share: i128,
    sent: i128,
    received: i128,
    net: i128,
}

/// The key of one member's spending in one category.
// Derived: by `member`, then `category`, which keeps each member's rows
// together for `Summary::member_category_totals`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MemberCategory {
    member: MemberId,
    category: CategoryId,
}

/// A row of one of the ledger's maps: its key and its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Row<K, V> {
    key: K,
    value: V,
}

/// Changes to apply to a [`Ledger`] at once, in exact 128-bit arithmetic.
#[derive(Debug, Default)]
pub struct Delta {
    total_spent: i128,
    members: BTreeMap<MemberId, TotalsChange>,
    categories: BTreeMap<CategoryId, i128>,
    member_categories: BTreeMap<MemberCategory, i128>,
    pairs: BTreeMap<MemberPair, i128>,
}

impl Delta {
    /// Adds (`sign` 1) or removes (`sign` −1) one expense.
    pub fn add_expense(&mut self, resolved: &ResolvedExpense, category: CategoryId, sign: i128) {
        let total = sign * i128::from(resolved.shares.total.minor_units());
        self.total_spent += total;
        *self.categories.entry(category).or_default() += total;
        for share in &resolved.shares.members {
            let paid = sign * i128::from(share.paid.minor_units());
            let consumed = sign * i128::from(share.share.minor_units());
            let change = self.members.entry(share.member).or_default();
            change.net += paid - consumed;
            change.paid += paid;
            change.share += consumed;
            let key = MemberCategory {
                member: share.member,
                category,
            };
            *self.member_categories.entry(key).or_default() += consumed;
        }
        for debt in &resolved.debts {
            self.add_debt(debt.members, sign * i128::from(debt.amount));
        }
    }

    /// Adds (`sign` 1) or removes (`sign` −1) one transfer of `amount` minor
    /// units in the group currency, from `pair.from` to `pair.to`.
    pub fn add_transfer(&mut self, pair: MemberPair, amount: i64, sign: i128) {
        let amount = sign * i128::from(amount);
        let sender = self.members.entry(pair.from).or_default();
        sender.net += amount;
        sender.sent += amount;
        let recipient = self.members.entry(pair.to).or_default();
        recipient.net -= amount;
        recipient.received += amount;
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
pub struct Ledger {
    summary: Summary,
    /// Keyed with the lower member as `from`: what `from` owes `to`,
    /// negative when `to` owes `from`.
    pairs: BTreeMap<MemberPair, i64>,
}

impl Ledger {
    /// Returns an empty ledger in `currency`.
    #[must_use]
    pub const fn new(currency: Currency) -> Self {
        Self {
            summary: Summary::new(currency),
            pairs: BTreeMap::new(),
        }
    }

    /// The group's totals.
    #[must_use]
    pub const fn summary(&self) -> &Summary {
        &self.summary
    }

    /// The group currency, which every total is in.
    #[must_use]
    pub const fn currency(&self) -> Currency {
        self.summary.currency
    }

    /// Moves every total of the category `from` to the category `to`, in
    /// the group and per member; no balance changes.
    ///
    /// Category rows are never negative, the group's add up to the total
    /// spent and one member's to their share, and both are in range, so the
    /// sums moved into `to` are in range too.
    pub fn move_category(&mut self, from: CategoryId, to: CategoryId) {
        let summary = &mut self.summary;
        if let Some(total) = summary.categories.remove(&from) {
            *summary.categories.entry(to).or_default() += total;
        }
        let moved: Vec<Row<MemberCategory, i64>> = summary
            .member_categories
            .iter()
            .filter(|&(key, _)| key.category == from)
            .map(|(&key, &value)| Row { key, value })
            .collect();
        for row in moved {
            summary.member_categories.remove(&row.key);
            let key = MemberCategory {
                member: row.key.member,
                category: to,
            };
            *summary.member_categories.entry(key).or_default() += row.value;
        }
    }

    /// The netted debt of every pair of members, keyed with the lower member
    /// as `from`: what `from` owes `to`, negative when `to` owes `from`.
    #[must_use]
    pub const fn pairs(&self) -> &BTreeMap<MemberPair, i64> {
        &self.pairs
    }

    /// Applies `delta` if every resulting value stays in range; otherwise
    /// changes nothing.
    ///
    /// # Errors
    ///
    /// [`MoneyError::Overflow`] when a resulting value would leave the range
    /// of an amount; the ledger is unchanged then.
    pub fn apply(&mut self, delta: &Delta) -> Result<(), MoneyError> {
        let summary = &self.summary;
        let total_spent = fit(i128::from(summary.total_spent) + delta.total_spent)?;
        let members = delta
            .members
            .iter()
            .map(|(&member, change)| {
                let old = summary.members.get(&member).copied().unwrap_or_default();
                let value = Totals {
                    paid: fit(i128::from(old.paid) + change.paid)?,
                    share: fit(i128::from(old.share) + change.share)?,
                    sent: fit(i128::from(old.sent) + change.sent)?,
                    received: fit(i128::from(old.received) + change.received)?,
                    net: fit(i128::from(old.net) + change.net)?,
                };
                Ok(Row { key: member, value })
            })
            .collect::<Result<Vec<_>, MoneyError>>()?;
        let categories = updated(&summary.categories, &delta.categories)?;
        let member_categories = updated(&summary.member_categories, &delta.member_categories)?;
        let pairs = updated(&self.pairs, &delta.pairs)?;

        self.summary.total_spent = total_spent;
        for row in members {
            store(&mut self.summary.members, row, Totals::is_zero);
        }
        for row in categories {
            store(&mut self.summary.categories, row, |total| 0 == total);
        }
        for row in member_categories {
            store(&mut self.summary.member_categories, row, |total| 0 == total);
        }
        for row in pairs {
            store(&mut self.pairs, row, |amount| 0 == amount);
        }
        Ok(())
    }
}

/// The new values of the rows `changes` touches, range-checked.
fn updated<K: Ord + Copy>(
    current: &BTreeMap<K, i64>,
    changes: &BTreeMap<K, i128>,
) -> Result<Vec<Row<K, i64>>, MoneyError> {
    changes
        .iter()
        .map(|(&key, &change)| {
            let value = fit(i128::from(current.get(&key).copied().unwrap_or(0)) + change)?;
            Ok(Row { key, value })
        })
        .collect()
}

/// Writes a row, or removes it when its value is zero.
fn store<K: Ord, V: Copy>(map: &mut BTreeMap<K, V>, row: Row<K, V>, is_zero: impl Fn(V) -> bool) {
    let Row { key, value } = row;
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
fn fit(value: i128) -> Result<i64, MoneyError> {
    i64::try_from(value)
        .ok()
        .filter(|&value| i64::MIN != value)
        .ok_or(MoneyError::Overflow)
}

#[cfg(test)]
mod tests;
