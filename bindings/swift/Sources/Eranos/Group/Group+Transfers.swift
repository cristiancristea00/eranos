internal import EranosGenerated

// The group's transfers.
public extension Group {
    /// Every transfer, in the order they were created.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func transfers() throws(EranosError) -> [Transfer] {
        try converting { try handle.transfers().map(Transfer.init) }
    }

    /// Every transfer, in the order they were created.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func transfers() async throws(EranosError) -> [Transfer] {
        try await converting { [handle] cancellation in
            try await handle.transfersAsync(cancellation: cancellation).map(Transfer.init)
        }
    }

    /// The transfer with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another call holds the
    ///   lock.
    func transfer(withID id: Transfer.ID) throws(EranosError) -> Transfer? {
        try converting { try handle.transfer(id: id.rawValue).map(Transfer.init) }
    }

    /// The transfer with the identifier `id`, if the group holds it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func transfer(withID id: Transfer.ID) async throws(EranosError) -> Transfer? {
        try await converting { [handle] cancellation in
            try await handle.transferAsync(id: id.rawValue, cancellation: cancellation).map(Transfer.init)
        }
    }

    /// The amount in the group currency of the transfer with the identifier `id`, if the group
    /// holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one; on wasm,
    ///   ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another call holds the
    ///   lock.
    func amount(of id: Transfer.ID) throws(EranosError) -> Money? {
        try converting { try handle.transferAmount(id: id.rawValue).map(Money.init) }
    }

    /// The amount in the group currency of the transfer with the identifier `id`, if the group
    /// holds it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func amount(of id: Transfer.ID) async throws(EranosError) -> Money? {
        try await converting { [handle] cancellation in
            try await handle.transferAmountAsync(id: id.rawValue, cancellation: cancellation).map(Money.init)
        }
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for a transfer with an identifier,
    ///   ``EranosError/transfer(_:detail:)`` for one that does not build, such as
    ///   ``TransferFailure/sameMember``, and ``EranosError/group(_:detail:)`` with
    ///   ``GroupFailure/unknownMember(_:)``; on wasm, ``EranosError/group(_:detail:)`` with
    ///   ``GroupFailure/busy`` while another call holds the lock.
    @discardableResult
    func add(_ transfer: Transfer) throws(EranosError) -> Transfer.ID {
        try converting { Transfer.ID(rawValue: try handle.addTransfer(transfer: transfer.generated)) }
    }

    /// Adds `transfer`, which has no identifier yet, and returns the one the group gives it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierGiven(detail:)`` for a transfer with an identifier,
    ///   ``EranosError/transfer(_:detail:)`` for one that does not build, such as
    ///   ``TransferFailure/sameMember``, and ``EranosError/group(_:detail:)`` with
    ///   ``GroupFailure/unknownMember(_:)``; ``EranosError/cancelled(detail:)`` when the task is
    ///   cancelled first.
    @discardableResult
    func add(_ transfer: Transfer) async throws(EranosError) -> Transfer.ID {
        try await converting { [handle] cancellation in
            Transfer.ID(rawValue: try await handle.addTransferAsync(transfer: transfer.generated, cancellation: cancellation))
        }
    }

    /// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for a transfer without an identifier,
    ///   ``EranosError/group(_:detail:)`` with ``GroupFailure/unknownTransfer(_:)``, and the failures
    ///   of adding one; on wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while
    ///   another call holds the lock.
    @discardableResult
    func replace(_ transfer: Transfer) throws(EranosError) -> Transfer {
        try converting { Transfer(try handle.replaceTransfer(transfer: transfer.generated)) }
    }

    /// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/identifierMissing(detail:)`` for a transfer without an identifier,
    ///   ``EranosError/group(_:detail:)`` with ``GroupFailure/unknownTransfer(_:)``, and the failures
    ///   of adding one; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func replace(_ transfer: Transfer) async throws(EranosError) -> Transfer {
        try await converting { [handle] cancellation in
            Transfer(try await handle.replaceTransferAsync(transfer: transfer.generated, cancellation: cancellation))
        }
    }

    /// Removes the transfer with the identifier `id` and returns it.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/unknownTransfer(_:)``; on
    ///   wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another call holds
    ///   the lock.
    @discardableResult
    func remove(_ id: Transfer.ID) throws(EranosError) -> Transfer {
        try converting { Transfer(try handle.removeTransfer(id: id.rawValue)) }
    }

    /// Removes the transfer with the identifier `id` and returns it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/unknownTransfer(_:)``;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    @discardableResult
    func remove(_ id: Transfer.ID) async throws(EranosError) -> Transfer {
        try await converting { [handle] cancellation in
            Transfer(try await handle.removeTransferAsync(id: id.rawValue, cancellation: cancellation))
        }
    }
}
