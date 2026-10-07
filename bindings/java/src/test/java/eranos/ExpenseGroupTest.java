// The group, in both forms of every call, and its read handle. A call that answered another way
// than the group holds, a change that did not take effect, or an asynchronous form that differed
// from its blocking twin would show an app one group from a future and another from a thread.

package eranos;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionException;

/** Every call of the group, blocking and asynchronous, and every read of its read handle. */
class ExpenseGroupTest {
    /**
     * A read that answered another way than the group holds would show an app entries, totals, or
     * plans the group does not have.
     */
    @Test
    void everyBlockingReadAnswersWhatTheGroupHolds() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        assertFalse(group.id().value().isEmpty());
        List<Category> categories = group.categories();
        assertEquals(
                List.of("Uncategorised", "Food"), categories.stream().map(Category::name).toList());
        assertEquals(CategoryId.UNCATEGORISED, categories.get(0).id());
        assertEquals(Optional.of(categories.get(1)), group.category(flat.food));
        assertEquals(flat.euro, group.currency());
        List<Expense> expenses = group.expenses();
        assertEquals(List.of("Dinner"), expenses.stream().map(Expense::name).toList());
        assertEquals(Optional.of(expenses.get(0)), group.expense(flat.dinner));
        assertEquals(flat.euros(3_000), group.expenseShares(flat.dinner).orElseThrow().total());
        assertEquals(flat.euros(3_000), group.summary().totalSpent());
        List<Member> members = group.members();
        assertEquals(List.of("Ana", "Bob"), members.stream().map(Member::firstName).toList());
        assertEquals(Optional.of(members.get(0)), group.member(flat.ana));
        assertFalse(group.isArchived(flat.ana));
        assertEquals("Flat", group.name());
        assertEquals(Optional.of("Rent"), group.description());
        assertEquals(Optional.of(Language.parse("pt-PT")), group.language());
        assertEquals(
                List.of(flat.euros(500)),
                group.settlement(SettlementStrategy.GREEDY).payments().stream()
                        .map(Payment::amount)
                        .toList());
        List<Transfer> transfers = group.transfers();
        assertEquals(1, transfers.size());
        assertEquals(Optional.of(transfers.get(0)), group.transfer(flat.repayment));
        assertEquals(Optional.of(flat.euros(1_000)), group.transferAmount(flat.repayment));
        String json = group.save();
        assertEquals(json, ExpenseGroup.load(json).save());
        EranosException.Id invalid =
                assertThrows(EranosException.Id.class, () -> group.member(new MemberId(" nobody")));
        assertEquals(new IdFailure.SurroundingWhitespace(), invalid.failure());
        assertEquals("an identifier cannot start or end with whitespace", invalid.getMessage());
    }

    /**
     * An asynchronous read that answered another way than its blocking twin would show an app one
     * group from a future and another from a thread.
     */
    @Test
    void everyAsyncReadAnswersAsItsBlockingTwin() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        assertEquals(group.id(), group.idAsync().join());
        assertEquals(group.categories(), group.categoriesAsync().join());
        assertEquals(group.category(flat.food), group.categoryAsync(flat.food).join());
        assertEquals(group.currency(), group.currencyAsync().join());
        assertEquals(group.expenses(), group.expensesAsync().join());
        assertEquals(group.expense(flat.dinner), group.expenseAsync(flat.dinner).join());
        assertEquals(
                group.expenseShares(flat.dinner), group.expenseSharesAsync(flat.dinner).join());
        assertEquals(group.summary(), group.summaryAsync().join());
        assertEquals(group.members(), group.membersAsync().join());
        assertEquals(group.member(flat.bob), group.memberAsync(flat.bob).join());
        assertEquals(group.isArchived(flat.bob), group.isArchivedAsync(flat.bob).join());
        assertEquals(group.name(), group.nameAsync().join());
        assertEquals(group.description(), group.descriptionAsync().join());
        assertEquals(group.language(), group.languageAsync().join());
        for (SettlementStrategy strategy : SettlementStrategy.values()) {
            assertEquals(group.settlement(strategy), group.settlementAsync(strategy).join());
        }
        assertEquals(group.transfers(), group.transfersAsync().join());
        assertEquals(group.transfer(flat.repayment), group.transferAsync(flat.repayment).join());
        assertEquals(
                group.transferAmount(flat.repayment),
                group.transferAmountAsync(flat.repayment).join());
        String json = group.saveAsync().join();
        assertEquals(group.save(), json);
        assertEquals(json, ExpenseGroup.loadAsync(json).join().save());
    }

    /**
     * A lookup of an identifier the group does not hold that found an entry would show an app an
     * entry of another group.
     */
    @Test
    void aLookupOfAnIdentifierTheGroupDoesNotHoldFindsNothing() throws EranosException {
        Flat flat = new Flat();
        Flat stranger = new Flat();
        ExpenseGroup group = flat.group;
        List<Optional<?>> lookups =
                List.of(
                        group.member(stranger.ana),
                        group.category(stranger.food),
                        group.expense(stranger.dinner),
                        group.expenseShares(stranger.dinner),
                        group.transfer(stranger.repayment),
                        group.transferAmount(stranger.repayment),
                        group.memberAsync(stranger.ana).join(),
                        group.categoryAsync(stranger.food).join(),
                        group.expenseAsync(stranger.dinner).join(),
                        group.expenseSharesAsync(stranger.dinner).join(),
                        group.transferAsync(stranger.repayment).join(),
                        group.transferAmountAsync(stranger.repayment).join());
        for (Optional<?> lookup : lookups) {
            assertEquals(Optional.empty(), lookup);
        }
        try (ExpenseGroupReader reader = group.read()) {
            for (Optional<?> lookup :
                    List.of(
                            reader.member(stranger.ana),
                            reader.category(stranger.food),
                            reader.expense(stranger.dinner),
                            reader.expenseShares(stranger.dinner),
                            reader.transfer(stranger.repayment),
                            reader.transferAmount(stranger.repayment))) {
                assertEquals(Optional.empty(), lookup);
            }
        }
        try (ExpenseGroupReader reader = ExpenseGroup.builder("Trip", flat.euro).build().read()) {
            assertEquals(Optional.empty(), reader.language());
        }
    }

    /**
     * A blocking change that did not take effect, or took another, would leave the group other than
     * the app changed it.
     */
    @Test
    void everyBlockingChangeTakesEffect() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        MemberId carol = group.addMember(Member.builder("Carol").build());
        Member renamed = group.member(carol).orElseThrow().toBuilder().lastName("Popa").build();
        assertEquals("Carol", group.replaceMember(renamed).firstName());
        assertEquals(Optional.of("Popa"), group.member(carol).orElseThrow().lastName());
        group.archiveMember(carol);
        assertTrue(group.isArchived(carol));
        group.unarchiveMember(carol);
        assertFalse(group.isArchived(carol));
        assertEquals(renamed, group.removeMember(carol));
        EranosException.ExpenseGroup inUse =
                assertThrows(
                        EranosException.ExpenseGroup.class, () -> group.removeMember(flat.ana));
        assertEquals(
                new ExpenseGroupFailure.MemberInUse(
                        flat.ana, List.of(flat.dinner), List.of(flat.repayment)),
                inUse.failure());

        CategoryId travel = group.addCategory(new Category("Travel"));
        assertEquals(
                "Travel",
                group.replaceCategory(group.category(travel).orElseThrow().renamed("Trips"))
                        .name());
        assertEquals("Trips", group.removeCategory(travel).name());

        ExpenseId lunch =
                group.addExpense(flat.expense("Lunch", flat.bob, List.of(flat.ana, flat.bob)));
        assertEquals(
                "Lunch",
                group.replaceExpense(
                                group.expense(lunch).orElseThrow().toBuilder()
                                        .name("Brunch")
                                        .build())
                        .name());
        assertEquals("Brunch", group.removeExpense(lunch).name());

        TransferId cash =
                group.addTransfer(
                        Transfer.builder(
                                        flat.ana,
                                        flat.bob,
                                        new Cost.Local(flat.euros(100)),
                                        flat.day)
                                .build());
        assertEquals(
                Optional.empty(),
                group.replaceTransfer(
                                group.transfer(cash).orElseThrow().toBuilder().note("Cash").build())
                        .note());
        assertEquals(Optional.of("Cash"), group.removeTransfer(cash).note());

        group.setName("Home");
        group.clearDescription();
        group.clearLanguage();
        assertEquals("Home", group.name());
        assertEquals(Optional.empty(), group.description());
        assertEquals(Optional.empty(), group.language());
        group.setDescription("Shared flat");
        group.setLanguage(Language.parse("ro"));
        assertEquals(Optional.of("Shared flat"), group.description());
        assertEquals(Optional.of(Language.parse("ro")), group.language());
        CurrencyCode lei = CurrencyCode.of("RON");
        group.changeCurrency(lei, Map.of(flat.euro, ExchangeRate.parse("5")));
        assertEquals(lei, group.currency());
        assertEquals(new Money(15_000, lei), group.summary().totalSpent());
    }

    /**
     * An asynchronous change that took effect another way than its blocking twin would change the
     * group differently from a future.
     */
    @Test
    void everyAsyncChangeTakesEffectAsItsBlockingTwinDoes() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        MemberId carol = group.addMemberAsync(Member.builder("Carol").build()).join();
        Member stored = group.member(carol).orElseThrow();
        assertEquals(
                stored,
                group.replaceMemberAsync(stored.toBuilder().lastName("Popa").build()).join());
        group.archiveMemberAsync(carol).join();
        assertTrue(group.isArchived(carol));
        group.unarchiveMemberAsync(carol).join();
        assertFalse(group.isArchived(carol));
        assertEquals(Optional.of("Popa"), group.removeMemberAsync(carol).join().lastName());

        CategoryId travel = group.addCategoryAsync(new Category("Travel")).join();
        assertEquals(
                "Travel",
                group.replaceCategoryAsync(group.category(travel).orElseThrow().renamed("Trips"))
                        .join()
                        .name());
        assertEquals("Trips", group.removeCategoryAsync(travel).join().name());

        ExpenseId lunch =
                group.addExpenseAsync(flat.expense("Lunch", flat.bob, List.of(flat.ana))).join();
        assertEquals(
                "Lunch",
                group.replaceExpenseAsync(
                                group.expense(lunch).orElseThrow().toBuilder()
                                        .name("Brunch")
                                        .build())
                        .join()
                        .name());
        assertEquals("Brunch", group.removeExpenseAsync(lunch).join().name());

        TransferId cash =
                group.addTransferAsync(
                                Transfer.builder(
                                                flat.ana,
                                                flat.bob,
                                                new Cost.Local(flat.euros(100)),
                                                flat.day)
                                        .build())
                        .join();
        assertEquals(
                Optional.empty(),
                group.replaceTransferAsync(
                                group.transfer(cash).orElseThrow().toBuilder().note("Cash").build())
                        .join()
                        .note());
        assertEquals(Optional.of("Cash"), group.removeTransferAsync(cash).join().note());

        group.setNameAsync("Home").join();
        group.setDescriptionAsync("Shared flat").join();
        group.setLanguageAsync(Language.parse("ro")).join();
        assertEquals("Home", group.name());
        assertEquals(Optional.of("Shared flat"), group.description());
        group.clearDescriptionAsync().join();
        group.clearLanguageAsync().join();
        assertEquals(Optional.empty(), group.description());
        assertEquals(Optional.empty(), group.language());
        CurrencyCode lei = CurrencyCode.of("RON");
        group.changeCurrencyAsync(lei, Map.of(flat.euro, ExchangeRate.parse("5"))).join();
        assertEquals(lei, group.currency());
        CompletionException refused =
                assertThrows(
                        CompletionException.class, () -> group.removeMemberAsync(flat.ana).join());
        assertInstanceOf(EranosException.ExpenseGroup.class, refused.getCause());
    }

    /**
     * A new group that dropped what it starts with, the identifier the app gave it included, two
     * new groups under one generated identifier, or a saved one that loaded from text that is not
     * the saved form, would start an app from another group.
     */
    @Test
    void aGroupStartsWithWhatItIsGivenAndLoadsOnlyItsSavedForm() throws EranosException {
        CurrencyCode euro = CurrencyCode.of("EUR");
        ExpenseGroup plain = ExpenseGroup.builder("Trip", euro).build();
        assertEquals(Optional.empty(), plain.description());
        assertEquals(Optional.empty(), plain.language());
        assertNotEquals(plain.id(), ExpenseGroup.builder("Trip", euro).build().id());
        ExpenseGroupId lisbon = ExpenseGroupId.parse("lisbon-2026");
        assertEquals(lisbon, ExpenseGroup.builder(lisbon, "Trip", euro).build().id());
        EranosException.Id refused =
                assertThrows(
                        EranosException.Id.class,
                        () ->
                                ExpenseGroup.builder("Trip", euro)
                                        .id(new ExpenseGroupId(" x"))
                                        .build());
        assertEquals(new IdFailure.SurroundingWhitespace(), refused.failure());
        EranosException.ExpenseGroup blank =
                assertThrows(
                        EranosException.ExpenseGroup.class,
                        () -> ExpenseGroup.builder(" ", euro).build());
        assertEquals(new ExpenseGroupFailure.EmptyName(), blank.failure());
        assertEquals("the group name must not be empty", blank.getMessage());
        EranosException.InvalidJson json =
                assertThrows(EranosException.InvalidJson.class, () -> ExpenseGroup.load("{"));
        assertEquals(1L, json.column());
        assertEquals(1L, json.line());
        assertEquals("the saved group ends early, at line 1, column 1", json.getMessage());
        CompletionException loading =
                assertThrows(CompletionException.class, () -> ExpenseGroup.loadAsync("{").join());
        assertInstanceOf(EranosException.InvalidJson.class, loading.getCause());
    }

    /**
     * A layer that ordered identifiers itself, by UTF-16 code units as {@link String#compareTo}
     * does, would list text above U+FFFF before text just below it: another order than the
     * library's, and than the other languages'.
     */
    @Test
    void entriesComeInTheLibrarysIdentifierOrder() throws EranosException {
        ExpenseGroup group = ExpenseGroup.builder("Trip", CurrencyCode.of("EUR")).build();
        // U+1F600 is four bytes of UTF-8 from 0xF0, and two UTF-16 units from 0xD83D; U+FF61 is
        // three bytes from 0xEF, and one unit.
        MemberId above =
                group.addMember(Member.builder(new MemberId("\uD83D\uDE00"), "Ana").build());
        MemberId below = group.addMember(Member.builder(new MemberId("\uFF61"), "Bob").build());
        assertEquals(List.of(below, above), group.members().stream().map(Member::id).toList());
    }

    /**
     * A read handle that answered another way than the group, kept reading after it was closed, or
     * kept the lock would show a report of a group that has changed since.
     */
    @Test
    void aReadHandleReadsAsTheGroupDoesUntilItIsClosed() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        ExpenseGroupReader reader = group.read();
        assertEquals(group.id(), reader.id());
        assertEquals(group.categories(), reader.categories());
        assertEquals(group.category(flat.food), reader.category(flat.food));
        assertEquals(flat.euro, reader.currency());
        assertEquals(group.expenses(), reader.expenses());
        assertEquals(group.expense(flat.dinner), reader.expense(flat.dinner));
        assertEquals(group.expenseShares(flat.dinner), reader.expenseShares(flat.dinner));
        assertEquals(group.summary(), reader.summary());
        assertEquals(group.members(), reader.members());
        assertEquals(group.member(flat.ana), reader.member(flat.ana));
        assertFalse(reader.isArchived(flat.ana));
        assertEquals("Flat", reader.name());
        assertEquals(Optional.of("Rent"), reader.description());
        assertEquals(group.language(), reader.language());
        assertEquals(
                group.settlement(SettlementStrategy.GREEDY),
                reader.settlement(SettlementStrategy.GREEDY));
        assertEquals(group.transfers(), reader.transfers());
        assertEquals(group.transfer(flat.repayment), reader.transfer(flat.repayment));
        assertEquals(Optional.of(flat.euros(1_000)), reader.transferAmount(flat.repayment));
        assertEquals(group.save(), reader.save());
        reader.close();
        reader.close();
        IllegalStateException released = assertThrows(IllegalStateException.class, reader::name);
        assertEquals("the read handle was released", released.getMessage());
        try (ExpenseGroupReader async = group.readAsync().join()) {
            assertEquals("Flat", async.name());
        }
        group.setName("Home");
        assertEquals("Home", group.name());
    }

    /**
     * A cancelled future whose call still ran would change the group after the app gave the change
     * up; one whose call was never cancelled would keep waiting for a lock it no longer needs, and
     * a waiting call that never resumed would hang.
     */
    @Test
    void cancellingAFutureCancelsItsCallAndLeavesTheGroupAsItWas() throws EranosException {
        Flat flat = new Flat();
        ExpenseGroup group = flat.group;
        ExpenseGroupReader reader = group.read();
        CompletableFuture<MemberId> adding = group.addMemberAsync(Member.builder("Carol").build());
        assertTrue(adding.cancel(true));
        CompletableFuture<Void> renaming = group.setNameAsync("Home");
        reader.close();
        renaming.join();
        assertEquals("Home", group.name());
        assertEquals(2, group.members().size());
        assertThrows(CancellationException.class, adding::join);
    }

    /**
     * A closed group that still answered would read memory the library freed; one that could not be
     * closed would hold it until collected.
     */
    @Test
    void aClosedGroupAnswersNoMore() throws EranosException {
        ExpenseGroup group = ExpenseGroup.builder("Trip", CurrencyCode.of("EUR")).build();
        group.close();
        group.close();
        assertThrows(IllegalStateException.class, group::name);
    }
}
