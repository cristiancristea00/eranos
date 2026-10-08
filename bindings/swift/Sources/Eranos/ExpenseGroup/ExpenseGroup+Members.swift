internal import EranosGenerated

// The group's members.
public extension ExpenseGroup {
    /// Every member, archived ones included, in identifier order, which is creation order for
    /// generated identifiers.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    func members() throws(EranosError) -> [Member] {
        try converting { try handle.members().map(Member.init) }
    }

    /// Every member, archived ones included, in identifier order, which is creation order for
    /// generated identifiers.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func members() async throws(EranosError) -> [Member] {
        try await converting { [handle] cancellation in
            try await handle.membersAsync(cancellation: cancellation).map(Member.init)
        }
    }

    /// The member with the identifier `id`, if the group holds them.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func member(withID id: Member.ID) throws(EranosError) -> Member? {
        try converting { try handle.member(id: id.rawValue).map(Member.init) }
    }

    /// The member with the identifier `id`, if the group holds them.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func member(withID id: Member.ID) async throws(EranosError) -> Member? {
        try await converting { [handle] cancellation in
            try await handle.memberAsync(id: id.rawValue, cancellation: cancellation).map(Member.init)
        }
    }

    /// Whether the member with the identifier `id` is archived; `nil` for a member the group does
    /// not hold.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func isArchived(_ id: Member.ID) throws(EranosError) -> Bool? {
        try converting { try handle.isArchived(id: id.rawValue) }
    }

    /// Whether the member with the identifier `id` is archived; `nil` for a member the group does
    /// not hold.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func isArchived(_ id: Member.ID) async throws(EranosError) -> Bool? {
        try await converting { [handle] cancellation in
            try await handle.isArchivedAsync(id: id.rawValue, cancellation: cancellation)
        }
    }

    /// Adds `member` under their identifier, and returns the identifier.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateMember(_:)`` for one
    ///   the group holds, and ``EranosError/member(_:detail:)`` with ``MemberFailure/emptyFirstName``
    ///   for a blank first name; on wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    @discardableResult
    func add(_ member: Member) throws(EranosError) -> Member.ID {
        try converting { Member.ID(rawValue: try handle.addMember(member: member.generated)) }
    }

    /// Adds `member` under their identifier, and returns the identifier.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateMember(_:)`` for one
    ///   the group holds, and ``EranosError/member(_:detail:)`` with ``MemberFailure/emptyFirstName``
    ///   for a blank first name; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func add(_ member: Member) async throws(EranosError) -> Member.ID {
        try await converting { [handle] cancellation in
            Member.ID(rawValue: try await handle.addMemberAsync(member: member.generated, cancellation: cancellation))
        }
    }

    /// Replaces the details of the member that `member` names by their identifier, and returns them
    /// as they were; an archived member stays archived.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``
    ///   for an identifier the group does not hold, and the failures of adding one; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call
    ///   holds the lock.
    @discardableResult
    func replace(_ member: Member) throws(EranosError) -> Member {
        try converting { Member(try handle.replaceMember(member: member.generated)) }
    }

    /// Replaces the details of the member that `member` names by their identifier, and returns them
    /// as they were; an archived member stays archived.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``
    ///   for an identifier the group does not hold, and the failures of adding one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func replace(_ member: Member) async throws(EranosError) -> Member {
        try await converting { [handle] cancellation in
            Member(try await handle.replaceMemberAsync(member: member.generated, cancellation: cancellation))
        }
    }

    /// Archives the member with the identifier `id`: they keep their history and balance, and join
    /// no new expense.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func archive(_ id: Member.ID) throws(EranosError) {
        try converting { try handle.archiveMember(id: id.rawValue) }
    }

    /// Archives the member with the identifier `id`: they keep their history and balance, and join
    /// no new expense.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func archive(_ id: Member.ID) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.archiveMemberAsync(id: id.rawValue, cancellation: cancellation)
        }
    }

    /// Makes the archived member with the identifier `id` active again.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``; on wasm,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call holds the
    ///   lock.
    func unarchive(_ id: Member.ID) throws(EranosError) {
        try converting { try handle.unarchiveMember(id: id.rawValue) }
    }

    /// Makes the archived member with the identifier `id` active again.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func unarchive(_ id: Member.ID) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.unarchiveMemberAsync(id: id.rawValue, cancellation: cancellation)
        }
    }

    /// Removes the member with the identifier `id`, who appears in no entry, and returns them.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``, and
    ///   ``ExpenseGroupFailure/memberInUse(member:expenses:transfers:)`` with the entries that still name
    ///   them; archive such a member instead; on wasm, ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/busy`` while another call holds the lock.
    @discardableResult
    func remove(_ id: Member.ID) throws(EranosError) -> Member {
        try converting { Member(try handle.removeMember(id: id.rawValue)) }
    }

    /// Removes the member with the identifier `id`, who appears in no entry, and returns them.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unknownMember(_:)``, and
    ///   ``ExpenseGroupFailure/memberInUse(member:expenses:transfers:)`` with the entries that still name
    ///   them; archive such a member instead; ``EranosError/cancelled(detail:)`` when the task is
    ///   cancelled first.
    @discardableResult
    func remove(_ id: Member.ID) async throws(EranosError) -> Member {
        try await converting { [handle] cancellation in
            Member(try await handle.removeMemberAsync(id: id.rawValue, cancellation: cancellation))
        }
    }
}
