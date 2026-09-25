//! Resolving expenses into group-currency shares.
//!
//! Every figure a user entered is in the expense's own currency and is
//! divided there first; a foreign expense's total is then converted once,
//! and its shares are re-divided from the converted total, so both sides
//! still add up exactly. Each division rotates its ties by the expense ID
//! and a pass number, so the same expense resolves identically on every
//! device.

use std::collections::BTreeMap;

use eranos_core::MemberId;
use eranos_ledger::{Debt, MemberPair, MemberShare, ResolvedExpense};
use eranos_money::allocation::{allocate, rotation};
use eranos_money::{Currency, ExchangeRate, Money, MoneyError};

use crate::{Expense, ExpenseSplit, TipDistribution};

/// Allocation passes; each gets its own tie rotation.
const PASS_SPLIT: u64 = 0;
const PASS_TIP: u64 = 1;
const PASS_PAYERS: u64 = 2;
const PASS_CONVERTED_SHARES: u64 = 3;
const PASS_CONVERTED_PAYERS: u64 = 4;
const PASS_CONVERTED_TIP: u64 = 5;
/// Plus the item's position.
const PASS_ITEMS: u64 = 1 << 32;
/// Plus the payer's position.
const PASS_DEBTS: u64 = 2 << 32;

/// Divides one expense among its members in the group currency.
///
/// The group has checked the kind of the expense's cost against
/// `currency` (`Group::check_cost`, or the rates of a currency change).
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_expense(
    expense: &Expense,
    currency: Currency,
) -> Result<ResolvedExpense, MoneyError> {
    let seed = expense.id().to_bytes();
    let pass = |number: u64| rotation(seed, number);
    let amount = expense.amount().minor_units();
    let tip = expense.tip_amount().minor_units();
    let total = expense.total().minor_units();

    let base = match expense.split() {
        ExpenseSplit::Itemised(items) => {
            let mut totals: BTreeMap<MemberId, i64> = BTreeMap::new();
            for (position, item) in (0_u64..).zip(items) {
                let line = item
                    .split()
                    .divide(item.line_total().minor_units(), pass(PASS_ITEMS + position));
                for (member, share) in line {
                    // The line totals add up to the amount, so no sum overflows.
                    *totals.entry(member).or_default() += share;
                }
            }
            totals.into_iter().collect()
        }
        ExpenseSplit::Whole(split) => split.divide(amount, pass(PASS_SPLIT)),
    };

    let mut shares = base.clone();
    if 0 < tip {
        let weights: Vec<u64> = match expense.tip().distribution() {
            TipDistribution::Equal => vec![1; base.len()],
            TipDistribution::Proportional => base
                .iter()
                .map(|&(_, share)| share.unsigned_abs())
                .collect(),
        };
        let tips = allocate(tip, &weights, pass(PASS_TIP));
        for ((_, share), portion) in shares.iter_mut().zip(tips) {
            // Shares and tips add up to the total, so no sum overflows.
            *share += portion;
        }
    }
    let paid = expense.payers().divide(total, pass(PASS_PAYERS));

    let (total, tip, shares, paid) =
        match convert(expense.total(), expense.cost().conversion(), currency)? {
            None => (total, tip, shares, paid),
            Some(converted) => {
                let reallocate = |parts: &[(MemberId, i64)], number: u64| {
                    let weights: Vec<u64> =
                        parts.iter().map(|&(_, part)| part.unsigned_abs()).collect();
                    parts
                        .iter()
                        .map(|&(member, _)| member)
                        .zip(allocate(converted, &weights, pass(number)))
                        .collect::<Vec<_>>()
                };
                // The converted total split back into amount and tip; the
                // tip is the second of the two parts.
                let tip_weights = [amount.unsigned_abs(), tip.unsigned_abs()];
                let converted_tip = allocate(converted, &tip_weights, pass(PASS_CONVERTED_TIP))
                    .last()
                    .copied()
                    .unwrap_or_default();
                (
                    converted,
                    converted_tip,
                    reallocate(&shares, PASS_CONVERTED_SHARES),
                    reallocate(&paid, PASS_CONVERTED_PAYERS),
                )
            }
        };

    let debts = debts(&shares, &paid, |position| pass(PASS_DEBTS + position));
    let mut members: BTreeMap<MemberId, (i64, i64)> = BTreeMap::new();
    for &(member, payment) in &paid {
        members.entry(member).or_default().0 = payment;
    }
    for &(member, consumed) in &shares {
        members.entry(member).or_default().1 = consumed;
    }
    let money = |minor_units| Money::from_minor_units(currency, minor_units);
    let members = members
        .into_iter()
        .map(|(member, (payment, consumed))| {
            MemberShare::new(member, money(payment), money(consumed))
        })
        .collect();
    Ok(ResolvedExpense::new(
        debts,
        members,
        money(tip),
        money(total),
    ))
}

/// Converts `amount` at `rate` into `currency`: `None` for an amount without
/// a rate, which the group's checks have kept to amounts in `currency`.
fn convert(
    amount: Money,
    rate: Option<ExchangeRate>,
    currency: Currency,
) -> Result<Option<i64>, MoneyError> {
    rate.map(|rate| Ok(rate.convert(amount, currency)?.minor_units()))
        .transpose()
}

/// What every participant owes every payer.
///
/// Each payer's amount is divided among the participants in proportion to
/// what they have left to cover, payer by payer in member order. Every
/// participant's debts then add up to their share and every payer's credits
/// to what they paid, exactly: the last payer receives exactly what is left.
fn debts(
    shares: &[(MemberId, i64)],
    paid: &[(MemberId, i64)],
    rotation: impl Fn(u64) -> u64,
) -> Vec<Debt> {
    let mut remaining: Vec<i64> = shares.iter().map(|&(_, share)| share).collect();
    let mut debts = Vec::new();
    for (position, &(creditor, amount)) in (0_u64..).zip(paid) {
        if 0 == amount {
            continue;
        }
        let weights: Vec<u64> = remaining.iter().map(|share| share.unsigned_abs()).collect();
        let portions = allocate(amount, &weights, rotation(position));
        for ((&(debtor, _), left), portion) in shares.iter().zip(&mut remaining).zip(portions) {
            // A portion never exceeds what is left, so `left` stays ≥ 0.
            *left -= portion;
            if debtor != creditor && 0 < portion {
                debts.push(Debt {
                    amount: portion,
                    members: MemberPair {
                        from: debtor,
                        to: creditor,
                    },
                });
            }
        }
    }
    debts
}

#[cfg(test)]
mod tests;
