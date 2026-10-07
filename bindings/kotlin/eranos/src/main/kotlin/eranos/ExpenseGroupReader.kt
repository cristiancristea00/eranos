package eranos

/**
 * A group's reads under one shared lock, so that they see no change in between: changes wait until the handle is closed.
 *
 * [ExpenseGroup.withRead] closes the handle when its block returns. A handle from [ExpenseGroup.read] holds the lock until [close], or
 * until the handle is collected; a blocking change on the thread that holds one waits for it forever, so close it first.
 * The reads never wait: the handle holds the lock they need.
 */
@Suppress("TooManyFunctions") // One function per read of the library: the class is its surface.
public class ExpenseGroupReader internal constructor(
    /** The generated read handle this one forwards to. */
    private val handle: eranos.internal.ExpenseGroupReader,
) : AutoCloseable {
    /** Releases the lock; every read after it throws [IllegalStateException], and closing a closed handle does nothing. */
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
    public fun currency(): Currency = converting { Currency.known(handle.currency()) }

    /**
     * The group's description.
     *
     * @throws IllegalStateException after [close].
     */
    public fun description(): String? = converting { handle.description() }

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
     * Every member, archived ones included, in the order they were added.
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
     * Whether the member with the identifier [id] is archived; `false` for a member the group does not hold.
     *
     * @throws EranosException.Id for an identifier that is not one.
     * @throws IllegalStateException after [close].
     */
    @Throws(EranosException::class)
    public fun isArchived(id: MemberId): Boolean = converting { handle.isArchived(id.value) }

    /**
     * Every category, the built-in one first.
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
     * Every expense, in the order they were created.
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
     * Every transfer, in the order they were created.
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
    public fun settlement(strategy: SettlementStrategy): Settlement = converting { handle.settlement(strategy.generated).converted() }
}
