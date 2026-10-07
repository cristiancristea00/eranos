// The group, in both forms of every call, and its read handle. A call that answered another way than the group holds, a change
// that did not take effect, or a suspending form that differed from its blocking twin would show an app one group from a
// coroutine and another from a plain thread.

package eranos

import kotlinx.coroutines.async
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.yield
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.Assertions.assertNotEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows

/** Every call of the group, blocking and suspending, and every read of its read handle. */
class ExpenseGroupTest {
    /**
     * A read that answered another way than the group holds would show an app entries, totals, or plans the group does not
     * have.
     */
    @Test
    fun everyBlockingReadAnswersWhatTheGroupHolds() {
        val flat = Flat()
        val group = flat.group
        assertTrue(group.idBlocking().value.isNotEmpty())
        assertEquals("Flat", group.nameBlocking())
        assertEquals(flat.euro, group.currencyBlocking())
        assertEquals("Rent", group.descriptionBlocking())
        assertEquals("pt-PT", group.languageBlocking().toString())
        val members = group.membersBlocking()
        assertEquals(listOf("Ana", "Bob"), members.map { it.firstName })
        assertEquals(members[0], group.memberBlocking(flat.ana))
        assertFalse(group.isArchivedBlocking(flat.ana))
        val categories = group.categoriesBlocking()
        assertEquals(listOf("Uncategorised", "Food"), categories.map { it.name })
        assertEquals(categories[1], group.categoryBlocking(flat.food))
        val expenses = group.expensesBlocking()
        assertEquals(listOf("Dinner"), expenses.map { it.name })
        assertEquals(expenses[0], group.expenseBlocking(flat.dinner))
        assertEquals(flat.euros(3_000), present(group.expenseSharesBlocking(flat.dinner)).total)
        val transfers = group.transfersBlocking()
        assertEquals(1, transfers.size)
        assertEquals(transfers[0], group.transferBlocking(flat.repayment))
        assertEquals(flat.euros(1_000), group.transferAmountBlocking(flat.repayment))
        assertEquals(flat.euros(3_000), group.summaryBlocking().totalSpent)
        assertEquals(listOf(flat.euros(500)), group.settlementBlocking(SettlementStrategy.GREEDY).payments.map { it.amount })
        val json = group.saveBlocking()
        assertEquals(json, ExpenseGroup.loadBlocking(json).saveBlocking())
    }

    /**
     * A suspending read that answered another way than its blocking twin would show an app one group from a coroutine and
     * another from a thread.
     */
    @Test
    fun everySuspendingReadAnswersAsItsBlockingTwin() =
        runTest {
            val flat = Flat()
            val group = flat.group
            assertEquals(group.idBlocking(), group.id())
            assertEquals(group.nameBlocking(), group.name())
            assertEquals(group.currencyBlocking(), group.currency())
            assertEquals(group.descriptionBlocking(), group.description())
            assertEquals(group.languageBlocking(), group.language())
            assertEquals(group.membersBlocking(), group.members())
            assertEquals(group.memberBlocking(flat.bob), group.member(flat.bob))
            assertEquals(group.isArchivedBlocking(flat.bob), group.isArchived(flat.bob))
            assertEquals(group.categoriesBlocking(), group.categories())
            assertEquals(group.categoryBlocking(flat.food), group.category(flat.food))
            assertEquals(group.expensesBlocking(), group.expenses())
            assertEquals(group.expenseBlocking(flat.dinner), group.expense(flat.dinner))
            assertEquals(group.expenseSharesBlocking(flat.dinner), group.expenseShares(flat.dinner))
            assertEquals(group.transfersBlocking(), group.transfers())
            assertEquals(group.transferBlocking(flat.repayment), group.transfer(flat.repayment))
            assertEquals(group.transferAmountBlocking(flat.repayment), group.transferAmount(flat.repayment))
            assertEquals(group.summaryBlocking(), group.summary())
            for (strategy in SettlementStrategy.entries) {
                assertEquals(group.settlementBlocking(strategy), group.settlement(strategy))
            }
            val json = group.save()
            assertEquals(group.saveBlocking(), json)
            assertEquals(json, ExpenseGroup.load(json).save())
        }

    /**
     * A lookup of an identifier the group does not hold that found an entry would show an app an entry of another group.
     */
    @Test
    fun aLookupOfAnIdentifierTheGroupDoesNotHoldFindsNothing() =
        runTest {
            val flat = Flat()
            val stranger = Flat()
            val group = flat.group
            assertNull(group.memberBlocking(stranger.ana))
            assertNull(group.categoryBlocking(stranger.food))
            assertNull(group.expenseBlocking(stranger.dinner))
            assertNull(group.expenseSharesBlocking(stranger.dinner))
            assertNull(group.transferBlocking(stranger.repayment))
            assertNull(group.transferAmountBlocking(stranger.repayment))
            assertNull(group.member(stranger.ana))
            assertNull(group.category(stranger.food))
            assertNull(group.expense(stranger.dinner))
            assertNull(group.expenseShares(stranger.dinner))
            assertNull(group.transfer(stranger.repayment))
            assertNull(group.transferAmount(stranger.repayment))
            group.withReadBlocking { reader ->
                assertNull(reader.member(stranger.ana))
                assertNull(reader.category(stranger.food))
                assertNull(reader.expense(stranger.dinner))
                assertNull(reader.expenseShares(stranger.dinner))
                assertNull(reader.transfer(stranger.repayment))
                assertNull(reader.transferAmount(stranger.repayment))
            }
            assertNull(ExpenseGroup("Trip", flat.euro).withRead { it.language() })
        }

    /** A blocking change that did not take effect, or took another, would leave the group other than the app changed it. */
    @Test
    fun everyBlockingChangeTakesEffect() {
        val flat = Flat()
        val group = flat.group
        val carol = group.addBlocking(Member("Carol"))
        assertEquals(Member("Carol"), present(group.memberBlocking(carol)).let { Member(it.firstName) })
        val renamed = present(group.memberBlocking(carol)).copy(lastName = "Popa")
        assertEquals("Carol", group.replaceBlocking(renamed).firstName)
        assertEquals("Popa", present(group.memberBlocking(carol)).lastName)
        group.archiveBlocking(carol)
        assertTrue(group.isArchivedBlocking(carol))
        group.unarchiveBlocking(carol)
        assertFalse(group.isArchivedBlocking(carol))
        assertEquals(renamed, group.removeBlocking(carol))
        val inUse = assertThrows<EranosException.ExpenseGroup> { group.removeBlocking(flat.ana) }
        assertEquals(ExpenseGroupFailure.MemberInUse(flat.ana, listOf(flat.dinner), listOf(flat.repayment)), inUse.failure)

        val travel = group.addBlocking(Category("Travel"))
        val stored = present(group.categoryBlocking(travel))
        assertEquals("Travel", group.replaceBlocking(stored.copy(name = "Trips")).name)
        assertEquals("Trips", group.removeBlocking(travel).name)

        val lunch = group.addBlocking(flat.expense("Lunch", flat.bob, listOf(flat.ana, flat.bob)))
        val stored2 = present(group.expenseBlocking(lunch))
        assertEquals("Lunch", group.replaceBlocking(stored2.copy(name = "Brunch")).name)
        assertEquals("Brunch", group.removeBlocking(lunch).name)

        val cash = group.addBlocking(Transfer(flat.ana, flat.bob, Cost.Local(flat.euros(100)), flat.day))
        val stored3 = present(group.transferBlocking(cash))
        assertEquals(null, group.replaceBlocking(stored3.copy(note = "Cash")).note)
        assertEquals("Cash", group.removeBlocking(cash).note)

        group.renameBlocking("Home")
        val lei = CurrencyCode.of("RON")
        group.changeCurrencyBlocking(lei, mapOf(flat.euro to ExchangeRate.parse("5")))
        group.setDescriptionBlocking(null)
        group.setLanguageBlocking(null)
        assertEquals("Home", group.nameBlocking())
        assertEquals(lei, group.currencyBlocking())
        assertEquals(Money(15_000, lei), group.summaryBlocking().totalSpent)
        assertNull(group.descriptionBlocking())
        assertNull(group.languageBlocking())
    }

    /**
     * A suspending change that took effect another way than its blocking twin would change the group differently from a
     * coroutine.
     */
    @Test
    fun everySuspendingChangeTakesEffectAsItsBlockingTwinDoes() =
        runTest {
            val flat = Flat()
            val group = flat.group
            val carol = group.add(Member("Carol"))
            val stored = present(group.member(carol))
            assertEquals(stored, group.replace(stored.copy(lastName = "Popa")))
            group.archive(carol)
            assertTrue(group.isArchived(carol))
            group.unarchive(carol)
            assertFalse(group.isArchived(carol))
            assertEquals("Popa", group.remove(carol).lastName)

            val travel = group.add(Category("Travel"))
            assertEquals("Travel", group.replace(present(group.category(travel)).copy(name = "Trips")).name)
            assertEquals("Trips", group.remove(travel).name)

            val lunch = group.add(flat.expense("Lunch", flat.bob, listOf(flat.ana)))
            assertEquals("Lunch", group.replace(present(group.expense(lunch)).copy(name = "Brunch")).name)
            assertEquals("Brunch", group.remove(lunch).name)

            val cash = group.add(Transfer(flat.ana, flat.bob, Cost.Local(flat.euros(100)), flat.day))
            assertNull(group.replace(present(group.transfer(cash)).copy(note = "Cash")).note)
            assertEquals("Cash", group.remove(cash).note)

            group.rename("Home")
            val lei = CurrencyCode.of("RON")
            group.changeCurrency(lei, mapOf(flat.euro to ExchangeRate.parse("5")))
            group.setDescription("Shared flat")
            group.setLanguage(Language.parse("ro"))
            assertEquals("Home", group.name())
            assertEquals(lei, group.currency())
            assertEquals("Shared flat", group.description())
            assertEquals("ro", group.language().toString())
        }

    /**
     * A new group that dropped what it starts with, its identifier included, started from an identifier the rules refuse, or
     * a saved one that loaded from text that is not the saved form, would start an app from another group.
     */
    @Test
    fun aGroupStartsWithWhatItIsGivenAndLoadsOnlyItsSavedForm() =
        runTest {
            val euro = CurrencyCode.of("EUR")
            val plain = ExpenseGroup("Trip", euro)
            assertNull(plain.description())
            assertNull(plain.language())
            val named = ExpenseGroup(ExpenseGroupId.parse("flat-12"), "Flat", euro, "Rent", Language.parse("ro"))
            assertEquals(ExpenseGroupId("flat-12"), named.id())
            assertEquals("Rent", named.description())
            assertEquals(ExpenseGroupId("flat-12"), ExpenseGroup.load(named.save()).id())
            assertNotEquals(named.id(), plain.id())
            val refused = assertThrows<EranosException.Id> { ExpenseGroup(ExpenseGroupId("flat\n12"), "Flat", euro) }
            assertEquals(IdFailure.ControlCharacter, refused.failure)
            val blank = assertThrows<EranosException.ExpenseGroup> { ExpenseGroup(" ", euro) }
            assertEquals(ExpenseGroupFailure.EmptyName, blank.failure)
            assertEquals("the group name must not be empty", blank.message)
            val json = assertThrows<EranosException.InvalidJson> { ExpenseGroup.loadBlocking("{") }
            assertEquals(1L, json.line)
            assertEquals(1L, json.column)
            assertEquals("the saved group ends early, at line 1, column 1", json.message)
            assertThrows<EranosException.InvalidJson> { ExpenseGroup.load("{") }
        }

    /**
     * A read handle that answered another way than the group, kept reading after it was closed, or kept the lock would show
     * a report of a group that has changed since.
     */
    @Test
    fun aReadHandleReadsAsTheGroupDoesUntilItIsClosed() {
        val flat = Flat()
        val group = flat.group
        val reader = group.readBlocking()
        assertEquals(group.idBlocking(), reader.id())
        assertEquals("Flat", reader.name())
        assertEquals(flat.euro, reader.currency())
        assertEquals("Rent", reader.description())
        assertEquals("pt-PT", reader.language().toString())
        assertEquals(group.saveBlocking(), reader.save())
        assertEquals(group.membersBlocking(), reader.members())
        assertEquals(group.memberBlocking(flat.ana), reader.member(flat.ana))
        assertFalse(reader.isArchived(flat.ana))
        assertEquals(group.categoriesBlocking(), reader.categories())
        assertEquals(group.categoryBlocking(flat.food), reader.category(flat.food))
        assertEquals(group.expensesBlocking(), reader.expenses())
        assertEquals(group.expenseBlocking(flat.dinner), reader.expense(flat.dinner))
        assertEquals(group.expenseSharesBlocking(flat.dinner), reader.expenseShares(flat.dinner))
        assertEquals(group.transfersBlocking(), reader.transfers())
        assertEquals(group.transferBlocking(flat.repayment), reader.transfer(flat.repayment))
        assertEquals(flat.euros(1_000), reader.transferAmount(flat.repayment))
        assertEquals(group.summaryBlocking(), reader.summary())
        assertEquals(group.settlementBlocking(SettlementStrategy.GREEDY), reader.settlement(SettlementStrategy.GREEDY))
        reader.close()
        reader.close()
        val released = assertThrows<IllegalStateException> { reader.name() }
        assertEquals("the read handle was released", released.message)
        group.renameBlocking("Home")
        assertEquals("Home", group.withReadBlocking { it.name() })
    }

    /**
     * A suspending read handle that answered another way than the blocking one, or held the lock after its block returned,
     * would stop every later change.
     */
    @Test
    fun aSuspendingReadHandleReleasesTheLockWhenItsBlockReturns() =
        runTest {
            val flat = Flat()
            val group = flat.group
            group.read().use { assertEquals("Flat", it.name()) }
            assertEquals("Flat", group.withRead { it.name() })
            group.rename("Home")
            assertEquals("Home", group.name())
        }

    /**
     * A cancelled coroutine whose call still ran would change the group after the app gave the change up; one whose call was
     * never cancelled would keep waiting for a lock it no longer needs, and a waiting call that never resumed would hang.
     */
    @Test
    fun cancellingACoroutineCancelsItsCallAndLeavesTheGroupAsItWas() =
        runTest {
            val flat = Flat()
            val group = flat.group
            val reader = group.readBlocking()
            val adding = launch { group.add(Member("Carol")) }
            yield()
            assertTrue(adding.isActive)
            adding.cancelAndJoin()
            reader.close()
            assertEquals(2, group.members().size)

            val waiting = group.readBlocking()
            val renaming = async { group.rename("Home") }
            yield()
            assertTrue(renaming.isActive)
            waiting.close()
            renaming.await()
            assertEquals("Home", group.name())
        }

    /**
     * A closed group that still answered would read memory the library freed; one that could not be closed would hold it
     * until collected.
     */
    @Test
    fun aClosedGroupAnswersNoMore() {
        val group = ExpenseGroup("Trip", CurrencyCode.of("EUR"))
        group.close()
        group.close()
        assertThrows<IllegalStateException> { group.nameBlocking() }
    }
}
