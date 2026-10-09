package eranos;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

/**
 * A group's reads and changes under one exclusive lock, so that no other call runs in between:
 * every other call waits until the handle is closed.
 *
 * <p>A handle holds the lock until {@link #close}, which a {@code try}-with-resources statement
 * calls, or until the handle is collected; any blocking call of the group on the thread that holds
 * one waits forever, since the handle holds the lock the call needs and the thread cannot close it
 * meanwhile, so close it first. The calls never wait: the handle holds the lock they need.
 */
public final class ExpenseGroupWriter implements AutoCloseable {
    /** The generated write handle this one forwards to. */
    private final eranos.internal.ExpenseGroupWriter handle;

    ExpenseGroupWriter(eranos.internal.ExpenseGroupWriter handle) {
        this.handle = handle;
    }

    /**
     * Releases the lock; every call after it throws {@link IllegalStateException}, and closing a
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
        return Converting.callUnfailing(() -> CurrencyCode.converted(handle.currency()));
    }

    /**
     * The group's note, if it has one.
     *
     * @return the note, if the group has one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<String> note() {
        return Converting.callUnfailing(() -> Optional.ofNullable(handle.note()));
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
     * Every member, archived ones included, in identifier order, which is creation order for
     * generated identifiers.
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
     * Whether the member with the identifier {@code id} is archived; empty for a member the group
     * does not hold.
     *
     * @param id the identifier
     * @return whether the member is archived, if the group holds them
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Optional<Boolean> isArchived(MemberId id) throws EranosException {
        return Converting.call(() -> Optional.ofNullable(handle.isArchived(id.value())));
    }

    /**
     * Every category: the built-in one first, then the others in identifier order, which is
     * creation order for generated identifiers.
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
     * Every expense, in identifier order, which is creation order for generated identifiers.
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
     * Every transfer, in identifier order, which is creation order for generated identifiers.
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

    /**
     * Renames the group.
     *
     * @param name the new name
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.EmptyName} for a blank
     *     name
     * @throws IllegalStateException after {@link #close}
     */
    public void setName(String name) throws EranosException {
        Converting.run(() -> handle.setName(name));
    }

    /**
     * Moves the group to {@code currency}, converting every entry with the rate {@code rates} gives
     * its currency.
     *
     * @param currency the new group currency
     * @param rates the rate into the new group currency of every other currency the entries use
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.MissingExchangeRate} for
     *     a currency in use without a rate, and {@link ExpenseGroupFailure.UnexpectedExchangeRate}
     *     for a rate of the new group currency
     * @throws IllegalStateException after {@link #close}
     */
    public void changeCurrency(CurrencyCode currency, Map<CurrencyCode, ExchangeRate> rates)
            throws EranosException {
        Converting.run(() -> handle.changeCurrency(currency.code(), ExpenseGroup.rates(rates)));
    }

    /**
     * Sets the group's note, trimmed; blank text clears it, as {@link #clearNote} does.
     *
     * @param note the new note
     * @throws IllegalStateException after {@link #close}
     */
    public void setNote(String note) {
        Objects.requireNonNull(note, "note");
        Converting.runUnfailing(() -> handle.setNote(note));
    }

    /**
     * Clears the group's note, which {@link #setNote} sets.
     *
     * @throws IllegalStateException after {@link #close}
     */
    public void clearNote() {
        Converting.runUnfailing(() -> handle.setNote(null));
    }

    /**
     * Sets the language the apps show the group in.
     *
     * @param language the new language
     * @throws IllegalStateException after {@link #close}
     */
    public void setLanguage(Language language) {
        Converting.runUnfailing(() -> handle.setLanguage(language.toString()));
    }

    /**
     * Clears the language the apps show the group in.
     *
     * @throws IllegalStateException after {@link #close}
     */
    public void clearLanguage() {
        Converting.runUnfailing(() -> handle.setLanguage(null));
    }

    /**
     * Adds {@code member} under their identifier, and returns the identifier.
     *
     * @param member the member
     * @return the member's identifier
     * @throws EranosException.Member with {@link MemberFailure.EmptyFirstName} for a blank first
     *     name
     * @throws EranosException.PaymentAccount with {@link PaymentAccountFailure.EmptyName} for an
     *     account with a blank name
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.DuplicateMember} for an
     *     identifier the group holds
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public MemberId addMember(Member member) throws EranosException {
        return Converting.call(() -> new MemberId(handle.addMember(member.generated())));
    }

    /**
     * Replaces the details of the member that {@code member} names by their identifier, and returns
     * them as they were; an archived member stays archived.
     *
     * @param member the member
     * @return the member as they were
     * @throws EranosException.Member as adding one does
     * @throws EranosException.PaymentAccount as adding one does
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownMember} for an
     *     identifier the group does not hold
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Member replaceMember(Member member) throws EranosException {
        return Converting.call(() -> Member.converted(handle.replaceMember(member.generated())));
    }

    /**
     * Archives the member with the identifier {@code id}: they keep their history and balance, and
     * join no new expense.
     *
     * @param id the identifier
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public void archiveMember(MemberId id) throws EranosException {
        Converting.run(() -> handle.archiveMember(id.value()));
    }

    /**
     * Makes the archived member with the identifier {@code id} active again.
     *
     * @param id the identifier
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public void unarchiveMember(MemberId id) throws EranosException {
        Converting.run(() -> handle.unarchiveMember(id.value()));
    }

    /**
     * Removes the member with the identifier {@code id}, who appears in no entry, and returns them.
     *
     * @param id the identifier
     * @return the member as they were
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownMember}, and
     *     {@link ExpenseGroupFailure.MemberInUse} with the entries that still name them; archive
     *     such a member instead
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Member removeMember(MemberId id) throws EranosException {
        return Converting.call(() -> Member.converted(handle.removeMember(id.value())));
    }

    /**
     * Adds {@code category} under its identifier, and returns the identifier.
     *
     * @param category the category
     * @return the category's identifier
     * @throws EranosException.Category with {@link CategoryFailure.EmptyName} for a blank name
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.DuplicateCategory} for
     *     an identifier the group holds, and {@link ExpenseGroupFailure.DuplicateCategoryName} for
     *     a name another category has, compared caselessly
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public CategoryId addCategory(Category category) throws EranosException {
        return Converting.call(() -> new CategoryId(handle.addCategory(category.generated())));
    }

    /**
     * Renames the category that {@code category} names by its identifier, and returns it as it was.
     *
     * @param category the category
     * @return the category as it was
     * @throws EranosException.Category with {@link CategoryFailure.EmptyName} for a blank name
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownCategory} for an
     *     identifier the group does not hold, {@link ExpenseGroupFailure.BuiltInCategory}, or
     *     {@link ExpenseGroupFailure.DuplicateCategoryName}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Category replaceCategory(Category category) throws EranosException {
        return Converting.call(
                () -> Category.converted(handle.replaceCategory(category.generated())));
    }

    /**
     * Removes the category with the identifier {@code id} and returns it; its expenses move to the
     * built-in one.
     *
     * @param id the identifier
     * @return the category as it was
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownCategory} or
     *     {@link ExpenseGroupFailure.BuiltInCategory}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Category removeCategory(CategoryId id) throws EranosException {
        return Converting.call(() -> Category.converted(handle.removeCategory(id.value())));
    }

    /**
     * Adds {@code expense} under its identifier, and returns the identifier.
     *
     * @param expense the expense
     * @return the expense's identifier
     * @throws EranosException.Expense for an expense that does not build, such as one whose amounts
     *     do not add up
     * @throws EranosException.ExpenseGroup for one that does not fit the group, such as {@link
     *     ExpenseGroupFailure.DuplicateExpense} for an identifier the group holds, {@link
     *     ExpenseGroupFailure.UnknownMember}, or {@link ExpenseGroupFailure.ArchivedMember}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public ExpenseId addExpense(Expense expense) throws EranosException {
        return Converting.call(() -> new ExpenseId(handle.addExpense(expense.generated())));
    }

    /**
     * Replaces the expense that {@code expense} names by its identifier, and returns it as it was.
     *
     * @param expense the expense
     * @return the expense as it was
     * @throws EranosException.Expense as adding one does
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownExpense} for an
     *     identifier the group does not hold, and as adding one does
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Expense replaceExpense(Expense expense) throws EranosException {
        return Converting.call(() -> Expense.converted(handle.replaceExpense(expense.generated())));
    }

    /**
     * Removes the expense with the identifier {@code id} and returns it.
     *
     * @param id the identifier
     * @return the expense as it was
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownExpense}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Expense removeExpense(ExpenseId id) throws EranosException {
        return Converting.call(() -> Expense.converted(handle.removeExpense(id.value())));
    }

    /**
     * Adds {@code transfer} under its identifier, and returns the identifier.
     *
     * @param transfer the transfer
     * @return the transfer's identifier
     * @throws EranosException.Transfer for a transfer that does not build, such as {@link
     *     TransferFailure.SameMember}
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.DuplicateTransfer} for
     *     an identifier the group holds, and {@link ExpenseGroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public TransferId addTransfer(Transfer transfer) throws EranosException {
        return Converting.call(() -> new TransferId(handle.addTransfer(transfer.generated())));
    }

    /**
     * Replaces the transfer that {@code transfer} names by its identifier, and returns it as it
     * was.
     *
     * @param transfer the transfer
     * @return the transfer as it was
     * @throws EranosException.Transfer as adding one does
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownTransfer} for an
     *     identifier the group does not hold, and as adding one does
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Transfer replaceTransfer(Transfer transfer) throws EranosException {
        return Converting.call(
                () -> Transfer.converted(handle.replaceTransfer(transfer.generated())));
    }

    /**
     * Removes the transfer with the identifier {@code id} and returns it.
     *
     * @param id the identifier
     * @return the transfer as it was
     * @throws EranosException.ExpenseGroup with {@link ExpenseGroupFailure.UnknownTransfer}
     * @throws EranosException.Id for an identifier that is not one
     * @throws IllegalStateException after {@link #close}
     */
    public Transfer removeTransfer(TransferId id) throws EranosException {
        return Converting.call(() -> Transfer.converted(handle.removeTransfer(id.value())));
    }
}
