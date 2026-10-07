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

use crate::split::Share;
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
/// `currency` (`ExpenseGroup::check_cost`, or the rates of a currency change).
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_expense(
    expense: &Expense,
    currency: Currency,
) -> Result<ResolvedExpense, MoneyError> {
    resolve_expense_at(expense, expense.cost().conversion(), currency)
}

/// Divides one expense as [`resolve_expense`] does, at the rate `conversion`.
///
/// The rate replaces the expense's own, and `None` makes it a local cost:
/// a currency change resolves each expense at its new rate before it gives
/// the expense that rate.
///
/// The caller has checked the kind of cost that `conversion` makes against
/// `currency`, as a currency change does with its rates.
///
/// # Errors
///
/// [`MoneyError::Overflow`] when the conversion leaves the range of an
/// amount.
pub fn resolve_expense_at(
    expense: &Expense,
    conversion: Option<ExchangeRate>,
    currency: Currency,
) -> Result<ResolvedExpense, MoneyError> {
    let seed = expense.id().to_bytes();
    let pass = |number: u64| rotation(seed, number);
    let amount = expense.amount().minor_units();
    let tip = expense.tip_amount().minor_units();
    let total = expense.total().minor_units();

    let base = match expense.split() {
        ExpenseSplit::Whole(split) => split.divide(amount, pass(PASS_SPLIT)),
        ExpenseSplit::Itemised(items) => {
            let mut totals: BTreeMap<MemberId, i64> = BTreeMap::new();
            for (position, item) in (0_u64..).zip(items) {
                let line = item
                    .split()
                    .divide(item.line_total().minor_units(), pass(PASS_ITEMS + position));
                for share in line {
                    // The line totals add up to the amount, so no sum overflows.
                    *totals.entry(share.member).or_default() += share.amount;
                }
            }
            totals
                .into_iter()
                .map(|(member, summed)| Share {
                    member,
                    amount: summed,
                })
                .collect()
        }
    };

    let mut shares = base.clone();
    if 0 < tip {
        let weights: Vec<u64> = match expense.tip().distribution() {
            TipDistribution::Proportional => base
                .iter()
                .map(|share| share.amount.unsigned_abs())
                .collect(),
            TipDistribution::Equal => vec![1; base.len()],
        };
        let tips = allocate(tip, &weights, pass(PASS_TIP));
        for (share, portion) in shares.iter_mut().zip(tips) {
            // Shares and tips add up to the total, so no sum overflows.
            share.amount += portion;
        }
    }
    let paid = expense.payers().divide(total, pass(PASS_PAYERS));

    let (total, tip, shares, paid) = match convert(expense.total(), conversion, currency)? {
        None => (total, tip, shares, paid),
        Some(converted) => {
            let reallocate = |parts: &[Share], number: u64| {
                let weights: Vec<u64> = parts
                    .iter()
                    .map(|part| part.amount.unsigned_abs())
                    .collect();
                parts
                    .iter()
                    .zip(allocate(converted, &weights, pass(number)))
                    .map(|(part, reallocated)| Share {
                        member: part.member,
                        amount: reallocated,
                    })
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
    let mut members: BTreeMap<MemberId, PaidAndConsumed> = BTreeMap::new();
    for payment in &paid {
        members.entry(payment.member).or_default().paid = payment.amount;
    }
    for share in &shares {
        members.entry(share.member).or_default().consumed = share.amount;
    }
    let money = |minor_units| Money::from_minor_units(minor_units, currency);
    let members = members
        .into_iter()
        .map(|(member, totals)| {
            MemberShare::new(member, money(totals.paid), money(totals.consumed))
        })
        .collect();
    Ok(ResolvedExpense::new(
        members,
        money(total),
        money(tip),
        debts,
    ))
}

/// What one member paid and consumed of an expense, in minor units.
#[derive(Debug, Clone, Copy, Default)]
struct PaidAndConsumed {
    paid: i64,
    consumed: i64,
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
fn debts(shares: &[Share], paid: &[Share], rotation: impl Fn(u64) -> u64) -> Vec<Debt> {
    let mut remaining: Vec<i64> = shares.iter().map(|share| share.amount).collect();
    let mut debts = Vec::new();
    for (position, payment) in (0_u64..).zip(paid) {
        if 0 == payment.amount {
            continue;
        }
        let weights: Vec<u64> = remaining.iter().map(|left| left.unsigned_abs()).collect();
        let portions = allocate(payment.amount, &weights, rotation(position));
        for ((share, left), portion) in shares.iter().zip(&mut remaining).zip(portions) {
            // A portion never exceeds what is left, so `left` stays ≥ 0.
            *left -= portion;
            if share.member != payment.member && 0 < portion {
                debts.push(Debt {
                    members: MemberPair {
                        from: share.member,
                        to: payment.member,
                    },
                    amount: portion,
                });
            }
        }
    }
    debts
}

#[cfg(test)]
mod tests;
