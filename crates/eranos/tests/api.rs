//! Apps move groups and their values between threads: a background thread
//! saves the group the interface thread edits, and the bindings call the
//! library from whichever thread their language runs on. These tests keep
//! every public type `Send` and `Sync`, so a field that is neither (an `Rc`,
//! a `Cell`, a raw pointer) fails the build here instead of in an app.
#![forbid(unsafe_code)]

/// Compiles only when `T` can move to another thread and be shared between
/// threads.
const fn assert_send_and_sync<T: Send + Sync>() {}

/// An app could not hand a group, an entry, or an error to another thread
/// if a public type lost `Send` or `Sync`.
#[test]
fn every_public_type_is_send_and_sync() {
    const {
        // Calendar values and categories.
        assert_send_and_sync::<eranos::calendar::CalendarDate>();
        assert_send_and_sync::<eranos::calendar::TimeOfDay>();
        assert_send_and_sync::<eranos::category::Category>();
        // Cooperative futures, around a future that is itself both.
        assert_send_and_sync::<eranos::cooperative::Cooperative<std::future::Ready<()>>>();
        // Errors.
        assert_send_and_sync::<eranos::error::PaymentAccountError>();
        assert_send_and_sync::<eranos::error::CalendarError>();
        assert_send_and_sync::<eranos::error::CategoryError>();
        assert_send_and_sync::<eranos::error::ContactError>();
        assert_send_and_sync::<eranos::error::ExpenseError>();
        assert_send_and_sync::<eranos::error::ExpenseGroupError>();
        assert_send_and_sync::<eranos::error::IbanError>();
        assert_send_and_sync::<eranos::error::IdError>();
        assert_send_and_sync::<eranos::error::LanguageError>();
        assert_send_and_sync::<eranos::error::MemberError>();
        assert_send_and_sync::<eranos::error::MoneyError>();
        assert_send_and_sync::<eranos::error::SettlementError>();
        assert_send_and_sync::<eranos::error::TransferError>();
        // Expenses and how they divide.
        assert_send_and_sync::<eranos::expense::Expense>();
        assert_send_and_sync::<eranos::expense::ExpenseBuilder>();
        assert_send_and_sync::<eranos::expense::ExpenseSplit>();
        assert_send_and_sync::<eranos::expense::Item>();
        assert_send_and_sync::<eranos::expense::ItemBuilder>();
        assert_send_and_sync::<eranos::expense::MemberAmount>();
        assert_send_and_sync::<eranos::expense::MemberParts>();
        assert_send_and_sync::<eranos::expense::MemberPercentage>();
        assert_send_and_sync::<eranos::expense::Split>();
        assert_send_and_sync::<eranos::expense::Tip>();
        assert_send_and_sync::<eranos::expense::TipDistribution>();
        assert_send_and_sync::<eranos::expense::TipValue>();
        // Groups, and how they are built and loaded.
        assert_send_and_sync::<eranos::expense_group::ExpenseGroup>();
        assert_send_and_sync::<eranos::expense_group::ExpenseGroupBuilder>();
        assert_send_and_sync::<eranos::expense_group::Language>();
        #[cfg(feature = "serde")]
        assert_send_and_sync::<eranos::expense_group::SavedExpenseGroup>();
        assert_send_and_sync::<eranos::expense_group::ExpenseGroupReadGuard>();
        assert_send_and_sync::<eranos::expense_group::ExpenseGroupWriteGuard>();
        assert_send_and_sync::<eranos::expense_group::SharedExpenseGroup>();
        // Identifiers.
        assert_send_and_sync::<eranos::id::CategoryId>();
        assert_send_and_sync::<eranos::id::ExpenseId>();
        assert_send_and_sync::<eranos::id::ExpenseGroupId>();
        assert_send_and_sync::<eranos::id::MemberId>();
        assert_send_and_sync::<eranos::id::TransferId>();
        // Totals.
        assert_send_and_sync::<eranos::ledger::Balance>();
        assert_send_and_sync::<eranos::ledger::CategoryTotal>();
        assert_send_and_sync::<eranos::ledger::ExpenseShares>();
        assert_send_and_sync::<eranos::ledger::MemberShare>();
        assert_send_and_sync::<eranos::ledger::MemberSummary>();
        assert_send_and_sync::<eranos::ledger::Summary>();
        // Members, their contact details, and their accounts.
        assert_send_and_sync::<eranos::member::PaymentAccount>();
        assert_send_and_sync::<eranos::member::PaymentAccountDetails>();
        assert_send_and_sync::<eranos::member::PaymentAccountKind>();
        assert_send_and_sync::<eranos::member::Email>();
        assert_send_and_sync::<eranos::member::Iban>();
        assert_send_and_sync::<eranos::member::Member>();
        assert_send_and_sync::<eranos::member::MemberBuilder>();
        assert_send_and_sync::<eranos::member::PhoneNumber>();
        assert_send_and_sync::<eranos::member::Username>();
        // Money.
        assert_send_and_sync::<eranos::money::Cost>();
        assert_send_and_sync::<eranos::money::Currency>();
        assert_send_and_sync::<eranos::money::CurrencyRate>();
        assert_send_and_sync::<eranos::money::ExchangeRate>();
        assert_send_and_sync::<eranos::money::Money>();
        assert_send_and_sync::<eranos::money::Parts>();
        assert_send_and_sync::<eranos::money::Percentage>();
        // Settlements and transfers.
        assert_send_and_sync::<eranos::settlement::Payment>();
        assert_send_and_sync::<eranos::settlement::SettlementPlan>();
        assert_send_and_sync::<eranos::settlement::SettlementStrategy>();
        assert_send_and_sync::<eranos::transfer::Transfer>();
        assert_send_and_sync::<eranos::transfer::TransferBuilder>();
    }
}
