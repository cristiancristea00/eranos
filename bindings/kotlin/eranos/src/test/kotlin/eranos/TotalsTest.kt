// The totals. A summary, a balance, a share, or a settlement that converted with an amount changed or dropped would show an app
// other totals than the group holds; one that printed on its own would show it other words than the library's.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test

/** The summary, the balances, the shares of an expense, and the settlements. */
class TotalsTest {
    /** A summary that converted with a total changed or dropped would show an app other totals than the group holds. */
    @Test
    fun theSummaryHoldsTheGroupsTotals() {
        val flat = Flat()
        val summary = flat.group.summaryBlocking()
        assertEquals(flat.euros(3_000), summary.totalSpent)
        assertEquals(listOf(flat.food), summary.categoryTotals.map { it.category })
        assertEquals(listOf(flat.euros(3_000)), summary.categoryTotals.map { it.total })
        val (ana, bob) = summary.members
        assertEquals(flat.ana, ana.member)
        assertEquals(flat.bob, bob.member)
        assertEquals(flat.euros(3_000), ana.paid)
        assertEquals(flat.euros(1_500), ana.share)
        assertEquals(flat.euros(0), ana.sent)
        assertEquals(flat.euros(1_000), ana.received)
        assertEquals(flat.euros(500), ana.net)
        assertEquals(Balance.GetsBack(flat.euros(500)), ana.balance)
        assertEquals(Balance.Owes(flat.euros(500)), bob.balance)
        assertEquals(flat.euros(1_000), bob.sent)
        assertEquals(flat.euros(0), bob.received)
        val (anaTotals, bobTotals) = summary.memberCategoryTotals
        assertEquals(flat.ana, anaTotals.member)
        assertEquals(flat.bob, bobTotals.member)
        assertEquals(listOf(flat.euros(1_500)), anaTotals.totals.map { it.total })

        val again = flat.group.summaryBlocking()
        val other = Flat().group.summaryBlocking()
        assertValue(summary, again, other)
        assertValue(ana, again.members[0], bob)
        assertValue(summary.categoryTotals[0], again.categoryTotals[0], other.categoryTotals[0])
        assertValue(anaTotals, again.memberCategoryTotals[0], bobTotals)
    }

    /** A balance that printed on its own, or converted to another kind, would tell a member they owe what they are owed. */
    @Test
    fun aBalancePrintsAsTheLibraryWritesIt() {
        val flat = Flat()
        val getsBack = Balance.GetsBack(flat.euros(1_200))
        val owes = Balance.Owes(flat.euros(1_200))
        assertEquals("gets back 12.00 EUR", getsBack.toString())
        assertEquals("owes 12.00 EUR", owes.toString())
        assertEquals("settled", Balance.Settled.toString())
        assertValue(getsBack, Balance.GetsBack(flat.euros(1_200)), Balance.GetsBack(flat.euros(1)))
        assertValue(owes, Balance.Owes(flat.euros(1_200)), Balance.Owes(flat.euros(1)))
        for (balance in listOf(getsBack, owes, Balance.Settled)) {
            assertEquals(balance, balance.generated.converted())
        }
    }

    /**
     * An expense's shares or a settlement that converted with an amount changed would tell members to pay what they do not
     * owe.
     */
    @Test
    fun sharesAndSettlementsHoldTheGroupsAmounts() {
        val flat = Flat()
        val shares = present(flat.group.expenseSharesBlocking(flat.dinner))
        assertEquals(flat.euros(3_000), shares.total)
        assertEquals(flat.euros(0), shares.tip)
        val (ana, bob) = shares.members
        assertEquals(flat.ana, ana.member)
        assertEquals(flat.euros(3_000), ana.paid)
        assertEquals(flat.euros(1_500), ana.share)
        val again = present(flat.group.expenseSharesBlocking(flat.dinner))
        assertValue(shares, again, ExpenseShares(shares.members, flat.euros(1), shares.total))
        assertValue(ana, again.members[0], bob)

        assertEquals(
            listOf(SettlementStrategy.DIRECT, SettlementStrategy.GREEDY, SettlementStrategy.MINIMUM_TRANSFERS),
            SettlementStrategy.entries,
        )
        for (strategy in SettlementStrategy.entries) {
            val settlement = flat.group.settlementBlocking(strategy)
            assertEquals(strategy, settlement.strategy)
            assertEquals(strategy, strategy.generated.converted())
            val (payment) = settlement.payments
            assertEquals(flat.bob, payment.from)
            assertEquals(flat.ana, payment.to)
            assertEquals(flat.euros(500), payment.amount)
            assertValue(settlement, flat.group.settlementBlocking(strategy), Settlement(emptyList(), strategy))
            assertValue(payment, Payment(flat.euros(500), flat.bob, flat.ana), Payment(flat.euros(500), flat.ana, flat.bob))
        }
    }
}
