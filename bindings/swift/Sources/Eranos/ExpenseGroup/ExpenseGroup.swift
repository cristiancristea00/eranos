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

    /// A new group with no members and no entries, under `id`, which the app gives: its own, or one
    /// ``Identifier/generate()`` makes; ``init(name:currency:note:language:)`` generates it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse, and
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName`` for a blank name.
    public init(
        id: ID,
        name: String,
        currency: CurrencyCode,
        note: String? = nil,
        language: Language? = nil
    ) throws(EranosError) {
        let group = NewExpenseGroup(
            id: id.rawValue,
            name: name,
            currency: currency.code,
            note: note,
            language: language?.description
        )
        handle = try converting { try EranosGenerated.ExpenseGroup(group: group) }
    }

    /// A new group with no members and no entries, under an identifier ``Identifier/generate()``
    /// makes when this is called; ``init(id:name:currency:note:language:)`` takes the app's
    /// own.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName``
    ///   for a blank name.
    public convenience init(
        name: String,
        currency: CurrencyCode,
        note: String? = nil,
        language: Language? = nil
    ) throws(EranosError) {
        try self.init(id: .generate(), name: name, currency: currency, note: note, language: language)
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
    /// - Throws: Nothing on the platforms the package builds for: the call it makes can fail only on wasm,
    ///   where another call can hold the lock.
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

    /// A write handle, which holds the exclusive lock until it is released or freed, so that its
    /// reads and changes run with no other call in between. Any synchronous call of the group on the
    /// thread that holds one waits forever: release or free the handle first.
    ///
    /// - Throws: Nothing on the platforms the package builds for: the call it makes can fail only on wasm,
    ///   where another call can hold the lock.
    public func write() throws(EranosError) -> ExpenseGroupWriter {
        let writer = try converting { try handle.write() }
        return ExpenseGroupWriter(writer)
    }

    /// A write handle, which holds the exclusive lock until it is released or freed, so that its
    /// reads and changes run with no other call in between.
    ///
    /// Waits for the lock without blocking a thread; cancelling the task cancels the wait.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    public func write() async throws(EranosError) -> ExpenseGroupWriter {
        let writer = try await converting { [handle] cancellation in
            try await handle.writeAsync(cancellation: cancellation)
        }
        return ExpenseGroupWriter(writer)
    }

    /// Runs `body` with a write handle, and releases it when `body` returns or throws, so that no
    /// other call runs between the reads and changes of `body`.
    ///
    /// - Throws: As `write()` does, and what `body` throws.
    public func withWrite<Value>(_ body: (ExpenseGroupWriter) throws(EranosError) -> Value) throws(EranosError) -> Value {
        let writer = try write()
        defer {
            writer.release()
        }
        return try body(writer)
    }

    /// Runs `body` with a write handle, and releases it when `body` returns or throws, so that no
    /// other call runs between the reads and changes of `body`.
    ///
    /// Waits for the lock without blocking a thread.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first, and what
    ///   `body` throws.
    public func withWrite<Value>(_ body: (ExpenseGroupWriter) async throws(EranosError) -> Value) async throws(EranosError) -> Value {
        let writer = try await write()
        defer {
            writer.release()
        }
        return try await body(writer)
    }

    // A closure that throws errors other than the library's, or whose thrown type Swift infers as
    // `any Error`, as it does for every closure not marked with one, takes the forms below. They
    // are disfavoured: without that, a closure that throws nothing would match both forms and make
    // the call ambiguous.

    /// Runs `body` with a read handle, and releases it when `body` returns or throws, as the form
    /// whose `body` throws ``EranosError`` alone does, for a `body` that throws other errors too.
    ///
    /// - Throws: As `read()` does, and what `body` throws.
    @_disfavoredOverload
    public func withRead<Value>(_ body: (ExpenseGroupReader) throws -> Value) throws -> Value {
        let reader = try read()
        defer {
            reader.release()
        }
        return try body(reader)
    }

    /// Runs `body` with a read handle, and releases it when `body` returns or throws, as the form
    /// whose `body` throws ``EranosError`` alone does, for a `body` that throws other errors too.
    ///
    /// Waits for the lock without blocking a thread.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first, and what
    ///   `body` throws.
    @_disfavoredOverload
    public func withRead<Value>(_ body: (ExpenseGroupReader) async throws -> Value) async throws -> Value {
        let reader = try await read()
        defer {
            reader.release()
        }
        return try await body(reader)
    }

    /// Runs `body` with a write handle, and releases it when `body` returns or throws, as the form
    /// whose `body` throws ``EranosError`` alone does, for a `body` that throws other errors too.
    ///
    /// - Throws: As `write()` does, and what `body` throws.
    @_disfavoredOverload
    public func withWrite<Value>(_ body: (ExpenseGroupWriter) throws -> Value) throws -> Value {
        let writer = try write()
        defer {
            writer.release()
        }
        return try body(writer)
    }

    /// Runs `body` with a write handle, and releases it when `body` returns or throws, as the form
    /// whose `body` throws ``EranosError`` alone does, for a `body` that throws other errors too.
    ///
    /// Waits for the lock without blocking a thread.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first, and what
    ///   `body` throws.
    @_disfavoredOverload
    public func withWrite<Value>(_ body: (ExpenseGroupWriter) async throws -> Value) async throws -> Value {
        let writer = try await write()
        defer {
            writer.release()
        }
        return try await body(writer)
    }
}
