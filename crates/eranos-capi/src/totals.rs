//! The totals of the C and C++ APIs: the summary, the members' balances, an expense's shares, and the
//! settlements.

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
    use crate::values::ffi::EranosMoney;
    use diplomat_runtime::DiplomatWrite;

    /// How a settlement plans its payments.
    #[derive(Debug)]
    pub enum EranosSettlementStrategy {
        /// Every debt between two members, unsimplified.
        Direct,
        /// A fast plan that pairs the largest debtor with the largest creditor.
        Greedy,
        /// The fewest payments possible.
        MinimumTransfers,
    }

    /// Where a member stands: what they paid, consumed, sent, and received.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberSummary(pub(crate) eranos_bindings::MemberSummary);

    impl EranosMemberSummary {
        /// Where the member stands.
        #[must_use]
        pub fn balance(&self) -> Box<EranosBalance> {
            Box::new(EranosBalance(self.0.balance.clone()))
        }

        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// What the member is owed, less what they owe: positive when they get money back.
        #[must_use]
        pub fn net(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.net.clone()))
        }

        /// What the member paid for expenses.
        #[must_use]
        pub fn paid(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.paid.clone()))
        }

        /// What the member received in transfers.
        #[must_use]
        pub fn received(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.received.clone()))
        }

        /// What the member sent in transfers.
        #[must_use]
        pub fn sent(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.sent.clone()))
        }

        /// The member's share of the expenses: what they consumed.
        #[must_use]
        pub fn share(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.share.clone()))
        }
    }

    /// What the expenses of one category add up to.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCategoryTotal(pub(crate) eranos_bindings::CategoryTotal);

    impl EranosCategoryTotal {
        /// The category's identifier.
        pub fn category(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.category);
        }

        /// The total.
        #[must_use]
        pub fn total(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total.clone()))
        }
    }

    /// A member's share of each category.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberCategoryTotals(pub(crate) eranos_bindings::MemberCategoryTotals);

    impl EranosMemberCategoryTotals {
        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// The member's total in every category they have a share in.
        #[must_use]
        pub fn totals(&self) -> Box<EranosCategoryTotalList> {
            Box::new(EranosCategoryTotalList(self.0.totals.clone()))
        }
    }

    /// The group's totals.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSummary(pub(crate) eranos_bindings::Summary);

    impl EranosSummary {
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

        /// Where every member stands.
        #[must_use]
        pub fn members(&self) -> Box<EranosMemberSummaryList> {
            Box::new(EranosMemberSummaryList(self.0.members.clone()))
        }

        /// What the expenses add up to.
        #[must_use]
        pub fn total_spent(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total_spent.clone()))
        }
    }

    /// A member's part of an expense.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMemberShare(pub(crate) eranos_bindings::MemberShare);

    impl EranosMemberShare {
        /// The member's identifier.
        pub fn member(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.member);
        }

        /// What the member paid of it.
        #[must_use]
        pub fn paid(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.paid.clone()))
        }

        /// The member's share of it.
        #[must_use]
        pub fn share(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.share.clone()))
        }
    }

    /// An expense's shares, in the group currency.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosExpenseShares(pub(crate) eranos_bindings::ExpenseShares);

    impl EranosExpenseShares {
        /// Every member's part.
        #[must_use]
        pub fn members(&self) -> Box<EranosMemberShareList> {
            Box::new(EranosMemberShareList(self.0.members.clone()))
        }

        /// The tip.
        #[must_use]
        pub fn tip(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.tip.clone()))
        }

        /// The amount with the tip.
        #[must_use]
        pub fn total(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.total.clone()))
        }
    }

    /// A payment of a settlement.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosPayment(pub(crate) eranos_bindings::Payment);

    impl EranosPayment {
        /// What is paid.
        #[must_use]
        pub fn amount(&self) -> Box<EranosMoney> {
            Box::new(EranosMoney(self.0.amount.clone()))
        }

        /// The identifier of the member who pays.
        pub fn from(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.from);
        }

        /// The identifier of the member who is paid.
        pub fn to(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.to);
        }
    }

    /// A plan of payments that settles every balance.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosSettlement(pub(crate) eranos_bindings::Settlement);

    impl EranosSettlement {
        /// The payments.
        #[must_use]
        pub fn payments(&self) -> Box<EranosPaymentList> {
            Box::new(EranosPaymentList(self.0.payments.clone()))
        }

        /// The strategy that made the plan.
        #[must_use]
        pub fn strategy(&self) -> EranosSettlementStrategy {
            self.0.strategy.into()
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

    /// The kind of a balance.
    #[derive(Debug)]
    pub enum EranosBalanceKind {
        /// The others owe the member `amount`.
        GetsBack,
        /// The member owes the others `amount`.
        Owes,
        /// The member owes nothing and is owed nothing.
        Settled,
    }

    /// Where a member stands.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosBalance(pub(crate) eranos_bindings::Balance);

    impl EranosBalance {
        /// The others owe the member `amount`.
        #[must_use]
        pub fn create_gets_back(amount: &EranosMoney) -> Box<Self> {
            Box::new(Self(eranos_bindings::Balance::GetsBack {
                amount: amount.0.clone(),
            }))
        }

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

        /// Which kind of balance it is.
        #[must_use]
        pub const fn kind(&self) -> EranosBalanceKind {
            match &self.0 {
                eranos_bindings::Balance::GetsBack { .. } => EranosBalanceKind::GetsBack,
                eranos_bindings::Balance::Owes { .. } => EranosBalanceKind::Owes,
                eranos_bindings::Balance::Settled => EranosBalanceKind::Settled,
            }
        }

        /// What the others owe the member. Present in the kinds `GetsBack`, `Owes` alone.
        #[must_use]
        pub fn amount(&self) -> Option<Box<EranosMoney>> {
            match &self.0 {
                eranos_bindings::Balance::GetsBack { amount, .. }
                | eranos_bindings::Balance::Owes { amount, .. } => {
                    Some(Box::new(EranosMoney(amount.clone())))
                }
                eranos_bindings::Balance::Settled => None,
            }
        }

        /// Writes the balance as the library does: "gets back 12.00 EUR", "owes 12.00 EUR", or "settled".
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
}

impl From<ffi::EranosSettlementStrategy> for eranos_bindings::SettlementStrategy {
    fn from(value: ffi::EranosSettlementStrategy) -> Self {
        match value {
            ffi::EranosSettlementStrategy::Direct => Self::Direct,
            ffi::EranosSettlementStrategy::Greedy => Self::Greedy,
            ffi::EranosSettlementStrategy::MinimumTransfers => Self::MinimumTransfers,
        }
    }
}

impl From<eranos_bindings::SettlementStrategy> for ffi::EranosSettlementStrategy {
    fn from(value: eranos_bindings::SettlementStrategy) -> Self {
        match value {
            eranos_bindings::SettlementStrategy::Direct => Self::Direct,
            eranos_bindings::SettlementStrategy::Greedy => Self::Greedy,
            eranos_bindings::SettlementStrategy::MinimumTransfers => Self::MinimumTransfers,
        }
    }
}
