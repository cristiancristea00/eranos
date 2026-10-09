internal import EranosGenerated

/// A group's reads and changes under one exclusive lock, so that no other call runs in between:
/// every other call waits until the handle is released or freed.
///
/// `ExpenseGroup.withWrite(_:)` releases the handle when its closure returns. A handle from
/// `ExpenseGroup.write()` holds the lock until ``release()`` or until it is freed; any synchronous call
/// of the group on the thread that holds one waits forever, since the handle holds the lock the call
/// needs and the thread cannot release it meanwhile, so release it first.
public final class ExpenseGroupWriter: Sendable {
    private let handle: EranosGenerated.ExpenseGroupWriter

    init(_ handle: EranosGenerated.ExpenseGroupWriter) {
        self.handle = handle
    }

    /// Releases the lock; every call after it throws ``EranosError/released(detail:)``, and
    /// releasing a released handle does nothing.
    public func release() {
        handle.release()
    }

    /// The group's identifier.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func id() throws(EranosError) -> ExpenseGroup.ID {
        try converting { ExpenseGroup.ID(rawValue: try handle.id()) }
    }

    /// The group's name.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func name() throws(EranosError) -> String {
        try converting { try handle.name() }
    }

    /// The group currency, which every total is in.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func currency() throws(EranosError) -> CurrencyCode {
        try converting { CurrencyCode(try handle.currency()) }
    }

    /// The group's note.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func note() throws(EranosError) -> String? {
        try converting { try handle.note() }
    }

    /// The language the apps show the group in.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func language() throws(EranosError) -> Language? {
        try converting { try handle.language().map { Language(known: $0) } }
    }

    /// Every member, archived ones included, in identifier order, which is creation order for
    /// generated identifiers.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func members() throws(EranosError) -> [Member] {
        try converting { try handle.members().map(Member.init) }
    }

    /// The member with the identifier `id`, if the group holds them.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func member(withID id: Member.ID) throws(EranosError) -> Member? {
        try converting { try handle.member(id: id.rawValue).map(Member.init) }
    }

    /// Whether the member with the identifier `id` is archived; `nil` for a member the group does
    /// not hold.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func isArchived(_ id: Member.ID) throws(EranosError) -> Bool? {
        try converting { try handle.isArchived(id: id.rawValue) }
    }

    /// Every category: the built-in one first, then the others in identifier order, which is
    /// creation order for generated identifiers.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func categories() throws(EranosError) -> [Category] {
        try converting { try handle.categories().map(Category.init) }
    }

    /// The category with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func category(withID id: Category.ID) throws(EranosError) -> Category? {
        try converting { try handle.category(id: id.rawValue).map(Category.init) }
    }

    /// Every expense, in identifier order, which is creation order for generated identifiers.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func expenses() throws(EranosError) -> [Expense] {
        try converting { try handle.expenses().map(Expense.init) }
    }

    /// The expense with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func expense(withID id: Expense.ID) throws(EranosError) -> Expense? {
        try converting { try handle.expense(id: id.rawValue).map(Expense.init) }
    }

    /// How the expense with the identifier `id` divides, in the group currency, if the group holds
    /// it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func shares(of id: Expense.ID) throws(EranosError) -> ExpenseShares? {
        try converting { try handle.expenseShares(id: id.rawValue).map(ExpenseShares.init) }
    }

    /// Every transfer, in identifier order, which is creation order for generated identifiers.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func transfers() throws(EranosError) -> [Transfer] {
        try converting { try handle.transfers().map(Transfer.init) }
    }

    /// The transfer with the identifier `id`, if the group holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func transfer(withID id: Transfer.ID) throws(EranosError) -> Transfer? {
        try converting { try handle.transfer(id: id.rawValue).map(Transfer.init) }
    }

    /// The amount in the group currency of the transfer with the identifier `id`, if the group
    /// holds it.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func amount(of id: Transfer.ID) throws(EranosError) -> Money? {
        try converting { try handle.transferAmount(id: id.rawValue).map(Money.init) }
    }

    /// The group's totals.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func summary() throws(EranosError) -> Summary {
        try converting { Summary(try handle.summary()) }
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// - Throws: ``EranosError/settlement(_:detail:)`` with
    ///   ``SettlementFailure/outOfMemory(balances:)`` for an exact plan of more open balances than
    ///   the table fits; ``EranosError/released(detail:)`` after ``release()``.
    public func settlement(by strategy: SettlementStrategy) throws(EranosError) -> SettlementPlan {
        try converting { SettlementPlan(try handle.settlement(strategy: strategy.generated)) }
    }

    /// The group in its saved form, as JSON, which `ExpenseGroup.load(_:)` reads back.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func save() throws(EranosError) -> String {
        try converting { try handle.save() }
    }

    /// Adds `category` under its identifier, and returns the identifier.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse, and
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateCategory(_:)``
    ///   for an identifier the group holds, Uncategorised's included, or
    ///   ``ExpenseGroupFailure/duplicateCategoryName(_:)`` for a name another category has,
    ///   compared caselessly; ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func add(_ category: Category) throws(EranosError) -> Category.ID {
        try converting { Category.ID(rawValue: try handle.addCategory(category: category.generated)) }
    }

    /// Renames the category that `category` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownCategory(_:)`` for an identifier the group does not hold,
    ///   ``ExpenseGroupFailure/builtInCategory``, or
    ///   ``ExpenseGroupFailure/duplicateCategoryName(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    @discardableResult
    public func replace(_ category: Category) throws(EranosError) -> Category {
        try converting { Category(try handle.replaceCategory(category: category.generated)) }
    }

    /// Removes the category with the identifier `id` and returns it; its expenses move to the
    /// built-in one.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownCategory(_:)`` or ``ExpenseGroupFailure/builtInCategory``;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func remove(_ id: Category.ID) throws(EranosError) -> Category {
        try converting { Category(try handle.removeCategory(id: id.rawValue)) }
    }

    /// Adds `expense` under its identifier, and returns the identifier.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse,
    ///   ``EranosError/expense(_:detail:)`` for an expense that does not build, and
    ///   ``EranosError/expenseGroup(_:detail:)`` for one that does not fit the group, such as
    ///   ``ExpenseGroupFailure/duplicateExpense(_:)`` for an identifier the group holds,
    ///   ``ExpenseGroupFailure/unknownMember(_:)``, or ``ExpenseGroupFailure/archivedMember(_:)``;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func add(_ expense: Expense) throws(EranosError) -> Expense.ID {
        try converting { Expense.ID(rawValue: try handle.addExpense(expense: expense.generated)) }
    }

    /// Replaces the expense that `expense` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownExpense(_:)`` for an identifier the group does not hold, and
    ///   the failures of adding one; ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func replace(_ expense: Expense) throws(EranosError) -> Expense {
        try converting { Expense(try handle.replaceExpense(expense: expense.generated)) }
    }

    /// Removes the expense with the identifier `id` and returns it.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownExpense(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    @discardableResult
    public func remove(_ id: Expense.ID) throws(EranosError) -> Expense {
        try converting { Expense(try handle.removeExpense(id: id.rawValue)) }
    }

    /// Adds `member` under their identifier, and returns the identifier.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse,
    ///   ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/duplicateMember(_:)``
    ///   for one the group holds, and ``EranosError/member(_:detail:)`` with
    ///   ``MemberFailure/emptyFirstName`` for a blank first name; ``EranosError/released(detail:)``
    ///   after ``release()``.
    @discardableResult
    public func add(_ member: Member) throws(EranosError) -> Member.ID {
        try converting { Member.ID(rawValue: try handle.addMember(member: member.generated)) }
    }

    /// Replaces the details of the member that `member` names by their identifier, and returns them
    /// as they were; an archived member stays archived.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownMember(_:)`` for an identifier the group does not hold, and
    ///   the failures of adding one; ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func replace(_ member: Member) throws(EranosError) -> Member {
        try converting { Member(try handle.replaceMember(member: member.generated)) }
    }

    /// Archives the member with the identifier `id`: they keep their history and balance, and join
    /// no new expense.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownMember(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    public func archive(_ id: Member.ID) throws(EranosError) {
        try converting { try handle.archiveMember(id: id.rawValue) }
    }

    /// Makes the archived member with the identifier `id` active again.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownMember(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    public func unarchive(_ id: Member.ID) throws(EranosError) {
        try converting { try handle.unarchiveMember(id: id.rawValue) }
    }

    /// Removes the member with the identifier `id`, who appears in no entry, and returns them.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownMember(_:)``, and
    ///   ``ExpenseGroupFailure/memberInUse(member:expenses:transfers:)`` with the entries that
    ///   still name them; archive such a member instead; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    @discardableResult
    public func remove(_ id: Member.ID) throws(EranosError) -> Member {
        try converting { Member(try handle.removeMember(id: id.rawValue)) }
    }

    /// Renames the group.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with ``ExpenseGroupFailure/emptyName`` for
    ///   a blank name; ``EranosError/released(detail:)`` after ``release()``.
    public func rename(to name: String) throws(EranosError) {
        try converting { try handle.setName(name: name) }
    }

    /// Moves the group to `currency`, converting every entry with the rate `rates` gives its
    /// currency.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/missingExchangeRate(_:)`` for a currency in use without a rate, and
    ///   ``ExpenseGroupFailure/unexpectedExchangeRate`` for a rate of the new group currency;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func changeCurrency(to currency: CurrencyCode, rates: [CurrencyCode: ExchangeRate]) throws(EranosError) {
        try converting { try handle.changeCurrency(currency: currency.code, rates: rates.map { CurrencyRate(currency: $0.key.code, rate: $0.value.description) }) }
    }

    /// Sets the group's note, trimmed; nil or blank text clears it.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func changeNote(to note: String?) throws(EranosError) {
        try converting { try handle.setNote(note: note) }
    }

    /// Sets the language the apps show the group in, or clears it with nil.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func changeLanguage(to language: Language?) throws(EranosError) {
        try converting { try handle.setLanguage(language: language?.description) }
    }

    /// Adds `transfer` under its identifier, and returns the identifier.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier the rules refuse,
    ///   ``EranosError/transfer(_:detail:)`` for a transfer that does not build, such as
    ///   ``TransferFailure/sameMember``, and ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/duplicateTransfer(_:)`` for an identifier the group holds, or
    ///   ``ExpenseGroupFailure/unknownMember(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    @discardableResult
    public func add(_ transfer: Transfer) throws(EranosError) -> Transfer.ID {
        try converting { Transfer.ID(rawValue: try handle.addTransfer(transfer: transfer.generated)) }
    }

    /// Replaces the transfer that `transfer` names by its identifier, and returns it as it was.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownTransfer(_:)`` for an identifier the group does not hold, and
    ///   the failures of adding one; ``EranosError/released(detail:)`` after ``release()``.
    @discardableResult
    public func replace(_ transfer: Transfer) throws(EranosError) -> Transfer {
        try converting { Transfer(try handle.replaceTransfer(transfer: transfer.generated)) }
    }

    /// Removes the transfer with the identifier `id` and returns it.
    ///
    /// - Throws: ``EranosError/expenseGroup(_:detail:)`` with
    ///   ``ExpenseGroupFailure/unknownTransfer(_:)``; ``EranosError/released(detail:)`` after
    ///   ``release()``.
    @discardableResult
    public func remove(_ id: Transfer.ID) throws(EranosError) -> Transfer {
        try converting { Transfer(try handle.removeTransfer(id: id.rawValue)) }
    }
}
