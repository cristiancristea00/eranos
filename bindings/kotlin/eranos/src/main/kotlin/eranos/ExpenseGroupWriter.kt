package eranos

/**
 * A group's reads and changes under one exclusive lock, so that no other call runs in between: every other call waits until
 * the handle is closed.
 *
 * [ExpenseGroup.withWrite] closes the handle when its block returns. A handle from [ExpenseGroup.write] holds the lock until
 * [close], or until the handle is collected; any blocking call of the group on the thread that holds one waits forever, since
 * the handle holds the lock the call needs and the thread cannot close it meanwhile, so close it first. The calls never wait:
 * the handle holds the lock they need.
 */
@Suppress("TooManyFunctions") // One function per call of the library: the class is its surface.
public class ExpenseGroupWriter internal constructor(
    /** The generated write handle this one forwards to. */
    private val handle: eranos.internal.ExpenseGroupWriter,
) : AutoCloseable {
    /** Releases the lock; every call after it throws [IllegalStateException], and closing a closed handle does nothing. */
    override fun close() {
        handle.release()
    }

    /**
     * The group's identifier.
     *
     * @throws IllegalStateException after [close].
     */
    public fun id(): ExpenseGroupId = converting { ExpenseGroupId(handle.id()) }

    /**
     * The group's name.
     *
     * @throws IllegalStateException after [close].
     */
    public fun name(): String = converting { handle.name() }

    /**
     * The group currency, which every total is in.
     *
     * @throws IllegalStateException after [close].
     */
    public fun currency(): CurrencyCode = converting { handle.currency().converted() }

    /**
     * The group's note.
     *
     * @throws IllegalStateException after [close].
     */
    public fun note(): String? = converting { handle.note() }

    /**
     * The language the apps show the group in.
     *
     * @throws IllegalStateException after [close].
     */
    public fun language(): Language? = converting { handle.language()?.let(::Language) }

    /**
     * The group in its saved form, as JSON, which [ExpenseGroup.load] reads back.
     *
     * @throws IllegalStateException after [close].
     */
    public fun save(): String = converting { handle.save() }

    /**
     * Every member, archived ones included, in identifier order, which is creation order for generated identifiers.
     *
     * @throws IllegalStateException after [close].
     */
    public fun members(): List<Member> = converting { handle.members().map { it.converted() } }

    /**
     * The member with the identifier [id], if the group holds them.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun member(id: MemberId): Member? = converting { handle.member(id.value)?.converted() }

    /**
     * Whether the member with the identifier [id] is archived; `null` for a member the group does not hold.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun isArchived(id: MemberId): Boolean? = converting { handle.isArchived(id.value) }

    /**
     * Every category: the built-in one first, then the others in identifier order, which is creation order for generated
     * identifiers.
     *
     * @throws IllegalStateException after [close].
     */
    public fun categories(): List<Category> = converting { handle.categories().map { it.converted() } }

    /**
     * The category with the identifier [id], if the group holds it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun category(id: CategoryId): Category? = converting { handle.category(id.value)?.converted() }

    /**
     * Every expense, in identifier order, which is creation order for generated identifiers.
     *
     * @throws IllegalStateException after [close].
     */
    public fun expenses(): List<Expense> = converting { handle.expenses().map { it.converted() } }

    /**
     * The expense with the identifier [id], if the group holds it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun expense(id: ExpenseId): Expense? = converting { handle.expense(id.value)?.converted() }

    /**
     * How the expense with the identifier [id] divides, in the group currency, if the group holds it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun expenseShares(id: ExpenseId): ExpenseShares? = converting { handle.expenseShares(id.value)?.converted() }

    /**
     * Every transfer, in identifier order, which is creation order for generated identifiers.
     *
     * @throws IllegalStateException after [close].
     */
    public fun transfers(): List<Transfer> = converting { handle.transfers().map { it.converted() } }

    /**
     * The transfer with the identifier [id], if the group holds it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun transfer(id: TransferId): Transfer? = converting { handle.transfer(id.value)?.converted() }

    /**
     * The amount in the group currency of the transfer with the identifier [id], if the group holds it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun transferAmount(id: TransferId): Money? = converting { handle.transferAmount(id.value)?.converted() }

    /**
     * The group's totals.
     *
     * @throws IllegalStateException after [close].
     */
    public fun summary(): Summary = converting { handle.summary().converted() }

    /**
     * The payments that settle every balance by [strategy].
     *
     * @throws EranosException.Settlement with [SettlementFailure.OutOfMemory] for an exact plan of more open balances than
     *   its table fits.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun settlement(strategy: SettlementStrategy): SettlementPlan = converting { handle.settlement(strategy.generated).converted() }

    /**
     * Renames the group.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.EmptyName] for a blank name.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun rename(name: String): Unit = converting { handle.setName(name) }

    /**
     * Moves the group to [currency], converting every entry with the rate [rates] gives its currency.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.MissingExchangeRate] for a currency in use without a rate, and
     *   [ExpenseGroupFailure.UnexpectedExchangeRate] for a rate of the new group currency.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun changeCurrency(
        currency: CurrencyCode,
        rates: Map<CurrencyCode, ExchangeRate>,
    ): Unit = converting { handle.changeCurrency(currency.code, rates.generated()) }

    /**
     * Sets the group's note, trimmed; `null` or blank text clears it.
     *
     * @throws IllegalStateException after [close].
     */
    public fun setNote(note: String?): Unit = converting { handle.setNote(note) }

    /**
     * Sets the language the apps show the group in, or clears it with `null`.
     *
     * @throws IllegalStateException after [close].
     */
    public fun setLanguage(language: Language?): Unit = converting { handle.setLanguage(language?.toString()) }

    /**
     * Adds [member] under their identifier and returns the identifier.
     *
     * @throws EranosException.Member with [MemberFailure.EmptyFirstName] for a blank first name.
     * @throws EranosException.PaymentAccount with [PaymentAccountFailure.EmptyName] for an account with a blank name.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.DuplicateMember] for an identifier another member has.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun add(member: Member): MemberId = converting { MemberId(handle.addMember(member.generated)) }

    /**
     * Replaces the details of the member that [member] names by their identifier, and returns them as they were; an
     * archived member stays archived.
     *
     * @throws EranosException.Member as adding one does.
     * @throws EranosException.PaymentAccount as adding one does.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun replace(member: Member): Member = converting { handle.replaceMember(member.generated).converted() }

    /**
     * Archives the member with the identifier [id]: they keep their history and balance, and join no new expense.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun archive(id: MemberId): Unit = converting { handle.archiveMember(id.value) }

    /**
     * Makes the archived member with the identifier [id] active again.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun unarchive(id: MemberId): Unit = converting { handle.unarchiveMember(id.value) }

    /**
     * Removes the member with the identifier [id], who appears in no entry, and returns them.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownMember], and [ExpenseGroupFailure.MemberInUse] with the entries that
     *   still name them; archive such a member instead.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun remove(id: MemberId): Member = converting { handle.removeMember(id.value).converted() }

    /**
     * Adds [category] under its identifier and returns the identifier.
     *
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.DuplicateCategory] for an identifier another category has,
     *   the built-in one's included, and [ExpenseGroupFailure.DuplicateCategoryName] for a name another category has, compared
     *   caselessly.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun add(category: Category): CategoryId = converting { CategoryId(handle.addCategory(category.generated)) }

    /**
     * Renames the category that [category] names by its identifier, and returns it as it was.
     *
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownCategory], [ExpenseGroupFailure.BuiltInCategory], or
     *   [ExpenseGroupFailure.DuplicateCategoryName].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun replace(category: Category): Category = converting { handle.replaceCategory(category.generated).converted() }

    /**
     * Removes the category with the identifier [id] and returns it; its expenses move to the built-in one.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownCategory] or [ExpenseGroupFailure.BuiltInCategory].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun remove(id: CategoryId): Category = converting { handle.removeCategory(id.value).converted() }

    /**
     * Adds [expense] under its identifier and returns the identifier.
     *
     * @throws EranosException.Expense for an expense that does not build, such as one whose amounts do not add up.
     * @throws EranosException.ExpenseGroup for one that does not fit the group, such as [ExpenseGroupFailure.UnknownMember] or
     *   [ExpenseGroupFailure.ArchivedMember], and with [ExpenseGroupFailure.DuplicateExpense] for an identifier another expense
     *   has.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun add(expense: Expense): ExpenseId = converting { ExpenseId(handle.addExpense(expense.generated)) }

    /**
     * Replaces the expense that [expense] names by its identifier, and returns it as it was.
     *
     * @throws EranosException.Expense as adding one does.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownExpense], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun replace(expense: Expense): Expense = converting { handle.replaceExpense(expense.generated).converted() }

    /**
     * Removes the expense with the identifier [id] and returns it.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownExpense].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun remove(id: ExpenseId): Expense = converting { handle.removeExpense(id.value).converted() }

    /**
     * Adds [transfer] under its identifier and returns the identifier.
     *
     * @throws EranosException.Transfer for a transfer that does not build, such as [TransferFailure.SameMember].
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownMember], and [ExpenseGroupFailure.DuplicateTransfer]
     *   for an identifier another transfer has.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun add(transfer: Transfer): TransferId = converting { TransferId(handle.addTransfer(transfer.generated)) }

    /**
     * Replaces the transfer that [transfer] names by its identifier, and returns it as it was.
     *
     * @throws EranosException.Transfer as adding one does.
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownTransfer], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun replace(transfer: Transfer): Transfer = converting { handle.replaceTransfer(transfer.generated).converted() }

    /**
     * Removes the transfer with the identifier [id] and returns it.
     *
     * @throws EranosException.ExpenseGroup with [ExpenseGroupFailure.UnknownTransfer].
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun remove(id: TransferId): Transfer = converting { handle.removeTransfer(id.value).converted() }
}
