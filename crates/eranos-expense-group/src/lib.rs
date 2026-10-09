//! Groups: the members, categories, expenses, and transfers of a trip, a
//! household, or any shared spending, with their totals kept current.
//!
//! An internal crate of eranos: depend on `eranos`, which re-exports the
//! types callers use. The facade does not re-export [`FromExpenseGroupError`] and
//! [`FromLanguageError`], through which a crate that tells every failure
//! apart converts one without the wildcard arm a `match` outside this crate
//! would need.
#![forbid(unsafe_code)]
#![doc(test(attr(deny(warnings))))]

mod builder;
mod error;
mod language;
#[cfg(feature = "serde")]
mod saved;
mod shared;

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;

pub use builder::ExpenseGroupBuilder;
pub use error::{ExpenseGroupError, FromExpenseGroupError, FromLanguageError, LanguageError};
pub use language::Language;
#[cfg(feature = "serde")]
pub use saved::SavedExpenseGroup;
pub use shared::{ExpenseGroupReadGuard, ExpenseGroupWriteGuard, SharedExpenseGroup};

use eranos_category::Category;
use eranos_core::cooperative::{Cooperative, Pacer};
use eranos_core::{CategoryId, CategoryKey, ExpenseGroupId, ExpenseId, MemberId, TransferId, text};
use eranos_expense::{Expense, resolve_expense, resolve_expense_at};
use eranos_ledger::{Delta, ExpenseShares, Ledger, MemberPair, ResolvedExpense, Summary};
use eranos_member::Member;
use eranos_money::{Cost, CurrencyCode, CurrencyRate, ExchangeRate, Money};
use eranos_settlement::{SettlementError, SettlementPlan, SettlementStrategy, Settlements};
use eranos_transfer::{Transfer, resolve_transfer, resolve_transfer_at};

/// The version of the saved form. Format 4 names a transfer's members
/// `sender` and `receiver`. Every other version is refused: a newer one
/// would be read with fields silently dropped, and no earlier one is read.
pub(crate) const FORMAT: u32 = 4;

/// The entries a currency change re-resolves, or a load replays, between
/// two yields.
const ENTRIES_PER_YIELD: u64 = 1_024;

/// A group of people sharing expenses: a trip, a day out, a household.
///
/// The group owns its members, categories, expenses, and transfers, and
/// keeps them consistent: every entry references existing members and
/// categories, every foreign-currency entry has a rate into the group
/// currency, and a failed change leaves the group exactly as it was. Its
/// [`Summary`] is updated with every change, in time proportional to the
/// entry's size, and settlements are computed on first request and cached
/// until the balances change.
///
/// # Examples
///
/// ```
/// use eranos::{CurrencyCode, CalendarDate, Expense, ExpenseGroup, Member, Money, SettlementStrategy, Split};
///
/// let mut group = ExpenseGroup::builder("Lisbon", CurrencyCode::EUR).build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?)?;
/// let bob = group.add_member(Member::builder("Bob").build()?)?;
/// let carol = group.add_member(Member::builder("Carol").build()?)?;
///
/// let day = CalendarDate::new(2026, 9, 23)?;
/// let eur = |text| Money::parse(text, CurrencyCode::EUR);
/// let everyone = Split::equal([ana, bob, carol]);
/// group.add_expense(Expense::builder("Hotel", eur("300.00")?, ana, everyone.clone(), day).build()?)?;
/// group.add_expense(Expense::builder("Dinner", eur("60.00")?, bob, everyone, day).build()?)?;
///
/// assert_eq!(group.summary().total_spent(), eur("360.00")?);
/// // Everyone's share is 120.00: Ana gets back 180.00, Bob owes 60.00, Carol 120.00.
/// let settlement = group.settlement(SettlementStrategy::MinimumTransfers)?;
/// assert_eq!(settlement.len(), 2); // Bob pays Ana 60.00, Carol pays Ana 120.00
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct ExpenseGroup {
    id: ExpenseGroupId,
    name: String,
    currency: CurrencyCode,
    note: Option<String>,
    language: Option<Language>,
    members: BTreeMap<MemberId, MemberRecord>,
    categories: BTreeMap<CategoryKey, Category>,
    expenses: BTreeMap<ExpenseId, ExpenseRecord>,
    transfers: BTreeMap<TransferId, TransferRecord>,
    ledger: Ledger,
    settlements: Settlements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemberRecord {
    member: Member,
    archived: bool,
}

#[derive(Debug, Clone)]
struct ExpenseRecord {
    expense: Expense,
    resolved: ResolvedExpense,
}

/// An entry's rate into the new currency and its resolution at that rate,
/// kept until a currency change's last yield.
#[derive(Debug)]
struct Rerated<Resolution> {
    rate: Option<ExchangeRate>,
    resolution: Resolution,
}

#[derive(Debug, Clone)]
struct TransferRecord {
    transfer: Transfer,
    /// The amount in the group currency, in minor units.
    amount: i64,
}

impl ExpenseGroup {
    /// Starts a group with its required name and currency, under an
    /// identifier generated when it is called;
    /// [`builder_with_id`](Self::builder_with_id) takes one instead.
    ///
    /// Every total, balance, and settlement is in the group currency.
    pub fn builder(name: impl Into<String>, currency: CurrencyCode) -> ExpenseGroupBuilder {
        Self::builder_with_id(ExpenseGroupId::generate(), name, currency)
    }

    /// Starts a group with its required name and currency under the
    /// identifier `id`, such as the key an app's own store gives it;
    /// [`builder`](Self::builder) generates one instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, ExpenseGroup, ExpenseGroupId};
    ///
    /// let id = ExpenseGroupId::new("lisbon-2026")?;
    /// let group = ExpenseGroup::builder_with_id(id, "Lisbon", CurrencyCode::EUR).build()?;
    /// assert_eq!(group.id().as_str(), "lisbon-2026");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder_with_id(
        id: ExpenseGroupId,
        name: impl Into<String>,
        currency: CurrencyCode,
    ) -> ExpenseGroupBuilder {
        ExpenseGroupBuilder::new(id, name.into(), currency)
    }

    fn new(
        id: ExpenseGroupId,
        name: String,
        currency: CurrencyCode,
        note: Option<String>,
        language: Option<Language>,
    ) -> Self {
        Self {
            id,
            name,
            currency,
            note,
            language,
            members: BTreeMap::new(),
            categories: BTreeMap::from([(
                CategoryKey::new(CategoryId::UNCATEGORISED),
                Category::uncategorised(),
            )]),
            expenses: BTreeMap::new(),
            transfers: BTreeMap::new(),
            ledger: Ledger::new(currency),
            settlements: Settlements::default(),
        }
    }

    // --- Identifier ----------------------------------------------------------

    /// The identifier.
    pub const fn id(&self) -> ExpenseGroupId {
        self.id
    }

    // --- Name ----------------------------------------------------------------

    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Renames the group.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::EmptyName`] when the name is blank.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<(), ExpenseGroupError> {
        self.name = text::required(name, ExpenseGroupError::EmptyName)?;
        Ok(())
    }

    // --- Currency ------------------------------------------------------------

    /// The currency of every total, balance, and settlement.
    pub const fn currency(&self) -> CurrencyCode {
        self.currency
    }

    /// Moves the group to another currency, re-converting every entry.
    ///
    /// `rates` must hold, for every currency the group's entries use other
    /// than `currency`, its rate into `currency`; entries in the old group
    /// currency count too. Every entry in such a currency gets a converted
    /// cost at that rate, and entries in `currency` a local one. Everything is
    /// recomputed, which takes time proportional to the number of entries;
    /// on an error nothing changes. Moving to the current currency does
    /// nothing.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::DuplicateExchangeRate`] when `rates` lists a currency
    ///   twice.
    /// - [`ExpenseGroupError::MissingExchangeRate`] when a currency in use has no
    ///   rate.
    /// - [`ExpenseGroupError::Overflow`] when a converted total leaves the range of
    ///   an amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{
    ///     CalendarDate, CurrencyCode, CurrencyRate, ExchangeRate, Expense, ExpenseGroup, Member, Money, Split,
    /// };
    ///
    /// let mut group = ExpenseGroup::builder("Trip", CurrencyCode::EUR).build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// let (amount, day) = (Money::parse("20.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
    /// group.add_expense(Expense::builder("Museum", amount, ana, Split::equal([ana]), day).build()?)?;
    ///
    /// let rate = CurrencyRate::new(CurrencyCode::EUR, ExchangeRate::parse("4.97")?);
    /// group.change_currency(CurrencyCode::RON, &[rate])?;
    /// assert_eq!(group.summary().total_spent(), Money::parse("99.40", CurrencyCode::RON)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn change_currency(
        &mut self,
        currency: CurrencyCode,
        rates: &[CurrencyRate],
    ) -> Result<(), ExpenseGroupError> {
        self.change_currency_async(currency, rates).complete()
    }

    /// Moves the group to another currency, as
    /// [`change_currency`](Self::change_currency) does, yielding while it
    /// re-converts the entries.
    ///
    /// It yields once per 1 024 entries and changes the group only after
    /// its last yield, so a call dropped before it completes leaves the
    /// group as it was.
    ///
    /// # Errors
    ///
    /// As for [`change_currency`](Self::change_currency).
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::cooperative::block_on;
    /// use eranos::{
    ///     CalendarDate, CurrencyCode, CurrencyRate, ExchangeRate, Expense, ExpenseGroup, Member, Money, Split,
    /// };
    ///
    /// let mut group = ExpenseGroup::builder("Trip", CurrencyCode::EUR).build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// let (amount, day) = (Money::parse("20.00", CurrencyCode::EUR)?, CalendarDate::new(2026, 9, 23)?);
    /// group.add_expense(Expense::builder("Museum", amount, ana, Split::equal([ana]), day).build()?)?;
    ///
    /// // Awaited like any future; `block_on` runs it on this thread.
    /// let rate = CurrencyRate::new(CurrencyCode::EUR, ExchangeRate::parse("4.97")?);
    /// block_on(group.change_currency_async(CurrencyCode::RON, &[rate]))?;
    /// assert_eq!(group.summary().total_spent(), Money::parse("99.40", CurrencyCode::RON)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn change_currency_async(
        &mut self,
        currency: CurrencyCode,
        rates: &[CurrencyRate],
    ) -> Cooperative<impl Future<Output = Result<(), ExpenseGroupError>> + Send> {
        Cooperative::new(self.change_currency_paced(currency, rates))
    }

    /// The computation of
    /// [`change_currency_async`](Self::change_currency_async).
    async fn change_currency_paced(
        &mut self,
        currency: CurrencyCode,
        rates: &[CurrencyRate],
    ) -> Result<(), ExpenseGroupError> {
        if currency == self.currency {
            return Ok(());
        }
        // A change names a rate for each of a few currencies, so scanning the
        // list finds one sooner than a map keyed by currency would.
        for (position, rate) in rates.iter().enumerate() {
            if rates
                .iter()
                .take(position)
                .any(|earlier| earlier.currency() == rate.currency())
            {
                return Err(ExpenseGroupError::DuplicateExchangeRate {
                    currency: rate.currency(),
                });
            }
        }
        let rate_for = |source: CurrencyCode| -> Result<Option<ExchangeRate>, ExpenseGroupError> {
            if source == currency {
                Ok(None)
            } else {
                rates
                    .iter()
                    .find(|rate| rate.currency() == source)
                    .map(|rate| Some(rate.rate()))
                    .ok_or(ExpenseGroupError::MissingExchangeRate { currency: source })
            }
        };

        // `rate_for` gives every other currency its rate and `currency` none,
        // so every cost has the kind `check_cost` asks for.
        let mut pacer = Pacer::new(ENTRIES_PER_YIELD);
        let mut delta = Delta::default();
        let mut expenses = Vec::with_capacity(self.expenses.len());
        for record in self.expenses.values() {
            let rate = rate_for(record.expense.amount().currency())?;
            let resolution = resolve_expense_at(&record.expense, rate, currency)
                .map_err(|_| ExpenseGroupError::Overflow)?;
            delta.add_expense(&resolution, record.expense.category(), 1);
            expenses.push(Rerated { rate, resolution });
            pacer.advance(1).await;
        }
        let mut transfers = Vec::with_capacity(self.transfers.len());
        for record in self.transfers.values() {
            let rate = rate_for(record.transfer.amount().currency())?;
            let resolution = resolve_transfer_at(&record.transfer, rate, currency)
                .map_err(|_| ExpenseGroupError::Overflow)?;
            delta.add_transfer(pair(&record.transfer), resolution, 1);
            transfers.push(Rerated { rate, resolution });
            pacer.advance(1).await;
        }
        // Nothing is written before the last yield, so a call dropped at
        // any of them leaves the group as it was.
        let mut ledger = Ledger::new(currency);
        ledger
            .apply(&delta)
            .map_err(|_| ExpenseGroupError::Overflow)?;

        // Each entry is moved into its new record, in the same order as its
        // resolution, rather than copied.
        self.expenses = std::mem::take(&mut self.expenses)
            .into_iter()
            .zip(expenses)
            .map(|((id, record), rerated)| {
                let expense = record.expense.with_exchange_rate(rerated.rate);
                let resolved = rerated.resolution;
                (id, ExpenseRecord { expense, resolved })
            })
            .collect();
        self.transfers = std::mem::take(&mut self.transfers)
            .into_iter()
            .zip(transfers)
            .map(|((id, record), rerated)| {
                let transfer = record.transfer.with_exchange_rate(rerated.rate);
                let amount = rerated.resolution;
                (id, TransferRecord { transfer, amount })
            })
            .collect();
        self.currency = currency;
        self.ledger = ledger;
        self.invalidate_settlements();
        Ok(())
    }

    // --- Note ----------------------------------------------------------------

    /// The note, if any.
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    /// Sets the note, trimmed; blank text clears it, as
    /// [`clear_note`](Self::clear_note) does.
    pub fn set_note(&mut self, note: impl Into<String>) {
        self.note = text::optional(note);
    }

    /// Clears the note, which
    /// [`set_note`](Self::set_note) sets.
    pub fn clear_note(&mut self) {
        self.note = None;
    }

    // --- Language ------------------------------------------------------------

    /// The language the apps show the group in, if set.
    pub const fn language(&self) -> Option<&Language> {
        self.language.as_ref()
    }

    /// Sets the language, or clears it with `None`.
    pub fn set_language(&mut self, language: impl Into<Option<Language>>) {
        self.language = language.into();
    }

    // --- Members -------------------------------------------------------------

    /// Every member, archived ones included, in identifier order, which is
    /// creation order for generated identifiers; each carries its
    /// identifier.
    pub fn members(&self) -> impl ExactSizeIterator<Item = &Member> + '_ {
        self.members.values().map(|record| &record.member)
    }

    /// The member with this identifier.
    pub fn member(&self, id: MemberId) -> Option<&Member> {
        self.members.get(&id).map(|record| &record.member)
    }

    /// Whether the member with this identifier is archived; `None` for a
    /// member the group does not hold, as [`member`](Self::member) finds
    /// none.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{CurrencyCode, ExpenseGroup, Member, MemberId};
    ///
    /// let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR).build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// assert_eq!(group.is_archived(ana), Some(false));
    /// group.archive_member(ana)?;
    /// assert_eq!(group.is_archived(ana), Some(true));
    /// assert_eq!(group.is_archived(MemberId::new("stranger")?), None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_archived(&self, id: MemberId) -> Option<bool> {
        self.members.get(&id).map(|record| record.archived)
    }

    /// Adds a member under its identifier.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::DuplicateMember`] when the group already has a member
    /// with this identifier.
    pub fn add_member(&mut self, member: Member) -> Result<MemberId, ExpenseGroupError> {
        let id = member.id();
        match self.members.entry(id) {
            Entry::Occupied(_) => Err(ExpenseGroupError::DuplicateMember(id)),
            Entry::Vacant(slot) => {
                slot.insert(MemberRecord {
                    member,
                    archived: false,
                });
                Ok(id)
            }
        }
    }

    /// Replaces the details of the member with the identifier of `member`,
    /// returning the previous ones; an archived member stays archived.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::UnknownMember`] when there is no such member.
    pub fn replace_member(&mut self, member: Member) -> Result<Member, ExpenseGroupError> {
        let id = member.id();
        let record = self
            .members
            .get_mut(&id)
            .ok_or(ExpenseGroupError::UnknownMember(id))?;
        Ok(std::mem::replace(&mut record.member, member))
    }

    /// Archives a member: they keep their history and balance, and can still
    /// send and receive transfers, but cannot join new expenses. Archiving
    /// an archived member does nothing.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::UnknownMember`] when there is no such member.
    pub fn archive_member(&mut self, id: MemberId) -> Result<(), ExpenseGroupError> {
        self.members
            .get_mut(&id)
            .ok_or(ExpenseGroupError::UnknownMember(id))?
            .archived = true;
        Ok(())
    }

    /// Makes an archived member active again. Unarchiving an active member
    /// does nothing.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::UnknownMember`] when there is no such member.
    pub fn unarchive_member(&mut self, id: MemberId) -> Result<(), ExpenseGroupError> {
        self.members
            .get_mut(&id)
            .ok_or(ExpenseGroupError::UnknownMember(id))?
            .archived = false;
        Ok(())
    }

    /// Removes a member who appears in no expense and no transfer, returning
    /// their details.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::UnknownMember`] when there is no such member.
    /// - [`ExpenseGroupError::MemberInUse`] listing the entries that still
    ///   reference them; archive the member instead to keep that history.
    pub fn remove_member(&mut self, id: MemberId) -> Result<Member, ExpenseGroupError> {
        if !self.members.contains_key(&id) {
            return Err(ExpenseGroupError::UnknownMember(id));
        }
        let expenses: Vec<ExpenseId> = self
            .expenses
            .iter()
            .filter(|(_, record)| record.expense.members().contains(&id))
            .map(|(&id, _)| id)
            .collect();
        let transfers: Vec<TransferId> = self
            .transfers
            .iter()
            .filter(|(_, record)| {
                record.transfer.sender() == id || record.transfer.receiver() == id
            })
            .map(|(&id, _)| id)
            .collect();
        if !expenses.is_empty() || !transfers.is_empty() {
            return Err(ExpenseGroupError::MemberInUse {
                member: id,
                expenses,
                transfers,
            });
        }
        self.members
            .remove(&id)
            .map(|record| record.member)
            .ok_or(ExpenseGroupError::UnknownMember(id))
    }

    // --- Categories ----------------------------------------------------------

    /// Every category: the built-in Uncategorised one first, then the
    /// others in identifier order, which is creation order for generated
    /// identifiers; each carries its identifier.
    pub fn categories(&self) -> impl ExactSizeIterator<Item = &Category> + '_ {
        self.categories.values()
    }

    /// The category with this identifier.
    pub fn category(&self, id: CategoryId) -> Option<&Category> {
        self.categories.get(&CategoryKey::new(id))
    }

    /// Adds a category under its identifier.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::DuplicateCategory`] when the group already has a
    ///   category with this identifier.
    /// - [`ExpenseGroupError::DuplicateCategoryName`] when another category has the
    ///   same name under Unicode canonical caseless matching.
    pub fn add_category(&mut self, category: Category) -> Result<CategoryId, ExpenseGroupError> {
        let id = category.id();
        if self.categories.contains_key(&CategoryKey::new(id)) {
            return Err(ExpenseGroupError::DuplicateCategory(id));
        }
        self.check_category_name(&category, None)?;
        self.categories.insert(CategoryKey::new(id), category);
        Ok(id)
    }

    /// Replaces the category with the identifier of `category`, returning
    /// the previous version; its expenses keep it.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::BuiltInCategory`] for the built-in Uncategorised
    ///   category, whose name is fixed.
    /// - [`ExpenseGroupError::UnknownCategory`] when there is no such category.
    /// - [`ExpenseGroupError::DuplicateCategoryName`] when another category has the
    ///   new name.
    pub fn replace_category(&mut self, category: Category) -> Result<Category, ExpenseGroupError> {
        let id = category.id();
        if CategoryId::UNCATEGORISED == id {
            return Err(ExpenseGroupError::BuiltInCategory);
        }
        if !self.categories.contains_key(&CategoryKey::new(id)) {
            return Err(ExpenseGroupError::UnknownCategory(id));
        }
        self.check_category_name(&category, Some(id))?;
        self.categories
            .insert(CategoryKey::new(id), category)
            .ok_or(ExpenseGroupError::UnknownCategory(id))
    }

    /// Removes a category, returning it; its expenses move to
    /// [`CategoryId::UNCATEGORISED`], and their totals with them.
    ///
    /// No balance changes, so the cached settlements are kept. The time is
    /// proportional to the number of expenses and category totals.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::BuiltInCategory`] for the built-in Uncategorised
    ///   category.
    /// - [`ExpenseGroupError::UnknownCategory`] when there is no such category.
    pub fn remove_category(&mut self, id: CategoryId) -> Result<Category, ExpenseGroupError> {
        if CategoryId::UNCATEGORISED == id {
            return Err(ExpenseGroupError::BuiltInCategory);
        }
        let category = self
            .categories
            .remove(&CategoryKey::new(id))
            .ok_or(ExpenseGroupError::UnknownCategory(id))?;
        for record in self.expenses.values_mut() {
            if record.expense.category() == id {
                record.expense = record
                    .expense
                    .clone()
                    .with_category(CategoryId::UNCATEGORISED);
            }
        }
        self.ledger.move_category(id, CategoryId::UNCATEGORISED);
        Ok(category)
    }

    fn check_category_name(
        &self,
        category: &Category,
        except: Option<CategoryId>,
    ) -> Result<(), ExpenseGroupError> {
        let name = category.folded_name();
        self.categories
            .values()
            .find(|existing| Some(existing.id()) != except && existing.folded_name() == name)
            .map_or(Ok(()), |existing| {
                Err(ExpenseGroupError::DuplicateCategoryName {
                    existing: existing.id(),
                })
            })
    }

    // --- Expenses ------------------------------------------------------------

    /// Every expense, in identifier order, which is creation order for
    /// generated identifiers; each carries its identifier.
    pub fn expenses(&self) -> impl ExactSizeIterator<Item = &Expense> + '_ {
        self.expenses.values().map(|record| &record.expense)
    }

    /// The expense with this identifier.
    pub fn expense(&self, id: ExpenseId) -> Option<&Expense> {
        self.expenses.get(&id).map(|record| &record.expense)
    }

    /// How an expense divides in the group currency.
    pub fn expense_shares(&self, id: ExpenseId) -> Option<&ExpenseShares> {
        self.expenses.get(&id).map(|record| &record.resolved.shares)
    }

    /// Adds an expense under its identifier.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::DuplicateExpense`] when the group already has an
    ///   expense with this identifier.
    /// - [`ExpenseGroupError::UnknownMember`] or [`ExpenseGroupError::ArchivedMember`] when
    ///   a payer or participant is not an active member.
    /// - [`ExpenseGroupError::UnknownCategory`] when the category does not exist.
    /// - [`ExpenseGroupError::MissingExchangeRate`] when the expense is in another
    ///   currency and its cost is local, and
    ///   [`ExpenseGroupError::UnexpectedExchangeRate`] when it is in the group
    ///   currency and its cost is converted.
    /// - [`ExpenseGroupError::Overflow`] when a total would leave the range of an
    ///   amount.
    pub fn add_expense(&mut self, expense: Expense) -> Result<ExpenseId, ExpenseGroupError> {
        let record = self.checked_expense(expense)?;
        let mut delta = Delta::default();
        delta.add_expense(&record.resolved, record.expense.category(), 1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        Ok(self.insert_expense(record))
    }

    /// The record of `expense`, checked against the group as
    /// [`add_expense`](Self::add_expense) checks it and resolved in the
    /// group currency, not yet stored.
    fn checked_expense(&self, expense: Expense) -> Result<ExpenseRecord, ExpenseGroupError> {
        let id = expense.id();
        if self.expenses.contains_key(&id) {
            return Err(ExpenseGroupError::DuplicateExpense(id));
        }
        self.check_expense(&expense, None)?;
        let resolved =
            resolve_expense(&expense, self.currency).map_err(|_| ExpenseGroupError::Overflow)?;
        Ok(ExpenseRecord { expense, resolved })
    }

    /// Stores `record`, whose totals are committed or staged, returning its
    /// identifier.
    fn insert_expense(&mut self, record: ExpenseRecord) -> ExpenseId {
        let id = record.expense.id();
        self.expenses.insert(id, record);
        id
    }

    /// Adds `expense` as [`add_expense`](Self::add_expense) does, but adds
    /// its totals to `delta` rather than to the ledger, for a load that
    /// commits every entry's at once; until that succeeds, the ledger lacks
    /// the entry, so a load whose commit fails drops the group.
    ///
    /// # Errors
    ///
    /// As for [`add_expense`](Self::add_expense), but
    /// [`ExpenseGroupError::Overflow`] only for a resolution out of range.
    #[cfg(feature = "serde")]
    pub(crate) fn stage_expense(
        &mut self,
        expense: Expense,
        delta: &mut Delta,
    ) -> Result<(), ExpenseGroupError> {
        let record = self.checked_expense(expense)?;
        delta.add_expense(&record.resolved, record.expense.category(), 1);
        self.insert_expense(record);
        Ok(())
    }

    /// Replaces the expense with the identifier of `expense`, returning the
    /// previous version.
    ///
    /// The replacement may keep a member who was archived after the
    /// original was added, in the role the original gave them, as a payer
    /// or as a participant, but may not add an archived member or move one
    /// to the other role.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::UnknownExpense`] when there is no such expense, and the
    /// errors of [`add_expense`](Self::add_expense) about the expense itself.
    pub fn replace_expense(&mut self, expense: Expense) -> Result<Expense, ExpenseGroupError> {
        let id = expense.id();
        let previous = self
            .expenses
            .get(&id)
            .ok_or(ExpenseGroupError::UnknownExpense(id))?;
        self.check_expense(&expense, Some(&previous.expense))?;
        let resolved =
            resolve_expense(&expense, self.currency).map_err(|_| ExpenseGroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_expense(&previous.resolved, previous.expense.category(), -1);
        delta.add_expense(&resolved, expense.category(), 1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        // The lookup above found the expense, so `insert` hands it back.
        self.expenses
            .insert(id, ExpenseRecord { expense, resolved })
            .map(|replaced| replaced.expense)
            .ok_or(ExpenseGroupError::UnknownExpense(id))
    }

    /// Removes an expense, returning it.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::UnknownExpense`] when there is no such expense.
    /// - [`ExpenseGroupError::Overflow`] when the remaining totals would leave the
    ///   range of an amount.
    pub fn remove_expense(&mut self, id: ExpenseId) -> Result<Expense, ExpenseGroupError> {
        let Entry::Occupied(slot) = self.expenses.entry(id) else {
            return Err(ExpenseGroupError::UnknownExpense(id));
        };
        let record = slot.get();
        let mut delta = Delta::default();
        delta.add_expense(&record.resolved, record.expense.category(), -1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        Ok(slot.remove().expense)
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in identifier order, which is creation order for
    /// generated identifiers; each carries its identifier.
    pub fn transfers(&self) -> impl ExactSizeIterator<Item = &Transfer> + '_ {
        self.transfers.values().map(|record| &record.transfer)
    }

    /// The transfer with this identifier.
    pub fn transfer(&self, id: TransferId) -> Option<&Transfer> {
        self.transfers.get(&id).map(|record| &record.transfer)
    }

    /// A transfer's amount in the group currency.
    pub fn transfer_amount(&self, id: TransferId) -> Option<Money> {
        self.transfers
            .get(&id)
            .map(|record| Money::from_minor_units(record.amount, self.currency))
    }

    /// Adds a transfer under its identifier.
    ///
    /// Archived members may send and receive transfers, so they can settle
    /// up.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::DuplicateTransfer`] when the group already has a
    ///   transfer with this identifier.
    /// - [`ExpenseGroupError::UnknownMember`] when the sender or the recipient is
    ///   not a member.
    /// - [`ExpenseGroupError::MissingExchangeRate`] or
    ///   [`ExpenseGroupError::UnexpectedExchangeRate`] as for expenses.
    /// - [`ExpenseGroupError::Overflow`] when a total would leave the range of an
    ///   amount.
    pub fn add_transfer(&mut self, transfer: Transfer) -> Result<TransferId, ExpenseGroupError> {
        let record = self.checked_transfer(transfer)?;
        let mut delta = Delta::default();
        delta.add_transfer(pair(&record.transfer), record.amount, 1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        Ok(self.insert_transfer(record))
    }

    /// The record of `transfer`, checked against the group as
    /// [`add_transfer`](Self::add_transfer) checks it and resolved in the
    /// group currency, not yet stored.
    fn checked_transfer(&self, transfer: Transfer) -> Result<TransferRecord, ExpenseGroupError> {
        let id = transfer.id();
        if self.transfers.contains_key(&id) {
            return Err(ExpenseGroupError::DuplicateTransfer(id));
        }
        self.check_transfer(&transfer)?;
        let amount =
            resolve_transfer(&transfer, self.currency).map_err(|_| ExpenseGroupError::Overflow)?;
        Ok(TransferRecord { transfer, amount })
    }

    /// Stores `record`, whose amount is committed or staged, returning its
    /// identifier.
    fn insert_transfer(&mut self, record: TransferRecord) -> TransferId {
        let id = record.transfer.id();
        self.transfers.insert(id, record);
        id
    }

    /// Adds `transfer` as [`add_transfer`](Self::add_transfer) does, but
    /// adds its amount to `delta` rather than to the ledger, as
    /// [`stage_expense`](Self::stage_expense) does for an expense.
    ///
    /// # Errors
    ///
    /// As for [`add_transfer`](Self::add_transfer), but
    /// [`ExpenseGroupError::Overflow`] only for a resolution out of range.
    #[cfg(feature = "serde")]
    pub(crate) fn stage_transfer(
        &mut self,
        transfer: Transfer,
        delta: &mut Delta,
    ) -> Result<(), ExpenseGroupError> {
        let record = self.checked_transfer(transfer)?;
        delta.add_transfer(pair(&record.transfer), record.amount, 1);
        self.insert_transfer(record);
        Ok(())
    }

    /// Replaces the transfer with the identifier of `transfer`, returning
    /// the previous version.
    ///
    /// # Errors
    ///
    /// [`ExpenseGroupError::UnknownTransfer`] when there is no such transfer, and
    /// the errors of [`add_transfer`](Self::add_transfer) about the transfer
    /// itself.
    pub fn replace_transfer(&mut self, transfer: Transfer) -> Result<Transfer, ExpenseGroupError> {
        let id = transfer.id();
        let previous = self
            .transfers
            .get(&id)
            .ok_or(ExpenseGroupError::UnknownTransfer(id))?;
        self.check_transfer(&transfer)?;
        let amount =
            resolve_transfer(&transfer, self.currency).map_err(|_| ExpenseGroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_transfer(pair(&previous.transfer), previous.amount, -1);
        delta.add_transfer(pair(&transfer), amount, 1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        // The lookup above found the transfer, so `insert` hands it back.
        self.transfers
            .insert(id, TransferRecord { transfer, amount })
            .map(|replaced| replaced.transfer)
            .ok_or(ExpenseGroupError::UnknownTransfer(id))
    }

    /// Removes a transfer, returning it.
    ///
    /// # Errors
    ///
    /// - [`ExpenseGroupError::UnknownTransfer`] when there is no such transfer.
    /// - [`ExpenseGroupError::Overflow`] when the remaining totals would leave the
    ///   range of an amount.
    pub fn remove_transfer(&mut self, id: TransferId) -> Result<Transfer, ExpenseGroupError> {
        let Entry::Occupied(slot) = self.transfers.entry(id) else {
            return Err(ExpenseGroupError::UnknownTransfer(id));
        };
        let record = slot.get();
        let mut delta = Delta::default();
        delta.add_transfer(pair(&record.transfer), record.amount, -1);
        commit(&mut self.ledger, &mut self.settlements, &delta)?;
        Ok(slot.remove().transfer)
    }

    // --- Ledger --------------------------------------------------------------

    /// The group's totals: overall, per member, per category, and per member
    /// per category.
    pub const fn summary(&self) -> &Summary {
        self.ledger.summary()
    }

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance, computed with `strategy` on
    /// first request and cached until a balance changes.
    ///
    /// # Errors
    ///
    /// [`SettlementError::OutOfMemory`] when
    /// [`SettlementStrategy::MinimumTransfers`] cannot allocate its table;
    /// the other strategies never fail, [`SettlementStrategy::Adaptive`]
    /// settling greedily instead.
    pub fn settlement(
        &self,
        strategy: SettlementStrategy,
    ) -> Result<&SettlementPlan, SettlementError> {
        self.settlements.get(strategy, &self.ledger)
    }

    /// The payments that settle every balance, as
    /// [`settlement`](Self::settlement) computes and caches them, yielding
    /// while the exact solver fills its table.
    ///
    /// Only the exact solver yields, while it fills its table, and never for
    /// 16 open balances or fewer: [`SettlementStrategy::MinimumTransfers`]
    /// always, and [`SettlementStrategy::Adaptive`] when it settles exactly.
    /// The plan is
    /// cached once it is complete, and [`settlement`](Self::settlement)
    /// reads it; a call dropped before then caches nothing.
    ///
    /// # Errors
    ///
    /// As for [`settlement`](Self::settlement).
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::cooperative::block_on;
    /// use eranos::{CalendarDate, CurrencyCode, ExpenseGroup, Member, Money, SettlementStrategy, Transfer};
    ///
    /// let mut group = ExpenseGroup::builder("Flat", CurrencyCode::EUR).build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// let bob = group.add_member(Member::builder("Bob").build()?)?;
    /// let day = CalendarDate::new(2026, 9, 23)?;
    /// // Bob paid Ana 20.00 too much.
    /// group.add_transfer(Transfer::builder(bob, ana, Money::parse("20.00", CurrencyCode::EUR)?, day).build()?)?;
    ///
    /// // Awaited like any future; `block_on` runs it on this thread.
    /// let settlement = block_on(group.settlement_async(SettlementStrategy::MinimumTransfers))?;
    /// assert_eq!(settlement.payments()[0].sender(), ana);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn settlement_async(
        &self,
        strategy: SettlementStrategy,
    ) -> Cooperative<impl Future<Output = Result<&SettlementPlan, SettlementError>> + Send> {
        self.settlements.get_async(strategy, &self.ledger)
    }

    fn invalidate_settlements(&mut self) {
        self.settlements.clear();
    }

    // --- Checks --------------------------------------------------------------

    /// Checks that `cost` suits the group currency: a local cost in it, and a
    /// converted one in any other currency, even at a rate of 1.
    fn check_cost(&self, cost: Cost) -> Result<(), ExpenseGroupError> {
        let currency = cost.amount().currency();
        match (cost.conversion(), currency == self.currency) {
            (None, false) => Err(ExpenseGroupError::MissingExchangeRate { currency }),
            (None, true) | (Some(_), false) => Ok(()),
            (Some(_), true) => Err(ExpenseGroupError::UnexpectedExchangeRate),
        }
    }

    /// Checks the references and the cost of `expense`; `previous` is the
    /// version it replaces, whose archived members it may keep in the role,
    /// payer or participant, the version gave them.
    fn check_expense(
        &self,
        expense: &Expense,
        previous: Option<&Expense>,
    ) -> Result<(), ExpenseGroupError> {
        for member in expense.payers().members() {
            self.check_role(member, || {
                previous.map(|previous| previous.payers().members().collect())
            })?;
        }
        for member in expense.split().participants() {
            self.check_role(member, || {
                previous.map(|previous| previous.split().participants())
            })?;
        }
        let category = expense.category();
        if !self.categories.contains_key(&CategoryKey::new(category)) {
            return Err(ExpenseGroupError::UnknownCategory(category));
        }
        self.check_cost(expense.cost())
    }

    /// Checks that `member` is in the group and, if archived, among `kept`,
    /// the members the version replaced had in the same role; only an
    /// archived member needs them, so they are listed only then.
    fn check_role(
        &self,
        member: MemberId,
        kept: impl FnOnce() -> Option<BTreeSet<MemberId>>,
    ) -> Result<(), ExpenseGroupError> {
        let record = self
            .members
            .get(&member)
            .ok_or(ExpenseGroupError::UnknownMember(member))?;
        if record.archived && !kept().is_some_and(|kept| kept.contains(&member)) {
            return Err(ExpenseGroupError::ArchivedMember(member));
        }
        Ok(())
    }

    /// Checks the members and the cost of `transfer`.
    fn check_transfer(&self, transfer: &Transfer) -> Result<(), ExpenseGroupError> {
        for member in [transfer.sender(), transfer.receiver()] {
            if !self.members.contains_key(&member) {
                return Err(ExpenseGroupError::UnknownMember(member));
            }
        }
        self.check_cost(transfer.cost())
    }
}

impl PartialEq for ExpenseGroup {
    /// Groups are equal when their details and entries are; the derived
    /// totals and cached settlements follow from those.
    fn eq(&self, other: &Self) -> bool {
        let same_expenses = self.expenses.len() == other.expenses.len()
            && self.expenses.iter().zip(&other.expenses).all(
                |((id, record), (other_id, other_record))| {
                    id == other_id && record.expense == other_record.expense
                },
            );
        let same_transfers = self.transfers.len() == other.transfers.len()
            && self.transfers.iter().zip(&other.transfers).all(
                |((id, record), (other_id, other_record))| {
                    id == other_id && record.transfer == other_record.transfer
                },
            );
        self.id == other.id
            && self.name == other.name
            && self.currency == other.currency
            && self.note == other.note
            && self.language == other.language
            && self.members == other.members
            && self.categories == other.categories
            && same_expenses
            && same_transfers
    }
}

impl Eq for ExpenseGroup {}

/// Applies `delta` to `ledger`, which range-checks every value before it
/// writes any, and drops the kept `settlements`: with the change to the
/// entries that follows it, the one way an expense's or a transfer's totals
/// change a group, so a refused delta leaves the group as it was.
///
/// # Errors
///
/// [`ExpenseGroupError::Overflow`] when a total would leave the range of an
/// amount.
fn commit(
    ledger: &mut Ledger,
    settlements: &mut Settlements,
    delta: &Delta,
) -> Result<(), ExpenseGroupError> {
    ledger
        .apply(delta)
        .map_err(|_| ExpenseGroupError::Overflow)?;
    settlements.clear();
    Ok(())
}

/// The sender and the recipient of `transfer`, as the ledger keys them.
const fn pair(transfer: &Transfer) -> MemberPair {
    MemberPair {
        from: transfer.sender(),
        to: transfer.receiver(),
    }
}

#[cfg(test)]
mod tests;
