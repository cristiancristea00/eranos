internal import EranosGenerated

/// A group's reads under one shared lock, so that they see no change in between: changes wait
/// until the handle is released or freed.
///
/// `Group.withRead(_:)` releases the handle when its closure returns. A handle from
/// `Group.read()` holds the lock until ``release()`` or until it is freed; a synchronous change on
/// the thread that holds one waits for it forever, so release it first.
public final class GroupReader: Sendable {
    private let handle: EranosGenerated.GroupReader

    init(_ handle: EranosGenerated.GroupReader) {
        self.handle = handle
    }

    /// Releases the lock; every read after it throws ``EranosError/released(detail:)``, and
    /// releasing a released handle does nothing.
    public func release() {
        handle.release()
    }

    /// The group's identifier.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func id() throws(EranosError) -> Group.ID {
        try converting { Group.ID(rawValue: try handle.id()) }
    }

    /// Every category, the built-in one first.
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

    /// The group currency, which every total is in.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func currency() throws(EranosError) -> Currency {
        try converting { Currency(known: try handle.currency()) }
    }

    /// Every expense, in the order they were created.
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

    /// The group's totals.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func summary() throws(EranosError) -> Summary {
        try converting { Summary(try handle.summary()) }
    }

    /// Every member, archived ones included, in the order they were added.
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

    /// Whether the member with the identifier `id` is archived; false for a member the group does
    /// not hold.
    ///
    /// - Throws: ``EranosError/id(_:detail:)`` for an identifier that is not one;
    ///   ``EranosError/released(detail:)`` after ``release()``.
    public func isArchived(_ id: Member.ID) throws(EranosError) -> Bool {
        try converting { try handle.isArchived(id: id.rawValue) }
    }

    /// The group's name.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func name() throws(EranosError) -> String {
        try converting { try handle.name() }
    }

    /// The payments that settle every balance by `strategy`.
    ///
    /// - Throws: ``EranosError/settlement(_:detail:)`` with
    ///   ``SettlementFailure/outOfMemory(balances:)`` for an exact plan of more open balances than
    ///   the table fits; ``EranosError/released(detail:)`` after ``release()``.
    public func settlement(by strategy: SettlementStrategy) throws(EranosError) -> Settlement {
        try converting { Settlement(try handle.settlement(strategy: strategy.generated)) }
    }

    /// Every transfer, in the order they were created.
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

    /// The group's description.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func description() throws(EranosError) -> String? {
        try converting { try handle.description() }
    }

    /// The language the apps show the group in.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func language() throws(EranosError) -> Language? {
        try converting { try handle.language().map { Language(known: $0) } }
    }

    /// The group in its saved form, as JSON, which `Group.load(_:)` reads back.
    ///
    /// - Throws: ``EranosError/released(detail:)`` after ``release()``.
    public func save() throws(EranosError) -> String {
        try converting { try handle.save() }
    }
}
