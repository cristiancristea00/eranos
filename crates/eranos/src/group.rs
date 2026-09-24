//! Groups: the members, categories, expenses, and transfers of a trip, a
//! household, or any shared spending, with their totals kept current.

mod builder;
mod language;
#[cfg(feature = "serde")]
mod serde;

use std::collections::BTreeMap;
use std::sync::OnceLock;

pub use builder::GroupBuilder;
pub use language::Language;

use crate::category::Category;
use crate::error::{Field, GroupError, SettlementError, ValueError};
use crate::expense::Expense;
use crate::id::{CategoryId, ExpenseId, GroupId, MemberId, TransferId};
use crate::ledger::{Delta, ExpenseShares, Ledger, Summary};
use crate::member::Member;
use crate::money::{Currency, ExchangeRate, Money};
use crate::resolution::{resolve_expense, resolve_transfer};
use crate::settlement::{self, Settlement, SettlementStrategy};
use crate::text;
use crate::transfer::Transfer;

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
/// use eranos::{Currency, Date, Expense, Group, Member, Money, SettlementStrategy, Split};
///
/// let mut group = Group::builder(Currency::EUR, "Lisbon").build()?;
/// let ana = group.add_member(Member::builder("Ana").build()?);
/// let bob = group.add_member(Member::builder("Bob").build()?);
/// let carol = group.add_member(Member::builder("Carol").build()?);
///
/// let day = Date::new(23, 9, 2026)?;
/// let eur = |text| Money::parse(text, Currency::EUR);
/// group.add_expense(
///     Expense::builder(eur("300.00")?, day, "Hotel").paid_by(ana).split(Split::equal([ana, bob, carol])).build()?,
/// )?;
/// group.add_expense(Expense::builder(eur("60.00")?, day, "Dinner").paid_by(bob).split(Split::equal([ana, bob, carol])).build()?)?;
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
    shares: ExpenseShares,
}

#[derive(Debug, Clone)]
struct TransferRecord {
    /// The amount in the group currency, in minor units.
    amount: i64,
    transfer: Transfer,
}

/// One lazily computed settlement per strategy, cleared whenever a balance
/// changes.
#[derive(Debug, Clone, Default)]
struct Settlements {
    direct: OnceLock<Result<Settlement, SettlementError>>,
    greedy: OnceLock<Result<Settlement, SettlementError>>,
    minimum_transfers: OnceLock<Result<Settlement, SettlementError>>,
}

impl Settlements {
    const fn slot(
        &self,
        strategy: SettlementStrategy,
    ) -> &OnceLock<Result<Settlement, SettlementError>> {
        match strategy {
            SettlementStrategy::MinimumTransfers => &self.minimum_transfers,
            SettlementStrategy::Greedy => &self.greedy,
            SettlementStrategy::Direct => &self.direct,
        }
    }

    fn clear(&mut self) {
        self.minimum_transfers.take();
        self.greedy.take();
        self.direct.take();
    }
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
            categories: BTreeMap::new(),
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

    /// Every category, in the order they were created.
    pub fn categories(&self) -> impl ExactSizeIterator<Item = (CategoryId, &Category)> + '_ {
        self.categories.iter().map(|(&id, category)| (id, category))
    }

    /// The category with this identifier.
    pub fn category(&self, id: CategoryId) -> Option<&Category> {
        self.categories.get(&id)
    }

    /// Adds a category under a newly generated identifier.
    ///
    /// # Errors
    ///
    /// [`GroupError::DuplicateCategoryName`] when another category has the
    /// same name, compared case-insensitively.
    pub fn add_category(&mut self, category: Category) -> Result<CategoryId, GroupError> {
        let id = unused_id(&self.categories, CategoryId::generate);
        self.insert_category(id, category)?;
        Ok(id)
    }

    /// Adds a category under an existing identifier.
    ///
    /// # Errors
    ///
    /// - [`GroupError::DuplicateCategory`] when the identifier is taken.
    /// - [`GroupError::DuplicateCategoryName`] when another category has the
    ///   same name, compared case-insensitively.
    pub fn insert_category(
        &mut self,
        id: CategoryId,
        category: Category,
    ) -> Result<(), GroupError> {
        if self.categories.contains_key(&id) {
            return Err(GroupError::DuplicateCategory(id));
        }
        self.check_category_name(&category, None)?;
        self.categories.insert(id, category);
        Ok(())
    }

    /// Replaces a category's details, returning the previous ones; its
    /// expenses keep it.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownCategory`] when there is no such category.
    /// - [`GroupError::DuplicateCategoryName`] when another category has the
    ///   new name.
    pub fn rename_category(
        &mut self,
        id: CategoryId,
        category: Category,
    ) -> Result<Category, GroupError> {
        if !self.categories.contains_key(&id) {
            return Err(GroupError::UnknownCategory(id));
        }
        self.check_category_name(&category, Some(id))?;
        self.categories
            .insert(id, category)
            .ok_or(GroupError::UnknownCategory(id))
    }

    /// Removes a category no expense uses, returning it.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownCategory`] when there is no such category.
    /// - [`GroupError::CategoryInUse`] listing the expenses that still use
    ///   it.
    pub fn remove_category(&mut self, id: CategoryId) -> Result<Category, GroupError> {
        if !self.categories.contains_key(&id) {
            return Err(GroupError::UnknownCategory(id));
        }
        let expenses: Vec<ExpenseId> = self
            .expenses
            .iter()
            .filter(|(_, record)| record.expense.category() == Some(id))
            .map(|(&id, _)| id)
            .collect();
        if !expenses.is_empty() {
            return Err(GroupError::CategoryInUse {
                category: id,
                expenses,
            });
        }
        self.categories
            .remove(&id)
            .ok_or(GroupError::UnknownCategory(id))
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
    /// than `currency`, the rate into `currency`; entries in the old group
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
    /// use eranos::{Currency, Date, ExchangeRate, Expense, Group, Member, Money, Split};
    ///
    /// let mut group = Group::builder(Currency::EUR, "Trip").build()?;
    /// let ana = group.add_member(Member::builder("Ana").build()?);
    /// let day = Date::new(23, 9, 2026)?;
    /// group.add_expense(
    ///     Expense::builder(Money::parse("20.00", Currency::EUR)?, day, "Museum")
    ///         .paid_by(ana)
    ///         .split(Split::equal([ana]))
    ///         .build()?,
    /// )?;
    ///
    /// group.change_currency(Currency::RON, &[(Currency::EUR, ExchangeRate::parse("4.97")?)])?;
    /// assert_eq!(group.summary().total_spent(), Money::parse("99.40", Currency::RON)?);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn change_currency(
        &mut self,
        currency: Currency,
        rates: &[(Currency, ExchangeRate)],
    ) -> Result<(), GroupError> {
        if currency == self.currency {
            return Ok(());
        }
        let mut table = BTreeMap::new();
        for &(source, rate) in rates {
            if table.insert(source, rate).is_some() {
                return Err(GroupError::DuplicateExchangeRate { currency: source });
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

        let mut delta = Delta::default();
        let mut expenses = BTreeMap::new();
        for (&id, record) in &self.expenses {
            let mut expense = record.expense.clone();
            expense.set_exchange_rate(rate_for(expense.amount().currency())?);
            let shares = resolve_expense(id, &expense, currency)?;
            delta.add_expense(&shares, expense.category(), 1);
            expenses.insert(id, ExpenseRecord { expense, shares });
        }
        let mut transfers = BTreeMap::new();
        for (&id, record) in &self.transfers {
            let mut transfer = record.transfer.clone();
            transfer.set_exchange_rate(rate_for(transfer.amount().currency())?);
            let amount = resolve_transfer(&transfer, currency)?;
            delta.add_transfer(transfer.pair(), amount, 1);
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

    /// Every expense, in the order they were created.
    pub fn expenses(&self) -> impl ExactSizeIterator<Item = (ExpenseId, &Expense)> + '_ {
        self.expenses
            .iter()
            .map(|(&id, record)| (id, &record.expense))
    }

    /// The expense with this identifier.
    pub fn expense(&self, id: ExpenseId) -> Option<&Expense> {
        self.expenses.get(&id).map(|record| &record.expense)
    }

    /// How an expense divides in the group currency.
    pub fn expense_shares(&self, id: ExpenseId) -> Option<&ExpenseShares> {
        self.expenses.get(&id).map(|record| &record.shares)
    }

    /// Adds an expense under a newly generated identifier.
    ///
    /// # Errors
    ///
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
        let id = unused_id(&self.expenses, ExpenseId::generate);
        self.insert_expense(id, expense)?;
        Ok(id)
    }

    /// Adds an expense under an existing identifier, such as one synced from
    /// another device.
    ///
    /// # Errors
    ///
    /// [`GroupError::DuplicateExpense`] when the identifier is taken, and
    /// the errors of [`add_expense`](Self::add_expense).
    pub fn insert_expense(&mut self, id: ExpenseId, expense: Expense) -> Result<(), GroupError> {
        if self.expenses.contains_key(&id) {
            return Err(GroupError::DuplicateExpense(id));
        }
        self.check_expense(&expense, None)?;
        let shares = resolve_expense(id, &expense, self.currency)?;
        let mut delta = Delta::default();
        delta.add_expense(&shares, expense.category(), 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.expenses.insert(id, ExpenseRecord { expense, shares });
        self.invalidate_settlements();
        Ok(())
    }

    /// Replaces an expense, returning the previous version.
    ///
    /// The replacement may keep members who were archived after the
    /// original was added, but may not add archived members.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownExpense`] when there is no such expense, and the
    /// errors of [`add_expense`](Self::add_expense).
    pub fn replace_expense(
        &mut self,
        id: ExpenseId,
        expense: Expense,
    ) -> Result<Expense, GroupError> {
        let previous = self
            .expenses
            .get(&id)
            .ok_or(GroupError::UnknownExpense(id))?;
        self.check_expense(&expense, Some(&previous.expense))?;
        let shares = resolve_expense(id, &expense, self.currency)?;
        let mut delta = Delta::default();
        delta.add_expense(&previous.shares, previous.expense.category(), -1);
        delta.add_expense(&shares, expense.category(), 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        let replaced = self.expenses.insert(id, ExpenseRecord { expense, shares });
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
        delta.add_expense(&record.shares, record.expense.category(), -1);
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

    /// Every member, archived ones included, in the order they were created.
    pub fn members(&self) -> impl ExactSizeIterator<Item = (MemberId, &Member)> + '_ {
        self.members
            .iter()
            .map(|(&id, record)| (id, &record.member))
    }

    /// The member with this identifier.
    pub fn member(&self, id: MemberId) -> Option<&Member> {
        self.members.get(&id).map(|record| &record.member)
    }

    /// Whether the member is archived; `false` for an unknown member.
    pub fn is_archived(&self, id: MemberId) -> bool {
        self.members.get(&id).is_some_and(|record| record.archived)
    }

    /// Adds a member under a newly generated identifier.
    pub fn add_member(&mut self, member: Member) -> MemberId {
        let id = unused_id(&self.members, MemberId::generate);
        self.members.insert(
            id,
            MemberRecord {
                archived: false,
                member,
            },
        );
        id
    }

    /// Adds a member under an existing identifier, such as one synced from
    /// another device.
    ///
    /// # Errors
    ///
    /// [`GroupError::DuplicateMember`] when the identifier is taken.
    pub fn insert_member(&mut self, id: MemberId, member: Member) -> Result<(), GroupError> {
        if self.members.contains_key(&id) {
            return Err(GroupError::DuplicateMember(id));
        }
        self.members.insert(
            id,
            MemberRecord {
                archived: false,
                member,
            },
        );
        Ok(())
    }

    /// Replaces a member's details, returning the previous ones.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownMember`] when there is no such member.
    pub fn update_member(&mut self, id: MemberId, member: Member) -> Result<Member, GroupError> {
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
    /// [`ValueError::Empty`] when the name is blank.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<(), ValueError> {
        self.name = text::required(name, Field::GroupName)?;
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
        self.settlements
            .slot(strategy)
            .get_or_init(|| settlement::settle(strategy, &self.ledger))
            .as_ref()
            .map_err(|&error| error)
    }

    fn invalidate_settlements(&mut self) {
        self.settlements.clear();
    }

    // --- Transfers -----------------------------------------------------------

    /// Every transfer, in the order they were created.
    pub fn transfers(&self) -> impl ExactSizeIterator<Item = (TransferId, &Transfer)> + '_ {
        self.transfers
            .iter()
            .map(|(&id, record)| (id, &record.transfer))
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

    /// Adds a transfer under a newly generated identifier.
    ///
    /// Archived members may send and receive transfers, so they can settle
    /// up.
    ///
    /// # Errors
    ///
    /// - [`GroupError::UnknownMember`] when the sender or the recipient is
    ///   not a member.
    /// - [`GroupError::MissingExchangeRate`] or
    ///   [`GroupError::UnexpectedExchangeRate`] as for expenses.
    /// - [`GroupError::Overflow`] when a total would leave the range of an
    ///   amount.
    pub fn add_transfer(&mut self, transfer: Transfer) -> Result<TransferId, GroupError> {
        let id = unused_id(&self.transfers, TransferId::generate);
        self.insert_transfer(id, transfer)?;
        Ok(id)
    }

    /// Adds a transfer under an existing identifier.
    ///
    /// # Errors
    ///
    /// [`GroupError::DuplicateTransfer`] when the identifier is taken, and
    /// the errors of [`add_transfer`](Self::add_transfer).
    pub fn insert_transfer(
        &mut self,
        id: TransferId,
        transfer: Transfer,
    ) -> Result<(), GroupError> {
        if self.transfers.contains_key(&id) {
            return Err(GroupError::DuplicateTransfer(id));
        }
        self.check_transfer(&transfer)?;
        let amount = resolve_transfer(&transfer, self.currency)?;
        let mut delta = Delta::default();
        delta.add_transfer(transfer.pair(), amount, 1);
        self.ledger
            .apply(&delta)
            .map_err(|_| GroupError::Overflow)?;
        self.transfers
            .insert(id, TransferRecord { amount, transfer });
        self.invalidate_settlements();
        Ok(())
    }

    /// Replaces a transfer, returning the previous version.
    ///
    /// # Errors
    ///
    /// [`GroupError::UnknownTransfer`] when there is no such transfer, and
    /// the errors of [`add_transfer`](Self::add_transfer).
    pub fn replace_transfer(
        &mut self,
        id: TransferId,
        transfer: Transfer,
    ) -> Result<Transfer, GroupError> {
        let previous = self
            .transfers
            .get(&id)
            .ok_or(GroupError::UnknownTransfer(id))?;
        self.check_transfer(&transfer)?;
        let amount = resolve_transfer(&transfer, self.currency)?;
        let mut delta = Delta::default();
        delta.add_transfer(previous.transfer.pair(), previous.amount, -1);
        delta.add_transfer(transfer.pair(), amount, 1);
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
        delta.add_transfer(record.transfer.pair(), record.amount, -1);
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

    /// Checks the references of `expense`; `previous` is the version it
    /// replaces, whose archived members it may keep.
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
        match expense.category() {
            Some(category) if !self.categories.contains_key(&category) => {
                Err(GroupError::UnknownCategory(category))
            }
            _ => Ok(()),
        }
    }

    fn check_transfer(&self, transfer: &Transfer) -> Result<(), GroupError> {
        for member in [transfer.from(), transfer.to()] {
            if !self.members.contains_key(&member) {
                return Err(GroupError::UnknownMember(member));
            }
        }
        Ok(())
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

/// An identifier from `generate` that `taken` does not hold yet.
///
/// A UUID v7 collision is all but impossible, but a silent overwrite would
/// lose an entry, so a taken identifier is drawn again.
fn unused_id<Id: Ord, Value>(taken: &BTreeMap<Id, Value>, mut generate: impl FnMut() -> Id) -> Id {
    loop {
        let id = generate();
        if !taken.contains_key(&id) {
            return id;
        }
    }
}

#[cfg(test)]
mod tests;
