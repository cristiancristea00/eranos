//! Tests for turning a ledger into a settlement. The payments are what the
//! apps ask members to send, so they must be in the group currency and
//! sorted the same way on every device, a kept plan must last exactly until
//! it is cleared, and a strategy converts through its own method.

use std::future::IntoFuture;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use eranos_ledger::Delta;
use eranos_money::Currency;

use super::*;

fn member(number: u8) -> MemberId {
    MemberId::from_bytes([number; 16])
}

/// A ledger where members 3 and 4 owe members 1 and 2 through transfers
/// received and sent.
fn ledger() -> Ledger {
    let mut delta = Delta::default();
    // A transfer from 1 to 3 leaves 3 owing 1; the same for 2 and 4.
    delta.add_transfer(
        MemberPair {
            from: member(1),
            to: member(3),
        },
        500,
        1,
    );
    delta.add_transfer(
        MemberPair {
            from: member(2),
            to: member(4),
        },
        300,
        1,
    );
    let mut ledger = Ledger::new(Currency::RON);
    ledger.apply(&delta).unwrap();
    ledger
}

/// A ledger where members 1 to `creditors` each sent member 100 a different
/// amount: `creditors + 1` open balances and no exact opposites among them.
fn owed_by_a_hub(creditors: u8) -> Ledger {
    let hub = member(100);
    let mut delta = Delta::default();
    for number in 1..=creditors {
        delta.add_transfer(
            MemberPair {
                from: member(number),
                to: hub,
            },
            i64::from(number),
            1,
        );
    }
    let mut ledger = Ledger::new(Currency::EUR);
    ledger.apply(&delta).unwrap();
    ledger
}

/// Every strategy records itself, pays in the group currency, and sorts its
/// payments by payer and recipient.
#[test]
fn every_strategy_pays_in_the_group_currency_in_order() {
    for strategy in SettlementStrategy::all() {
        let settlement = settle(strategy, &ledger()).unwrap();
        assert_eq!(settlement.strategy(), strategy);
        assert_eq!(settlement.len(), 2);
        assert!(!settlement.is_empty());
        let payments: Vec<(MemberId, MemberId, Money)> = settlement
            .payments()
            .iter()
            .map(|payment| (payment.from(), payment.to(), payment.amount()))
            .collect();
        assert_eq!(
            payments,
            vec![
                (
                    member(3),
                    member(1),
                    Money::from_minor_units(Currency::RON, 500)
                ),
                (
                    member(4),
                    member(2),
                    Money::from_minor_units(Currency::RON, 300)
                ),
            ]
        );
    }
}

/// A plan would be computed again on every request, which the exact solver
/// makes slow, or kept after the balances it came from changed.
#[test]
fn a_kept_settlement_lasts_until_it_is_cleared() {
    let mut kept = Settlements::default();
    let empty = Ledger::new(Currency::RON);
    for strategy in SettlementStrategy::all() {
        // The first request computes the plan; later ones return it.
        let first = kept.get(strategy, &ledger()).unwrap();
        let second = kept.get(strategy, &empty).unwrap();
        assert!(std::ptr::eq(first, second));
        assert_eq!(second.len(), 2);
    }
    // Cleared, every plan is computed again.
    kept.clear();
    for strategy in SettlementStrategy::all() {
        assert!(kept.get(strategy, &empty).unwrap().is_empty());
    }
}

/// A request dropped at a yield would keep a plan it never finished, or
/// leave the next request nothing to compute: it keeps nothing, and the
/// next request computes the plan in full. Seventeen open balances make the
/// exact solver yield once.
#[test]
fn a_dropped_request_keeps_no_plan() {
    let kept = Settlements::default();
    let ledger = owed_by_a_hub(16);
    let mut context = Context::from_waker(Waker::noop());
    {
        let request = kept.get_async(SettlementStrategy::MinimumTransfers, &ledger);
        let mut request = pin!(request.into_future());
        assert!(request.as_mut().poll(&mut context).is_pending());
    }
    assert!(kept.minimum_transfers.get().is_none());
    assert_eq!(
        kept.get(SettlementStrategy::MinimumTransfers, &ledger)
            .map(Settlement::len),
        Ok(16)
    );
}

/// Two requests computing at once would each keep a plan of their own: the
/// first to complete keeps its plan, and the other returns that one.
#[test]
fn requests_computing_at_once_keep_the_first_plan() {
    let kept = Settlements::default();
    let ledger = owed_by_a_hub(16);
    let mut context = Context::from_waker(Waker::noop());
    let first = kept.get_async(SettlementStrategy::MinimumTransfers, &ledger);
    let second = kept.get_async(SettlementStrategy::MinimumTransfers, &ledger);
    let mut first = pin!(first.into_future());
    let mut second = pin!(second.into_future());
    assert!(first.as_mut().poll(&mut context).is_pending());
    assert!(second.as_mut().poll(&mut context).is_pending());
    let Poll::Ready(Ok(winner)) = first.as_mut().poll(&mut context) else {
        panic!("the first request did not complete after its one yield");
    };
    let Poll::Ready(Ok(other)) = second.as_mut().poll(&mut context) else {
        panic!("the second request did not complete after its one yield");
    };
    assert!(std::ptr::eq(winner, other));
}

/// An app listing the strategies from `all` would show them in another
/// order than their documentation.
#[test]
fn all_lists_every_strategy_in_declaration_order() {
    assert_eq!(
        SettlementStrategy::all(),
        [
            SettlementStrategy::Direct,
            SettlementStrategy::Greedy,
            SettlementStrategy::MinimumTransfers
        ]
    );
}

/// A settled group has an empty settlement under every strategy.
#[test]
fn a_settled_ledger_has_no_payments() {
    for strategy in SettlementStrategy::all() {
        assert!(
            settle(strategy, &Ledger::new(Currency::EUR))
                .unwrap()
                .is_empty()
        );
    }
}

/// The exact solver's refusal reaches the caller, and the other strategies
/// still settle the same balances.
#[test]
fn an_exact_plan_too_large_to_compute_is_an_error() {
    let ledger = owed_by_a_hub(63);
    assert_eq!(
        settle(SettlementStrategy::MinimumTransfers, &ledger),
        Err(SettlementError::OutOfMemory { balances: 64 })
    );
    for strategy in [SettlementStrategy::Greedy, SettlementStrategy::Direct] {
        assert_eq!(settle(strategy, &ledger).unwrap().len(), 63);
    }
}

/// A strategy as a crate outside this one converts it: to its name.
struct StrategyName(&'static str);

impl FromSettlementStrategy for StrategyName {
    fn direct() -> Self {
        Self("direct")
    }

    fn greedy() -> Self {
        Self("greedy")
    }

    fn minimum_transfers() -> Self {
        Self("minimum transfers")
    }
}

/// A conversion that called another variant's method would show the
/// bindings a plan made by another strategy than the one it names.
#[test]
fn a_strategy_converts_through_the_method_of_its_variant() {
    let names: Vec<&str> = SettlementStrategy::all()
        .into_iter()
        .map(|strategy| StrategyName::from_settlement_strategy(strategy).0)
        .collect();
    assert_eq!(names, ["direct", "greedy", "minimum transfers"]);
}
