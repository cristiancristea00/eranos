package eranos;

import java.util.List;
import java.util.Optional;

/**
 * A group's reads under one shared lock, so that they see no change in between: changes wait until
 * the handle is closed.
 *
 * <p>A handle holds the lock until {@link #close}, which a {@code try}-with-resources statement
 * calls, or until the handle is collected; a blocking change on the thread that holds one waits for
 * it forever, so close it first. The reads never wait: the handle holds the lock they need.
 */
public final class ExpenseGroupReader implements AutoCloseable {
    /** The generated read handle this one forwards to. */
    private final eranos.internal.ExpenseGroupReader handle;

    ExpenseGroupReader(eranos.internal.ExpenseGroupReader handle) {
        this.handle = handle;
    }

    /**
     * Releases the lock; every read after it throws {@link IllegalStateException}, and closing a
     * closed handle does nothing.
     */
    @Override
    public void close() {
        handle.release();
    }

    /**
     * The group's identifier.
     *
     * @return the identifier
     * @throws IllegalStateException after {@link #close}
     */
    public ExpenseGroupId id() {
        return Converting.callUnfailing(() -> new ExpenseGroupId(handle.id()));
    }

    /**
     * The group's name.
     *
     * @return the text
     * @throws IllegalStateException after {@link #close}
     */
    public String name() {
        return Converting.callUnfailing(() -> handle.name());
    }

    /**
     * The group currency, which every total is in.
     *
     * @return the currency
     * @throws IllegalStateException after {@link #close}
     */
    public CurrencyCode currency() {
        return Converting.callUnfailing(() -> CurrencyCode.known(handle.currency()));
    }

    /**
     * The group's description, if it has one.
     *
     * @return the description, if the group has one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<String> description() {
        return Converting.callUnfailing(() -> Optional.ofNullable(handle.description()));
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * @return the language, if the group has one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Language> language() {
        return Converting.callUnfailing(
                () -> Optional.ofNullable(handle.language()).map(Language::known));
    }

    /**
     * The group in its saved form, as JSON, which {@link ExpenseGroup#load} reads back.
     *
     * @return the text
     * @throws IllegalStateException after {@link #close}
     */
    public String save() {
        return Converting.callUnfailing(() -> handle.save());
    }

    /**
     * Every member, archived ones included, in the order they were added.
     *
     * @return the members
     * @throws IllegalStateException after {@link #close}
     */
    public List<Member> members() {
        return Converting.callUnfailing(
                () -> handle.members().stream().map(Member::converted).toList());
    }

    /**
     * The member with the identifier {@code id}, if the group holds them.
     *
     * @param id the identifier
     * @return the member, if the group holds them
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Member> member(MemberId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.member(id.value())).map(Member::converted));
    }

    /**
     * Whether the member with the identifier {@code id} is archived; {@code false} for a member the
     * group does not hold.
     *
     * @param id the identifier
     * @return whether the member is archived
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public boolean isArchived(MemberId id) throws EranosException {
        return Converting.call(() -> handle.isArchived(id.value()));
    }

    /**
     * Every category, the built-in one first.
     *
     * @return the categories
     * @throws IllegalStateException after {@link #close}
     */
    public List<Category> categories() {
        return Converting.callUnfailing(
                () -> handle.categories().stream().map(Category::converted).toList());
    }

    /**
     * The category with the identifier {@code id}, if the group holds it.
     *
     * @param id the identifier
     * @return the category, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Category> category(CategoryId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.category(id.value())).map(Category::converted));
    }

    /**
     * Every expense, in the order they were created.
     *
     * @return the expenses
     * @throws IllegalStateException after {@link #close}
     */
    public List<Expense> expenses() {
        return Converting.callUnfailing(
                () -> handle.expenses().stream().map(Expense::converted).toList());
    }

    /**
     * The expense with the identifier {@code id}, if the group holds it.
     *
     * @param id the identifier
     * @return the expense, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Expense> expense(ExpenseId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.expense(id.value())).map(Expense::converted));
    }

    /**
     * How the expense with the identifier {@code id} divides, in the group currency, if the group
     * holds it.
     *
     * @param id the identifier
     * @return the shares, if the group holds the expense
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<ExpenseShares> expenseShares(ExpenseId id) throws EranosException {
        return Converting.call(
                () ->
                        Optional.ofNullable(handle.expenseShares(id.value()))
                                .map(ExpenseShares::converted));
    }

    /**
     * Every transfer, in the order they were created.
     *
     * @return the transfers
     * @throws IllegalStateException after {@link #close}
     */
    public List<Transfer> transfers() {
        return Converting.callUnfailing(
                () -> handle.transfers().stream().map(Transfer::converted).toList());
    }

    /**
     * The transfer with the identifier {@code id}, if the group holds it.
     *
     * @param id the identifier
     * @return the transfer, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Transfer> transfer(TransferId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.transfer(id.value())).map(Transfer::converted));
    }

    /**
     * The amount in the group currency of the transfer with the identifier {@code id}, if the group
     * holds it.
     *
     * @param id the identifier
     * @return the amount, if the group holds the transfer
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Money> transferAmount(TransferId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.transferAmount(id.value())).map(Money::converted));
    }

    /**
     * The group's totals.
     *
     * @return the totals
     * @throws IllegalStateException after {@link #close}
     */
    public Summary summary() {
        return Converting.callUnfailing(() -> Summary.converted(handle.summary()));
    }

    /**
     * The payments that settle every balance by {@code strategy}.
     *
     * @param strategy the strategy
     * @return the payments
     * @throws EranosException.Settlement with {@link SettlementFailure.OutOfMemory} for an exact
     *     plan of more open balances than its table fits
     * @throws IllegalStateException after {@link #close}
     */
    public SettlementPlan settlement(SettlementStrategy strategy) throws EranosException {
        return Converting.call(
                () -> SettlementPlan.converted(handle.settlement(strategy.generated())));
    }
}
