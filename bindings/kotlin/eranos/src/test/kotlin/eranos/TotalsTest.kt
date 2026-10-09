// The totals. A summary, a balance, a share, or a settlement that converted with an amount changed or dropped would show an app
// other totals than the group holds; one that printed on its own would show it other words than the library's.

package eranos

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNull
import org.junit.jupiter.api.Test

/** The summary, the balances, the shares of an expense, and the settlements. */
class TotalsTest {
    /** A summary that converted with a total changed or dropped would show an app other totals than the group holds. */
    @Test
    fun theSummaryHoldsTheGroupsTotals() {
        val flat = Flat()
        val summary = flat.group.summaryBlocking()
        assertEquals(flat.euros(3_000), summary.totalSpent)
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
        assertEquals(listOf(flat.food), summary.categoryTotals.map { it.category })
        assertEquals(listOf(flat.euros(3_000)), summary.categoryTotals.map { it.total })
        val (anaTotals, bobTotals) = summary.memberCategoryTotals
        assertEquals(flat.ana, anaTotals.member)
        assertEquals(flat.bob, bobTotals.member)
        assertEquals(listOf(flat.euros(1_500)), anaTotals.totals.map { it.total })
        assertEquals(ana, summary.member(flat.ana))
        val stranger = MemberId("nobody")
        val nobody = summary.member(stranger)
        assertEquals(stranger, nobody.member)
        assertEquals(List(5) { flat.euros(0) }, listOf(nobody.paid, nobody.share, nobody.sent, nobody.received, nobody.net))
        assertEquals(Balance.Settled, nobody.balance)
        assertEquals(flat.euros(3_000), summary.categoryTotal(flat.food))
        assertEquals(flat.euros(0), summary.categoryTotal(CategoryId.UNCATEGORISED))
        assertEquals(flat.euros(1_500), summary.memberCategoryTotal(flat.ana, flat.food))
        assertEquals(flat.euros(0), summary.memberCategoryTotal(flat.ana, CategoryId.UNCATEGORISED))
        assertEquals(flat.euros(0), summary.memberCategoryTotal(stranger, flat.food))

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
        val owes = Balance.Owes(flat.euros(1_200))
        val getsBack = Balance.GetsBack(flat.euros(1_200))
        assertEquals("owes 12.00 EUR", owes.toString())
        assertEquals("settled", Balance.Settled.toString())
        assertEquals("gets back 12.00 EUR", getsBack.toString())
        assertValue(owes, Balance.Owes(flat.euros(1_200)), Balance.Owes(flat.euros(1)))
        assertValue(getsBack, Balance.GetsBack(flat.euros(1_200)), Balance.GetsBack(flat.euros(1)))
        for (balance in listOf(owes, Balance.Settled, getsBack)) {
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
        assertValue(shares, again, ExpenseShares(shares.members, shares.total, flat.euros(1)))
        assertValue(ana, again.members[0], bob)

        assertEquals(
            listOf(
                SettlementStrategy.ADAPTIVE,
                SettlementStrategy.MINIMUM_TRANSFERS,
                SettlementStrategy.GREEDY,
                SettlementStrategy.DIRECT,
            ),
            SettlementStrategy.entries,
        )
        for (strategy in SettlementStrategy.entries) {
            val settlement = flat.group.settlementBlocking(strategy)
            // Two members' balances are well within the adaptive strategy's limit, so it settles them exactly.
            assertEquals(if (SettlementStrategy.ADAPTIVE == strategy) SettlementStrategy.MINIMUM_TRANSFERS else strategy, settlement.strategy)
            assertEquals(strategy, strategy.generated.converted())
            val (payment) = settlement.payments
            assertEquals(flat.bob, payment.sender)
            assertEquals(flat.ana, payment.receiver)
            assertEquals(flat.euros(500), payment.amount)
            assertValue(settlement, flat.group.settlementBlocking(strategy), SettlementPlan(emptyList(), settlement.strategy))
            assertValue(payment, Payment(flat.bob, flat.ana, flat.euros(500)), Payment(flat.ana, flat.bob, flat.euros(500)))
        }
        val (payment) = flat.group.settlementBlocking(SettlementStrategy.GREEDY).payments
        val repayment = Transfer(payment, flat.day, note = "Settled")
        assertEquals(Transfer(repayment.id, flat.bob, flat.ana, Cost.Local(flat.euros(500)), flat.day, note = "Settled"), repayment)
        assertEquals(repayment, Transfer(repayment.id, payment, flat.day, note = "Settled"))
        assertEquals(repayment.copy(note = null), Transfer(repayment.id, payment, flat.day))
        assertNull(Transfer(payment, flat.day).note)
    }
}
