internal import EranosGenerated

// The group's own details: its identifier, name, currency, note, language, and saved form.
public extension ExpenseGroup {
    /// The group's identifier.
    var id: ExpenseGroup.ID {
        ExpenseGroup.ID(rawValue: handle.id())
    }

    /// The group's identifier.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func id() async throws(EranosError) -> ExpenseGroup.ID {
        try await converting { [handle] cancellation in
            ExpenseGroup.ID(rawValue: try await handle.idAsync(cancellation: cancellation))
        }
    }

    /// The group's name.
    var name: String {
        handle.name()
    }

    /// The group's name.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func name() async throws(EranosError) -> String {
        try await converting { [handle] cancellation in
            try await handle.nameAsync(cancellation: cancellation)
        }
    }

    /// Renames the group.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName`` for a blank name;
    ///   on wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another call
    ///   holds the lock.
    func rename(to name: String) throws(EranosError) {
        try converting { try handle.setName(name: name) }
    }

    /// Renames the group.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName`` for a blank name;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func rename(to name: String) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.setNameAsync(name: name, cancellation: cancellation)
        }
    }

    /// The group currency, which every total is in.
    var currency: CurrencyCode {
        CurrencyCode(handle.currency())
    }

    /// The group currency, which every total is in.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func currency() async throws(EranosError) -> CurrencyCode {
        try await converting { [handle] cancellation in
            CurrencyCode(try await handle.currencyAsync(cancellation: cancellation))
        }
    }

    /// Moves the group to `currency`, converting every entry with the rate `rates` gives its
    /// currency.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/missingExchangeRate(_:)`` for
    ///   a currency in use without a rate, and ``ExpenseGroupFailure/unexpectedExchangeRate`` for a rate of
    ///   the new group currency; on wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy``
    ///   while another call holds the lock.
    func changeCurrency(to currency: CurrencyCode, rates: [CurrencyCode: ExchangeRate]) throws(EranosError) {
        try converting { try handle.changeCurrency(currency: currency.code, rates: rates.map { CurrencyRate(currency: $0.key.code, rate: $0.value.description) }) }
    }

    /// Moves the group to `currency`, converting every entry with the rate `rates` gives its
    /// currency.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/missingExchangeRate(_:)`` for
    ///   a currency in use without a rate, and ``ExpenseGroupFailure/unexpectedExchangeRate`` for a rate of
    ///   the new group currency; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func changeCurrency(to currency: CurrencyCode, rates: [CurrencyCode: ExchangeRate]) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.changeCurrencyAsync(currency: currency.code, rates: rates.map { CurrencyRate(currency: $0.key.code, rate: $0.value.description) }, cancellation: cancellation)
        }
    }

    /// The group's note.
    var note: String? {
        handle.note()
    }

    /// The group's note.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func note() async throws(EranosError) -> String? {
        try await converting { [handle] cancellation in
            try await handle.noteAsync(cancellation: cancellation)
        }
    }

    /// Sets the group's note, trimmed; nil or blank text clears it.
    func changeNote(to note: String?) {
        handle.setNote(note: note)
    }

    /// Sets the group's note, trimmed; nil or blank text clears it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func changeNote(to note: String?) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.setNoteAsync(note: note, cancellation: cancellation)
        }
    }

    /// The language the apps show the group in.
    var language: Language? {
        handle.language().map { Language(known: $0) }
    }

    /// The language the apps show the group in.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func language() async throws(EranosError) -> Language? {
        try await converting { [handle] cancellation in
            try await handle.languageAsync(cancellation: cancellation).map { Language(known: $0) }
        }
    }

    /// Sets the language the apps show the group in, or clears it with nil.
    ///
    /// - Throws: On wasm, ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/busy`` while another
    ///   call holds the lock.
    func changeLanguage(to language: Language?) throws(EranosError) {
        try converting { try handle.setLanguage(language: language?.description) }
    }

    /// Sets the language the apps show the group in, or clears it with nil.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func changeLanguage(to language: Language?) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.setLanguageAsync(language: language?.description, cancellation: cancellation)
        }
    }

    /// The group in its saved form, as JSON, which `ExpenseGroup.load(_:)` reads back.
    func save() -> String {
        handle.save()
    }

    /// The group in its saved form, as JSON, which `ExpenseGroup.load(_:)` reads back.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func save() async throws(EranosError) -> String {
        try await converting { [handle] cancellation in
            try await handle.saveAsync(cancellation: cancellation)
        }
    }
}
