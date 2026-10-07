package eranos

import eranos.internal.CurrencyRate
import eranos.internal.NewGroup
import eranos.internal.loadGroupAsync
import kotlin.coroutines.cancellation.CancellationException

/**
 * A group of people who share expenses, safe to share between coroutines and threads.
 *
 * Every call comes in two forms. The suspending one waits for the group's lock without blocking its thread, and cancelling
 * its coroutine cancels the call, leaving the group as it was; its twin, whose name ends in `Blocking`, waits by blocking
 * the thread, for code outside a coroutine:
 *
 * ```kotlin
 * val group = Group("Flat", Currency.of("EUR"))
 * val ana = group.addBlocking(Member("Ana")) // outside a coroutine
 * val bob = group.add(Member("Bob")) // in a coroutine
 * ```
 *
 * Reads share the lock and run together; a change takes it alone, after the reads in progress. [close] frees the group;
 * every call after it throws [IllegalStateException].
 */
@Suppress("TooManyFunctions") // One function per call of the library, twice over: the class is its surface.
public class Group private constructor(
    /** The generated group this one forwards to. */
    private val handle: eranos.internal.Group,
) : AutoCloseable {
    /**
     * A new group with no members and no entries, called [name], whose totals are in [currency].
     *
     * @throws EranosException.Group with [GroupFailure.EmptyName] for a blank name.
     */
    @Throws(EranosException::class)
    public constructor(
        name: String,
        currency: Currency,
        description: String? = null,
        language: Language? = null,
    ) : this(converting { eranos.internal.Group(NewGroup(name, currency.code, description, language?.toString())) })

    /** Frees the group; every call after it throws [IllegalStateException], and closing a closed group does nothing. */
    override fun close() {
        handle.close()
    }

    /**
     * A read handle, which holds the shared lock until it is closed.
     *
     * Waits for the lock without blocking its thread; cancelling the coroutine cancels the wait. [readBlocking] blocks
     * instead.
     */
    public suspend fun read(): GroupReader = converting { GroupReader(handle.readAsync(null)) }

    /**
     * A read handle, which holds the shared lock until it is closed.
     *
     * Waits for the lock by blocking its thread; in a coroutine, [read] waits without blocking it.
     */
    public fun readBlocking(): GroupReader = converting { GroupReader(handle.read()) }

    /**
     * Runs [block] with a read handle, and closes the handle when [block] returns or throws, so that every read of [block]
     * sees the group as one state.
     *
     * Waits for the lock without blocking its thread.
     */
    public suspend inline fun <T> withRead(block: (GroupReader) -> T): T = read().use(block)

    /**
     * Runs [block] with a read handle, and closes the handle when [block] returns or throws, so that every read of [block]
     * sees the group as one state.
     *
     * Waits for the lock by blocking its thread.
     */
    public inline fun <T> withReadBlocking(block: (GroupReader) -> T): T = readBlocking().use(block)

    // The group's own details: its identifier, name, currency, description, language, and saved form.

    /**
     * The group's identifier.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [idBlocking] blocks instead.
     */
    public suspend fun id(): GroupId = converting { GroupId(handle.idAsync(null)) }

    /**
     * The group's identifier.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [id] waits without blocking it.
     */
    public fun idBlocking(): GroupId = converting { GroupId(handle.id()) }

    /**
     * The group's name.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [nameBlocking] blocks instead.
     */
    public suspend fun name(): String = converting { handle.nameAsync(null) }

    /**
     * The group's name.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [name] waits without blocking it.
     */
    public fun nameBlocking(): String = converting { handle.name() }

    /**
     * Renames the group.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [renameBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.EmptyName] for a blank name.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun rename(name: String): Unit = converting { handle.setNameAsync(name, null) }

    /**
     * Renames the group.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [rename] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.EmptyName] for a blank name.
     */
    @Throws(EranosException::class)
    public fun renameBlocking(name: String): Unit = converting { handle.setName(name) }

    /**
     * The group currency, which every total is in.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [currencyBlocking] blocks instead.
     */
    public suspend fun currency(): Currency = converting { Currency.known(handle.currencyAsync(null)) }

    /**
     * The group currency, which every total is in.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [currency] waits without blocking it.
     */
    public fun currencyBlocking(): Currency = converting { Currency.known(handle.currency()) }

    /**
     * Moves the group to [currency], converting every entry with the rate [rates] gives its currency.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [changeCurrencyBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.MissingExchangeRate] for a currency in use without a rate, and
     *   [GroupFailure.UnexpectedExchangeRate] for a rate of the new group currency.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun changeCurrency(
        currency: Currency,
        rates: Map<Currency, ExchangeRate>,
    ): Unit = converting { handle.changeCurrencyAsync(currency.code, rates.generated(), null) }

    /**
     * Moves the group to [currency], converting every entry with the rate [rates] gives its currency.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [changeCurrency] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.MissingExchangeRate] for a currency in use without a rate, and
     *   [GroupFailure.UnexpectedExchangeRate] for a rate of the new group currency.
     */
    @Throws(EranosException::class)
    public fun changeCurrencyBlocking(
        currency: Currency,
        rates: Map<Currency, ExchangeRate>,
    ): Unit = converting { handle.changeCurrency(currency.code, rates.generated()) }

    /**
     * The group's description.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [descriptionBlocking] blocks instead.
     */
    public suspend fun description(): String? = converting { handle.descriptionAsync(null) }

    /**
     * The group's description.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [description] waits without blocking it.
     */
    public fun descriptionBlocking(): String? = converting { handle.description() }

    /**
     * Sets the group's description; `null` or blank text clears it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [setDescriptionBlocking] blocks instead.
     */
    public suspend fun setDescription(description: String?): Unit = converting { handle.setDescriptionAsync(description.orEmpty(), null) }

    /**
     * Sets the group's description; `null` or blank text clears it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [setDescription] waits without blocking it.
     */
    public fun setDescriptionBlocking(description: String?): Unit = converting { handle.setDescription(description.orEmpty()) }

    /**
     * The language the apps show the group in.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [languageBlocking] blocks instead.
     */
    public suspend fun language(): Language? = converting { handle.languageAsync(null)?.let(::Language) }

    /**
     * The language the apps show the group in.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [language] waits without blocking it.
     */
    public fun languageBlocking(): Language? = converting { handle.language()?.let(::Language) }

    /**
     * Sets the language the apps show the group in, or clears it with `null`.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [setLanguageBlocking] blocks instead.
     */
    public suspend fun setLanguage(language: Language?): Unit = converting { handle.setLanguageAsync(language?.toString(), null) }

    /**
     * Sets the language the apps show the group in, or clears it with `null`.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [setLanguage] waits without blocking it.
     */
    public fun setLanguageBlocking(language: Language?): Unit = converting { handle.setLanguage(language?.toString()) }

    /**
     * The group in its saved form, as JSON, which [Group.load] reads back.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [saveBlocking] blocks instead.
     */
    public suspend fun save(): String = converting { handle.saveAsync(null) }

    /**
     * The group in its saved form, as JSON, which [Group.load] reads back.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [save] waits without blocking it.
     */
    public fun saveBlocking(): String = converting { handle.save() }

    // The group's members.

    /**
     * Every member, archived ones included, in the order they were added.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [membersBlocking] blocks instead.
     */
    public suspend fun members(): List<Member> = converting { handle.membersAsync(null).map { it.converted() } }

    /**
     * Every member, archived ones included, in the order they were added.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [members] waits without blocking it.
     */
    public fun membersBlocking(): List<Member> = converting { handle.members().map { it.converted() } }

    /**
     * The member with the identifier [id], if the group holds them.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [memberBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun member(id: MemberId): Member? = converting { handle.memberAsync(id.value, null)?.converted() }

    /**
     * The member with the identifier [id], if the group holds them.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [member] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun memberBlocking(id: MemberId): Member? = converting { handle.member(id.value)?.converted() }

    /**
     * Whether the member with the identifier [id] is archived; `false` for a member the group does not hold.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [isArchivedBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun isArchived(id: MemberId): Boolean = converting { handle.isArchivedAsync(id.value, null) }

    /**
     * Whether the member with the identifier [id] is archived; `false` for a member the group does not hold.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [isArchived] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun isArchivedBlocking(id: MemberId): Boolean = converting { handle.isArchived(id.value) }

    /**
     * Adds [member], who has no identifier yet, and returns the one the group gives them.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [addBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a member with an identifier.
     * @throws EranosException.Member with [MemberFailure.EmptyFirstName] for a blank first name.
     * @throws EranosException.Account with [AccountFailure.EmptyName] for an account with a blank name.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun add(member: Member): MemberId = converting { MemberId(handle.addMemberAsync(member.generated, null)) }

    /**
     * Adds [member], who has no identifier yet, and returns the one the group gives them.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [add] waits without blocking it.
     *
     * @throws IllegalArgumentException for a member with an identifier.
     * @throws EranosException.Member with [MemberFailure.EmptyFirstName] for a blank first name.
     * @throws EranosException.Account with [AccountFailure.EmptyName] for an account with a blank name.
     */
    @Throws(EranosException::class)
    public fun addBlocking(member: Member): MemberId = converting { MemberId(handle.addMember(member.generated)) }

    /**
     * Replaces the details of the member that [member] names by their identifier, and returns them as they were; an
     * archived member stays archived.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [replaceBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a member without an identifier.
     * @throws EranosException.Member as adding one does.
     * @throws EranosException.Account as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun replace(member: Member): Member = converting { handle.replaceMemberAsync(member.generated, null).converted() }

    /**
     * Replaces the details of the member that [member] names by their identifier, and returns them as they were; an
     * archived member stays archived.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [replace] waits without blocking it.
     *
     * @throws IllegalArgumentException for a member without an identifier.
     * @throws EranosException.Member as adding one does.
     * @throws EranosException.Account as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun replaceBlocking(member: Member): Member = converting { handle.replaceMember(member.generated).converted() }

    /**
     * Archives the member with the identifier [id]: they keep their history and balance, and join no new expense.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [archiveBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun archive(id: MemberId): Unit = converting { handle.archiveMemberAsync(id.value, null) }

    /**
     * Archives the member with the identifier [id]: they keep their history and balance, and join no new expense.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [archive] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun archiveBlocking(id: MemberId): Unit = converting { handle.archiveMember(id.value) }

    /**
     * Makes the archived member with the identifier [id] active again.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [unarchiveBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun unarchive(id: MemberId): Unit = converting { handle.unarchiveMemberAsync(id.value, null) }

    /**
     * Makes the archived member with the identifier [id] active again.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [unarchive] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun unarchiveBlocking(id: MemberId): Unit = converting { handle.unarchiveMember(id.value) }

    /**
     * Removes the member with the identifier [id], who appears in no entry, and returns them.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [removeBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember], and [GroupFailure.MemberInUse] with the entries that
     *   still name them; archive such a member instead.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun remove(id: MemberId): Member = converting { handle.removeMemberAsync(id.value, null).converted() }

    /**
     * Removes the member with the identifier [id], who appears in no entry, and returns them.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [remove] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownMember], and [GroupFailure.MemberInUse] with the entries that
     *   still name them; archive such a member instead.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun removeBlocking(id: MemberId): Member = converting { handle.removeMember(id.value).converted() }

    // The group's categories.

    /**
     * Every category, the built-in one first.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [categoriesBlocking] blocks instead.
     */
    public suspend fun categories(): List<Category> = converting { handle.categoriesAsync(null).map { it.converted() } }

    /**
     * Every category, the built-in one first.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [categories] waits without blocking it.
     */
    public fun categoriesBlocking(): List<Category> = converting { handle.categories().map { it.converted() } }

    /**
     * The category with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [categoryBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun category(id: CategoryId): Category? = converting { handle.categoryAsync(id.value, null)?.converted() }

    /**
     * The category with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [category] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun categoryBlocking(id: CategoryId): Category? = converting { handle.category(id.value)?.converted() }

    /**
     * Adds [category], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [addBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a category with an identifier.
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.Group with [GroupFailure.DuplicateCategoryName] for a name another category has, compared
     *   caselessly.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun add(category: Category): CategoryId = converting { CategoryId(handle.addCategoryAsync(category.generated, null)) }

    /**
     * Adds [category], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [add] waits without blocking it.
     *
     * @throws IllegalArgumentException for a category with an identifier.
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.Group with [GroupFailure.DuplicateCategoryName] for a name another category has, compared
     *   caselessly.
     */
    @Throws(EranosException::class)
    public fun addBlocking(category: Category): CategoryId = converting { CategoryId(handle.addCategory(category.generated)) }

    /**
     * Renames the category that [category] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [replaceBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a category without an identifier.
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.Group with [GroupFailure.UnknownCategory], [GroupFailure.BuiltInCategory], or
     *   [GroupFailure.DuplicateCategoryName].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun replace(category: Category): Category = converting { handle.replaceCategoryAsync(category.generated, null).converted() }

    /**
     * Renames the category that [category] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [replace] waits without blocking it.
     *
     * @throws IllegalArgumentException for a category without an identifier.
     * @throws EranosException.Category with [CategoryFailure.EmptyName] for a blank name.
     * @throws EranosException.Group with [GroupFailure.UnknownCategory], [GroupFailure.BuiltInCategory], or
     *   [GroupFailure.DuplicateCategoryName].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun replaceBlocking(category: Category): Category = converting { handle.replaceCategory(category.generated).converted() }

    /**
     * Removes the category with the identifier [id] and returns it; its expenses move to the built-in one.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [removeBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownCategory] or [GroupFailure.BuiltInCategory].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun remove(id: CategoryId): Category = converting { handle.removeCategoryAsync(id.value, null).converted() }

    /**
     * Removes the category with the identifier [id] and returns it; its expenses move to the built-in one.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [remove] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownCategory] or [GroupFailure.BuiltInCategory].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun removeBlocking(id: CategoryId): Category = converting { handle.removeCategory(id.value).converted() }

    // The group's expenses.

    /**
     * Every expense, in the order they were created.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [expensesBlocking] blocks instead.
     */
    public suspend fun expenses(): List<Expense> = converting { handle.expensesAsync(null).map { it.converted() } }

    /**
     * Every expense, in the order they were created.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [expenses] waits without blocking it.
     */
    public fun expensesBlocking(): List<Expense> = converting { handle.expenses().map { it.converted() } }

    /**
     * The expense with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [expenseBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun expense(id: ExpenseId): Expense? = converting { handle.expenseAsync(id.value, null)?.converted() }

    /**
     * The expense with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [expense] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun expenseBlocking(id: ExpenseId): Expense? = converting { handle.expense(id.value)?.converted() }

    /**
     * How the expense with the identifier [id] divides, in the group currency, if the group holds it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [expenseSharesBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun expenseShares(id: ExpenseId): ExpenseShares? = converting { handle.expenseSharesAsync(id.value, null)?.converted() }

    /**
     * How the expense with the identifier [id] divides, in the group currency, if the group holds it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [expenseShares] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun expenseSharesBlocking(id: ExpenseId): ExpenseShares? = converting { handle.expenseShares(id.value)?.converted() }

    /**
     * Adds [expense], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [addBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for an expense with an identifier.
     * @throws EranosException.Expense for an expense that does not build, such as one whose amounts do not add up.
     * @throws EranosException.Group for one that does not fit the group, such as [GroupFailure.UnknownMember] or
     *   [GroupFailure.ArchivedMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun add(expense: Expense): ExpenseId = converting { ExpenseId(handle.addExpenseAsync(expense.generated, null)) }

    /**
     * Adds [expense], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [add] waits without blocking it.
     *
     * @throws IllegalArgumentException for an expense with an identifier.
     * @throws EranosException.Expense for an expense that does not build, such as one whose amounts do not add up.
     * @throws EranosException.Group for one that does not fit the group, such as [GroupFailure.UnknownMember] or
     *   [GroupFailure.ArchivedMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun addBlocking(expense: Expense): ExpenseId = converting { ExpenseId(handle.addExpense(expense.generated)) }

    /**
     * Replaces the expense that [expense] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [replaceBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for an expense without an identifier.
     * @throws EranosException.Expense as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownExpense], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun replace(expense: Expense): Expense = converting { handle.replaceExpenseAsync(expense.generated, null).converted() }

    /**
     * Replaces the expense that [expense] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [replace] waits without blocking it.
     *
     * @throws IllegalArgumentException for an expense without an identifier.
     * @throws EranosException.Expense as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownExpense], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun replaceBlocking(expense: Expense): Expense = converting { handle.replaceExpense(expense.generated).converted() }

    /**
     * Removes the expense with the identifier [id] and returns it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [removeBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownExpense].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun remove(id: ExpenseId): Expense = converting { handle.removeExpenseAsync(id.value, null).converted() }

    /**
     * Removes the expense with the identifier [id] and returns it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [remove] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownExpense].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun removeBlocking(id: ExpenseId): Expense = converting { handle.removeExpense(id.value).converted() }

    // The group's transfers.

    /**
     * Every transfer, in the order they were created.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [transfersBlocking] blocks instead.
     */
    public suspend fun transfers(): List<Transfer> = converting { handle.transfersAsync(null).map { it.converted() } }

    /**
     * Every transfer, in the order they were created.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [transfers] waits without blocking it.
     */
    public fun transfersBlocking(): List<Transfer> = converting { handle.transfers().map { it.converted() } }

    /**
     * The transfer with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [transferBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun transfer(id: TransferId): Transfer? = converting { handle.transferAsync(id.value, null)?.converted() }

    /**
     * The transfer with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [transfer] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun transferBlocking(id: TransferId): Transfer? = converting { handle.transfer(id.value)?.converted() }

    /**
     * The amount in the group currency of the transfer with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [transferAmountBlocking] blocks instead.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun transferAmount(id: TransferId): Money? = converting { handle.transferAmountAsync(id.value, null)?.converted() }

    /**
     * The amount in the group currency of the transfer with the identifier [id], if the group holds it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [transferAmount] waits without blocking it.
     *
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun transferAmountBlocking(id: TransferId): Money? = converting { handle.transferAmount(id.value)?.converted() }

    /**
     * Adds [transfer], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [addBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a transfer with an identifier.
     * @throws EranosException.Transfer for a transfer that does not build, such as [TransferFailure.SameMember].
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun add(transfer: Transfer): TransferId = converting { TransferId(handle.addTransferAsync(transfer.generated, null)) }

    /**
     * Adds [transfer], which has no identifier yet, and returns the one the group gives it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [add] waits without blocking it.
     *
     * @throws IllegalArgumentException for a transfer with an identifier.
     * @throws EranosException.Transfer for a transfer that does not build, such as [TransferFailure.SameMember].
     * @throws EranosException.Group with [GroupFailure.UnknownMember].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun addBlocking(transfer: Transfer): TransferId = converting { TransferId(handle.addTransfer(transfer.generated)) }

    /**
     * Replaces the transfer that [transfer] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [replaceBlocking] blocks instead.
     *
     * @throws IllegalArgumentException for a transfer without an identifier.
     * @throws EranosException.Transfer as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownTransfer], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun replace(transfer: Transfer): Transfer = converting { handle.replaceTransferAsync(transfer.generated, null).converted() }

    /**
     * Replaces the transfer that [transfer] names by its identifier, and returns it as it was.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [replace] waits without blocking it.
     *
     * @throws IllegalArgumentException for a transfer without an identifier.
     * @throws EranosException.Transfer as adding one does.
     * @throws EranosException.Group with [GroupFailure.UnknownTransfer], and as adding one does.
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun replaceBlocking(transfer: Transfer): Transfer = converting { handle.replaceTransfer(transfer.generated).converted() }

    /**
     * Removes the transfer with the identifier [id] and returns it.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [removeBlocking] blocks instead.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownTransfer].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun remove(id: TransferId): Transfer = converting { handle.removeTransferAsync(id.value, null).converted() }

    /**
     * Removes the transfer with the identifier [id] and returns it.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [remove] waits without blocking it.
     *
     * @throws EranosException.Group with [GroupFailure.UnknownTransfer].
     * @throws EranosException.Id for an identifier that is not one.
     */
    @Throws(EranosException::class)
    public fun removeBlocking(id: TransferId): Transfer = converting { handle.removeTransfer(id.value).converted() }

    // The group's totals and its settlements.

    /**
     * The group's totals.
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [summaryBlocking] blocks instead.
     */
    public suspend fun summary(): Summary = converting { handle.summaryAsync(null).converted() }

    /**
     * The group's totals.
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [summary] waits without blocking it.
     */
    public fun summaryBlocking(): Summary = converting { handle.summary().converted() }

    /**
     * The payments that settle every balance by [strategy].
     *
     * Waits for the group's lock without blocking its thread; cancelling the coroutine cancels the call, which leaves the
     * group as it was. [settlementBlocking] blocks instead.
     *
     * @throws EranosException.Settlement with [SettlementFailure.OutOfMemory] for an exact plan of more open balances than
     *   its table fits.
     */
    @Throws(EranosException::class, CancellationException::class)
    public suspend fun settlement(strategy: SettlementStrategy): Settlement =
        converting { handle.settlementAsync(strategy.generated, null).converted() }

    /**
     * The payments that settle every balance by [strategy].
     *
     * Waits for the group's lock by blocking its thread; in a coroutine, [settlement] waits without blocking it.
     *
     * @throws EranosException.Settlement with [SettlementFailure.OutOfMemory] for an exact plan of more open balances than
     *   its table fits.
     */
    @Throws(EranosException::class)
    public fun settlementBlocking(strategy: SettlementStrategy): Settlement = converting { handle.settlement(strategy.generated).converted() }

    /** The loading of a saved group. */
    public companion object {
        /**
         * The group saved as [json], loaded through the checks that adding its entries makes.
         *
         * Yields while it replays the entries; cancelling the coroutine cancels the load. [loadBlocking] blocks instead.
         *
         * @throws EranosException.InvalidJson for text that is not the saved form.
         * @throws EranosException.Group with [GroupFailure.UnsupportedFormat] for a format this release cannot read, and
         *   the failures of entries that do not fit together.
         */
        @Throws(EranosException::class, CancellationException::class)
        public suspend fun load(json: String): Group = converting { Group(loadGroupAsync(json, null)) }

        /**
         * The group saved as [json], loaded through the checks that adding its entries makes.
         *
         * Blocks its thread until the load completes; in a coroutine, [load] yields while it replays the entries.
         *
         * @throws EranosException.InvalidJson for text that is not the saved form.
         * @throws EranosException.Group with [GroupFailure.UnsupportedFormat] for a format this release cannot read, and
         *   the failures of entries that do not fit together.
         */
        @Throws(EranosException::class)
        public fun loadBlocking(json: String): Group = converting { Group(eranos.internal.Group.load(json)) }
    }
}

/** The rates as the generated code takes them: each currency beside its rate into the group currency. */
private fun Map<Currency, ExchangeRate>.generated(): List<CurrencyRate> = map { (currency, rate) -> CurrencyRate(currency.code, rate.toString()) }
