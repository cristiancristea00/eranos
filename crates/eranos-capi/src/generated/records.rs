// Written by `cargo xtask generate` from the binding core's records and calls: edit those,
// or the generator in eranos-glue, and run the command again, never this file.

/// The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it
/// in the namespace `eranos::detail`.
#[expect(
    clippy::unnecessary_box_returns,
    reason = "the functions Diplomat writes hand out an opaque type only in a box"
)]
#[diplomat::bridge]
#[diplomat::attr(cpp, namespace = "eranos::detail")]
pub mod ffi {
    use crate::error::ffi::EranosError;
    use crate::values::ffi::{EranosCalendarDate, EranosMoney, EranosTextList, EranosTimeOfDay};
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};

    /// Where a member stands with the group.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosBalance(pub(crate) eranos_bindings::Balance);

    impl EranosBalance {
        /// The member owes the others `amount`.
        #[must_use]
        pub fn create_owes(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::Balance::Owes {
                amount: amount.0.clone(),
            }))
        }

        /// The member owes nothing and is owed nothing.
        #[must_use]
        pub fn create_settled() -> Box<Self> {
            Box::new(Self(eranos_bindings::Balance::Settled))
        }

        /// The others owe the member `amount`.
        #[must_use]
        pub fn create_gets_back(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::Balance::GetsBack {
                amount: amount.0.clone(),
            }))
        }

        /// Which kind of balance it is.
        #[must_use]
        pub const fn kind(&self) -> EranosBalanceKind {
            match &self.0 {
                eranos_bindings::Balance::Owes { .. } => EranosBalanceKind::Owes,
                eranos_bindings::Balance::Settled => EranosBalanceKind::Settled,
                eranos_bindings::Balance::GetsBack { .. } => EranosBalanceKind::GetsBack,
            }
        }

        /// In `Owes`, what the member owes; in `GetsBack`, what the member gets back. Present in
        /// the kinds `Owes` and `GetsBack` alone.
        #[must_use]
        pub fn amount(&self) -> Option<Box<EranosMoney>> {
            match &self.0 {
                eranos_bindings::Balance::Owes { amount, .. }
                | eranos_bindings::Balance::GetsBack { amount, .. } => {
                    Some(Box::new(EranosMoney(amount.clone())))
                }
                eranos_bindings::Balance::Settled => None,
            }
        }

        /// Writes the balance as the library does: "gets back 12.00 EUR", "owes 12.00 EUR", or
        /// "settled".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a currency the library does not list.
        pub fn format(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::balance_format(&self.0).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }

    /// The kinds of balance.
    #[derive(Debug)]
    pub enum EranosBalanceKind {
        /// The member owes the others `amount`.
        Owes,
        /// The member owes nothing and is owed nothing.
        Settled,
        /// The others owe the member `amount`.
        GetsBack,
    }

    /// A label the group gives expenses.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosCategory(pub(crate) eranos_bindings::Category);

    impl EranosCategory {
        /// A category of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(name: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Category {
                id: None,
                name: crate::text(name)?,
            })))
        }

        /// The identifier: absent for a category to add, present for one the group holds or one to
        /// rename. Writes it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a category to add, present for one the group holds or
        /// one to rename.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }

        /// Clears the identifier: absent for a category to add, present for one the group holds or
        /// one to rename.
        pub fn clear_id(&mut self) {
            self.0.id = None;
        }

        /// The name, which no other category of the group shares.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets the name, which no other category of the group shares.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }
    }

    /// A list of category records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryList(pub(crate) Vec<eranos_bindings::Category>);

    impl EranosCategoryList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosCategory>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosCategory(element.clone())))
        }
    }

    /// What the expenses of a category add up to, in the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryTotal(pub(crate) eranos_bindings::CategoryTotal);

    impl EranosCategoryTotal {
        /// The category, as its identifier.
        pub fn category(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.category);
        }

        /// The total.
        #[must_use]
        pub fn total(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total.clone()))
        }
    }

    /// A list of category total records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryTotalList(pub(crate) Vec<eranos_bindings::CategoryTotal>);

    impl EranosCategoryTotalList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosCategoryTotal>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosCategoryTotal(element.clone())))
        }
    }

    /// What an entry cost: an amount in the group currency, or one in another currency with its
    /// exchange rate into the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCost(pub(crate) eranos_bindings::Cost);

    impl EranosCost {
        /// An amount in the group currency.
        #[must_use]
        pub fn create_local(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::Cost::Local {
                amount: amount.0.clone(),
            }))
        }

        /// An amount in another currency than the group's, converted at `rate`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create_converted(
            amount: &EranosMoney,
            rate: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Cost::Converted {
                amount: amount.0.clone(),
                rate: crate::text(rate)?,
            })))
        }

        /// Which kind of cost it is.
        #[must_use]
        pub const fn kind(&self) -> EranosCostKind {
            match &self.0 {
                eranos_bindings::Cost::Local { .. } => EranosCostKind::Local,
                eranos_bindings::Cost::Converted { .. } => EranosCostKind::Converted,
            }
        }

        /// In `Local`, the amount; in `Converted`, the amount, in its own currency.
        #[must_use]
        pub fn amount(&self) -> Box<EranosMoney> {
            match &self.0 {
                eranos_bindings::Cost::Local { amount, .. }
                | eranos_bindings::Cost::Converted { amount, .. } => {
                    Box::new(EranosMoney(amount.clone()))
                }
            }
        }

        /// The rate into the group currency, as a decimal: "4.97". Present in the kind `Converted`
        /// alone. Writes it, if present; absent otherwise.
        pub fn rate(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::Cost::Converted { rate, .. } => {
                    crate::write_optional(write, Some(rate))
                }
                eranos_bindings::Cost::Local { .. } => None,
            }
        }
    }

    /// The kinds of cost.
    #[derive(Debug)]
    pub enum EranosCostKind {
        /// An amount in the group currency.
        Local,
        /// An amount in another currency than the group's, converted at `rate`.
        Converted,
    }

    /// What ISO 4217 says of a currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCurrencyDetails(pub(crate) eranos_bindings::CurrencyDetails);

    impl EranosCurrencyDetails {
        /// The three-letter code: "EUR".
        pub fn code(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.code);
        }

        /// The English name: "Euro".
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// The three-digit numeric code: 978 for the euro.
        #[must_use]
        pub const fn numeric_code(&self) -> i32 {
            self.0.numeric_code
        }

        /// How many decimal places its minor unit has: 2 for the euro.
        #[must_use]
        pub const fn decimal_places(&self) -> i32 {
            self.0.decimal_places
        }

        /// The currency of the ISO 4217 code `code`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a code the library does not list, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn of(code: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::currency_details(&crate::text(code)?)
                .map(|details| Box::new(Self(details)))
                .map_err(crate::failed)
        }
    }

    /// A list of currency details records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCurrencyDetailsList(pub(crate) Vec<eranos_bindings::CurrencyDetails>);

    impl EranosCurrencyDetailsList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosCurrencyDetails(element.clone())))
        }

        /// Every currency of the ISO 4217 list that the library knows, by code.
        #[must_use]
        pub fn all() -> Box<Self> {
            Box::new(Self(eranos_bindings::currencies()))
        }
    }

    /// The rate of a currency into another: one of the rates a currency change takes.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosCurrencyRate(pub(crate) eranos_bindings::CurrencyRate);

    impl EranosCurrencyRate {
        /// A currency rate of its required fields; every other field starts absent, empty, or at
        /// its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            currency: &DiplomatStr,
            rate: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::CurrencyRate {
                currency: crate::text(currency)?,
                rate: crate::text(rate)?,
            })))
        }

        /// The ISO 4217 code of the currency the rate converts from.
        pub fn currency(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.currency);
        }

        /// Sets the ISO 4217 code of the currency the rate converts from.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_currency(&mut self, currency: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.currency = crate::text(currency)?;
            Ok(())
        }

        /// The rate, as a decimal: "4.97".
        pub fn rate(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.rate);
        }

        /// Sets the rate, as a decimal: "4.97".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_rate(&mut self, rate: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.rate = crate::text(rate)?;
            Ok(())
        }
    }

    /// A list of currency rate records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosCurrencyRateList(pub(crate) Vec<eranos_bindings::CurrencyRate>);

    impl EranosCurrencyRateList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosCurrencyRate) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosCurrencyRate>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosCurrencyRate(element.clone())))
        }
    }

    /// Something bought for the group.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosExpense(pub(crate) eranos_bindings::Expense);

    impl EranosExpense {
        /// An expense of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            name: &DiplomatStr,
            cost: &EranosCost,
            payers: &EranosSplit,
            split: &EranosExpenseSplit,
            date: EranosCalendarDate,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Expense {
                id: None,
                name: crate::text(name)?,
                cost: cost.0.clone(),
                payers: payers.0.clone(),
                split: split.0.clone(),
                date: date.into(),
                tip: None,
                time: None,
                category: None,
                description: None,
            })))
        }

        /// The identifier: absent for an expense to add, present for one the group holds or one to
        /// replace. Writes it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for an expense to add, present for one the group holds or
        /// one to replace.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }

        /// Clears the identifier: absent for an expense to add, present for one the group holds or
        /// one to replace.
        pub fn clear_id(&mut self) {
            self.0.id = None;
        }

        /// What was bought.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets what was bought.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }

        /// What it cost.
        #[must_use]
        pub fn cost(&self) -> Box<EranosCost> {
            Box::new(EranosCost(self.0.cost.clone()))
        }

        /// Sets what it cost.
        pub fn set_cost(&mut self, cost: &EranosCost) {
            self.0.cost.clone_from(&cost.0);
        }

        /// Who paid the total, amount plus tip, and how much each.
        #[must_use]
        pub fn payers(&self) -> Box<EranosSplit> {
            Box::new(EranosSplit(self.0.payers.clone()))
        }

        /// Sets who paid the total, amount plus tip, and how much each.
        pub fn set_payers(&mut self, payers: &EranosSplit) {
            self.0.payers.clone_from(&payers.0);
        }

        /// How the amount is shared.
        #[must_use]
        pub fn split(&self) -> Box<EranosExpenseSplit> {
            Box::new(EranosExpenseSplit(self.0.split.clone()))
        }

        /// Sets how the amount is shared.
        pub fn set_split(&mut self, split: &EranosExpenseSplit) {
            self.0.split.clone_from(&split.0);
        }

        /// The day it was bought.
        #[must_use]
        pub fn date(&self) -> EranosCalendarDate {
            self.0.date.into()
        }

        /// Sets the day it was bought.
        pub fn set_date(&mut self, date: EranosCalendarDate) {
            self.0.date = date.into();
        }

        /// The tip on top of the amount.
        #[must_use]
        pub fn tip(&self) -> Option<Box<EranosTip>> {
            self.0.tip.clone().map(|value| Box::new(EranosTip(value)))
        }

        /// Sets the tip on top of the amount.
        pub fn set_tip(&mut self, tip: &EranosTip) {
            self.0.tip = Some(tip.0.clone());
        }

        /// Clears the tip on top of the amount.
        pub fn clear_tip(&mut self) {
            self.0.tip = None;
        }

        /// The time of day it was bought.
        #[must_use]
        pub fn time(&self) -> Option<EranosTimeOfDay> {
            self.0.time.map(Into::into)
        }

        /// Sets the time of day it was bought.
        pub fn set_time(&mut self, time: EranosTimeOfDay) {
            self.0.time = Some(time.into());
        }

        /// Clears the time of day it was bought.
        pub const fn clear_time(&mut self) {
            self.0.time = None;
        }

        /// The category, as its identifier; absent for the built-in one. Writes it, if present;
        /// absent otherwise.
        pub fn category(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.category.as_deref())
        }

        /// Sets the category, as its identifier; absent for the built-in one.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_category(&mut self, category: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.category = Some(crate::text(category)?);
            Ok(())
        }

        /// Clears the category, as its identifier; absent for the built-in one.
        pub fn clear_category(&mut self) {
            self.0.category = None;
        }

        /// A longer description. Writes it, if present; absent otherwise.
        pub fn description(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.description.as_deref())
        }

        /// Sets a longer description.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_description(
            &mut self,
            description: &DiplomatStr,
        ) -> Result<(), Box<EranosError>> {
            self.0.description = Some(crate::text(description)?);
            Ok(())
        }

        /// Clears a longer description.
        pub fn clear_description(&mut self) {
            self.0.description = None;
        }

        /// The tip in the expense's currency, zero without one.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the kind
        /// `Money` for an amount out of range.
        pub fn tip_amount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
            eranos_bindings::expense_tip_amount(&self.0)
                .map(|money| Box::new(EranosMoney(money)))
                .map_err(crate::failed)
        }

        /// The amount with the tip, what the payers paid.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the kind
        /// `Money` for an amount out of range.
        pub fn total(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
            eranos_bindings::expense_total(&self.0)
                .map(|money| Box::new(EranosMoney(money)))
                .map_err(crate::failed)
        }

        /// The identifiers of every member the expense names, payers and participants, each once
        /// and in the order of their identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Id` for an identifier that is not one.
        pub fn participants(&self) -> Result<Box<EranosTextList>, Box<EranosError>> {
            eranos_bindings::expense_members(&self.0)
                .map(|members| Box::new(EranosTextList(members)))
                .map_err(crate::failed)
        }
    }

    /// A list of expense records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseList(pub(crate) Vec<eranos_bindings::Expense>);

    impl EranosExpenseList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosExpense>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosExpense(element.clone())))
        }
    }

    /// How one expense divides, in the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseShares(pub(crate) eranos_bindings::ExpenseShares);

    impl EranosExpenseShares {
        /// What every member paid and consumed.
        #[must_use]
        pub fn members(&self) -> Box<EranosMemberShareList> {
            Box::new(EranosMemberShareList(self.0.members.clone()))
        }

        /// The total: the amount and the tip.
        #[must_use]
        pub fn total(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total.clone()))
        }

        /// The tip.
        #[must_use]
        pub fn tip(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.tip.clone()))
        }
    }

    /// How an expense's cost is shared: as a whole, or item by item.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseSplit(pub(crate) eranos_bindings::ExpenseSplit);

    impl EranosExpenseSplit {
        /// The whole amount is divided by one split.
        #[must_use]
        pub fn create_whole(split: &EranosSplit) -> Box<Self> {
            Box::new(Self(eranos_bindings::ExpenseSplit::Whole {
                split: split.0.clone(),
            }))
        }

        /// Every item is divided by its own split.
        #[must_use]
        pub fn create_itemised(items: &EranosItemList) -> Box<Self> {
            Box::new(Self(eranos_bindings::ExpenseSplit::Itemised {
                items: items.0.clone(),
            }))
        }

        /// Which kind of expense split it is.
        #[must_use]
        pub const fn kind(&self) -> EranosExpenseSplitKind {
            match &self.0 {
                eranos_bindings::ExpenseSplit::Whole { .. } => EranosExpenseSplitKind::Whole,
                eranos_bindings::ExpenseSplit::Itemised { .. } => EranosExpenseSplitKind::Itemised,
            }
        }

        /// The lines of the bill. Present in the kind `Itemised` alone.
        #[must_use]
        pub fn items(&self) -> Option<Box<EranosItemList>> {
            match &self.0 {
                eranos_bindings::ExpenseSplit::Itemised { items, .. } => {
                    Some(Box::new(EranosItemList(items.clone())))
                }
                eranos_bindings::ExpenseSplit::Whole { .. } => None,
            }
        }

        /// How the amount divides. Present in the kind `Whole` alone.
        #[must_use]
        pub fn split(&self) -> Option<Box<EranosSplit>> {
            match &self.0 {
                eranos_bindings::ExpenseSplit::Whole { split, .. } => {
                    Some(Box::new(EranosSplit(split.clone())))
                }
                eranos_bindings::ExpenseSplit::Itemised { .. } => None,
            }
        }
    }

    /// The kinds of expense split.
    #[derive(Debug)]
    pub enum EranosExpenseSplitKind {
        /// The whole amount is divided by one split.
        Whole,
        /// Every item is divided by its own split.
        Itemised,
    }

    /// A line of an itemised bill.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosItem(pub(crate) eranos_bindings::Item);

    impl EranosItem {
        /// An item of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            name: &DiplomatStr,
            unit_price: &EranosMoney,
            split: &EranosSplit,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Item {
                name: crate::text(name)?,
                unit_price: unit_price.0.clone(),
                split: split.0.clone(),
                quantity: 1,
            })))
        }

        /// What was bought.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets what was bought.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }

        /// The price of one unit, in the expense's currency.
        #[must_use]
        pub fn unit_price(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.unit_price.clone()))
        }

        /// Sets the price of one unit, in the expense's currency.
        pub fn set_unit_price(&mut self, unit_price: &EranosMoney) {
            self.0.unit_price.clone_from(&unit_price.0);
        }

        /// How the line total divides.
        #[must_use]
        pub fn split(&self) -> Box<EranosSplit> {
            Box::new(EranosSplit(self.0.split.clone()))
        }

        /// Sets how the line total divides.
        pub fn set_split(&mut self, split: &EranosSplit) {
            self.0.split.clone_from(&split.0);
        }

        /// How many units, from 1.
        #[must_use]
        pub const fn quantity(&self) -> i32 {
            self.0.quantity
        }

        /// Sets how many units, from 1.
        pub const fn set_quantity(&mut self, quantity: i32) {
            self.0.quantity = quantity;
        }
    }

    /// A list of item records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosItemList(pub(crate) Vec<eranos_bindings::Item>);

    impl EranosItemList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosItem) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosItem>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosItem(element.clone())))
        }
    }

    /// A person in the group.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMember(pub(crate) eranos_bindings::Member);

    impl EranosMember {
        /// A member of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(first_name: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Member {
                id: None,
                first_name: crate::text(first_name)?,
                last_name: None,
                email: None,
                phone: None,
                accounts: Vec::new(),
            })))
        }

        /// The identifier: absent for a member to add, present for one the group holds or one to
        /// replace. Writes it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a member to add, present for one the group holds or one
        /// to replace.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }

        /// Clears the identifier: absent for a member to add, present for one the group holds or
        /// one to replace.
        pub fn clear_id(&mut self) {
            self.0.id = None;
        }

        /// The first name.
        pub fn first_name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.first_name);
        }

        /// Sets the first name.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_first_name(&mut self, first_name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.first_name = crate::text(first_name)?;
            Ok(())
        }

        /// The last name. Writes it, if present; absent otherwise.
        pub fn last_name(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.last_name.as_deref())
        }

        /// Sets the last name.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_last_name(&mut self, last_name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.last_name = Some(crate::text(last_name)?);
            Ok(())
        }

        /// Clears the last name.
        pub fn clear_last_name(&mut self) {
            self.0.last_name = None;
        }

        /// The email address, without surrounding whitespace. Writes it, if present; absent
        /// otherwise.
        pub fn email(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.email.as_deref())
        }

        /// Sets the email address, without surrounding whitespace.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_email(&mut self, email: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.email = Some(crate::text(email)?);
            Ok(())
        }

        /// Clears the email address, without surrounding whitespace.
        pub fn clear_email(&mut self) {
            self.0.email = None;
        }

        /// The phone number, starting with `+` and the country code; a member the group holds has
        /// it in E.164 form: "+40712345678". Writes it, if present; absent otherwise.
        pub fn phone(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.phone.as_deref())
        }

        /// Sets the phone number, starting with `+` and the country code; a member the group holds
        /// has it in E.164 form: "+40712345678".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_phone(&mut self, phone: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.phone = Some(crate::text(phone)?);
            Ok(())
        }

        /// Clears the phone number, starting with `+` and the country code; a member the group
        /// holds has it in E.164 form: "+40712345678".
        pub fn clear_phone(&mut self) {
            self.0.phone = None;
        }

        /// Where the member can be paid, in the order they were given.
        #[must_use]
        pub fn accounts(&self) -> Box<EranosPaymentAccountList> {
            Box::new(EranosPaymentAccountList(self.0.accounts.clone()))
        }

        /// Sets where the member can be paid, in the order they were given.
        pub fn set_accounts(&mut self, accounts: &EranosPaymentAccountList) {
            self.0.accounts.clone_from(&accounts.0);
        }
    }

    /// An exact amount one member pays.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberAmount(pub(crate) eranos_bindings::MemberAmount);

    impl EranosMemberAmount {
        /// A member amount of its required fields; every other field starts absent, empty, or at
        /// its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            member: &DiplomatStr,
            amount: &EranosMoney,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberAmount {
                member: crate::text(member)?,
                amount: amount.0.clone(),
            })))
        }

        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member, as their identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }

        /// The amount.
        #[must_use]
        pub fn amount(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.amount.clone()))
        }

        /// Sets the amount.
        pub fn set_amount(&mut self, amount: &EranosMoney) {
            self.0.amount.clone_from(&amount.0);
        }
    }

    /// A list of member amount records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberAmountList(pub(crate) Vec<eranos_bindings::MemberAmount>);

    impl EranosMemberAmountList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosMemberAmount) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberAmount>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberAmount(element.clone())))
        }
    }

    /// What one member's shares add up to in each category.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberCategoryTotals(pub(crate) eranos_bindings::MemberCategoryTotals);

    impl EranosMemberCategoryTotals {
        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// The member's total in every category they have a share in.
        #[must_use]
        pub fn totals(&self) -> Box<EranosCategoryTotalList> {
            Box::new(EranosCategoryTotalList(self.0.totals.clone()))
        }
    }

    /// A list of member category totals records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberCategoryTotalsList(
        pub(crate) Vec<eranos_bindings::MemberCategoryTotals>,
    );

    impl EranosMemberCategoryTotalsList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberCategoryTotals>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberCategoryTotals(element.clone())))
        }
    }

    /// A list of member records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberList(pub(crate) Vec<eranos_bindings::Member>);

    impl EranosMemberList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMember>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMember(element.clone())))
        }
    }

    /// The parts one member pays in proportion to.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberParts(pub(crate) eranos_bindings::MemberParts);

    impl EranosMemberParts {
        /// A member parts of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            member: &DiplomatStr,
            hundredths: i32,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberParts {
                member: crate::text(member)?,
                hundredths,
            })))
        }

        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member, as their identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }

        /// The parts, in hundredths: 50 for half a part.
        #[must_use]
        pub const fn hundredths(&self) -> i32 {
            self.0.hundredths
        }

        /// Sets the parts, in hundredths: 50 for half a part.
        pub const fn set_hundredths(&mut self, hundredths: i32) {
            self.0.hundredths = hundredths;
        }
    }

    /// A list of member parts records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberPartsList(pub(crate) Vec<eranos_bindings::MemberParts>);

    impl EranosMemberPartsList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosMemberParts) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberParts>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberParts(element.clone())))
        }
    }

    /// The percentage one member pays.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberPercentage(pub(crate) eranos_bindings::MemberPercentage);

    impl EranosMemberPercentage {
        /// A member percentage of its required fields; every other field starts absent, empty, or
        /// at its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            member: &DiplomatStr,
            basis_points: i32,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberPercentage {
                member: crate::text(member)?,
                basis_points,
            })))
        }

        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member, as their identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }

        /// The percentage, in basis points: 2 500 for 25 %.
        #[must_use]
        pub const fn basis_points(&self) -> i32 {
            self.0.basis_points
        }

        /// Sets the percentage, in basis points: 2 500 for 25 %.
        pub const fn set_basis_points(&mut self, basis_points: i32) {
            self.0.basis_points = basis_points;
        }
    }

    /// A list of member percentage records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberPercentageList(pub(crate) Vec<eranos_bindings::MemberPercentage>);

    impl EranosMemberPercentageList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosMemberPercentage) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberPercentage>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberPercentage(element.clone())))
        }
    }

    /// What one member paid of an expense and what their share of it is.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberShare(pub(crate) eranos_bindings::MemberShare);

    impl EranosMemberShare {
        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// What the member paid, tip included.
        #[must_use]
        pub fn paid(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.paid.clone()))
        }

        /// The member's share, tip included.
        #[must_use]
        pub fn share(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.share.clone()))
        }
    }

    /// A list of member share records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberShareList(pub(crate) Vec<eranos_bindings::MemberShare>);

    impl EranosMemberShareList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberShare>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberShare(element.clone())))
        }
    }

    /// A member's totals, in the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberSummary(pub(crate) eranos_bindings::MemberSummary);

    impl EranosMemberSummary {
        /// The member, as their identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// What the member paid for expenses.
        #[must_use]
        pub fn paid(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.paid.clone()))
        }

        /// The member's share of the expenses: what they consumed.
        #[must_use]
        pub fn share(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.share.clone()))
        }

        /// What the member sent in transfers.
        #[must_use]
        pub fn sent(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.sent.clone()))
        }

        /// What the member received in transfers.
        #[must_use]
        pub fn received(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.received.clone()))
        }

        /// What the member is owed, less what they owe: positive when they get money back.
        #[must_use]
        pub fn net(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.net.clone()))
        }

        /// Where the member stands.
        #[must_use]
        pub fn balance(&self) -> Box<EranosBalance> {
            Box::new(EranosBalance(self.0.balance.clone()))
        }
    }

    /// A list of member summary records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberSummaryList(pub(crate) Vec<eranos_bindings::MemberSummary>);

    impl EranosMemberSummaryList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosMemberSummary>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosMemberSummary(element.clone())))
        }
    }

    /// What a new group starts with.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosNewExpenseGroup(pub(crate) eranos_bindings::NewExpenseGroup);

    impl EranosNewExpenseGroup {
        /// A new expense group of its required fields; every other field starts absent, empty, or
        /// at its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            name: &DiplomatStr,
            currency: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::NewExpenseGroup {
                name: crate::text(name)?,
                currency: crate::text(currency)?,
                description: None,
                language: None,
            })))
        }

        /// The group's name.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets the group's name.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }

        /// The ISO 4217 code of the group currency, which every total is in.
        pub fn currency(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.currency);
        }

        /// Sets the ISO 4217 code of the group currency, which every total is in.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_currency(&mut self, currency: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.currency = crate::text(currency)?;
            Ok(())
        }

        /// A longer description. Writes it, if present; absent otherwise.
        pub fn description(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.description.as_deref())
        }

        /// Sets a longer description.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_description(
            &mut self,
            description: &DiplomatStr,
        ) -> Result<(), Box<EranosError>> {
            self.0.description = Some(crate::text(description)?);
            Ok(())
        }

        /// Clears a longer description.
        pub fn clear_description(&mut self) {
            self.0.description = None;
        }

        /// The language the apps show the group in, as a BCP 47 tag. Writes it, if present; absent
        /// otherwise.
        pub fn language(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.language.as_deref())
        }

        /// Sets the language the apps show the group in, as a BCP 47 tag.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_language(&mut self, language: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.language = Some(crate::text(language)?);
            Ok(())
        }

        /// Clears the language the apps show the group in, as a BCP 47 tag.
        pub fn clear_language(&mut self) {
            self.0.language = None;
        }
    }

    /// One payment of a settlement: `from` pays `amount` to `to`.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosPayment(pub(crate) eranos_bindings::Payment);

    impl EranosPayment {
        /// The member who pays, as their identifier.
        pub fn from(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.from);
        }

        /// The member who is paid, as their identifier.
        pub fn to(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.to);
        }

        /// The amount, in the group currency.
        #[must_use]
        pub fn amount(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.amount.clone()))
        }
    }

    /// A place to send a member money: a bank account or a payment-app username, with a label.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosPaymentAccount(pub(crate) eranos_bindings::PaymentAccount);

    impl EranosPaymentAccount {
        /// A payment account of its required fields; every other field starts absent, empty, or at
        /// its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            name: &DiplomatStr,
            details: &EranosPaymentAccountDetails,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::PaymentAccount {
                name: crate::text(name)?,
                details: details.0.clone(),
            })))
        }

        /// The label: "Main account".
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets the label: "Main account".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }

        /// The IBAN or the username.
        #[must_use]
        pub fn details(&self) -> Box<EranosPaymentAccountDetails> {
            Box::new(EranosPaymentAccountDetails(self.0.details.clone()))
        }

        /// Sets the IBAN or the username.
        pub fn set_details(&mut self, details: &EranosPaymentAccountDetails) {
            self.0.details.clone_from(&details.0);
        }
    }

    /// Where a member can be paid, without its label.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosPaymentAccountDetails(pub(crate) eranos_bindings::PaymentAccountDetails);

    impl EranosPaymentAccountDetails {
        /// A bank account.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create_iban(iban: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(
                eranos_bindings::PaymentAccountDetails::Iban {
                    iban: crate::text(iban)?,
                },
            )))
        }

        /// A username in a payment app.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create_username(username: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(
                eranos_bindings::PaymentAccountDetails::Username {
                    username: crate::text(username)?,
                },
            )))
        }

        /// Which kind of payment account details it is.
        #[must_use]
        pub const fn kind(&self) -> EranosPaymentAccountDetailsKind {
            match &self.0 {
                eranos_bindings::PaymentAccountDetails::Iban { .. } => {
                    EranosPaymentAccountDetailsKind::Iban
                }
                eranos_bindings::PaymentAccountDetails::Username { .. } => {
                    EranosPaymentAccountDetailsKind::Username
                }
            }
        }

        /// The IBAN, without spaces, in capitals. Present in the kind `Iban` alone. Writes it, if
        /// present; absent otherwise.
        pub fn iban(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::PaymentAccountDetails::Iban { iban, .. } => {
                    crate::write_optional(write, Some(iban))
                }
                eranos_bindings::PaymentAccountDetails::Username { .. } => None,
            }
        }

        /// The username, without surrounding whitespace. Present in the kind `Username` alone.
        /// Writes it, if present; absent otherwise.
        pub fn username(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::PaymentAccountDetails::Username { username, .. } => {
                    crate::write_optional(write, Some(username))
                }
                eranos_bindings::PaymentAccountDetails::Iban { .. } => None,
            }
        }
    }

    /// The kinds of payment account details.
    #[derive(Debug)]
    pub enum EranosPaymentAccountDetailsKind {
        /// A bank account.
        Iban,
        /// A username in a payment app.
        Username,
    }

    /// A list of payment account records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosPaymentAccountList(pub(crate) Vec<eranos_bindings::PaymentAccount>);

    impl EranosPaymentAccountList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosPaymentAccount) {
            self.0.push(element.0.clone());
        }

        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosPaymentAccount>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosPaymentAccount(element.clone())))
        }
    }

    /// A list of payment records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosPaymentList(pub(crate) Vec<eranos_bindings::Payment>);

    impl EranosPaymentList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosPayment>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosPayment(element.clone())))
        }
    }

    /// The payments that settle every balance.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSettlement(pub(crate) eranos_bindings::Settlement);

    impl EranosSettlement {
        /// The payments, ordered by payer and then by recipient.
        #[must_use]
        pub fn payments(&self) -> Box<EranosPaymentList> {
            Box::new(EranosPaymentList(self.0.payments.clone()))
        }

        /// The strategy that produced them: the one asked for, or, for the adaptive strategy, the
        /// one whose plan it chose.
        #[must_use]
        pub fn strategy(&self) -> EranosSettlementStrategy {
            self.0.strategy.into()
        }
    }

    /// How balances turn into payments.
    #[derive(Debug)]
    #[diplomat::enum_convert(eranos_bindings::SettlementStrategy)]
    pub enum EranosSettlementStrategy {
        /// The fewest payments while at most 20 balances are left after exact opposites pay each
        /// other, the greedy plan beyond; the settlement names the strategy whose plan it is.
        Adaptive,
        /// The fewest payments possible.
        MinimumTransfers,
        /// The largest debtor pays the largest creditor, repeatedly, after exact opposites pay each
        /// other where that saves a payment.
        Greedy,
        /// Everyone pays back the people who paid for them.
        Direct,
    }

    /// How an amount divides among members.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSplit(pub(crate) eranos_bindings::Split);

    impl EranosSplit {
        /// Every member pays the same.
        #[must_use]
        pub fn create_equal(members: &EranosTextList) -> Box<Self> {
            Box::new(Self(eranos_bindings::Split::Equal {
                members: members.0.clone(),
            }))
        }

        /// Every member pays in proportion to their parts.
        #[must_use]
        pub fn create_parts(shares: &EranosMemberPartsList) -> Box<Self> {
            Box::new(Self(eranos_bindings::Split::Parts {
                shares: shares.0.clone(),
            }))
        }

        /// Every member pays a percentage; the percentages add up to 100 %.
        #[must_use]
        pub fn create_percentages(shares: &EranosMemberPercentageList) -> Box<Self> {
            Box::new(Self(eranos_bindings::Split::Percentages {
                shares: shares.0.clone(),
            }))
        }

        /// Every member pays an exact amount; the amounts add up to what they divide.
        #[must_use]
        pub fn create_amounts(shares: &EranosMemberAmountList) -> Box<Self> {
            Box::new(Self(eranos_bindings::Split::Amounts {
                shares: shares.0.clone(),
            }))
        }

        /// Which kind of split it is.
        #[must_use]
        pub const fn kind(&self) -> EranosSplitKind {
            match &self.0 {
                eranos_bindings::Split::Equal { .. } => EranosSplitKind::Equal,
                eranos_bindings::Split::Parts { .. } => EranosSplitKind::Parts,
                eranos_bindings::Split::Percentages { .. } => EranosSplitKind::Percentages,
                eranos_bindings::Split::Amounts { .. } => EranosSplitKind::Amounts,
            }
        }

        /// The amount of every member. Present in the kind `Amounts` alone.
        #[must_use]
        pub fn amounts_shares(&self) -> Option<Box<EranosMemberAmountList>> {
            match &self.0 {
                eranos_bindings::Split::Amounts { shares, .. } => {
                    Some(Box::new(EranosMemberAmountList(shares.clone())))
                }
                eranos_bindings::Split::Equal { .. }
                | eranos_bindings::Split::Parts { .. }
                | eranos_bindings::Split::Percentages { .. } => None,
            }
        }

        /// The members, as identifiers. Present in the kind `Equal` alone.
        #[must_use]
        pub fn members(&self) -> Option<Box<EranosTextList>> {
            match &self.0 {
                eranos_bindings::Split::Equal { members, .. } => {
                    Some(Box::new(EranosTextList(members.clone())))
                }
                eranos_bindings::Split::Parts { .. }
                | eranos_bindings::Split::Percentages { .. }
                | eranos_bindings::Split::Amounts { .. } => None,
            }
        }

        /// The parts of every member. Present in the kind `Parts` alone.
        #[must_use]
        pub fn parts_shares(&self) -> Option<Box<EranosMemberPartsList>> {
            match &self.0 {
                eranos_bindings::Split::Parts { shares, .. } => {
                    Some(Box::new(EranosMemberPartsList(shares.clone())))
                }
                eranos_bindings::Split::Equal { .. }
                | eranos_bindings::Split::Percentages { .. }
                | eranos_bindings::Split::Amounts { .. } => None,
            }
        }

        /// The percentage of every member. Present in the kind `Percentages` alone.
        #[must_use]
        pub fn percentages_shares(&self) -> Option<Box<EranosMemberPercentageList>> {
            match &self.0 {
                eranos_bindings::Split::Percentages { shares, .. } => {
                    Some(Box::new(EranosMemberPercentageList(shares.clone())))
                }
                eranos_bindings::Split::Equal { .. }
                | eranos_bindings::Split::Parts { .. }
                | eranos_bindings::Split::Amounts { .. } => None,
            }
        }

        /// The identifiers of the members the split divides among, in the order it lists them.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Id` for an identifier that is not one.
        pub fn participants(&self) -> Result<Box<EranosTextList>, Box<EranosError>> {
            eranos_bindings::split_members(&self.0)
                .map(|members| Box::new(EranosTextList(members)))
                .map_err(crate::failed)
        }
    }

    /// The kinds of split.
    #[derive(Debug)]
    pub enum EranosSplitKind {
        /// Every member pays the same.
        Equal,
        /// Every member pays in proportion to their parts.
        Parts,
        /// Every member pays a percentage; the percentages add up to 100 %.
        Percentages,
        /// Every member pays an exact amount; the amounts add up to what they divide.
        Amounts,
    }

    /// The group's totals: overall, per member, per category, and per member per category, all in
    /// the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSummary(pub(crate) eranos_bindings::Summary);

    impl EranosSummary {
        /// What the group spent.
        #[must_use]
        pub fn total_spent(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total_spent.clone()))
        }

        /// The totals of every member.
        #[must_use]
        pub fn members(&self) -> Box<EranosMemberSummaryList> {
            Box::new(EranosMemberSummaryList(self.0.members.clone()))
        }

        /// The total of every category.
        #[must_use]
        pub fn category_totals(&self) -> Box<EranosCategoryTotalList> {
            Box::new(EranosCategoryTotalList(self.0.category_totals.clone()))
        }

        /// The totals per category of every member.
        #[must_use]
        pub fn member_category_totals(&self) -> Box<EranosMemberCategoryTotalsList> {
            Box::new(EranosMemberCategoryTotalsList(
                self.0.member_category_totals.clone(),
            ))
        }
    }

    /// A tip on top of an expense, covered by its payers.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosTip(pub(crate) eranos_bindings::Tip);

    impl EranosTip {
        /// A tip of its required fields; every other field starts absent, empty, or at its default.
        #[must_use]
        pub fn create(value: &EranosTipValue, distribution: EranosTipDistribution) -> Box<Self> {
            Box::new(Self(eranos_bindings::Tip {
                value: value.0.clone(),
                distribution: distribution.into(),
            }))
        }

        /// How the tip is expressed.
        #[must_use]
        pub fn value(&self) -> Box<EranosTipValue> {
            Box::new(EranosTipValue(self.0.value.clone()))
        }

        /// Sets how the tip is expressed.
        pub fn set_value(&mut self, value: &EranosTipValue) {
            self.0.value.clone_from(&value.0);
        }

        /// How the tip is shared.
        #[must_use]
        pub fn distribution(&self) -> EranosTipDistribution {
            self.0.distribution.into()
        }

        /// Sets how the tip is shared.
        pub fn set_distribution(&mut self, distribution: EranosTipDistribution) {
            self.0.distribution = distribution.into();
        }
    }

    /// How a tip is shared among the participants.
    #[derive(Debug)]
    #[diplomat::enum_convert(eranos_bindings::TipDistribution)]
    pub enum EranosTipDistribution {
        /// In proportion to each participant's share of the amount.
        Proportional,
        /// The same for every participant.
        Equal,
    }

    /// How a tip is expressed.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTipValue(pub(crate) eranos_bindings::TipValue);

    impl EranosTipValue {
        /// A percentage of the expense's amount.
        #[must_use]
        pub fn create_percentage(basis_points: i32) -> Box<Self> {
            Box::new(Self(eranos_bindings::TipValue::Percentage { basis_points }))
        }

        /// A fixed amount in the expense's currency.
        #[must_use]
        pub fn create_fixed(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::TipValue::Fixed {
                amount: amount.0.clone(),
            }))
        }

        /// Which kind of tip value it is.
        #[must_use]
        pub const fn kind(&self) -> EranosTipValueKind {
            match &self.0 {
                eranos_bindings::TipValue::Percentage { .. } => EranosTipValueKind::Percentage,
                eranos_bindings::TipValue::Fixed { .. } => EranosTipValueKind::Fixed,
            }
        }

        /// The amount. Present in the kind `Fixed` alone.
        #[must_use]
        pub fn amount(&self) -> Option<Box<EranosMoney>> {
            match &self.0 {
                eranos_bindings::TipValue::Fixed { amount, .. } => {
                    Some(Box::new(EranosMoney(amount.clone())))
                }
                eranos_bindings::TipValue::Percentage { .. } => None,
            }
        }

        /// The percentage, in basis points: 1 000 for 10 %. Present in the kind `Percentage` alone.
        #[must_use]
        pub const fn basis_points(&self) -> Option<i32> {
            match &self.0 {
                eranos_bindings::TipValue::Percentage { basis_points, .. } => Some(*basis_points),
                eranos_bindings::TipValue::Fixed { .. } => None,
            }
        }
    }

    /// The kinds of tip value.
    #[derive(Debug)]
    pub enum EranosTipValueKind {
        /// A percentage of the expense's amount.
        Percentage,
        /// A fixed amount in the expense's currency.
        Fixed,
    }

    /// Money one member paid another, such as a repayment.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosTransfer(pub(crate) eranos_bindings::Transfer);

    impl EranosTransfer {
        /// A transfer of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            from: &DiplomatStr,
            to: &DiplomatStr,
            cost: &EranosCost,
            date: EranosCalendarDate,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Transfer {
                id: None,
                from: crate::text(from)?,
                to: crate::text(to)?,
                cost: cost.0.clone(),
                date: date.into(),
                time: None,
                note: None,
            })))
        }

        /// The identifier: absent for a transfer to add, present for one the group holds or one to
        /// replace. Writes it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a transfer to add, present for one the group holds or
        /// one to replace.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }

        /// Clears the identifier: absent for a transfer to add, present for one the group holds or
        /// one to replace.
        pub fn clear_id(&mut self) {
            self.0.id = None;
        }

        /// The member who paid, as their identifier.
        pub fn from(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.from);
        }

        /// Sets the member who paid, as their identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_from(&mut self, from: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.from = crate::text(from)?;
            Ok(())
        }

        /// The member who was paid, as their identifier.
        pub fn to(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.to);
        }

        /// Sets the member who was paid, as their identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_to(&mut self, to: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.to = crate::text(to)?;
            Ok(())
        }

        /// What was paid.
        #[must_use]
        pub fn cost(&self) -> Box<EranosCost> {
            Box::new(EranosCost(self.0.cost.clone()))
        }

        /// Sets what was paid.
        pub fn set_cost(&mut self, cost: &EranosCost) {
            self.0.cost.clone_from(&cost.0);
        }

        /// The day it was paid.
        #[must_use]
        pub fn date(&self) -> EranosCalendarDate {
            self.0.date.into()
        }

        /// Sets the day it was paid.
        pub fn set_date(&mut self, date: EranosCalendarDate) {
            self.0.date = date.into();
        }

        /// The time of day it was paid.
        #[must_use]
        pub fn time(&self) -> Option<EranosTimeOfDay> {
            self.0.time.map(Into::into)
        }

        /// Sets the time of day it was paid.
        pub fn set_time(&mut self, time: EranosTimeOfDay) {
            self.0.time = Some(time.into());
        }

        /// Clears the time of day it was paid.
        pub const fn clear_time(&mut self) {
            self.0.time = None;
        }

        /// A note on the transfer. Writes it, if present; absent otherwise.
        pub fn note(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.note.as_deref())
        }

        /// Sets a note on the transfer.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_note(&mut self, note: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.note = Some(crate::text(note)?);
            Ok(())
        }

        /// Clears a note on the transfer.
        pub fn clear_note(&mut self) {
            self.0.note = None;
        }
    }

    /// A list of transfer records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTransferList(pub(crate) Vec<eranos_bindings::Transfer>);

    impl EranosTransferList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosTransfer>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosTransfer(element.clone())))
        }
    }
}
