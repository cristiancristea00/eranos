//! The entries of the C and C++ APIs: costs, splits, items, tips, expenses, transfers, accounts, members, and
//! categories, each with a constructor of its required fields and a setter of every field.

/// The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it in the
/// namespace `eranos::detail`.
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

    /// How a tip is shared among the participants.
    #[derive(Debug)]
    pub enum EranosTipDistribution {
        /// The same for every participant.
        Equal,
        /// In proportion to each participant's share of the amount.
        Proportional,
    }

    /// A member's exact amount in a split.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberAmount(pub(crate) eranos_bindings::MemberAmount);

    impl EranosMemberAmount {
        /// A new member amount of its required fields; every other field starts absent, empty, or
        /// at its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            amount: &EranosMoney,
            member: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberAmount {
                amount: amount.0.clone(),
                member: crate::text(member)?,
            })))
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

        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member's identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }
    }

    /// A member's parts in a split, in hundredths of a part.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberParts(pub(crate) eranos_bindings::MemberParts);

    impl EranosMemberParts {
        /// A new member parts of its required fields; every other field starts absent, empty, or at
        /// its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            hundredths: i32,
            member: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberParts {
                hundredths,
                member: crate::text(member)?,
            })))
        }

        /// The parts, in hundredths: 100 for one part.
        #[must_use]
        pub const fn hundredths(&self) -> i32 {
            self.0.hundredths
        }

        /// Sets the parts, in hundredths: 100 for one part.
        pub const fn set_hundredths(&mut self, hundredths: i32) {
            self.0.hundredths = hundredths;
        }

        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member's identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }
    }

    /// A member's percentage in a split, in basis points.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMemberPercentage(pub(crate) eranos_bindings::MemberPercentage);

    impl EranosMemberPercentage {
        /// A new member percentage of its required fields; every other field starts absent, empty,
        /// or at its default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            basis_points: i32,
            member: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::MemberPercentage {
                basis_points,
                member: crate::text(member)?,
            })))
        }

        /// The percentage, in basis points: 10000 for 100 %.
        #[must_use]
        pub const fn basis_points(&self) -> i32 {
            self.0.basis_points
        }

        /// Sets the percentage, in basis points: 10000 for 100 %.
        pub const fn set_basis_points(&mut self, basis_points: i32) {
            self.0.basis_points = basis_points;
        }

        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// Sets the member's identifier.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_member(&mut self, member: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.member = crate::text(member)?;
            Ok(())
        }
    }

    /// A line of an itemised bill: `quantity` units of `name` at `unit_price`, which `split`
    /// divides.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosItem(pub(crate) eranos_bindings::Item);

    impl EranosItem {
        /// A new item of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            name: &DiplomatStr,
            split: &EranosSplit,
            unit_price: &EranosMoney,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Item {
                name: crate::text(name)?,
                split: split.0.clone(),
                unit_price: unit_price.0.clone(),
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

        /// How the line's total divides.
        #[must_use]
        pub fn split(&self) -> Box<EranosSplit> {
            Box::new(EranosSplit(self.0.split.clone()))
        }

        /// Sets how the line's total divides.
        pub fn set_split(&mut self, split: &EranosSplit) {
            self.0.split.clone_from(&split.0);
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

    /// A tip on top of an expense's amount.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosTip(pub(crate) eranos_bindings::Tip);

    impl EranosTip {
        /// A new tip of its required fields; every other field starts absent, empty, or at its
        /// default.
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

        /// How the tip is shared among the participants.
        #[must_use]
        pub fn distribution(&self) -> EranosTipDistribution {
            self.0.distribution.into()
        }

        /// Sets how the tip is shared among the participants.
        pub fn set_distribution(&mut self, distribution: EranosTipDistribution) {
            self.0.distribution = distribution.into();
        }
    }

    /// An expense: what was bought, what it cost, who paid, and how it divides.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosExpense(pub(crate) eranos_bindings::Expense);

    impl EranosExpense {
        /// A new expense of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            cost: &EranosCost,
            date: EranosCalendarDate,
            name: &DiplomatStr,
            payers: &EranosSplit,
            split: &EranosExpenseSplit,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Expense {
                id: None,
                cost: cost.0.clone(),
                date: date.into(),
                name: crate::text(name)?,
                payers: payers.0.clone(),
                split: split.0.clone(),
                category: None,
                description: None,
                time: None,
                tip: None,
            })))
        }

        /// The identifier: absent for an expense to add, present for one the group holds. Writes
        /// it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for an expense to add, present for one the group holds.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }
        /// Clears the identifier: absent for an expense to add, present for one the group holds.
        pub fn clear_id(&mut self) {
            self.0.id = None;
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

        /// The day it was bought.
        #[must_use]
        pub fn date(&self) -> EranosCalendarDate {
            self.0.date.into()
        }

        /// Sets the day it was bought.
        pub fn set_date(&mut self, date: EranosCalendarDate) {
            self.0.date = date.into();
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

        /// Who paid the total, amount and tip, and how much each.
        #[must_use]
        pub fn payers(&self) -> Box<EranosSplit> {
            Box::new(EranosSplit(self.0.payers.clone()))
        }

        /// Sets who paid the total, amount and tip, and how much each.
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

        /// The category's identifier; absent for the built-in one. Writes it, if present; absent
        /// otherwise.
        pub fn category(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.category.as_deref())
        }

        /// Sets the category's identifier; absent for the built-in one.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_category(&mut self, category: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.category = Some(crate::text(category)?);
            Ok(())
        }
        /// Clears the category's identifier; absent for the built-in one.
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

        /// The tip in the expense's currency, zero without one.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the kind `Money` for an
        /// amount out of range.
        pub fn tip_amount(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
            eranos_bindings::expense_tip_amount(&self.0)
                .map(|money| Box::new(EranosMoney(money)))
                .map_err(crate::failed)
        }

        /// The amount with the tip, what the payers paid.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Expense` for a tip the expense cannot have, and of the kind `Money` for an
        /// amount out of range.
        pub fn total(&self) -> Result<Box<EranosMoney>, Box<EranosError>> {
            eranos_bindings::expense_total(&self.0)
                .map(|money| Box::new(EranosMoney(money)))
                .map_err(crate::failed)
        }

        /// The identifiers of every member the expense names, payers and participants, each once and in the
        /// order of their identifiers.
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

    /// A transfer: money one member paid another.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosTransfer(pub(crate) eranos_bindings::Transfer);

    impl EranosTransfer {
        /// A new transfer of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            cost: &EranosCost,
            date: EranosCalendarDate,
            from: &DiplomatStr,
            to: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Transfer {
                id: None,
                cost: cost.0.clone(),
                date: date.into(),
                from: crate::text(from)?,
                to: crate::text(to)?,
                note: None,
                time: None,
            })))
        }

        /// The identifier: absent for a transfer to add, present for one the group holds. Writes
        /// it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a transfer to add, present for one the group holds.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }
        /// Clears the identifier: absent for a transfer to add, present for one the group holds.
        pub fn clear_id(&mut self) {
            self.0.id = None;
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

        /// The identifier of the member who paid.
        pub fn from(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.from);
        }

        /// Sets the identifier of the member who paid.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_from(&mut self, from: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.from = crate::text(from)?;
            Ok(())
        }

        /// The identifier of the member who was paid.
        pub fn to(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.to);
        }

        /// Sets the identifier of the member who was paid.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_to(&mut self, to: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.to = crate::text(to)?;
            Ok(())
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
    }

    /// An account a member can be paid into.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosAccount(pub(crate) eranos_bindings::Account);

    impl EranosAccount {
        /// A new account of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            details: &EranosAccountDetails,
            name: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Account {
                details: details.0.clone(),
                name: crate::text(name)?,
            })))
        }

        /// The IBAN or the username.
        #[must_use]
        pub fn details(&self) -> Box<EranosAccountDetails> {
            Box::new(EranosAccountDetails(self.0.details.clone()))
        }

        /// Sets the IBAN or the username.
        pub fn set_details(&mut self, details: &EranosAccountDetails) {
            self.0.details.clone_from(&details.0);
        }

        /// The label.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// Sets the label.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&mut self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.name = crate::text(name)?;
            Ok(())
        }
    }

    /// A member of the group.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosMember(pub(crate) eranos_bindings::Member);

    impl EranosMember {
        /// A new member of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(first_name: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Member {
                id: None,
                first_name: crate::text(first_name)?,
                accounts: Vec::new(),
                email: None,
                last_name: None,
                phone: None,
            })))
        }

        /// The identifier: absent for a member to add, present for one the group holds. Writes it,
        /// if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a member to add, present for one the group holds.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }
        /// Clears the identifier: absent for a member to add, present for one the group holds.
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

        /// Where the member can be paid, in the order they were given.
        #[must_use]
        pub fn accounts(&self) -> Box<EranosAccountList> {
            Box::new(EranosAccountList(self.0.accounts.clone()))
        }

        /// Sets where the member can be paid, in the order they were given.
        pub fn set_accounts(&mut self, accounts: &EranosAccountList) {
            self.0.accounts.clone_from(&accounts.0);
        }

        /// The email address. Writes it, if present; absent otherwise.
        pub fn email(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.email.as_deref())
        }

        /// Sets the email address.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_email(&mut self, email: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.email = Some(crate::text(email)?);
            Ok(())
        }
        /// Clears the email address.
        pub fn clear_email(&mut self) {
            self.0.email = None;
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

        /// The phone number. Writes it, if present; absent otherwise.
        pub fn phone(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.phone.as_deref())
        }

        /// Sets the phone number.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_phone(&mut self, phone: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.phone = Some(crate::text(phone)?);
            Ok(())
        }
        /// Clears the phone number.
        pub fn clear_phone(&mut self) {
            self.0.phone = None;
        }
    }

    /// A category of expenses.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosCategory(pub(crate) eranos_bindings::Category);

    impl EranosCategory {
        /// A new category of its required fields; every other field starts absent, empty, or at its
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

        /// The identifier: absent for a category to add, present for one the group holds. Writes
        /// it, if present; absent otherwise.
        pub fn id(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.id.as_deref())
        }

        /// Sets the identifier: absent for a category to add, present for one the group holds.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn set_id(&mut self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.id = Some(crate::text(id)?);
            Ok(())
        }
        /// Clears the identifier: absent for a category to add, present for one the group holds.
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

    /// A list of account records.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosAccountList(pub(crate) Vec<eranos_bindings::Account>);

    impl EranosAccountList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds a copy of `element` at the end.
        pub fn push(&mut self, element: &EranosAccount) {
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
        pub fn get(&self, index: usize) -> Option<Box<EranosAccount>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosAccount(element.clone())))
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

    /// The kind of a cost.
    #[derive(Debug)]
    pub enum EranosCostKind {
        /// An amount in another currency than the group's, converted at `rate`.
        Converted,
        /// An amount in the group currency.
        Local,
    }

    /// What an entry cost: an amount in the group currency, or one in another, converted at a rate.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCost(pub(crate) eranos_bindings::Cost);

    impl EranosCost {
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

        /// An amount in the group currency.
        #[must_use]
        pub fn create_local(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::Cost::Local {
                amount: amount.0.clone(),
            }))
        }

        /// Which kind of cost it is.
        #[must_use]
        pub const fn kind(&self) -> EranosCostKind {
            match &self.0 {
                eranos_bindings::Cost::Converted { .. } => EranosCostKind::Converted,
                eranos_bindings::Cost::Local { .. } => EranosCostKind::Local,
            }
        }

        /// The amount, in its own currency.
        #[must_use]
        pub fn amount(&self) -> Box<EranosMoney> {
            match &self.0 {
                eranos_bindings::Cost::Converted { amount, .. }
                | eranos_bindings::Cost::Local { amount, .. } => {
                    Box::new(EranosMoney(amount.clone()))
                }
            }
        }

        /// The rate into the group currency, as a decimal. Present in the kind `Converted` alone.
        /// Writes it, if present; absent otherwise.
        pub fn rate(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::Cost::Converted { rate, .. } => {
                    crate::write_optional(write, Some(rate))
                }
                eranos_bindings::Cost::Local { .. } => None,
            }
        }
    }

    /// The kind of a split.
    #[derive(Debug)]
    pub enum EranosSplitKind {
        /// Every member pays an exact amount; the amounts add up to what they divide.
        Amounts,
        /// Every member pays the same.
        Equal,
        /// Every member pays in proportion to their parts.
        Parts,
        /// Every member pays a percentage; the percentages add up to 100 %.
        Percentages,
    }

    /// How an amount is divided among members.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSplit(pub(crate) eranos_bindings::Split);

    impl EranosSplit {
        /// Every member pays an exact amount; the amounts add up to what they divide.
        #[must_use]
        pub fn create_amounts(shares: &EranosMemberAmountList) -> Box<Self> {
            Box::new(Self(eranos_bindings::Split::Amounts {
                shares: shares.0.clone(),
            }))
        }

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

        /// Which kind of split it is.
        #[must_use]
        pub const fn kind(&self) -> EranosSplitKind {
            match &self.0 {
                eranos_bindings::Split::Amounts { .. } => EranosSplitKind::Amounts,
                eranos_bindings::Split::Equal { .. } => EranosSplitKind::Equal,
                eranos_bindings::Split::Parts { .. } => EranosSplitKind::Parts,
                eranos_bindings::Split::Percentages { .. } => EranosSplitKind::Percentages,
            }
        }

        /// Each member's amount. Present in the kind `Amounts` alone.
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

        /// The identifiers of the members who share the amount. Present in the kind `Equal` alone.
        #[must_use]
        pub fn members(&self) -> Option<Box<EranosTextList>> {
            match &self.0 {
                eranos_bindings::Split::Equal { members, .. } => {
                    Some(Box::new(EranosTextList(members.clone())))
                }
                eranos_bindings::Split::Amounts { .. }
                | eranos_bindings::Split::Parts { .. }
                | eranos_bindings::Split::Percentages { .. } => None,
            }
        }

        /// Each member's parts. Present in the kind `Parts` alone.
        #[must_use]
        pub fn parts_shares(&self) -> Option<Box<EranosMemberPartsList>> {
            match &self.0 {
                eranos_bindings::Split::Parts { shares, .. } => {
                    Some(Box::new(EranosMemberPartsList(shares.clone())))
                }
                eranos_bindings::Split::Amounts { .. }
                | eranos_bindings::Split::Equal { .. }
                | eranos_bindings::Split::Percentages { .. } => None,
            }
        }

        /// Each member's percentage. Present in the kind `Percentages` alone.
        #[must_use]
        pub fn percentages_shares(&self) -> Option<Box<EranosMemberPercentageList>> {
            match &self.0 {
                eranos_bindings::Split::Percentages { shares, .. } => {
                    Some(Box::new(EranosMemberPercentageList(shares.clone())))
                }
                eranos_bindings::Split::Amounts { .. }
                | eranos_bindings::Split::Equal { .. }
                | eranos_bindings::Split::Parts { .. } => None,
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

    /// The kind of a expense split.
    #[derive(Debug)]
    pub enum EranosExpenseSplitKind {
        /// Every line is divided by its own split; the lines add up to the amount.
        Itemised,
        /// The whole amount is divided by one split.
        Whole,
    }

    /// How an expense's amount is shared.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseSplit(pub(crate) eranos_bindings::ExpenseSplit);

    impl EranosExpenseSplit {
        /// Every line is divided by its own split; the lines add up to the amount.
        #[must_use]
        pub fn create_itemised(items: &EranosItemList) -> Box<Self> {
            Box::new(Self(eranos_bindings::ExpenseSplit::Itemised {
                items: items.0.clone(),
            }))
        }

        /// The whole amount is divided by one split.
        #[must_use]
        pub fn create_whole(split: &EranosSplit) -> Box<Self> {
            Box::new(Self(eranos_bindings::ExpenseSplit::Whole {
                split: split.0.clone(),
            }))
        }

        /// Which kind of expense split it is.
        #[must_use]
        pub const fn kind(&self) -> EranosExpenseSplitKind {
            match &self.0 {
                eranos_bindings::ExpenseSplit::Itemised { .. } => EranosExpenseSplitKind::Itemised,
                eranos_bindings::ExpenseSplit::Whole { .. } => EranosExpenseSplitKind::Whole,
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

    /// The kind of a tip value.
    #[derive(Debug)]
    pub enum EranosTipValueKind {
        /// A fixed amount in the expense's currency.
        Fixed,
        /// A percentage of the expense's amount.
        Percentage,
    }

    /// How a tip is expressed.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTipValue(pub(crate) eranos_bindings::TipValue);

    impl EranosTipValue {
        /// A fixed amount in the expense's currency.
        #[must_use]
        pub fn create_fixed(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::TipValue::Fixed {
                amount: amount.0.clone(),
            }))
        }

        /// A percentage of the expense's amount.
        #[must_use]
        pub fn create_percentage(basis_points: i32) -> Box<Self> {
            Box::new(Self(eranos_bindings::TipValue::Percentage { basis_points }))
        }

        /// Which kind of tip value it is.
        #[must_use]
        pub const fn kind(&self) -> EranosTipValueKind {
            match &self.0 {
                eranos_bindings::TipValue::Fixed { .. } => EranosTipValueKind::Fixed,
                eranos_bindings::TipValue::Percentage { .. } => EranosTipValueKind::Percentage,
            }
        }

        /// The tip. Present in the kind `Fixed` alone.
        #[must_use]
        pub fn amount(&self) -> Option<Box<EranosMoney>> {
            match &self.0 {
                eranos_bindings::TipValue::Fixed { amount, .. } => {
                    Some(Box::new(EranosMoney(amount.clone())))
                }
                eranos_bindings::TipValue::Percentage { .. } => None,
            }
        }

        /// The tip's share of the amount, in basis points. Present in the kind `Percentage` alone.
        #[must_use]
        pub const fn basis_points(&self) -> Option<i32> {
            match &self.0 {
                eranos_bindings::TipValue::Percentage { basis_points, .. } => Some(*basis_points),
                eranos_bindings::TipValue::Fixed { .. } => None,
            }
        }
    }

    /// The kind of a account details.
    #[derive(Debug)]
    pub enum EranosAccountDetailsKind {
        /// A bank account.
        Iban,
        /// A username in a payment app.
        Username,
    }

    /// Where an account is: a bank account or a username in a payment app.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosAccountDetails(pub(crate) eranos_bindings::AccountDetails);

    impl EranosAccountDetails {
        /// A bank account.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create_iban(iban: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::AccountDetails::Iban {
                iban: crate::text(iban)?,
            })))
        }

        /// A username in a payment app.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create_username(username: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::AccountDetails::Username {
                username: crate::text(username)?,
            })))
        }

        /// Which kind of account details it is.
        #[must_use]
        pub const fn kind(&self) -> EranosAccountDetailsKind {
            match &self.0 {
                eranos_bindings::AccountDetails::Iban { .. } => EranosAccountDetailsKind::Iban,
                eranos_bindings::AccountDetails::Username { .. } => {
                    EranosAccountDetailsKind::Username
                }
            }
        }

        /// The account's IBAN. Present in the kind `Iban` alone. Writes it, if present; absent
        /// otherwise.
        pub fn iban(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::AccountDetails::Iban { iban, .. } => {
                    crate::write_optional(write, Some(iban))
                }
                eranos_bindings::AccountDetails::Username { .. } => None,
            }
        }

        /// The username. Present in the kind `Username` alone. Writes it, if present; absent
        /// otherwise.
        pub fn username(&self, write: &mut DiplomatWrite) -> Option<()> {
            match &self.0 {
                eranos_bindings::AccountDetails::Username { username, .. } => {
                    crate::write_optional(write, Some(username))
                }
                eranos_bindings::AccountDetails::Iban { .. } => None,
            }
        }
    }
}

impl From<ffi::EranosTipDistribution> for eranos_bindings::TipDistribution {
    fn from(value: ffi::EranosTipDistribution) -> Self {
        match value {
            ffi::EranosTipDistribution::Equal => Self::Equal,
            ffi::EranosTipDistribution::Proportional => Self::Proportional,
        }
    }
}

impl From<eranos_bindings::TipDistribution> for ffi::EranosTipDistribution {
    fn from(value: eranos_bindings::TipDistribution) -> Self {
        match value {
            eranos_bindings::TipDistribution::Equal => Self::Equal,
            eranos_bindings::TipDistribution::Proportional => Self::Proportional,
        }
    }
}
