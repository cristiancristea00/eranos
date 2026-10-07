internal import EranosGenerated

// The group's totals and its settlements.
public extension ExpenseGroup {
    /// The group's totals.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    func summary() throws(EranosError) -> Summary {
        try converting { Summary(try handle.summary()) }
    }

    /// The group's totals.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func summary() async throws(EranosError) -> Summary {
        try await converting { [handle] cancellation in
            Summary(try await handle.summaryAsync(cancellation: cancellation))
        }
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// - Throws: ``EranosError/settlement(_:detail:)`` with
    ///   ``SettlementFailure/outOfMemory(balances:)`` for an exact plan of more open balances than
    ///   the table fits; on wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while
    ///   another call holds the lock.
    func settlement(by strategy: SettlementStrategy) throws(EranosError) -> SettlementPlan {
        try converting { SettlementPlan(try handle.settlement(strategy: strategy.generated)) }
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/settlement(_:detail:)`` with
    ///   ``SettlementFailure/outOfMemory(balances:)`` for an exact plan of more open balances than
    ///   the table fits; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func settlement(by strategy: SettlementStrategy) async throws(EranosError) -> SettlementPlan {
        try await converting { [handle] cancellation in
            SettlementPlan(try await handle.settlementAsync(strategy: strategy.generated, cancellation: cancellation))
        }
    }
}
