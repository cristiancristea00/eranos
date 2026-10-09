//! Pairwise debts, netted but not simplified.

use std::collections::BTreeMap;

use eranos_ledger::MemberPair;
use eranos_money::{CurrencyCode, Money};

use super::Payment;

/// One payment in `currency` per pair of members with a netted debt,
/// ordered by payer and then by recipient; `pairs` maps every pair, keyed
/// with the lower member as `from`, to what `from` owes `to`.
pub(super) fn settle(pairs: &BTreeMap<MemberPair, i64>, currency: CurrencyCode) -> Vec<Payment> {
    let mut payments: Vec<Payment> = pairs
        .iter()
        .filter_map(|(&members, &amount)| match amount.signum() {
            1 => Some(Payment {
                members,
                amount: Money::from_minor_units(amount, currency),
            }),
            // The ledger keeps every value above i64::MIN, so negating fits.
            -1 => Some(Payment {
                members: members.reversed(),
                amount: Money::from_minor_units(-amount, currency),
            }),
            _ => None,
        })
        .collect();
    payments.sort_unstable_by_key(|payment| payment.members);
    payments
}

#[cfg(test)]
mod tests;
