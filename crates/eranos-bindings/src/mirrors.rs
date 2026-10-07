//! The list of every mirrored record, enum, and error, from which the glue
//! crates stamp their own copies.

/// Hands the mirror of every record, enum, and error the bindings expose to
/// the macro whose path it is given, one call each.
///
/// A glue crate writes `eranos_bindings::mirrors!(path::to::callback);`
/// where the types are in scope, and its callback stamps a copy of each.
/// Each mirror comes from `#[mirrored]` on its item's definition; a type the
/// list leaves out has no copy, so a glue crate's signature that names it
/// fails to build.
#[macro_export]
macro_rules! mirrors {
    ($($callback:tt)+) => {
        $crate::mirror_payment_account!($($callback)+);
        $crate::mirror_payment_account_details!($($callback)+);
        $crate::mirror_payment_account_failure!($($callback)+);
        $crate::mirror_balance!($($callback)+);
        $crate::mirror_calendar_date!($($callback)+);
        $crate::mirror_calendar_failure!($($callback)+);
        $crate::mirror_category!($($callback)+);
        $crate::mirror_category_failure!($($callback)+);
        $crate::mirror_category_total!($($callback)+);
        $crate::mirror_contact_failure!($($callback)+);
        $crate::mirror_cost!($($callback)+);
        $crate::mirror_currency_details!($($callback)+);
        $crate::mirror_currency_rate!($($callback)+);
        $crate::mirror_eranos_error!($($callback)+);
        $crate::mirror_expense!($($callback)+);
        $crate::mirror_expense_failure!($($callback)+);
        $crate::mirror_expense_shares!($($callback)+);
        $crate::mirror_expense_split!($($callback)+);
        $crate::mirror_expense_group_failure!($($callback)+);
        $crate::mirror_iban_reason!($($callback)+);
        $crate::mirror_id_failure!($($callback)+);
        $crate::mirror_item!($($callback)+);
        $crate::mirror_language_failure!($($callback)+);
        $crate::mirror_member!($($callback)+);
        $crate::mirror_member_amount!($($callback)+);
        $crate::mirror_member_category_totals!($($callback)+);
        $crate::mirror_member_failure!($($callback)+);
        $crate::mirror_member_parts!($($callback)+);
        $crate::mirror_member_percentage!($($callback)+);
        $crate::mirror_member_share!($($callback)+);
        $crate::mirror_member_summary!($($callback)+);
        $crate::mirror_money!($($callback)+);
        $crate::mirror_money_failure!($($callback)+);
        $crate::mirror_new_expense_group!($($callback)+);
        $crate::mirror_payment!($($callback)+);
        $crate::mirror_settlement_plan!($($callback)+);
        $crate::mirror_settlement_failure!($($callback)+);
        $crate::mirror_settlement_strategy!($($callback)+);
        $crate::mirror_split!($($callback)+);
        $crate::mirror_summary!($($callback)+);
        $crate::mirror_time_of_day!($($callback)+);
        $crate::mirror_tip!($($callback)+);
        $crate::mirror_tip_distribution!($($callback)+);
        $crate::mirror_tip_value!($($callback)+);
        $crate::mirror_transfer!($($callback)+);
        $crate::mirror_transfer_failure!($($callback)+);
    };
}
