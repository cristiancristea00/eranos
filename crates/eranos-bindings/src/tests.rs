//! The glue crates share every object the core hands out between threads,
//! and move every record and error between them: a foreign language calls
//! from whichever thread it runs on. A field that is neither `Send` nor
//! `Sync` (an `Rc`, a `Cell`, a guard tied to one thread) fails the build
//! here instead of in a glue crate's generated code, far from its cause.

use super::*;

/// Compiles only when `T` can move to another thread and be shared between
/// threads.
const fn assert_send_and_sync<T: Send + Sync>() {}

/// A glue crate could not export an object, or hand a record or an error to
/// another thread, if a public type lost `Send` or `Sync`.
#[test]
fn every_public_type_is_send_and_sync() {
    const {
        // The objects.
        assert_send_and_sync::<Cancellation>();
        assert_send_and_sync::<GroupHandle>();
        assert_send_and_sync::<GroupReader>();
        // The error.
        assert_send_and_sync::<EranosError>();
        assert_send_and_sync::<IbanReason>();
        // Values.
        assert_send_and_sync::<CalendarDate>();
        assert_send_and_sync::<Cost>();
        assert_send_and_sync::<CurrencyDetails>();
        assert_send_and_sync::<CurrencyRate>();
        assert_send_and_sync::<Money>();
        assert_send_and_sync::<TimeOfDay>();
        // Entries and how they divide.
        assert_send_and_sync::<Account>();
        assert_send_and_sync::<AccountDetails>();
        assert_send_and_sync::<Category>();
        assert_send_and_sync::<Expense>();
        assert_send_and_sync::<ExpenseSplit>();
        assert_send_and_sync::<Item>();
        assert_send_and_sync::<Member>();
        assert_send_and_sync::<MemberAmount>();
        assert_send_and_sync::<MemberParts>();
        assert_send_and_sync::<MemberPercentage>();
        assert_send_and_sync::<NewGroup>();
        assert_send_and_sync::<Split>();
        assert_send_and_sync::<Tip>();
        assert_send_and_sync::<TipDistribution>();
        assert_send_and_sync::<TipValue>();
        assert_send_and_sync::<Transfer>();
        // Totals and settlements.
        assert_send_and_sync::<Balance>();
        assert_send_and_sync::<CategoryTotal>();
        assert_send_and_sync::<ExpenseShares>();
        assert_send_and_sync::<MemberCategoryTotals>();
        assert_send_and_sync::<MemberShare>();
        assert_send_and_sync::<MemberSummary>();
        assert_send_and_sync::<Payment>();
        assert_send_and_sync::<Settlement>();
        assert_send_and_sync::<SettlementStrategy>();
        assert_send_and_sync::<Summary>();
    }
}
