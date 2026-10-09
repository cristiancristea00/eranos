//! Tests for turning a ledger into a settlement. The payments are what the
//! apps ask members to send, so they must be in the group currency and
//! sorted the same way on every device, a kept plan must last exactly until
//! it is cleared, and a strategy converts through its own method.

use std::future::IntoFuture;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use eranos_ledger::Delta;
use eranos_money::CurrencyCode;

use super::*;

/// The member `number`; members sort by number.
fn member(number: u8) -> MemberId {
    MemberId::new(&format!("member-{number:03}")).unwrap()
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
    let mut ledger = Ledger::new(CurrencyCode::RON);
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
    let mut ledger = Ledger::new(CurrencyCode::EUR);
    ledger.apply(&delta).unwrap();
    ledger
}

/// Every strategy names the strategy whose plan it is (the adaptive one the
/// exact solver's, for these four balances), pays in the group currency, and
/// sorts its payments by payer and recipient.
#[test]
fn every_strategy_pays_in_the_group_currency_in_order() {
    for &strategy in SettlementStrategy::all() {
        let settlement = settle(strategy, &ledger()).unwrap();
        let computed_by = if SettlementStrategy::Adaptive == strategy {
            SettlementStrategy::MinimumTransfers
        } else {
            strategy
        };
        assert_eq!(settlement.strategy(), computed_by);
        assert_eq!(settlement.len(), 2);
        assert!(!settlement.is_empty());
        assert_eq!(
            settlement.payments(),
            [
                Payment {
                    members: MemberPair {
                        from: member(3),
                        to: member(1),
                    },
                    amount: Money::from_minor_units(500, CurrencyCode::RON),
                },
                Payment {
                    members: MemberPair {
                        from: member(4),
                        to: member(2),
                    },
                    amount: Money::from_minor_units(300, CurrencyCode::RON),
                },
            ]
        );
    }
}

/// A plan would be computed again on every request, which the exact solver
/// makes slow, or kept after the balances it came from changed.
#[test]
fn a_kept_settlement_lasts_until_it_is_cleared() {
    let mut kept = Settlements::default();
    let empty = Ledger::new(CurrencyCode::RON);
    for &strategy in SettlementStrategy::all() {
        // The first request computes the plan; later ones return it.
        let first = kept.get(strategy, &ledger()).unwrap();
        let second = kept.get(strategy, &empty).unwrap();
        assert!(std::ptr::eq(first, second));
        assert_eq!(second.len(), 2);
    }
    // Cleared, every plan is computed again.
    kept.clear();
    for &strategy in SettlementStrategy::all() {
        assert!(kept.get(strategy, &empty).unwrap().is_empty());
    }
}

/// A request dropped at a yield would keep a plan it never finished, or
/// leave the next request nothing to compute: it keeps nothing, and the
/// next request computes the plan in full. Seventeen open balances make the
/// exact solver yield once, for the adaptive strategy and for the fewest
/// payments alike.
#[test]
fn a_dropped_request_keeps_no_plan() {
    let ledger = owed_by_a_hub(16);
    let mut context = Context::from_waker(Waker::noop());
    for strategy in [
        SettlementStrategy::Adaptive,
        SettlementStrategy::MinimumTransfers,
    ] {
        let kept = Settlements::default();
        {
            let request = kept.get_async(strategy, &ledger);
            let mut request = pin!(request.into_future());
            assert!(request.as_mut().poll(&mut context).is_pending());
        }
        assert!(kept.slot(strategy).get().is_none());
        assert_eq!(kept.get(strategy, &ledger).map(SettlementPlan::len), Ok(16));
    }
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
            SettlementStrategy::Adaptive,
            SettlementStrategy::MinimumTransfers,
            SettlementStrategy::Greedy,
            SettlementStrategy::Direct
        ]
    );
}

/// A settled group has an empty settlement under every strategy.
#[test]
fn a_settled_ledger_has_no_payments() {
    for &strategy in SettlementStrategy::all() {
        assert!(
            settle(strategy, &Ledger::new(CurrencyCode::EUR))
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

/// A refusal kept in the cache would refuse every later request until a
/// balance changed, where the exact solver's table, which the allocator
/// refused once, may fit on the next request: only a plan is kept.
#[test]
fn a_refused_settlement_is_not_kept() {
    let settlements = Settlements::default();
    let ledger = owed_by_a_hub(63);
    assert_eq!(
        settlements.get(SettlementStrategy::MinimumTransfers, &ledger),
        Err(SettlementError::OutOfMemory { balances: 64 })
    );
    assert!(settlements.minimum_transfers.get().is_none());
    assert_eq!(
        settlements
            .get(SettlementStrategy::Greedy, &ledger)
            .map(SettlementPlan::len),
        Ok(63)
    );
    assert!(settlements.greedy.get().is_some());
}

/// The adaptive strategy settles a group too large for the exact solver
/// greedily, and says so, instead of failing as the fewest payments do.
#[test]
fn the_adaptive_strategy_settles_what_the_exact_solver_cannot() {
    let settlement = settle(SettlementStrategy::Adaptive, &owed_by_a_hub(63)).unwrap();
    assert_eq!(settlement.strategy(), SettlementStrategy::Greedy);
    assert_eq!(settlement.len(), 63);
}

/// An adaptive plan is kept under the adaptive strategy, named by the
/// strategy that computed it, without filling that strategy's own slot.
#[test]
fn a_kept_adaptive_plan_names_the_strategy_that_computed_it() {
    let kept = Settlements::default();
    let plan = kept.get(SettlementStrategy::Adaptive, &ledger()).unwrap();
    assert_eq!(plan.strategy(), SettlementStrategy::MinimumTransfers);
    assert!(std::ptr::eq(
        plan,
        kept.get(SettlementStrategy::Adaptive, &ledger()).unwrap()
    ));
    assert!(kept.minimum_transfers.get().is_none());
}

/// A strategy as a crate outside this one converts it: to its name.
struct StrategyName(&'static str);

impl FromSettlementStrategy for StrategyName {
    fn adaptive() -> Self {
        Self("adaptive")
    }

    fn minimum_transfers() -> Self {
        Self("minimum transfers")
    }

    fn greedy() -> Self {
        Self("greedy")
    }

    fn direct() -> Self {
        Self("direct")
    }
}

/// A conversion that called another variant's method would show the
/// bindings a plan made by another strategy than the one it names.
#[test]
fn a_strategy_converts_through_the_method_of_its_variant() {
    let names: Vec<&str> = SettlementStrategy::all()
        .iter()
        .map(|&strategy| StrategyName::from_settlement_strategy(strategy).0)
        .collect();
    assert_eq!(names, ["adaptive", "minimum transfers", "greedy", "direct"]);
}
