internal import EranosGenerated

/// A group of people who share expenses, safe to share between tasks and threads.
///
/// Every call comes in two forms under one name. The synchronous one waits for the group's lock,
/// blocking its thread; the asynchronous one waits without blocking, and cancelling its task
/// cancels it, leaving the group as it was. Swift picks the asynchronous form in asynchronous code,
/// so a task never blocks a thread of the pool:
///
/// ```swift
/// let group = try Group(currency: Currency(code: "EUR"), name: "Flat")
/// let ana = try group.add(Member(firstName: "Ana"))   // synchronous code
/// let bob = try await group.add(Member(firstName: "Bob"))   // asynchronous code
/// ```
///
/// Reads share the lock and run together; a change takes it alone, after the reads in progress.
public final class Group: Sendable {
    // `ID` is the name Swift gives an identifier type, as `Identifiable.ID` does.
    /// The identifier of a group.
    public typealias ID = Identifier<Group> // swiftlint:disable:this type_name

    let handle: EranosGenerated.Group

    /// A new group with no members and no entries.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/emptyName`` for a blank name.
    public init(currency: Currency, name: String, description: String? = nil, language: Language? = nil) throws(EranosError) {
        let group = NewGroup(currency: currency.code, name: name, description: description, language: language?.description)
        handle = try converting { try EranosGenerated.Group(group: group) }
    }

    private init(handle: EranosGenerated.Group) {
        self.handle = handle
    }

    /// The group saved as `json`, loaded through the checks that adding its entries makes.
    ///
    /// - Throws: ``EranosError/invalidJSON(line:column:detail:)`` for text that is not the saved
    ///   form, ``EranosError/group(_:detail:)`` with ``GroupFailure/unsupportedFormat(format:)`` for a
    ///   format this release cannot read, and the failures of entries that do not fit together.
    public static func load(_ json: String) throws(EranosError) -> Group {
        let handle = try converting { try EranosGenerated.Group.load(json: json) }
        return Group(handle: handle)
    }

    /// The group saved as `json`, loaded through the checks that adding its entries makes.
    ///
    /// Yields while it replays the entries; cancelling the task cancels the load.
    ///
    /// - Throws: As the synchronous form does, and ``EranosError/cancelled(detail:)`` when the task
    ///   is cancelled first.
    public static func load(_ json: String) async throws(EranosError) -> Group {
        let handle = try await converting { cancellation in
            try await EranosGenerated.loadGroupAsync(json: json, cancellation: cancellation)
        }
        return Group(handle: handle)
    }

    /// A read handle, which holds the shared lock until it is released or freed.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while a change
    ///   holds the lock.
    public func read() throws(EranosError) -> GroupReader {
        let reader = try converting { try handle.read() }
        return GroupReader(reader)
    }

    /// A read handle, which holds the shared lock until it is released or freed.
    ///
    /// Waits for the lock without blocking a thread; cancelling the task cancels the wait.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    public func read() async throws(EranosError) -> GroupReader {
        let reader = try await converting { [handle] cancellation in
            try await handle.readAsync(cancellation: cancellation)
        }
        return GroupReader(reader)
    }

    /// Runs `body` with a read handle, and releases it when `body` returns or throws, so that
    /// every read of `body` sees the group as one state.
    ///
    /// - Throws: As `read()` does, and what `body` throws.
    public func withRead<Value>(_ body: (GroupReader) throws(EranosError) -> Value) throws(EranosError) -> Value {
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
    public func withRead<Value>(_ body: (GroupReader) async throws(EranosError) -> Value) async throws(EranosError) -> Value {
        let reader = try await read()
        defer {
            reader.release()
        }
        return try await body(reader)
    }
}
