//! Groups: the members, categories, expenses, and transfers of a trip, a
//! household, or any shared spending, with their totals kept current.

mod builder;
mod language;
#[cfg(feature = "serde")]
mod saved;

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

pub use builder::GroupBuilder;
pub use language::Language;
#[cfg(feature = "serde")]
pub use saved::SavedGroup;

use eranos_core::text;
use eranos_expense::resolve_expense;
use eranos_ledger::{Delta, ExpenseShares, Ledger, MemberPair, ResolvedExpense, Summary};
use eranos_settlement::Settlements;

use crate::category::Category;
use crate::error::{GroupError, SettlementError};
use crate::expense::Expense;
use crate::id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
use crate::member::Member;
use crate::money::{Cost, Currency, CurrencyRate, ExchangeRate, Money};
use crate::settlement::{Settlement, SettlementStrategy};
use crate::transfer::{Transfer, resolve_transfer};

/// The version of the saved form. Every other version is refused: a newer
/// one would be read with fields silently dropped, and no earlier one is
/// read.
pub(crate) const FORMAT: u32 = 2;

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
/// use eranos::{Currency, CalendarDate, Expense, Group, Member, Money, SettlementStrategy, Split};
///
/// let mut group = Group::builder(Currency::EUR, "Lisbon").build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?)?;
/// let bob = group.add_member(Member::builder("Bob").build()?)?;
/// let carol = group.add_member(Member::builder("Carol").build()?)?;
///
/// let day = CalendarDate::new(23, 9, 2026)?;
/// let eur = |text| Money::parse(text, Currency::EUR);
/// let everyone = Split::equal([ana, bob, carol]);
/// group.add_expense(Expense::builder(eur("300.00")?, day, "Hotel", ana, everyone.clone()).build()?)?;
/// group.add_expense(Expense::builder(eur("60.00")?, day, "Dinner", bob, everyone).build()?)?;
///
/// assert_eq!(group.summary().total_spent(), eur("360.00")?);
/// // Everyone's share is 120.00: Ana gets back 180.00, Bob owes 60.00, Carol 120.00.
/// let settlement = group.settlement(SettlementStrategy::MinimumTransfers)?;
/// assert_eq!(settlement.len(), 2); // Bob pays Ana 60.00, Carol pays Ana 120.00
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct Group {
    id: GroupId,
    categories: BTreeMap<CategoryId, Category>,
    currency: Currency,
    expenses: BTreeMap<ExpenseId, ExpenseRecord>,
    ledger: Ledger,
    members: BTreeMap<MemberId, MemberRecord>,
    name: String,
    settlements: Settlements,
    transfers: BTreeMap<TransferId, TransferRecord>,
    description: Option<String>,
    language: Option<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MemberRecord {
    archived: bool,
    member: Member,
}

#[derive(Debug, Clone)]
struct ExpenseRecord {
    expense: Expense,
    resolved: ResolvedExpense,
}

#[derive(Debug, Clone)]
struct TransferRecord {
    /// The amount in the group currency, in minor units.
    amount: i64,
    transfer: Transfer,
}

impl Group {
    /// Starts a group with its required currency and name.
    ///
    /// Every total, balance, and settlement is in the group currency.
    pub fn builder(currency: Currency, name: impl Into<String>) -> GroupBuilder {
        GroupBuilder::new(currency, name.into())
    }

    fn new(
        id: GroupId,
        currency: Currency,
        name: String,
        description: Option<String>,
        language: Option<Language>,
    ) -> Self {
        Self {
            id,
            categories: BTreeMap::from([(CategoryId::UNCATEGORISED, Category::uncategorised())]),
            currency,
            expenses: BTreeMap::new(),
            ledger: Ledger::new(currency),
            members: BTreeMap::new(),
            name,
            settlements: Settlements::default(),
            transfers: BTreeMap::new(),
            description,
            language,
        }
    }

    // --- Identifier ----------------------------------------------------------

    /// The identifier.
    pub const fn id(&self) -> GroupId {
        self.id
    }

    // --- Categories ----------------------------------------------------------

    /// Every category, in the order they were created, with the built-in
    /// Uncategorised one first; each carries its identifier.
    pub fn categories(&self) -> impl ExactSizeIterator<Item = &Category> + '_ {
        self.categories.values()
    }

    /// The category with this identifier.
    pub fn category(&self, id: CategoryId) -> Option<&Category> {
        self.categories.get(&id)
    }

    /// Adds a category under its identifier.
    ///
    /// # Errors
    ///
    /// - [`GroupError::DuplicateCategory`] when the group already has a
    ///   category with this identifier.
    /// - [`GroupError::DuplicateCategoryName`] when another category has the
    ///   same name under Unicode canonical caseless matching.
    pub fn add_category(&mut self, category: Category) -> Result<CategoryId, GroupError> {
        let id = category.id();
        if self.categories.contains_key(&id) {
            return Err(GroupError::DuplicateCategory(id));
        }
        self.check_category_name(&category, None)?;
        self.categories.insert(id, category);
        Ok(id)
    }

    /// Replaces the category with the identifier of `category`, returning
    /// the previous version; its expenses keep it.
    ///
    /// # Errors
    ///
    /// - [`GroupError::BuiltInCategory`] for the built-in Uncategorised
    ///   category, whose name is fixed.
    /// - [`GroupError::UnknownCategory`] when there is no such category.
    /// - [`GroupError::DuplicateCategoryName`] when another category has the
    ///   new name.
    pub fn replace_category(&mut self, category: Category) -> Result<Category, GroupError> {
        let id = category.id();
        if id == CategoryId::UNCATEGORISED {
            return Err(GroupError::BuiltInCategory);
        }
        if !self.categories.contains_key(&id) {
            return Err(GroupError::UnknownCategory(id));
        }
        self.check_category_name(&category, Some(id))?;
        self.categories
            .insert(id, category)
            .ok_or(GroupError::UnknownCategory(id))
    }

    /// Removes a category, returning it; its expenses move to
    /// [`CategoryId::UNCATEGORISED`], and their totals with them.
    ///
    /// No balance changes, so the cached settlements are kept. The time is
    /// proportional to the number of expenses and category totals.
    ///
    /// # Errors
    ///
    /// - [`GroupError::BuiltInCategory`] for the built-in Uncategorised
    ///   category.
    /// - [`GroupError::UnknownCategory`] when there is no such category.
    pub fn remove_category(&mut self, id: CategoryId) -> Result<Category, GroupError> {
        if id == CategoryId::UNCATEGORISED {
            return Err(GroupError::BuiltInCategory);
        }
        let category = self
            .categories
            .remove(&id)
            .ok_or(GroupError::UnknownCategory(id))?;
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
    ) -> Result<(), GroupError> {
        let name = category.folded_name();
        match self
            .categories
            .iter()
            .find(|&(&id, existing)| Some(id) != except && existing.folded_name() == name)
        {
            Some((&existing, _)) => Err(GroupError::DuplicateCategoryName { existing }),
            None => Ok(()),
        }
    }

    // --- Currency ------------------------------------------------------------

    /// The currency of every total, balance, and settlement.
    pub const fn currency(&self) -> Currency {
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
    /// - [`GroupError::DuplicateExchangeRate`] when `rates` lists a currency
    ///   twice.
    /// - [`GroupError::MissingExchangeRate`] when a currency in use has no
    ///   rate.
    /// - [`GroupError::Overflow`] when a converted total leaves the range of
    ///   an amount.
    ///
    /// # Examples
    ///
    /// ```
    /// use eranos::{
    ///     CalendarDate, Currency, CurrencyRate, ExchangeRate, Expense, Group, Member, Money, Split,
    /// };
    ///
    /// let mut group = Group::builder(Currency::EUR, "Trip").build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?)?;
    /// let (amount, day) = (Money::parse("20.00", Currency::EUR)?, CalendarDate::new(23, 9, 2026)?);
    /// group.add_expense(Expense::builder(amount, day, "Museum", ana, Split::equal([ana])).build()?)?;
    ///
    /// let rate = CurrencyRate::new(Currency::EUR, ExchangeRate::parse("4.97")?);
    /// group.change_currency(Currency::RON, &[rate])?;
    /// assert_eq!(group.summary().total_spent(), Money::parse("99.40", Currency::RON)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn change_currency(
        &mut self,
        currency: Currency,
        rates: &[CurrencyRate],
    ) -> Result<(), GroupError> {
        if currency == self.currency {
            return Ok(());
        }
        let mut table = BTreeMap::new();
        for rate in rates {
            if table.insert(rate.currency(), rate.rate()).is_some() {
                return Err(GroupError::DuplicateExchangeRate {
                    currency: rate.currency(),
                });
            }
        }
        let rate_for = |source: Currency| -> Result<Option<ExchangeRate>, GroupError> {
            if source == currency {
                Ok(None)
            } else {
                table
                    .get(&source)
                    .copied()
                    .map(Some)
                    .ok_or(GroupError::MissingExchangeRate { currency: source })
            }
        };

        // `rate_for` gives every other currency its rate and `currency` none,
        // so every cost has the kind `check_cost` asks for.
        let mut delta = Delta::default();
        let mut expenses = BTreeMap::new();
        for (&id, record) in &self.expenses {
            let rate = rate_for(record.expense.amount().currency())?;
            let expense = record.expense.clone().with_exchange_rate(rate);
            let resolved = resolve_expense(&expense, currency).map_err(|_| GroupError::Overflow)?;
            delta.add_expense(&resolved, expense.category(), 1);
            expenses.insert(id, ExpenseRecord { expense, resolved });
        }
        let mut transfers = BTreeMap::new();
        for (&id, record) in &self.transfers {
            let rate = rate_for(record.transfer.amount().currency())?;
            let transfer = record.transfer.clone().with_exchange_rate(rate);
            let amount = resolve_transfer(&transfer, currency).map_err(|_| GroupError::Overflow)?;
            delta.add_transfer(pair(&transfer), amount, 1);
            transfers.insert(id, TransferRecord { amount, transfer });
        }
        let mut ledger = Ledger::new(currency);
        ledger.apply(&delta).map_err(|_| GroupError::Overflow)?;

        self.currency = currency;
        self.expenses = expenses;
        self.transfers = transfers;
        self.ledger = ledger;
        self.invalidate_settlements();
        Ok(())
    }

    // --- Expenses ------------------------------------------------------------

    /// Every expense, in the order they were created; each carries its
    /// identifier.
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
    /// - [`GroupError::DuplicateExpense`] when the group already has an
    ///   expense with this identifier.
    /// - [`GroupError::UnknownMember`] or [`GroupError::ArchivedMember`] when
    ///   a payer or participant is not an active member.
    /// - [`GroupError::UnknownCategory`] when the category does not exist.
    /// - [`GroupError::MissingExchangeRate`] when the expense is in another
    ///   currency and its cost is local, and
    ///   [`GroupError::UnexpectedExchangeRate`] when it is in the group
    ///   currency and its cost is converted.
    /// - [`GroupError::Overflow`] when a total would leave the range of an
    ///   amount.
    pub fn add_expense(&mut self, expense: Expense) -> Result<ExpenseId, GroupError> {
        let id = expense.id();
        if self.expenses.contains_key(&id) {
            return Err(GroupError::DuplicateExpense(id));
        }
        self.check_expense(&expense, None)?;
        let resolved =
            resolve_expense(&expense, self.currency).map_err(|_| GroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_expense(&resolved, expense.category(), 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.expenses
            .insert(id, ExpenseRecord { expense, resolved });
        self.invalidate_settlements();
        Ok(id)
    }

    /// Replaces the expense with the identifier of `expense`, returning the
    /// previous version.
    ///
    /// The replacement may keep members who were archived after the
    /// original was added, but may not add archived members.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownExpense`] when there is no such expense, and the
    /// errors of [`add_expense`](Self::add_expense) about the expense itself.
    pub fn replace_expense(&mut self, expense: Expense) -> Result<Expense, GroupError> {
        let id = expense.id();
        let previous = self
            .expenses
            .get(&id)
            .ok_or(GroupError::UnknownExpense(id))?;
        self.check_expense(&expense, Some(&previous.expense))?;
        let resolved =
            resolve_expense(&expense, self.currency).map_err(|_| GroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_expense(&previous.resolved, previous.expense.category(), -1);
        delta.add_expense(&resolved, expense.category(), 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        let replaced = self
            .expenses
            .insert(id, ExpenseRecord { expense, resolved });
        self.invalidate_settlements();
        replaced
            .map(|replaced| replaced.expense)
            .ok_or(GroupError::UnknownExpense(id))
    }

    /// Removes an expense, returning it.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownExpense`] when there is no such expense.
    /// - [`GroupError::Overflow`] when the remaining totals would leave the
    ///   range of an amount.
    pub fn remove_expense(&mut self, id: ExpenseId) -> Result<Expense, GroupError> {
        let record = self
            .expenses
            .get(&id)
            .ok_or(GroupError::UnknownExpense(id))?;
        let mut delta = Delta::default();
        delta.add_expense(&record.resolved, record.expense.category(), -1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.invalidate_settlements();
        self.expenses
            .remove(&id)
            .map(|removed| removed.expense)
            .ok_or(GroupError::UnknownExpense(id))
    }

    // --- Ledger --------------------------------------------------------------

    /// The group's totals: overall, per member, per category, and per member
    /// per category.
    pub const fn summary(&self) -> &Summary {
        self.ledger.summary()
    }

    // --- Members -------------------------------------------------------------

    /// Every member, archived ones included, in the order they were created;
    /// each carries its identifier.
    pub fn members(&self) -> impl ExactSizeIterator<Item = &Member> + '_ {
        self.members.values().map(|record| &record.member)
    }

    /// The member with this identifier.
    pub fn member(&self, id: MemberId) -> Option<&Member> {
        self.members.get(&id).map(|record| &record.member)
    }

    /// Whether the member is archived; `false` for an unknown member.
    pub fn is_archived(&self, id: MemberId) -> bool {
        self.members.get(&id).is_some_and(|record| record.archived)
    }

    /// Adds a member under its identifier.
    ///
    /// # Errors
    ///
    /// [`GroupError::DuplicateMember`] when the group already has a member
    /// with this identifier.
    pub fn add_member(&mut self, member: Member) -> Result<MemberId, GroupError> {
        let id = member.id();
        match self.members.entry(id) {
            Entry::Occupied(_) => Err(GroupError::DuplicateMember(id)),
            Entry::Vacant(slot) => {
                slot.insert(MemberRecord {
                    archived: false,
                    member,
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
    /// [`GroupError::UnknownMember`] when there is no such member.
    pub fn replace_member(&mut self, member: Member) -> Result<Member, GroupError> {
        let id = member.id();
        let record = self
            .members
            .get_mut(&id)
            .ok_or(GroupError::UnknownMember(id))?;
        Ok(std::mem::replace(&mut record.member, member))
    }

    /// Archives a member: they keep their history and balance, and can still
    /// send and receive transfers, but cannot join new expenses. Archiving
    /// an archived member does nothing.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownMember`] when there is no such member.
    pub fn archive_member(&mut self, id: MemberId) -> Result<(), GroupError> {
        self.members
            .get_mut(&id)
            .ok_or(GroupError::UnknownMember(id))?
            .archived = true;
        Ok(())
    }

    /// Makes an archived member active again. Unarchiving an active member
    /// does nothing.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownMember`] when there is no such member.
    pub fn unarchive_member(&mut self, id: MemberId) -> Result<(), GroupError> {
        self.members
            .get_mut(&id)
            .ok_or(GroupError::UnknownMember(id))?
            .archived = false;
        Ok(())
    }

    /// Removes a member who appears in no expense and no transfer, returning
    /// their details.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownMember`] when there is no such member.
    /// - [`GroupError::MemberInUse`] listing the entries that still
    ///   reference them; archive the member instead to keep that history.
    pub fn remove_member(&mut self, id: MemberId) -> Result<Member, GroupError> {
        if !self.members.contains_key(&id) {
            return Err(GroupError::UnknownMember(id));
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
            .filter(|(_, record)| record.transfer.from() == id || record.transfer.to() == id)
            .map(|(&id, _)| id)
            .collect();
        if !expenses.is_empty() || !transfers.is_empty() {
            return Err(GroupError::MemberInUse {
                expenses,
                member: id,
                transfers,
            });
        }
        self.members
            .remove(&id)
            .map(|record| record.member)
            .ok_or(GroupError::UnknownMember(id))
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
    /// [`GroupError::EmptyName`] when the name is blank.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<(), GroupError> {
        self.name = text::required(name, GroupError::EmptyName)?;
        Ok(())
    }

    // --- Settlements ---------------------------------------------------------

    /// The payments that settle every balance, computed with `strategy` on
    /// first request and cached until a balance changes.
    ///
    /// # Errors
    ///
    /// [`SettlementError::OutOfMemory`] when
    /// [`SettlementStrategy::MinimumTransfers`] cannot allocate its table;
    /// the other strategies never fail.
    pub fn settlement(&self, strategy: SettlementStrategy) -> Result<&Settlement, SettlementError> {
        self.settlements.get(strategy, &self.ledger)
    }

    fn invalidate_settlements(&mut self) {
        self.settlements.clear();
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in the order they were created; each carries its
    /// identifier.
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
            .map(|record| Money::from_minor_units(self.currency, record.amount))
    }

    /// Adds a transfer under its identifier.
    ///
    /// Archived members may send and receive transfers, so they can settle
    /// up.
    ///
    /// # Errors
    ///
    /// - [`GroupError::DuplicateTransfer`] when the group already has a
    ///   transfer with this identifier.
    /// - [`GroupError::UnknownMember`] when the sender or the recipient is
    ///   not a member.
    /// - [`GroupError::MissingExchangeRate`] or
    ///   [`GroupError::UnexpectedExchangeRate`] as for expenses.
    /// - [`GroupError::Overflow`] when a total would leave the range of an
    ///   amount.
    pub fn add_transfer(&mut self, transfer: Transfer) -> Result<TransferId, GroupError> {
        let id = transfer.id();
        if self.transfers.contains_key(&id) {
            return Err(GroupError::DuplicateTransfer(id));
        }
        self.check_transfer(&transfer)?;
        let amount =
            resolve_transfer(&transfer, self.currency).map_err(|_| GroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_transfer(pair(&transfer), amount, 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.transfers
            .insert(id, TransferRecord { amount, transfer });
        self.invalidate_settlements();
        Ok(id)
    }

    /// Replaces the transfer with the identifier of `transfer`, returning
    /// the previous version.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownTransfer`] when there is no such transfer, and
    /// the errors of [`add_transfer`](Self::add_transfer) about the transfer
    /// itself.
    pub fn replace_transfer(&mut self, transfer: Transfer) -> Result<Transfer, GroupError> {
        let id = transfer.id();
        let previous = self
            .transfers
            .get(&id)
            .ok_or(GroupError::UnknownTransfer(id))?;
        self.check_transfer(&transfer)?;
        let amount =
            resolve_transfer(&transfer, self.currency).map_err(|_| GroupError::Overflow)?;
        let mut delta = Delta::default();
        delta.add_transfer(pair(&previous.transfer), previous.amount, -1);
        delta.add_transfer(pair(&transfer), amount, 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        let replaced = self
            .transfers
            .insert(id, TransferRecord { amount, transfer });
        self.invalidate_settlements();
        replaced
            .map(|record| record.transfer)
            .ok_or(GroupError::UnknownTransfer(id))
    }

    /// Removes a transfer, returning it.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownTransfer`] when there is no such transfer.
    /// - [`GroupError::Overflow`] when the remaining totals would leave the
    ///   range of an amount.
    pub fn remove_transfer(&mut self, id: TransferId) -> Result<Transfer, GroupError> {
        let record = self
            .transfers
            .get(&id)
            .ok_or(GroupError::UnknownTransfer(id))?;
        let mut delta = Delta::default();
        delta.add_transfer(pair(&record.transfer), record.amount, -1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.invalidate_settlements();
        self.transfers
            .remove(&id)
            .map(|removed| removed.transfer)
            .ok_or(GroupError::UnknownTransfer(id))
    }

    // --- Description ---------------------------------------------------------

    /// The description, if any.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the description; blank text clears it.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = text::optional(description);
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

    // --- Checks --------------------------------------------------------------

    /// Checks that `cost` suits the group currency: a local cost in it, and a
    /// converted one in any other currency, even at a rate of 1.
    fn check_cost(&self, cost: Cost) -> Result<(), GroupError> {
        let currency = cost.amount().currency();
        match (cost.conversion(), currency == self.currency) {
            (None, false) => Err(GroupError::MissingExchangeRate { currency }),
            (None, true) | (Some(_), false) => Ok(()),
            (Some(_), true) => Err(GroupError::UnexpectedExchangeRate),
        }
    }

    /// Checks the references and the cost of `expense`; `previous` is the
    /// version it replaces, whose archived members it may keep.
    fn check_expense(
        &self,
        expense: &Expense,
        previous: Option<&Expense>,
    ) -> Result<(), GroupError> {
        let kept = previous.map(Expense::members).unwrap_or_default();
        for member in expense.members() {
            let record = self
                .members
                .get(&member)
                .ok_or(GroupError::UnknownMember(member))?;
            if record.archived && !kept.contains(&member) {
                return Err(GroupError::ArchivedMember(member));
            }
        }
        let category = expense.category();
        if !self.categories.contains_key(&category) {
            return Err(GroupError::UnknownCategory(category));
        }
        self.check_cost(expense.cost())
    }

    /// Checks the members and the cost of `transfer`.
    fn check_transfer(&self, transfer: &Transfer) -> Result<(), GroupError> {
        for member in [transfer.from(), transfer.to()] {
            if !self.members.contains_key(&member) {
                return Err(GroupError::UnknownMember(member));
            }
        }
        self.check_cost(transfer.cost())
    }
}

impl PartialEq for Group {
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
            && self.description == other.description
            && self.currency == other.currency
            && self.language == other.language
            && self.members == other.members
            && self.categories == other.categories
            && same_expenses
            && same_transfers
    }
}

impl Eq for Group {}

/// The sender and the recipient of `transfer`, as the ledger keys them.
const fn pair(transfer: &Transfer) -> MemberPair {
    MemberPair {
        from: transfer.from(),
        to: transfer.to(),
    }
}

#[cfg(test)]
mod tests;
