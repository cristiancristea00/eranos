package eranos;

import eranos.internal.CurrencyRate;
import eranos.internal.Eranos;
import eranos.internal.NewGroup;

import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * A group of people who share expenses, safe to share between threads.
 *
 * <p>Every call comes in two forms. The one named for the call waits for the group's lock by
 * blocking its thread; its twin, whose name ends in {@code Async}, waits without blocking a thread
 * and returns a {@link CompletableFuture}, and cancelling the future cancels the call, which leaves
 * the group as it was:
 *
 * <pre>{@code
 * Group group = Group.builder(Currency.of("EUR"), "Flat").build();
 * MemberId ana = group.addMember(Member.builder("Ana").build());
 * CompletableFuture<MemberId> bob = group.addMemberAsync(Member.builder("Bob").build());
 * }</pre>
 *
 * <p>Reads share the lock and run together; a change takes it alone, after the reads in progress.
 * {@link #close} frees the group; every call after it throws {@link IllegalStateException}.
 */
public final class Group implements AutoCloseable {
    /** The generated group this one forwards to. */
    private final eranos.internal.Group handle;

    private Group(eranos.internal.Group handle) {
        this.handle = handle;
    }

    /**
     * A builder of a new group with no members and no entries, called {@code name}, whose totals
     * are in {@code currency}.
     *
     * @param currency the group currency
     * @param name the name
     * @return the builder
     */
    public static Builder builder(Currency currency, String name) {
        return new Builder(currency, name);
    }

    /**
     * The group saved as {@code json}, loaded through the checks that adding its entries makes.
     *
     * <p>Blocks its thread until the load completes; {@link #loadAsync} yields while it replays the
     * entries.
     *
     * @param json the saved group
     * @return the group
     * @throws EranosException.InvalidJson for text that is not the saved form
     * @throws EranosException.Group with {@link GroupFailure.UnsupportedFormat} for a format this
     *     release cannot read, and the failures of entries that do not fit together
     */
    public static Group load(String json) throws EranosException {
        return new Group(Converting.call(() -> eranos.internal.Group.load(json)));
    }

    /**
     * The group saved as {@code json}, loaded through the checks that adding its entries makes.
     *
     * <p>Yields while it replays the entries; cancelling the future cancels the load. The future
     * fails as {@link #load} throws.
     *
     * @param json the saved group
     * @return a future of the group
     */
    public static CompletableFuture<Group> loadAsync(String json) {
        return Converting.future(Eranos.loadGroupAsync(json, null), Group::new);
    }

    /**
     * Frees the group; every call after it throws {@link IllegalStateException}, and closing a
     * closed group does nothing.
     */
    @Override
    public void close() {
        handle.close();
    }

    /**
     * A read handle, which holds the shared lock until it is closed.
     *
     * <p>Waits for the lock by blocking its thread; {@link #readAsync} waits without blocking it.
     *
     * @return the read handle
     */
    public GroupReader read() {
        return new GroupReader(Converting.callUnfailing(handle::read));
    }

    /**
     * A read handle, which holds the shared lock until it is closed.
     *
     * <p>Waits for the lock without blocking a thread; cancelling the future cancels the wait.
     *
     * @return a future of the read handle
     */
    public CompletableFuture<GroupReader> readAsync() {
        return Converting.future(handle.readAsync(null), GroupReader::new);
    }

    // The group's own details: its identifier, name, description, language, currency, and saved
    // form.

    /**
     * The group's identifier.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #idAsync} waits without blocking
     * it.
     *
     * @return the identifier
     */
    public GroupId id() {
        return Converting.callUnfailing(() -> new GroupId(handle.id()));
    }

    /**
     * The group's identifier.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the identifier
     */
    public CompletableFuture<GroupId> idAsync() {
        return Converting.future(handle.idAsync(null), value -> new GroupId(value));
    }

    /**
     * The group currency, which every total is in.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #currencyAsync} waits without
     * blocking it.
     *
     * @return the currency
     */
    public Currency currency() {
        return Converting.callUnfailing(() -> Currency.known(handle.currency()));
    }

    /**
     * The group currency, which every total is in.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the currency
     */
    public CompletableFuture<Currency> currencyAsync() {
        return Converting.future(handle.currencyAsync(null), value -> Currency.known(value));
    }

    /**
     * Moves the group to {@code currency}, converting every entry with the rate {@code rates} gives
     * its currency.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #changeCurrencyAsync} waits
     * without blocking it.
     *
     * @param currency the new group currency
     * @param rates the rate into the new group currency of every other currency the entries use
     * @throws EranosException.Group with {@link GroupFailure.MissingExchangeRate} for a currency in
     *     use without a rate, and {@link GroupFailure.UnexpectedExchangeRate} for a rate of the new
     *     group currency
     */
    public void changeCurrency(Currency currency, Map<Currency, ExchangeRate> rates)
            throws EranosException {
        Converting.run(() -> handle.changeCurrency(currency.code(), rates(rates)));
    }

    /**
     * Moves the group to {@code currency}, converting every entry with the rate {@code rates} gives
     * its currency.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.MissingExchangeRate} for a currency in use without a rate, and {@link
     * GroupFailure.UnexpectedExchangeRate} for a rate of the new group currency.
     *
     * @param currency the new group currency
     * @param rates the rate into the new group currency of every other currency the entries use
     * @return a future of the call's end
     */
    public CompletableFuture<Void> changeCurrencyAsync(
            Currency currency, Map<Currency, ExchangeRate> rates) {
        return Converting.future(
                handle.changeCurrencyAsync(currency.code(), rates(rates), null), _ -> null);
    }

    /**
     * The group's name.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #nameAsync} waits without
     * blocking it.
     *
     * @return the text
     */
    public String name() {
        return Converting.callUnfailing(() -> handle.name());
    }

    /**
     * The group's name.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the text
     */
    public CompletableFuture<String> nameAsync() {
        return Converting.future(handle.nameAsync(null), value -> value);
    }

    /**
     * Renames the group.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #setNameAsync} waits without
     * blocking it.
     *
     * @param name the new name
     * @throws EranosException.Group with {@link GroupFailure.EmptyName} for a blank name
     */
    public void setName(String name) throws EranosException {
        Converting.run(() -> handle.setName(name));
    }

    /**
     * Renames the group.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link GroupFailure.EmptyName}
     * for a blank name.
     *
     * @param name the new name
     * @return a future of the call's end
     */
    public CompletableFuture<Void> setNameAsync(String name) {
        return Converting.future(handle.setNameAsync(name, null), _ -> null);
    }

    /**
     * The group's description, if it has one.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #descriptionAsync} waits without
     * blocking it.
     *
     * @return the description, if the group has one
     */
    public Optional<String> description() {
        return Converting.callUnfailing(() -> Optional.ofNullable(handle.description()));
    }

    /**
     * The group's description, if it has one.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the description, if the group has one
     */
    public CompletableFuture<Optional<String>> descriptionAsync() {
        return Converting.future(
                handle.descriptionAsync(null), value -> Optional.ofNullable(value));
    }

    /**
     * Sets the group's description; blank text clears it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #setDescriptionAsync} waits
     * without blocking it.
     *
     * @param description the new description
     */
    public void setDescription(String description) {
        Converting.runUnfailing(() -> handle.setDescription(description));
    }

    /**
     * Sets the group's description; blank text clears it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @param description the new description
     * @return a future of the call's end
     */
    public CompletableFuture<Void> setDescriptionAsync(String description) {
        return Converting.future(handle.setDescriptionAsync(description, null), _ -> null);
    }

    /**
     * Clears the group's description.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #clearDescriptionAsync} waits
     * without blocking it.
     */
    public void clearDescription() {
        Converting.runUnfailing(() -> handle.setDescription(""));
    }

    /**
     * Clears the group's description.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the call's end
     */
    public CompletableFuture<Void> clearDescriptionAsync() {
        return Converting.future(handle.setDescriptionAsync("", null), _ -> null);
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #languageAsync} waits without
     * blocking it.
     *
     * @return the language, if the group has one
     */
    public Optional<Language> language() {
        return Converting.callUnfailing(
                () -> Optional.ofNullable(handle.language()).map(Language::known));
    }

    /**
     * The language the apps show the group in, if it has one.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the language, if the group has one
     */
    public CompletableFuture<Optional<Language>> languageAsync() {
        return Converting.future(
                handle.languageAsync(null),
                value -> Optional.ofNullable(value).map(Language::known));
    }

    /**
     * Sets the language the apps show the group in.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #setLanguageAsync} waits without
     * blocking it.
     *
     * @param language the new language
     */
    public void setLanguage(Language language) {
        Converting.runUnfailing(() -> handle.setLanguage(language.toString()));
    }

    /**
     * Sets the language the apps show the group in.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @param language the new language
     * @return a future of the call's end
     */
    public CompletableFuture<Void> setLanguageAsync(Language language) {
        return Converting.future(handle.setLanguageAsync(language.toString(), null), _ -> null);
    }

    /**
     * Clears the language the apps show the group in.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #clearLanguageAsync} waits
     * without blocking it.
     */
    public void clearLanguage() {
        Converting.runUnfailing(() -> handle.setLanguage(null));
    }

    /**
     * Clears the language the apps show the group in.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the call's end
     */
    public CompletableFuture<Void> clearLanguageAsync() {
        return Converting.future(handle.setLanguageAsync(null, null), _ -> null);
    }

    /**
     * The group in its saved form, as JSON, which {@link Group#load} reads back.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #saveAsync} waits without
     * blocking it.
     *
     * @return the text
     */
    public String save() {
        return Converting.callUnfailing(() -> handle.save());
    }

    /**
     * The group in its saved form, as JSON, which {@link Group#load} reads back.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the text
     */
    public CompletableFuture<String> saveAsync() {
        return Converting.future(handle.saveAsync(null), value -> value);
    }

    // The group's members.

    /**
     * Every member, archived ones included, in the order they were added.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #membersAsync} waits without
     * blocking it.
     *
     * @return the members
     */
    public List<Member> members() {
        return Converting.callUnfailing(
                () -> handle.members().stream().map(Member::converted).toList());
    }

    /**
     * Every member, archived ones included, in the order they were added.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the members
     */
    public CompletableFuture<List<Member>> membersAsync() {
        return Converting.future(
                handle.membersAsync(null), value -> value.stream().map(Member::converted).toList());
    }

    /**
     * The member with the identifier {@code id}, if the group holds them.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #memberAsync} waits without
     * blocking it.
     *
     * @param id the identifier
     * @return the member, if the group holds them
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<Member> member(MemberId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.member(id.value())).map(Member::converted));
    }

    /**
     * The member with the identifier {@code id}, if the group holds them.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the member, if the group holds them
     */
    public CompletableFuture<Optional<Member>> memberAsync(MemberId id) {
        return Converting.future(
                handle.memberAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(Member::converted));
    }

    /**
     * Whether the member with the identifier {@code id} is archived; {@code false} for a member the
     * group does not hold.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #isArchivedAsync} waits without
     * blocking it.
     *
     * @param id the identifier
     * @return whether the member is archived
     * @throws EranosException.Id for an identifier that is not one
     */
    public boolean isArchived(MemberId id) throws EranosException {
        return Converting.call(() -> handle.isArchived(id.value()));
    }

    /**
     * Whether the member with the identifier {@code id} is archived; {@code false} for a member the
     * group does not hold.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of whether the member is archived
     */
    public CompletableFuture<Boolean> isArchivedAsync(MemberId id) {
        return Converting.future(handle.isArchivedAsync(id.value(), null), value -> value);
    }

    /**
     * Adds {@code member}, who has no identifier yet, and returns the one the group gives them.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #addMemberAsync} waits without
     * blocking it.
     *
     * @param member the member
     * @return the member's identifier
     * @throws IllegalArgumentException for a member with an identifier
     * @throws EranosException.Member with {@link MemberFailure.EmptyFirstName} for a blank first
     *     name
     * @throws EranosException.Account with {@link AccountFailure.EmptyName} for an account with a
     *     blank name
     */
    public MemberId addMember(Member member) throws EranosException {
        return Converting.call(() -> new MemberId(handle.addMember(member.generated())));
    }

    /**
     * Adds {@code member}, who has no identifier yet, and returns the one the group gives them.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a member with an identifier;
     * {@link EranosException.Member} with {@link MemberFailure.EmptyFirstName} for a blank first
     * name; {@link EranosException.Account} with {@link AccountFailure.EmptyName} for an account
     * with a blank name.
     *
     * @param member the member
     * @return a future of the member's identifier
     */
    public CompletableFuture<MemberId> addMemberAsync(Member member) {
        return Converting.future(
                handle.addMemberAsync(member.generated(), null), value -> new MemberId(value));
    }

    /**
     * Replaces the details of the member that {@code member} names by their identifier, and returns
     * them as they were; an archived member stays archived.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #replaceMemberAsync} waits
     * without blocking it.
     *
     * @param member the member
     * @return the member as they were
     * @throws IllegalArgumentException for a member without an identifier
     * @throws EranosException.Member as adding one does
     * @throws EranosException.Account as adding one does
     * @throws EranosException.Group with {@link GroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     */
    public Member replaceMember(Member member) throws EranosException {
        return Converting.call(() -> Member.converted(handle.replaceMember(member.generated())));
    }

    /**
     * Replaces the details of the member that {@code member} names by their identifier, and returns
     * them as they were; an archived member stays archived.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a member without an identifier;
     * {@link EranosException.Member} as adding one does; {@link EranosException.Account} as adding
     * one does; {@link EranosException.Group} with {@link GroupFailure.UnknownMember}; {@link
     * EranosException.Id} for an identifier that is not one.
     *
     * @param member the member
     * @return a future of the member as they were
     */
    public CompletableFuture<Member> replaceMemberAsync(Member member) {
        return Converting.future(
                handle.replaceMemberAsync(member.generated(), null),
                value -> Member.converted(value));
    }

    /**
     * Archives the member with the identifier {@code id}: they keep their history and balance, and
     * join no new expense.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #archiveMemberAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @throws EranosException.Group with {@link GroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     */
    public void archiveMember(MemberId id) throws EranosException {
        Converting.run(() -> handle.archiveMember(id.value()));
    }

    /**
     * Archives the member with the identifier {@code id}: they keep their history and balance, and
     * join no new expense.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownMember}; {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the call's end
     */
    public CompletableFuture<Void> archiveMemberAsync(MemberId id) {
        return Converting.future(handle.archiveMemberAsync(id.value(), null), _ -> null);
    }

    /**
     * Makes the archived member with the identifier {@code id} active again.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #unarchiveMemberAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @throws EranosException.Group with {@link GroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     */
    public void unarchiveMember(MemberId id) throws EranosException {
        Converting.run(() -> handle.unarchiveMember(id.value()));
    }

    /**
     * Makes the archived member with the identifier {@code id} active again.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownMember}; {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the call's end
     */
    public CompletableFuture<Void> unarchiveMemberAsync(MemberId id) {
        return Converting.future(handle.unarchiveMemberAsync(id.value(), null), _ -> null);
    }

    /**
     * Removes the member with the identifier {@code id}, who appears in no entry, and returns them.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #removeMemberAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the member as they were
     * @throws EranosException.Group with {@link GroupFailure.UnknownMember}, and {@link
     *     GroupFailure.MemberInUse} with the entries that still name them; archive such a member
     *     instead
     * @throws EranosException.Id for an identifier that is not one
     */
    public Member removeMember(MemberId id) throws EranosException {
        return Converting.call(() -> Member.converted(handle.removeMember(id.value())));
    }

    /**
     * Removes the member with the identifier {@code id}, who appears in no entry, and returns them.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownMember}, and {@link GroupFailure.MemberInUse} with the entries that still
     * name them; archive such a member instead; {@link EranosException.Id} for an identifier that
     * is not one.
     *
     * @param id the identifier
     * @return a future of the member as they were
     */
    public CompletableFuture<Member> removeMemberAsync(MemberId id) {
        return Converting.future(
                handle.removeMemberAsync(id.value(), null), value -> Member.converted(value));
    }

    // The group's categories.

    /**
     * Every category, the built-in one first.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #categoriesAsync} waits without
     * blocking it.
     *
     * @return the categories
     */
    public List<Category> categories() {
        return Converting.callUnfailing(
                () -> handle.categories().stream().map(Category::converted).toList());
    }

    /**
     * Every category, the built-in one first.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the categories
     */
    public CompletableFuture<List<Category>> categoriesAsync() {
        return Converting.future(
                handle.categoriesAsync(null),
                value -> value.stream().map(Category::converted).toList());
    }

    /**
     * The category with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #categoryAsync} waits without
     * blocking it.
     *
     * @param id the identifier
     * @return the category, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<Category> category(CategoryId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.category(id.value())).map(Category::converted));
    }

    /**
     * The category with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the category, if the group holds it
     */
    public CompletableFuture<Optional<Category>> categoryAsync(CategoryId id) {
        return Converting.future(
                handle.categoryAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(Category::converted));
    }

    /**
     * Adds {@code category}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #addCategoryAsync} waits without
     * blocking it.
     *
     * @param category the category
     * @return the category's identifier
     * @throws IllegalArgumentException for a category with an identifier
     * @throws EranosException.Category with {@link CategoryFailure.EmptyName} for a blank name
     * @throws EranosException.Group with {@link GroupFailure.DuplicateCategoryName} for a name
     *     another category has, compared caselessly
     */
    public CategoryId addCategory(Category category) throws EranosException {
        return Converting.call(() -> new CategoryId(handle.addCategory(category.generated())));
    }

    /**
     * Adds {@code category}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a category with an identifier;
     * {@link EranosException.Category} with {@link CategoryFailure.EmptyName} for a blank name;
     * {@link EranosException.Group} with {@link GroupFailure.DuplicateCategoryName} for a name
     * another category has, compared caselessly.
     *
     * @param category the category
     * @return a future of the category's identifier
     */
    public CompletableFuture<CategoryId> addCategoryAsync(Category category) {
        return Converting.future(
                handle.addCategoryAsync(category.generated(), null),
                value -> new CategoryId(value));
    }

    /**
     * Renames the category that {@code category} names by its identifier, and returns it as it was.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #replaceCategoryAsync} waits
     * without blocking it.
     *
     * @param category the category
     * @return the category as it was
     * @throws IllegalArgumentException for a category without an identifier
     * @throws EranosException.Category with {@link CategoryFailure.EmptyName} for a blank name
     * @throws EranosException.Group with {@link GroupFailure.UnknownCategory}, {@link
     *     GroupFailure.BuiltInCategory}, or {@link GroupFailure.DuplicateCategoryName}
     * @throws EranosException.Id for an identifier that is not one
     */
    public Category replaceCategory(Category category) throws EranosException {
        return Converting.call(
                () -> Category.converted(handle.replaceCategory(category.generated())));
    }

    /**
     * Renames the category that {@code category} names by its identifier, and returns it as it was.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a category without an
     * identifier; {@link EranosException.Category} with {@link CategoryFailure.EmptyName} for a
     * blank name; {@link EranosException.Group} with {@link GroupFailure.UnknownCategory}, {@link
     * GroupFailure.BuiltInCategory}, or {@link GroupFailure.DuplicateCategoryName}; {@link
     * EranosException.Id} for an identifier that is not one.
     *
     * @param category the category
     * @return a future of the category as it was
     */
    public CompletableFuture<Category> replaceCategoryAsync(Category category) {
        return Converting.future(
                handle.replaceCategoryAsync(category.generated(), null),
                value -> Category.converted(value));
    }

    /**
     * Removes the category with the identifier {@code id} and returns it; its expenses move to the
     * built-in one.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #removeCategoryAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the category as it was
     * @throws EranosException.Group with {@link GroupFailure.UnknownCategory} or {@link
     *     GroupFailure.BuiltInCategory}
     * @throws EranosException.Id for an identifier that is not one
     */
    public Category removeCategory(CategoryId id) throws EranosException {
        return Converting.call(() -> Category.converted(handle.removeCategory(id.value())));
    }

    /**
     * Removes the category with the identifier {@code id} and returns it; its expenses move to the
     * built-in one.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownCategory} or {@link GroupFailure.BuiltInCategory}; {@link
     * EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the category as it was
     */
    public CompletableFuture<Category> removeCategoryAsync(CategoryId id) {
        return Converting.future(
                handle.removeCategoryAsync(id.value(), null), value -> Category.converted(value));
    }

    // The group's expenses.

    /**
     * Every expense, in the order they were created.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #expensesAsync} waits without
     * blocking it.
     *
     * @return the expenses
     */
    public List<Expense> expenses() {
        return Converting.callUnfailing(
                () -> handle.expenses().stream().map(Expense::converted).toList());
    }

    /**
     * Every expense, in the order they were created.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the expenses
     */
    public CompletableFuture<List<Expense>> expensesAsync() {
        return Converting.future(
                handle.expensesAsync(null),
                value -> value.stream().map(Expense::converted).toList());
    }

    /**
     * The expense with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #expenseAsync} waits without
     * blocking it.
     *
     * @param id the identifier
     * @return the expense, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<Expense> expense(ExpenseId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.expense(id.value())).map(Expense::converted));
    }

    /**
     * The expense with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the expense, if the group holds it
     */
    public CompletableFuture<Optional<Expense>> expenseAsync(ExpenseId id) {
        return Converting.future(
                handle.expenseAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(Expense::converted));
    }

    /**
     * How the expense with the identifier {@code id} divides, in the group currency, if the group
     * holds it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #expenseSharesAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the shares, if the group holds the expense
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<ExpenseShares> expenseShares(ExpenseId id) throws EranosException {
        return Converting.call(
                () ->
                        Optional.ofNullable(handle.expenseShares(id.value()))
                                .map(ExpenseShares::converted));
    }

    /**
     * How the expense with the identifier {@code id} divides, in the group currency, if the group
     * holds it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the shares, if the group holds the expense
     */
    public CompletableFuture<Optional<ExpenseShares>> expenseSharesAsync(ExpenseId id) {
        return Converting.future(
                handle.expenseSharesAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(ExpenseShares::converted));
    }

    /**
     * Adds {@code expense}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #addExpenseAsync} waits without
     * blocking it.
     *
     * @param expense the expense
     * @return the expense's identifier
     * @throws IllegalArgumentException for an expense with an identifier
     * @throws EranosException.Expense for an expense that does not build, such as one whose amounts
     *     do not add up
     * @throws EranosException.Group for one that does not fit the group, such as {@link
     *     GroupFailure.UnknownMember} or {@link GroupFailure.ArchivedMember}
     * @throws EranosException.Id for an identifier that is not one
     */
    public ExpenseId addExpense(Expense expense) throws EranosException {
        return Converting.call(() -> new ExpenseId(handle.addExpense(expense.generated())));
    }

    /**
     * Adds {@code expense}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for an expense with an identifier;
     * {@link EranosException.Expense} for an expense that does not build, such as one whose amounts
     * do not add up; {@link EranosException.Group} for one that does not fit the group, such as
     * {@link GroupFailure.UnknownMember} or {@link GroupFailure.ArchivedMember}; {@link
     * EranosException.Id} for an identifier that is not one.
     *
     * @param expense the expense
     * @return a future of the expense's identifier
     */
    public CompletableFuture<ExpenseId> addExpenseAsync(Expense expense) {
        return Converting.future(
                handle.addExpenseAsync(expense.generated(), null), value -> new ExpenseId(value));
    }

    /**
     * Replaces the expense that {@code expense} names by its identifier, and returns it as it was.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #replaceExpenseAsync} waits
     * without blocking it.
     *
     * @param expense the expense
     * @return the expense as it was
     * @throws IllegalArgumentException for an expense without an identifier
     * @throws EranosException.Expense as adding one does
     * @throws EranosException.Group with {@link GroupFailure.UnknownExpense}, and as adding one
     *     does
     * @throws EranosException.Id for an identifier that is not one
     */
    public Expense replaceExpense(Expense expense) throws EranosException {
        return Converting.call(() -> Expense.converted(handle.replaceExpense(expense.generated())));
    }

    /**
     * Replaces the expense that {@code expense} names by its identifier, and returns it as it was.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for an expense without an
     * identifier; {@link EranosException.Expense} as adding one does; {@link EranosException.Group}
     * with {@link GroupFailure.UnknownExpense}, and as adding one does; {@link EranosException.Id}
     * for an identifier that is not one.
     *
     * @param expense the expense
     * @return a future of the expense as it was
     */
    public CompletableFuture<Expense> replaceExpenseAsync(Expense expense) {
        return Converting.future(
                handle.replaceExpenseAsync(expense.generated(), null),
                value -> Expense.converted(value));
    }

    /**
     * Removes the expense with the identifier {@code id} and returns it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #removeExpenseAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the expense as it was
     * @throws EranosException.Group with {@link GroupFailure.UnknownExpense}
     * @throws EranosException.Id for an identifier that is not one
     */
    public Expense removeExpense(ExpenseId id) throws EranosException {
        return Converting.call(() -> Expense.converted(handle.removeExpense(id.value())));
    }

    /**
     * Removes the expense with the identifier {@code id} and returns it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownExpense}; {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the expense as it was
     */
    public CompletableFuture<Expense> removeExpenseAsync(ExpenseId id) {
        return Converting.future(
                handle.removeExpenseAsync(id.value(), null), value -> Expense.converted(value));
    }

    // The group's transfers.

    /**
     * Every transfer, in the order they were created.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #transfersAsync} waits without
     * blocking it.
     *
     * @return the transfers
     */
    public List<Transfer> transfers() {
        return Converting.callUnfailing(
                () -> handle.transfers().stream().map(Transfer::converted).toList());
    }

    /**
     * Every transfer, in the order they were created.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the transfers
     */
    public CompletableFuture<List<Transfer>> transfersAsync() {
        return Converting.future(
                handle.transfersAsync(null),
                value -> value.stream().map(Transfer::converted).toList());
    }

    /**
     * The transfer with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #transferAsync} waits without
     * blocking it.
     *
     * @param id the identifier
     * @return the transfer, if the group holds it
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<Transfer> transfer(TransferId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.transfer(id.value())).map(Transfer::converted));
    }

    /**
     * The transfer with the identifier {@code id}, if the group holds it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the transfer, if the group holds it
     */
    public CompletableFuture<Optional<Transfer>> transferAsync(TransferId id) {
        return Converting.future(
                handle.transferAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(Transfer::converted));
    }

    /**
     * The amount in the group currency of the transfer with the identifier {@code id}, if the group
     * holds it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #transferAmountAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the amount, if the group holds the transfer
     * @throws EranosException.Id for an identifier that is not one
     */
    public Optional<Money> transferAmount(TransferId id) throws EranosException {
        return Converting.call(
                () -> Optional.ofNullable(handle.transferAmount(id.value())).map(Money::converted));
    }

    /**
     * The amount in the group currency of the transfer with the identifier {@code id}, if the group
     * holds it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the amount, if the group holds the transfer
     */
    public CompletableFuture<Optional<Money>> transferAmountAsync(TransferId id) {
        return Converting.future(
                handle.transferAmountAsync(id.value(), null),
                value -> Optional.ofNullable(value).map(Money::converted));
    }

    /**
     * Adds {@code transfer}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #addTransferAsync} waits without
     * blocking it.
     *
     * @param transfer the transfer
     * @return the transfer's identifier
     * @throws IllegalArgumentException for a transfer with an identifier
     * @throws EranosException.Transfer for a transfer that does not build, such as {@link
     *     TransferFailure.SameMember}
     * @throws EranosException.Group with {@link GroupFailure.UnknownMember}
     * @throws EranosException.Id for an identifier that is not one
     */
    public TransferId addTransfer(Transfer transfer) throws EranosException {
        return Converting.call(() -> new TransferId(handle.addTransfer(transfer.generated())));
    }

    /**
     * Adds {@code transfer}, which has no identifier yet, and returns the one the group gives it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a transfer with an identifier;
     * {@link EranosException.Transfer} for a transfer that does not build, such as {@link
     * TransferFailure.SameMember}; {@link EranosException.Group} with {@link
     * GroupFailure.UnknownMember}; {@link EranosException.Id} for an identifier that is not one.
     *
     * @param transfer the transfer
     * @return a future of the transfer's identifier
     */
    public CompletableFuture<TransferId> addTransferAsync(Transfer transfer) {
        return Converting.future(
                handle.addTransferAsync(transfer.generated(), null),
                value -> new TransferId(value));
    }

    /**
     * Replaces the transfer that {@code transfer} names by its identifier, and returns it as it
     * was.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #replaceTransferAsync} waits
     * without blocking it.
     *
     * @param transfer the transfer
     * @return the transfer as it was
     * @throws IllegalArgumentException for a transfer without an identifier
     * @throws EranosException.Transfer as adding one does
     * @throws EranosException.Group with {@link GroupFailure.UnknownTransfer}, and as adding one
     *     does
     * @throws EranosException.Id for an identifier that is not one
     */
    public Transfer replaceTransfer(Transfer transfer) throws EranosException {
        return Converting.call(
                () -> Transfer.converted(handle.replaceTransfer(transfer.generated())));
    }

    /**
     * Replaces the transfer that {@code transfer} names by its identifier, and returns it as it
     * was.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link IllegalArgumentException} for a transfer without an
     * identifier; {@link EranosException.Transfer} as adding one does; {@link
     * EranosException.Group} with {@link GroupFailure.UnknownTransfer}, and as adding one does;
     * {@link EranosException.Id} for an identifier that is not one.
     *
     * @param transfer the transfer
     * @return a future of the transfer as it was
     */
    public CompletableFuture<Transfer> replaceTransferAsync(Transfer transfer) {
        return Converting.future(
                handle.replaceTransferAsync(transfer.generated(), null),
                value -> Transfer.converted(value));
    }

    /**
     * Removes the transfer with the identifier {@code id} and returns it.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #removeTransferAsync} waits
     * without blocking it.
     *
     * @param id the identifier
     * @return the transfer as it was
     * @throws EranosException.Group with {@link GroupFailure.UnknownTransfer}
     * @throws EranosException.Id for an identifier that is not one
     */
    public Transfer removeTransfer(TransferId id) throws EranosException {
        return Converting.call(() -> Transfer.converted(handle.removeTransfer(id.value())));
    }

    /**
     * Removes the transfer with the identifier {@code id} and returns it.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Group} with {@link
     * GroupFailure.UnknownTransfer}; {@link EranosException.Id} for an identifier that is not one.
     *
     * @param id the identifier
     * @return a future of the transfer as it was
     */
    public CompletableFuture<Transfer> removeTransferAsync(TransferId id) {
        return Converting.future(
                handle.removeTransferAsync(id.value(), null), value -> Transfer.converted(value));
    }

    // The group's totals and its settlements.

    /**
     * The group's totals.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #summaryAsync} waits without
     * blocking it.
     *
     * @return the totals
     */
    public Summary summary() {
        return Converting.callUnfailing(() -> Summary.converted(handle.summary()));
    }

    /**
     * The group's totals.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * @return a future of the totals
     */
    public CompletableFuture<Summary> summaryAsync() {
        return Converting.future(handle.summaryAsync(null), value -> Summary.converted(value));
    }

    /**
     * The payments that settle every balance by {@code strategy}.
     *
     * <p>Waits for the group's lock by blocking its thread; {@link #settlementAsync} waits without
     * blocking it.
     *
     * @param strategy the strategy
     * @return the payments
     * @throws EranosException.Settlement with {@link SettlementFailure.OutOfMemory} for an exact
     *     plan of more open balances than its table fits
     */
    public Settlement settlement(SettlementStrategy strategy) throws EranosException {
        return Converting.call(() -> Settlement.converted(handle.settlement(strategy.generated())));
    }

    /**
     * The payments that settle every balance by {@code strategy}.
     *
     * <p>Waits for the group's lock without blocking a thread; cancelling the future cancels the
     * call, which leaves the group as it was.
     *
     * <p>The future fails with {@link EranosException.Settlement} with {@link
     * SettlementFailure.OutOfMemory} for an exact plan of more open balances than its table fits.
     *
     * @param strategy the strategy
     * @return a future of the payments
     */
    public CompletableFuture<Settlement> settlementAsync(SettlementStrategy strategy) {
        return Converting.future(
                handle.settlementAsync(strategy.generated(), null),
                value -> Settlement.converted(value));
    }

    /**
     * The rates as the generated code takes them: each currency beside its rate into the group
     * currency.
     *
     * @param rates the rates
     * @return the generated rates
     */
    private static List<CurrencyRate> rates(Map<Currency, ExchangeRate> rates) {
        return rates.entrySet().stream()
                .map(rate -> new CurrencyRate(rate.getKey().code(), rate.getValue().toString()))
                .toList();
    }

    /** A builder of a new group, whose every setter replaces what was set before. */
    public static final class Builder {
        private final Currency currency;
        private final String name;
        private String description;
        private Language language;

        private Builder(Currency currency, String name) {
            this.currency = Objects.requireNonNull(currency, "currency");
            this.name = Objects.requireNonNull(name, "name");
        }

        /**
         * Sets the description; {@code null} leaves the group without one.
         *
         * @param description the description, or {@code null}
         * @return this builder
         */
        public Builder description(String description) {
            this.description = description;
            return this;
        }

        /**
         * Sets the language the apps show the group in; {@code null} leaves the group without one.
         *
         * @param language the language, or {@code null}
         * @return this builder
         */
        public Builder language(Language language) {
            this.language = language;
            return this;
        }

        /**
         * The group.
         *
         * @return the group
         * @throws EranosException.Group with {@link GroupFailure.EmptyName} for a blank name
         */
        public Group build() throws EranosException {
            NewGroup group =
                    new NewGroup(
                            currency.code(),
                            name,
                            description,
                            null == language ? null : language.toString());
            return new Group(Converting.call(() -> new eranos.internal.Group(group)));
        }
    }
}
