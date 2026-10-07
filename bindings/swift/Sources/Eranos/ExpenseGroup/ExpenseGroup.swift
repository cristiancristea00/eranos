internal import EranosGenerated

/// A group of people who share expenses, safe to share between tasks and threads.
///
/// Every call comes in two forms under one name. The synchronous one waits for the group's lock,
/// blocking its thread; the asynchronous one waits without blocking, and cancelling its task
/// cancels it, leaving the group as it was. Swift picks the asynchronous form in asynchronous code,
/// so a task never blocks a thread of the pool:
///
/// ```swift
/// let group = try ExpenseGroup(name: "Flat", currency: CurrencyCode(code: "EUR"))
/// let ana = try group.add(Member(firstName: "Ana"))   // synchronous code
/// let bob = try await group.add(Member(firstName: "Bob"))   // asynchronous code
/// ```
///
/// Reads share the lock and run together; a change takes it alone, after the reads in progress.
public final class ExpenseGroup: Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a group.
    public typealias ID = Identifier<ExpenseGroup> // swiftlint:disable:this type_name

    let handle: EranosGenerated.ExpenseGroup

    /// A new group with no members and no entries.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName`` for a blank name.
    public init(name: String, currency: CurrencyCode, description: String? = nil, language: Language? = nil) throws(EranosError) {
        let group = NewExpenseGroup(name: name, currency: currency.code, description: description, language: language?.description)
        handle = try converting { try EranosGenerated.ExpenseGroup(group: group) }
    }

    private init(handle: EranosGenerated.ExpenseGroup) {
        self.handle = handle
    }

    /// The group saved as `json`, loaded through the checks that adding its entries makes.
    ///
    /// - Throws: ``EranosError/invalidJSON(line:column:detail:)`` for text that is not the saved
    ///   form, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/unsupportedFormat(format:)`` for a
    ///   format this release cannot read, and the failures of entries that do not fit together.
    public static func load(_ json: String) throws(EranosError) -> ExpenseGroup {
        let handle = try converting { try EranosGenerated.ExpenseGroup.load(json: json) }
        return ExpenseGroup(handle: handle)
    }

    /// The group saved as `json`, loaded through the checks that adding its entries makes.
    ///
    /// Yields while it replays the entries; cancelling the task cancels the load.
    ///
    /// - Throws: As the synchronous form does, and ``EranosError/cancelled(detail:)`` when the task
    ///   is cancelled first.
    public static func load(_ json: String) async throws(EranosError) -> ExpenseGroup {
        let handle = try await converting { cancellation in
            try await EranosGenerated.loadExpenseGroupAsync(json: json, cancellation: cancellation)
        }
        return ExpenseGroup(handle: handle)
    }

    /// A read handle, which holds the shared lock until it is released or freed.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while a change
    ///   holds the lock.
    public func read() throws(EranosError) -> ExpenseGroupReader {
        let reader = try converting { try handle.read() }
        return ExpenseGroupReader(reader)
    }

    /// A read handle, which holds the shared lock until it is released or freed.
    ///
    /// Waits for the lock without blocking a thread; cancelling the task cancels the wait.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    public func read() async throws(EranosError) -> ExpenseGroupReader {
        let reader = try await converting { [handle] cancellation in
            try await handle.readAsync(cancellation: cancellation)
        }
        return ExpenseGroupReader(reader)
    }

    /// Runs `body` with a read handle, and releases it when `body` returns or throws, so that
    /// every read of `body` sees the group as one state.
    ///
    /// - Throws: As `read()` does, and what `body` throws.
    public func withRead<Value>(_ body: (ExpenseGroupReader) throws(EranosError) -> Value) throws(EranosError) -> Value {
        let reader = try read()
        defer {
            reader.release()
        }
        return try body(reader)
    }

    /// Runs `body` with a read handle, and releases it when `body` returns or throws, so that
    /// every read of `body` sees the group as one state.
    ///
    /// Waits for the lock without blocking a thread.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first, and what
    ///   `body` throws.
    public func withRead<Value>(_ body: (ExpenseGroupReader) async throws(EranosError) -> Value) async throws(EranosError) -> Value {
        let reader = try await read()
        defer {
            reader.release()
        }
        return try await body(reader)
    }
}
