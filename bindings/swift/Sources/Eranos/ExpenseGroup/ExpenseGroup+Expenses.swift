internal import EranosGenerated

// The group's expenses.
public extension ExpenseGroup {
    /// Every expense, in the order they were created.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    func expenses() throws(EranosError) -> [Expense] {
        try converting { try handle.expenses().map(Expense.init) }
    }

    /// Every expense, in the order they were created.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func expenses() async throws(EranosError) -> [Expense] {
        try await converting { [handle] cancellation in
            try await handle.expensesAsync(cancellation: cancellation).map(Expense.init)
        }
    }

    /// The expense with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func expense(withID id: Expense.ID) throws(EranosError) -> Expense? {
        try converting { try handle.expense(id: id.rawValue).map(Expense.init) }
    }

    /// The expense with the identifier `id`, if the group holds it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func expense(withID id: Expense.ID) async throws(EranosError) -> Expense? {
        try await converting { [handle] cancellation in
            try await handle.expenseAsync(id: id.rawValue, cancellation: cancellation).map(Expense.init)
        }
    }

    /// How the expense with the identifier `id` divides, in the group currency, if the group holds
    /// it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func shares(of id: Expense.ID) throws(EranosError) -> ExpenseShares? {
        try converting { try handle.expenseShares(id: id.rawValue).map(ExpenseShares.init) }
    }

    /// How the expense with the identifier `id` divides, in the group currency, if the group holds
    /// it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func shares(of id: Expense.ID) async throws(EranosError) -> ExpenseShares? {
        try await converting { [handle] cancellation in
            try await handle.expenseSharesAsync(id: id.rawValue, cancellation: cancellation).map(ExpenseShares.init)
        }
    }

    /// Adds `expense`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for an expense with an identifier,
    ///   ``EranosError/expense(_:detail:)`` for one that does not build, and
    ///   ``EranosError/expenseGroup(_:detail:)`` for one that does not fit the group, such as
    ///   ``ExpenseGroupFailure/unknownMember(_:)`` or ``ExpenseGroupFailure/archivedMember(_:)``; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    @discardableResult
    func add(_ expense: Expense) throws(EranosError) -> Expense.ID {
        try converting { Expense.ID(rawValue: try handle.addExpense(expense: expense.generated)) }
    }

    /// Adds `expense`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for an expense with an identifier,
    ///   ``EranosError/expense(_:detail:)`` for one that does not build, and
    ///   ``EranosError/expenseGroup(_:detail:)`` for one that does not fit the group, such as
    ///   ``ExpenseGroupFailure/unknownMember(_:)`` or ``ExpenseGroupFailure/archivedMember(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func add(_ expense: Expense) async throws(EranosError) -> Expense.ID {
        try await converting { [handle] cancellation in
            Expense.ID(rawValue: try await handle.addExpenseAsync(expense: expense.generated, cancellation: cancellation))
        }
    }

    /// Replaces the expense that `expense` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for an expense without an identifier,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownExpense(_:)``, and the failures
    ///   of adding one; on wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while
    ///   another call holds the lock.
    @discardableResult
    func replace(_ expense: Expense) throws(EranosError) -> Expense {
        try converting { Expense(try handle.replaceExpense(expense: expense.generated)) }
    }

    /// Replaces the expense that `expense` names by its identifier, and returns it as it was.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for an expense without an identifier,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownExpense(_:)``, and the failures
    ///   of adding one; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func replace(_ expense: Expense) async throws(EranosError) -> Expense {
        try await converting { [handle] cancellation in
            Expense(try await handle.replaceExpenseAsync(expense: expense.generated, cancellation: cancellation))
        }
    }

    /// Removes the expense with the identifier `id` and returns it.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownExpense(_:)``; on
    ///   wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds
    ///   the lock.
    @discardableResult
    func remove(_ id: Expense.ID) throws(EranosError) -> Expense {
        try converting { Expense(try handle.removeExpense(id: id.rawValue)) }
    }

    /// Removes the expense with the identifier `id` and returns it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownExpense(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func remove(_ id: Expense.ID) async throws(EranosError) -> Expense {
        try await converting { [handle] cancellation in
            Expense(try await handle.removeExpenseAsync(id: id.rawValue, cancellation: cancellation))
        }
    }
}
