internal import EranosGenerated

// The group's categories.
public extension ExpenseGroup {
    /// Every category, the built-in one first.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    func categories() throws(EranosError) -> [Category] {
        try converting { try handle.categories().map(Category.init) }
    }

    /// Every category, the built-in one first.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func categories() async throws(EranosError) -> [Category] {
        try await converting { [handle] cancellation in
            try await handle.categoriesAsync(cancellation: cancellation).map(Category.init)
        }
    }

    /// The category with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func category(withID id: Category.ID) throws(EranosError) -> Category? {
        try converting { try handle.category(id: id.rawValue).map(Category.init) }
    }

    /// The category with the identifier `id`, if the group holds it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func category(withID id: Category.ID) async throws(EranosError) -> Category? {
        try await converting { [handle] cancellation in
            try await handle.categoryAsync(id: id.rawValue, cancellation: cancellation).map(Category.init)
        }
    }

    /// Adds `category`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for a category with an identifier, and
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateCategoryName(_:)`` for a name
    ///   another category has, compared caselessly; on wasm, ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/busy`` while another call holds the lock.
    @discardableResult
    func add(_ category: Category) throws(EranosError) -> Category.ID {
        try converting { Category.ID(rawValue: try handle.addCategory(category: category.generated)) }
    }

    /// Adds `category`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for a category with an identifier, and
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateCategoryName(_:)`` for a name
    ///   another category has, compared caselessly; ``EranosError/cancelled(detail:)`` when the task
    ///   is cancelled first.
    @discardableResult
    func add(_ category: Category) async throws(EranosError) -> Category.ID {
        try await converting { [handle] cancellation in
            Category.ID(rawValue: try await handle.addCategoryAsync(category: category.generated, cancellation: cancellation))
        }
    }

    /// Renames the category that `category` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for a category without an identifier,
    ///   and ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownCategory(_:)``,
    ///   ``ExpenseGroupFailure/builtInCategory``, or ``ExpenseGroupFailure/duplicateCategoryName(_:)``; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    @discardableResult
    func replace(_ category: Category) throws(EranosError) -> Category {
        try converting { Category(try handle.replaceCategory(category: category.generated)) }
    }

    /// Renames the category that `category` names by its identifier, and returns it as it was.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for a category without an identifier,
    ///   and ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownCategory(_:)``,
    ///   ``ExpenseGroupFailure/builtInCategory``, or ``ExpenseGroupFailure/duplicateCategoryName(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func replace(_ category: Category) async throws(EranosError) -> Category {
        try await converting { [handle] cancellation in
            Category(try await handle.replaceCategoryAsync(category: category.generated, cancellation: cancellation))
        }
    }

    /// Removes the category with the identifier `id` and returns it; its expenses move to the
    /// built-in one.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownCategory(_:)`` or
    ///   ``ExpenseGroupFailure/builtInCategory``; on wasm, ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/busy`` while another call holds the lock.
    @discardableResult
    func remove(_ id: Category.ID) throws(EranosError) -> Category {
        try converting { Category(try handle.removeCategory(id: id.rawValue)) }
    }

    /// Removes the category with the identifier `id` and returns it; its expenses move to the
    /// built-in one.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownCategory(_:)`` or
    ///   ``ExpenseGroupFailure/builtInCategory``; ``EranosError/cancelled(detail:)`` when the task is
    ///   cancelled first.
    @discardableResult
    func remove(_ id: Category.ID) async throws(EranosError) -> Category {
        try await converting { [handle] cancellation in
            Category(try await handle.removeCategoryAsync(id: id.rawValue, cancellation: cancellation))
        }
    }
}
