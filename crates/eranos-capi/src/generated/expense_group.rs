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
    use crate::generated::records::ffi::{
        EranosCategory, EranosCategoryList, EranosCurrencyRateList, EranosExpense,
        EranosExpenseList, EranosExpenseShares, EranosMember, EranosMemberList,
        EranosNewExpenseGroup, EranosSettlementPlan, EranosSettlementStrategy, EranosSummary,
        EranosTransfer, EranosTransferList,
    };
    use crate::values::ffi::EranosMoney;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};
    use std::sync::{Arc, OnceLock, mpsc};

    /// A cancellation of asynchronous calls, which one cancellation can be given to several of.
    ///
    /// Cancelling it stops the calls it was given at their next yield, or at once while they wait
    /// for the lock, and they report the kind `Cancelled`, leaving the group as it was.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCancellation(pub(crate) Arc<eranos_bindings::Cancellation>);

    impl EranosCancellation {
        /// A cancellation not yet cancelled.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Arc::default()))
        }

        /// Cancels every call it was given.
        pub fn cancel(&self) {
            self.0.cancel();
        }

        /// Whether it was cancelled.
        #[must_use]
        pub fn is_cancelled(&self) -> bool {
            self.0.is_cancelled()
        }
    }

    /// The outcome of an asynchronous call that returns a list of category records, which its
    /// callback receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryListOutcome(
        pub(crate) Result<Vec<eranos_bindings::Category>, crate::Failure>,
    );

    impl EranosCategoryListOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the list the call returned, if it succeeded.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosCategoryList>> {
            self.0
                .as_ref()
                .ok()
                .map(|list| Box::new(EranosCategoryList(list.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns a category, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryOutcome(
        pub(crate) Result<Option<eranos_bindings::Category>, crate::Failure>,
    );

    impl EranosCategoryOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the category the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosCategory>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosCategory(value.clone())))
        }
    }

    /// A group: its details, members, categories, expenses, transfers, totals, and settlements.
    ///
    /// Every call has a synchronous form, which blocks the calling thread while another call holds
    /// the lock, and an asynchronous one, which runs on the group's worker thread, started by the
    /// first, in the order of the calls, and reports to its callback there, so a callback must be
    /// safe to call from another thread. Destroying the group lets the queued calls finish and
    /// their callbacks run.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseGroup {
        pub(crate) handle: eranos_bindings::ExpenseGroupHandle,
        pub(crate) jobs: OnceLock<mpsc::Sender<crate::Job>>,
    }

    impl EranosExpenseGroup {
        /// A new group, with the details `details` holds, under the identifier it gives or a
        /// generated one.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Id` for an identifier the rules refuse, of the kind `Money`
        /// for a currency the library does not list, of the kind `ExpenseGroup` for a blank name,
        /// and of the kind `Language` for a language that is not a tag.
        pub fn create(details: &EranosNewExpenseGroup) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::ExpenseGroupHandle::new(&details.0)
                .map(|handle| Box::new(Self::from(handle)))
                .map_err(crate::failed)
        }

        /// The group saved as `json`, which `save` wrote.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidJson` for text that is not a saved group, of the
        /// kind `ExpenseGroup` for a format this release cannot read, of the module whose rule an
        /// entry breaks, and of the kind `InvalidText` for text that is not UTF-8.
        pub fn load(json: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::ExpenseGroupHandle::load(&crate::text(json)?)
                .map(|handle| Box::new(Self::from(handle)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `load`: it runs on a thread of its own and reports its outcome
        /// to `callback` there.
        pub fn load_async(
            json: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseGroupOutcome>) + 'static,
        ) {
            let json = crate::decoded(json);
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::run(
                async move {
                    eranos_bindings::ExpenseGroupHandle::load_async(json?, cancellation)
                        .await
                        .map(Self::from)
                        .map_err(crate::Failure::Library)
                },
                callback,
                |outcome| EranosExpenseGroupOutcome(outcome.map(Some)),
            );
        }

        /// The group's identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn id(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.id().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `id`: it runs on the group's worker thread, in the order of the
        /// calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops the
        /// call at its next yield, or at once while it waits for the lock, and reports the kind
        /// `Cancelled`, leaving the group as it was.
        pub fn id_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .id_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Every category: the built-in one first, then the others in identifier order, which is
        /// creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn categories(&self) -> Result<Box<EranosCategoryList>, Box<EranosError>> {
            self.handle
                .categories()
                .map(|list| Box::new(EranosCategoryList(list)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `categories`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn categories_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosCategoryListOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .categories_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosCategoryListOutcome,
            );
        }

        /// The category with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn category(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosCategory>>, Box<EranosError>> {
            self.handle
                .category(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosCategory(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `category`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn category_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosCategoryOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .category_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosCategoryOutcome,
            );
        }

        /// Adds `category` under its identifier, or one the library generates when it has none, and
        /// returns the identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn add_category(
            &self,
            category: &EranosCategory,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = self
                .handle
                .add_category(&category.0)
                .map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `add_category`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn add_category_async(
            &self,
            category: &EranosCategory,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let category = Ok(category.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .add_category_async(category?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Renames the category that `category` names by its identifier, and returns it as it was.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn replace_category(
            &self,
            category: &EranosCategory,
        ) -> Result<Box<EranosCategory>, Box<EranosError>> {
            self.handle
                .replace_category(&category.0)
                .map(|value| Box::new(EranosCategory(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `replace_category`: it runs on the group's worker thread, in
        /// the order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn replace_category_async(
            &self,
            category: &EranosCategory,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosCategoryOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let category = Ok(category.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .replace_category_async(category?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosCategoryOutcome,
            );
        }

        /// Removes the category with the identifier `id` and returns it; its expenses move to the
        /// built-in one.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn remove_category(
            &self,
            id: &DiplomatStr,
        ) -> Result<Box<EranosCategory>, Box<EranosError>> {
            self.handle
                .remove_category(&crate::text(id)?)
                .map(|value| Box::new(EranosCategory(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `remove_category`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn remove_category_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosCategoryOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .remove_category_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosCategoryOutcome,
            );
        }

        /// The ISO 4217 code of the group currency. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn currency(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.currency().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `currency`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn currency_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .currency_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Moves the group to `currency`, with a rate into it for every other currency its entries
        /// use.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn change_currency(
            &self,
            currency: &DiplomatStr,
            rates: &EranosCurrencyRateList,
        ) -> Result<(), Box<EranosError>> {
            self.handle
                .change_currency(&crate::text(currency)?, &rates.0)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `change_currency`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn change_currency_async(
            &self,
            currency: &DiplomatStr,
            rates: &EranosCurrencyRateList,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let currency = crate::decoded(currency);
            let rates = Ok(rates.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .change_currency_async(currency?, rates?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// Every expense, in identifier order, which is creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn expenses(&self) -> Result<Box<EranosExpenseList>, Box<EranosError>> {
            self.handle
                .expenses()
                .map(|list| Box::new(EranosExpenseList(list)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `expenses`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn expenses_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseListOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .expenses_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosExpenseListOutcome,
            );
        }

        /// The expense with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn expense(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosExpense>>, Box<EranosError>> {
            self.handle
                .expense(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosExpense(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `expense`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn expense_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .expense_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosExpenseOutcome,
            );
        }

        /// How the expense with the identifier `id` divides, in the group currency.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn expense_shares(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosExpenseShares>>, Box<EranosError>> {
            self.handle
                .expense_shares(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosExpenseShares(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `expense_shares`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn expense_shares_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseSharesOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .expense_shares_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosExpenseSharesOutcome,
            );
        }

        /// Adds `expense` under its identifier, or one the library generates when it has none, and
        /// returns the identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn add_expense(
            &self,
            expense: &EranosExpense,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = self.handle.add_expense(&expense.0).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `add_expense`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn add_expense_async(
            &self,
            expense: &EranosExpense,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let expense = Ok(expense.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .add_expense_async(expense?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Replaces the expense that `expense` names by its identifier, and returns it as it was.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn replace_expense(
            &self,
            expense: &EranosExpense,
        ) -> Result<Box<EranosExpense>, Box<EranosError>> {
            self.handle
                .replace_expense(&expense.0)
                .map(|value| Box::new(EranosExpense(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `replace_expense`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn replace_expense_async(
            &self,
            expense: &EranosExpense,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let expense = Ok(expense.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .replace_expense_async(expense?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosExpenseOutcome,
            );
        }

        /// Removes the expense with the identifier `id` and returns it.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn remove_expense(
            &self,
            id: &DiplomatStr,
        ) -> Result<Box<EranosExpense>, Box<EranosError>> {
            self.handle
                .remove_expense(&crate::text(id)?)
                .map(|value| Box::new(EranosExpense(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `remove_expense`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn remove_expense_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .remove_expense_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosExpenseOutcome,
            );
        }

        /// The group's totals.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn summary(&self) -> Result<Box<EranosSummary>, Box<EranosError>> {
            self.handle
                .summary()
                .map(|value| Box::new(EranosSummary(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `summary`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn summary_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosSummaryOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .summary_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosSummaryOutcome,
            );
        }

        /// Every member, archived ones included, in identifier order, which is creation order for
        /// generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn members(&self) -> Result<Box<EranosMemberList>, Box<EranosError>> {
            self.handle
                .members()
                .map(|list| Box::new(EranosMemberList(list)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `members`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn members_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosMemberListOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .members_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosMemberListOutcome,
            );
        }

        /// The member with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn member(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosMember>>, Box<EranosError>> {
            self.handle
                .member(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosMember(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `member`: it runs on the group's worker thread, in the order of
        /// the calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops
        /// the call at its next yield, or at once while it waits for the lock, and reports the kind
        /// `Cancelled`, leaving the group as it was.
        pub fn member_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosMemberOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .member_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosMemberOutcome,
            );
        }

        /// Whether the member with the identifier `id` is archived; `false` for an unknown member.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn is_archived(&self, id: &DiplomatStr) -> Result<bool, Box<EranosError>> {
            self.handle
                .is_archived(&crate::text(id)?)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `is_archived`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn is_archived_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosFlagOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .is_archived_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosFlagOutcome,
            );
        }

        /// Adds `member` under their identifier, or one the library generates when they have none,
        /// and returns the identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn add_member(
            &self,
            member: &EranosMember,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = self.handle.add_member(&member.0).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `add_member`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn add_member_async(
            &self,
            member: &EranosMember,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let member = Ok(member.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .add_member_async(member?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Replaces the member that `member` names by their identifier, and returns them as they
        /// were.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn replace_member(
            &self,
            member: &EranosMember,
        ) -> Result<Box<EranosMember>, Box<EranosError>> {
            self.handle
                .replace_member(&member.0)
                .map(|value| Box::new(EranosMember(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `replace_member`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn replace_member_async(
            &self,
            member: &EranosMember,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosMemberOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let member = Ok(member.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .replace_member_async(member?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosMemberOutcome,
            );
        }

        /// Archives the member with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn archive_member(&self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.handle
                .archive_member(&crate::text(id)?)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `archive_member`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn archive_member_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .archive_member_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// Unarchives the member with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn unarchive_member(&self, id: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.handle
                .unarchive_member(&crate::text(id)?)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `unarchive_member`: it runs on the group's worker thread, in
        /// the order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn unarchive_member_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .unarchive_member_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// Removes the member with the identifier `id`, whom no entry references, and returns them.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn remove_member(
            &self,
            id: &DiplomatStr,
        ) -> Result<Box<EranosMember>, Box<EranosError>> {
            self.handle
                .remove_member(&crate::text(id)?)
                .map(|value| Box::new(EranosMember(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `remove_member`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn remove_member_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosMemberOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .remove_member_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosMemberOutcome,
            );
        }

        /// The group's name. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn name(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.name().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `name`: it runs on the group's worker thread, in the order of
        /// the calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops
        /// the call at its next yield, or at once while it waits for the lock, and reports the kind
        /// `Cancelled`, leaving the group as it was.
        pub fn name_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .name_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Renames the group.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn set_name(&self, name: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.handle
                .set_name(&crate::text(name)?)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `set_name`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn set_name_async(
            &self,
            name: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let name = crate::decoded(name);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .set_name_async(name?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// The payments that settle every balance by `strategy`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        pub fn settlement(
            &self,
            strategy: EranosSettlementStrategy,
        ) -> Result<Box<EranosSettlementPlan>, Box<EranosError>> {
            self.handle
                .settlement(strategy.into())
                .map(|value| Box::new(EranosSettlementPlan(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `settlement`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn settlement_async(
            &self,
            strategy: EranosSettlementStrategy,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosSettlementPlanOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let strategy = Ok(strategy.into());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .settlement_async(strategy?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosSettlementPlanOutcome,
            );
        }

        /// Every transfer, in identifier order, which is creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn transfers(&self) -> Result<Box<EranosTransferList>, Box<EranosError>> {
            self.handle
                .transfers()
                .map(|list| Box::new(EranosTransferList(list)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `transfers`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn transfers_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTransferListOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .transfers_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTransferListOutcome,
            );
        }

        /// The transfer with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn transfer(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosTransfer>>, Box<EranosError>> {
            self.handle
                .transfer(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosTransfer(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `transfer`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn transfer_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTransferOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .transfer_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTransferOutcome,
            );
        }

        /// The amount in the group currency of the transfer with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while a change holds the lock or waits for it.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn transfer_amount(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosMoney>>, Box<EranosError>> {
            self.handle
                .transfer_amount(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosMoney(value))))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `transfer_amount`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn transfer_amount_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosMoneyOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .transfer_amount_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosMoneyOutcome,
            );
        }

        /// Adds `transfer` under its identifier, or one the library generates when it has none, and
        /// returns the identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn add_transfer(
            &self,
            transfer: &EranosTransfer,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = self
                .handle
                .add_transfer(&transfer.0)
                .map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `add_transfer`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn add_transfer_async(
            &self,
            transfer: &EranosTransfer,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let transfer = Ok(transfer.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .add_transfer_async(transfer?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn replace_transfer(
            &self,
            transfer: &EranosTransfer,
        ) -> Result<Box<EranosTransfer>, Box<EranosError>> {
            self.handle
                .replace_transfer(&transfer.0)
                .map(|value| Box::new(EranosTransfer(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `replace_transfer`: it runs on the group's worker thread, in
        /// the order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn replace_transfer_async(
            &self,
            transfer: &EranosTransfer,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTransferOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let transfer = Ok(transfer.0.clone());
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .replace_transfer_async(transfer?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosTransferOutcome,
            );
        }

        /// Removes the transfer with the identifier `id` and returns it.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn remove_transfer(
            &self,
            id: &DiplomatStr,
        ) -> Result<Box<EranosTransfer>, Box<EranosError>> {
            self.handle
                .remove_transfer(&crate::text(id)?)
                .map(|value| Box::new(EranosTransfer(value)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `remove_transfer`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn remove_transfer_async(
            &self,
            id: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTransferOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let id = crate::decoded(id);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .remove_transfer_async(id?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosTransferOutcome,
            );
        }

        /// The group's description. Writes it into `write` when there is one, and nothing
        /// otherwise.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn description(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.description().map_err(crate::failed)?;
            crate::write_optional(write, text.as_deref());
            Ok(())
        }

        /// The asynchronous form of `description`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn description_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosOptionalTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .description_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosOptionalTextOutcome,
            );
        }

        /// Sets the description; blank text clears it.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while another call holds the lock. [`EranosError`] of the kind `InvalidText` for text
        /// that is not UTF-8.
        pub fn set_description(&self, description: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.handle
                .set_description(&crate::text(description)?)
                .map_err(crate::failed)
        }

        /// The asynchronous form of `set_description`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn set_description_async(
            &self,
            description: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let description = crate::decoded(description);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .set_description_async(description?, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// The language the apps show the group in, as a BCP 47 tag. Writes it into `write` when
        /// there is one, and nothing otherwise.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn language(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.language().map_err(crate::failed)?;
            crate::write_optional(write, text.as_deref());
            Ok(())
        }

        /// The asynchronous form of `language`: it runs on the group's worker thread, in the order
        /// of the calls, and reports its outcome to `callback` there. Cancelling `cancellation`
        /// stops the call at its next yield, or at once while it waits for the lock, and reports
        /// the kind `Cancelled`, leaving the group as it was.
        pub fn language_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosOptionalTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .language_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosOptionalTextOutcome,
            );
        }

        /// Sets the language from a BCP 47 tag, or clears it when there is none.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock. [`EranosError`] of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn set_language(&self, language: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.handle
                .set_language(Some(&crate::text(language)?))
                .map_err(crate::failed)
        }

        /// The form of `set_language` that passes no `language`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `ExpenseGroup` with the failure
        /// `ExpenseGroupBusy` on wasm while another call holds the lock.
        pub fn clear_language(&self) -> Result<(), Box<EranosError>> {
            self.handle.set_language(None).map_err(crate::failed)
        }

        /// The asynchronous form of `set_language`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn set_language_async(
            &self,
            language: &DiplomatStr,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            let language = crate::decoded(language);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .set_language_async(Some(language?), cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// The asynchronous form of `clear_language`: it runs on the group's worker thread, in the
        /// order of the calls, and reports its outcome to `callback` there. Cancelling
        /// `cancellation` stops the call at its next yield, or at once while it waits for the lock,
        /// and reports the kind `Cancelled`, leaving the group as it was.
        pub fn clear_language_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosUnitOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .set_language_async(None, cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosUnitOutcome,
            );
        }

        /// The JSON of the group's saved form. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn save(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.handle.save().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The asynchronous form of `save`: it runs on the group's worker thread, in the order of
        /// the calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops
        /// the call at its next yield, or at once while it waits for the lock, and reports the kind
        /// `Cancelled`, leaving the group as it was.
        pub fn save_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosTextOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .save_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                },
                callback,
                EranosTextOutcome,
            );
        }

        /// A read handle, which holds the shared lock until it is released or dropped, so that its
        /// reads see no change in between.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `ExpenseGroup` with the failure `ExpenseGroupBusy` on wasm
        /// while a change holds the lock or waits for it.
        pub fn read(&self) -> Result<Box<EranosExpenseGroupReader>, Box<EranosError>> {
            self.handle
                .read()
                .map(|reader| Box::new(EranosExpenseGroupReader(reader)))
                .map_err(crate::failed)
        }

        /// The asynchronous form of `read`: it runs on the group's worker thread, in the order of
        /// the calls, and reports its outcome to `callback` there. Cancelling `cancellation` stops
        /// the call at its next yield, or at once while it waits for the lock, and reports the kind
        /// `Cancelled`, leaving the group as it was.
        pub fn read_async(
            &self,
            cancellation: Option<&EranosCancellation>,
            callback: impl Fn(Box<EranosExpenseGroupReaderOutcome>) + 'static,
        ) {
            let handle = self.handle.clone();
            let cancellation = cancellation.map(EranosCancellation::shared);
            crate::spawn(
                &self.jobs,
                async move {
                    handle
                        .read_async(cancellation)
                        .await
                        .map_err(crate::Failure::Library)
                        .map(Some)
                },
                callback,
                EranosExpenseGroupReaderOutcome,
            );
        }
    }

    /// The outcome of the asynchronous load, which its callback receives and destroys.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosExpenseGroupOutcome(
        pub(crate) Result<Option<EranosExpenseGroup>, crate::Failure>,
    );

    impl EranosExpenseGroupOutcome {
        /// Whether the load succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the load's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// Takes the group, if the load succeeded and it was not taken before.
        pub fn take(&mut self) -> Option<Box<EranosExpenseGroup>> {
            self.0.as_mut().ok().and_then(Option::take).map(Box::new)
        }
    }

    /// A read handle: it holds the group's shared lock until it is released or destroyed, so the
    /// reads of one report see no change in between; a change waits for it.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseGroupReader(pub(crate) eranos_bindings::ExpenseGroupReader);

    impl EranosExpenseGroupReader {
        /// Releases the lock at once; later reads report the kind `Released`.
        pub fn release(&self) {
            self.0.release();
        }

        /// The group's identifier. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn id(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.id().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Every category: the built-in one first, then the others in identifier order, which is
        /// creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn categories(&self) -> Result<Box<EranosCategoryList>, Box<EranosError>> {
            self.0
                .categories()
                .map(|list| Box::new(EranosCategoryList(list)))
                .map_err(crate::failed)
        }

        /// The category with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn category(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosCategory>>, Box<EranosError>> {
            self.0
                .category(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosCategory(value))))
                .map_err(crate::failed)
        }

        /// The ISO 4217 code of the group currency. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn currency(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.currency().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Every expense, in identifier order, which is creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn expenses(&self) -> Result<Box<EranosExpenseList>, Box<EranosError>> {
            self.0
                .expenses()
                .map(|list| Box::new(EranosExpenseList(list)))
                .map_err(crate::failed)
        }

        /// The expense with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn expense(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosExpense>>, Box<EranosError>> {
            self.0
                .expense(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosExpense(value))))
                .map_err(crate::failed)
        }

        /// How the expense with the identifier `id` divides, in the group currency.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn expense_shares(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosExpenseShares>>, Box<EranosError>> {
            self.0
                .expense_shares(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosExpenseShares(value))))
                .map_err(crate::failed)
        }

        /// The group's totals.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn summary(&self) -> Result<Box<EranosSummary>, Box<EranosError>> {
            self.0
                .summary()
                .map(|value| Box::new(EranosSummary(value)))
                .map_err(crate::failed)
        }

        /// Every member, archived ones included, in identifier order, which is creation order for
        /// generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn members(&self) -> Result<Box<EranosMemberList>, Box<EranosError>> {
            self.0
                .members()
                .map(|list| Box::new(EranosMemberList(list)))
                .map_err(crate::failed)
        }

        /// The member with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn member(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosMember>>, Box<EranosError>> {
            self.0
                .member(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosMember(value))))
                .map_err(crate::failed)
        }

        /// Whether the member with the identifier `id` is archived; `false` for an unknown member.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn is_archived(&self, id: &DiplomatStr) -> Result<bool, Box<EranosError>> {
            self.0.is_archived(&crate::text(id)?).map_err(crate::failed)
        }

        /// The group's name. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn name(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.name().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The payments that settle every balance by `strategy`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        pub fn settlement(
            &self,
            strategy: EranosSettlementStrategy,
        ) -> Result<Box<EranosSettlementPlan>, Box<EranosError>> {
            self.0
                .settlement(strategy.into())
                .map(|value| Box::new(EranosSettlementPlan(value)))
                .map_err(crate::failed)
        }

        /// Every transfer, in identifier order, which is creation order for generated identifiers.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn transfers(&self) -> Result<Box<EranosTransferList>, Box<EranosError>> {
            self.0
                .transfers()
                .map(|list| Box::new(EranosTransferList(list)))
                .map_err(crate::failed)
        }

        /// The transfer with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn transfer(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosTransfer>>, Box<EranosError>> {
            self.0
                .transfer(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosTransfer(value))))
                .map_err(crate::failed)
        }

        /// The amount in the group currency of the transfer with the identifier `id`.
        ///
        /// # Errors
        ///
        /// The library's refusal. [`EranosError`] of the kind `Released` after `release`.
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn transfer_amount(
            &self,
            id: &DiplomatStr,
        ) -> Result<Option<Box<EranosMoney>>, Box<EranosError>> {
            self.0
                .transfer_amount(&crate::text(id)?)
                .map(|value| value.map(|value| Box::new(EranosMoney(value))))
                .map_err(crate::failed)
        }

        /// The group's description. Writes it into `write` when there is one, and nothing
        /// otherwise.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn description(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.description().map_err(crate::failed)?;
            crate::write_optional(write, text.as_deref());
            Ok(())
        }

        /// The language the apps show the group in, as a BCP 47 tag. Writes it into `write` when
        /// there is one, and nothing otherwise.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn language(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.language().map_err(crate::failed)?;
            crate::write_optional(write, text.as_deref());
            Ok(())
        }

        /// The JSON of the group's saved form. Writes it into `write`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Released` after `release`.
        pub fn save(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = self.0.save().map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }

    /// The outcome of an asynchronous call that returns a read handle, which its callback receives
    /// and destroys.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosExpenseGroupReaderOutcome(
        pub(crate) Result<Option<eranos_bindings::ExpenseGroupReader>, crate::Failure>,
    );

    impl EranosExpenseGroupReaderOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// Takes the read handle, if the call succeeded and it was not taken before.
        pub fn take(&mut self) -> Option<Box<EranosExpenseGroupReader>> {
            self.0
                .as_mut()
                .ok()
                .and_then(Option::take)
                .map(|reader| Box::new(EranosExpenseGroupReader(reader)))
        }
    }

    /// The outcome of an asynchronous call that returns a list of expense records, which its
    /// callback receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseListOutcome(
        pub(crate) Result<Vec<eranos_bindings::Expense>, crate::Failure>,
    );

    impl EranosExpenseListOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the list the call returned, if it succeeded.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosExpenseList>> {
            self.0
                .as_ref()
                .ok()
                .map(|list| Box::new(EranosExpenseList(list.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns an expense, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseOutcome(
        pub(crate) Result<Option<eranos_bindings::Expense>, crate::Failure>,
    );

    impl EranosExpenseOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the expense the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosExpense>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosExpense(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns an expense shares, which its callback
    /// receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseSharesOutcome(
        pub(crate) Result<Option<eranos_bindings::ExpenseShares>, crate::Failure>,
    );

    impl EranosExpenseSharesOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the expense shares the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosExpenseShares>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosExpenseShares(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns an answer, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosFlagOutcome(pub(crate) Result<bool, crate::Failure>);

    impl EranosFlagOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// The answer, false when the call failed.
        #[must_use]
        pub const fn value(&self) -> bool {
            matches!(self.0, Ok(true))
        }
    }

    /// The outcome of an asynchronous call that returns a list of member records, which its
    /// callback receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberListOutcome(
        pub(crate) Result<Vec<eranos_bindings::Member>, crate::Failure>,
    );

    impl EranosMemberListOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the list the call returned, if it succeeded.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosMemberList>> {
            self.0
                .as_ref()
                .ok()
                .map(|list| Box::new(EranosMemberList(list.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns a member, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberOutcome(
        pub(crate) Result<Option<eranos_bindings::Member>, crate::Failure>,
    );

    impl EranosMemberOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the member the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosMember>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosMember(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns a money, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMoneyOutcome(
        pub(crate) Result<Option<eranos_bindings::Money>, crate::Failure>,
    );

    impl EranosMoneyOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the money the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosMoney>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosMoney(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns text it may not have, which its callback
    /// receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosOptionalTextOutcome(pub(crate) Result<Option<String>, crate::Failure>);

    impl EranosOptionalTextOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// Writes the text the call returned, if it succeeded and found some; absent otherwise.
        pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.as_ref().ok().and_then(Option::as_deref))
        }
    }

    /// The outcome of an asynchronous call that returns a settlement plan, which its callback
    /// receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSettlementPlanOutcome(
        pub(crate) Result<Option<eranos_bindings::SettlementPlan>, crate::Failure>,
    );

    impl EranosSettlementPlanOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the settlement plan the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosSettlementPlan>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosSettlementPlan(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns a summary, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSummaryOutcome(
        pub(crate) Result<Option<eranos_bindings::Summary>, crate::Failure>,
    );

    impl EranosSummaryOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the summary the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosSummary>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosSummary(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns text, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTextOutcome(pub(crate) Result<String, crate::Failure>);

    impl EranosTextOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// Writes the text the call returned, if it succeeded; absent otherwise.
        pub fn value(&self, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.as_deref().ok())
        }
    }

    /// The outcome of an asynchronous call that returns a list of transfer records, which its
    /// callback receives and destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTransferListOutcome(
        pub(crate) Result<Vec<eranos_bindings::Transfer>, crate::Failure>,
    );

    impl EranosTransferListOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the list the call returned, if it succeeded.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosTransferList>> {
            self.0
                .as_ref()
                .ok()
                .map(|list| Box::new(EranosTransferList(list.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns a transfer, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosTransferOutcome(
        pub(crate) Result<Option<eranos_bindings::Transfer>, crate::Failure>,
    );

    impl EranosTransferOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }

        /// A copy of the transfer the call returned, if it succeeded and found one.
        #[must_use]
        pub fn value(&self) -> Option<Box<EranosTransfer>> {
            self.0
                .as_ref()
                .ok()
                .and_then(Option::as_ref)
                .map(|value| Box::new(EranosTransfer(value.clone())))
        }
    }

    /// The outcome of an asynchronous call that returns nothing, which its callback receives and
    /// destroys.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosUnitOutcome(pub(crate) Result<(), crate::Failure>);

    impl EranosUnitOutcome {
        /// Whether the call succeeded.
        #[must_use]
        pub const fn is_ok(&self) -> bool {
            self.0.is_ok()
        }

        /// A copy of the call's failure, absent when it succeeded.
        #[must_use]
        pub fn error(&self) -> Option<Box<EranosError>> {
            self.0
                .as_ref()
                .err()
                .map(|failure| Box::new(EranosError::from(failure.clone())))
        }
    }
}
