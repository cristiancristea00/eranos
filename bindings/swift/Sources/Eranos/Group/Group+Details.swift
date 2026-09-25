internal import EranosGenerated

// The group's own details: its identifier, name, description, language, currency, and saved form.
public extension Group {
    /// The group's identifier.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func id() throws(EranosError) -> Group.ID {
        try converting { Group.ID(rawValue: try handle.id()) }
    }

    /// The group's identifier.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func id() async throws(EranosError) -> Group.ID {
        try await converting { [handle] cancellation in
            Group.ID(rawValue: try await handle.idAsync(cancellation: cancellation))
        }
    }

    /// The group currency, which every total is in.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func currency() throws(EranosError) -> Currency {
        try converting { Currency(known: try handle.currency()) }
    }

    /// The group currency, which every total is in.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func currency() async throws(EranosError) -> Currency {
        try await converting { [handle] cancellation in
            Currency(known: try await handle.currencyAsync(cancellation: cancellation))
        }
    }

    /// Moves the group to `currency`, converting every entry with the rate `rates` gives its
    /// currency.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/missingExchangeRate(_:)`` for
    ///   a currency in use without a rate, and ``GroupFailure/unexpectedExchangeRate`` for a rate of
    ///   the new group currency; on wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy``
    ///   while another call holds the lock.
    func changeCurrency(to currency: Currency, rates: [Currency: ExchangeRate]) throws(EranosError) {
        try converting { try handle.changeCurrency(currency: currency.code, rates: rates.map { CurrencyRate(currency: $0.key.code, rate: $0.value.description) }) }
    }

    /// Moves the group to `currency`, converting every entry with the rate `rates` gives its
    /// currency.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/missingExchangeRate(_:)`` for
    ///   a currency in use without a rate, and ``GroupFailure/unexpectedExchangeRate`` for a rate of
    ///   the new group currency; ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func changeCurrency(to currency: Currency, rates: [Currency: ExchangeRate]) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.changeCurrencyAsync(currency: currency.code, rates: rates.map { CurrencyRate(currency: $0.key.code, rate: $0.value.description) }, cancellation: cancellation)
        }
    }

    /// The group's name.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func name() throws(EranosError) -> String {
        try converting { try handle.name() }
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
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/emptyName`` for a blank name;
    ///   on wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another call
    ///   holds the lock.
    func rename(to name: String) throws(EranosError) {
        try converting { try handle.setName(name: name) }
    }

    /// Renames the group.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/group(_:detail:)`` with ``GroupFailure/emptyName`` for a blank name;
    ///   ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func rename(to name: String) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.setNameAsync(name: name, cancellation: cancellation)
        }
    }

    /// The group's description.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func description() throws(EranosError) -> String? {
        try converting { try handle.description() }
    }

    /// The group's description.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func description() async throws(EranosError) -> String? {
        try await converting { [handle] cancellation in
            try await handle.descriptionAsync(cancellation: cancellation)
        }
    }

    /// Sets the group's description; nil or blank text clears it.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func changeDescription(to description: String?) throws(EranosError) {
        try converting { try handle.setDescription(description: description ?? "") }
    }

    /// Sets the group's description; nil or blank text clears it.
    ///
    /// Waits for the group's lock without blocking a thread; cancelling the task cancels the call,
    /// which leaves the group as it was.
    ///
    /// - Throws: ``EranosError/cancelled(detail:)`` when the task is cancelled first.
    func changeDescription(to description: String?) async throws(EranosError) {
        try await converting { [handle] cancellation in
            try await handle.setDescriptionAsync(description: description ?? "", cancellation: cancellation)
        }
    }

    /// The language the apps show the group in.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func language() throws(EranosError) -> Language? {
        try converting { try handle.language().map { Language(known: $0) } }
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
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
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

    /// The group in its saved form, as JSON, which `Group.load(_:)` reads back.
    ///
    /// - Throws: On wasm, ``EranosError/group(_:detail:)`` with ``GroupFailure/busy`` while another
    ///   call holds the lock.
    func save() throws(EranosError) -> String {
        try converting { try handle.save() }
    }

    /// The group in its saved form, as JSON, which `Group.load(_:)` reads back.
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
