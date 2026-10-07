//! The glue crates share every object the core hands out between threads,
//! and move every record and error between them: a foreign language calls
//! from whichever thread it runs on. A field that is neither `Send` nor
//! `Sync` (an `Rc`, a `Cell`, a guard tied to one thread) fails the build
//! here instead of in a glue crate's generated code, far from its cause.
//! Every record to replace names its entry through one check, which would
//! otherwise let a replacement name no entry or text that is not one.

use eranos::MemberId;

use super::*;

/// A record to replace without an identifier could not name its entry, and
/// one whose identifier is not a UUID would name none.
#[test]
fn a_record_to_replace_names_its_entry_by_a_valid_identifier() {
    let id = MemberId::from_bytes([1; 16]);
    assert_eq!(identifier_to_replace(Some(&id.to_string())), Ok(id));
    assert_eq!(
        identifier_to_replace::<MemberId>(None),
        Err(EranosError::identifier_missing())
    );
    assert!(matches!(
        identifier_to_replace::<MemberId>(Some("nobody")),
        Err(EranosError::Id {
            failure: IdFailure::InvalidId,
            ..
        })
    ));
}

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
        assert_send_and_sync::<ExpenseGroupHandle>();
        assert_send_and_sync::<ExpenseGroupReader>();
        // The error, and the failure of every module.
        assert_send_and_sync::<PaymentAccountFailure>();
        assert_send_and_sync::<CalendarFailure>();
        assert_send_and_sync::<CategoryFailure>();
        assert_send_and_sync::<ContactFailure>();
        assert_send_and_sync::<EranosError>();
        assert_send_and_sync::<ExpenseFailure>();
        assert_send_and_sync::<ExpenseGroupFailure>();
        assert_send_and_sync::<IbanReason>();
        assert_send_and_sync::<IdFailure>();
        assert_send_and_sync::<LanguageFailure>();
        assert_send_and_sync::<MemberFailure>();
        assert_send_and_sync::<MoneyFailure>();
        assert_send_and_sync::<SettlementFailure>();
        assert_send_and_sync::<TransferFailure>();
        // Values.
        assert_send_and_sync::<CalendarDate>();
        assert_send_and_sync::<Cost>();
        assert_send_and_sync::<CurrencyDetails>();
        assert_send_and_sync::<CurrencyRate>();
        assert_send_and_sync::<Money>();
        assert_send_and_sync::<TimeOfDay>();
        // Entries and how they divide.
        assert_send_and_sync::<PaymentAccount>();
        assert_send_and_sync::<PaymentAccountDetails>();
        assert_send_and_sync::<Category>();
        assert_send_and_sync::<Expense>();
        assert_send_and_sync::<ExpenseSplit>();
        assert_send_and_sync::<ExpenseItem>();
        assert_send_and_sync::<Member>();
        assert_send_and_sync::<MemberAmount>();
        assert_send_and_sync::<MemberParts>();
        assert_send_and_sync::<MemberPercentage>();
        assert_send_and_sync::<NewExpenseGroup>();
        assert_send_and_sync::<Split>();
        assert_send_and_sync::<TipRule>();
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
        assert_send_and_sync::<SettlementPlan>();
        assert_send_and_sync::<SettlementStrategy>();
        assert_send_and_sync::<Summary>();
    }
}
