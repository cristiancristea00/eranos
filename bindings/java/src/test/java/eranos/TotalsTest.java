// The totals. A summary, a balance, a share, or a settlement that converted with an amount changed
// or dropped would show an app other totals than the group holds; one that printed on its own
// would show it other words than the library's.

package eranos;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Optional;

/** The summary, the balances, the shares of an expense, and the settlements. */
class TotalsTest {
    /** A summary that converted with a total changed or dropped would show other totals. */
    @Test
    void theSummaryHoldsTheGroupsTotals() throws EranosException {
        Flat flat = new Flat();
        Summary summary = flat.group.summary();
        assertEquals(flat.euros(3_000), summary.totalSpent());
        assertEquals(
                List.of(flat.food),
                summary.categoryTotals().stream().map(CategoryTotal::category).toList());
        assertEquals(
                List.of(flat.euros(3_000)),
                summary.categoryTotals().stream().map(CategoryTotal::total).toList());
        assertEquals(
                List.of(
                        new MemberSummary(
                                new Balance.GetsBack(flat.euros(500)),
                                flat.ana,
                                flat.euros(500),
                                flat.euros(3_000),
                                flat.euros(1_000),
                                flat.euros(0),
                                flat.euros(1_500)),
                        new MemberSummary(
                                new Balance.Owes(flat.euros(500)),
                                flat.bob,
                                flat.euros(-500),
                                flat.euros(0),
                                flat.euros(0),
                                flat.euros(1_000),
                                flat.euros(1_500))),
                summary.members());
        MemberCategoryTotals anaTotals = summary.memberCategoryTotals().get(0);
        assertEquals(flat.ana, anaTotals.member());
        assertEquals(
                List.of(flat.euros(1_500)),
                anaTotals.totals().stream().map(CategoryTotal::total).toList());
        assertEquals(summary, flat.group.summary());
    }

    /**
     * A balance that printed on its own, or converted to another kind, would tell a member they owe
     * what they are owed.
     */
    @Test
    void aBalancePrintsAsTheLibraryWritesIt() throws EranosException {
        Flat flat = new Flat();
        Balance getsBack = new Balance.GetsBack(flat.euros(1_200));
        Balance owes = new Balance.Owes(flat.euros(1_200));
        assertEquals("gets back 12.00 EUR", getsBack.toString());
        assertEquals("owes 12.00 EUR", owes.toString());
        assertEquals("settled", new Balance.Settled().toString());
        for (Balance balance : List.of(getsBack, owes, new Balance.Settled())) {
            assertEquals(balance, Conversions.converted(Conversions.generated(balance)));
        }
    }

    /**
     * An expense's shares or a settlement that converted with an amount changed would tell members
     * to pay what they do not owe.
     */
    @Test
    void sharesAndSettlementsHoldTheGroupsAmounts() throws EranosException {
        Flat flat = new Flat();
        assertEquals(
                Optional.of(
                        new ExpenseShares(
                                List.of(
                                        new MemberShare(
                                                flat.ana, flat.euros(3_000), flat.euros(1_500)),
                                        new MemberShare(
                                                flat.bob, flat.euros(0), flat.euros(1_500))),
                                flat.euros(0),
                                flat.euros(3_000))),
                flat.group.expenseShares(flat.dinner));
        assertEquals(
                List.of(
                        SettlementStrategy.ADAPTIVE,
                        SettlementStrategy.DIRECT,
                        SettlementStrategy.GREEDY,
                        SettlementStrategy.MINIMUM_TRANSFERS),
                List.of(SettlementStrategy.values()));
        for (SettlementStrategy strategy : SettlementStrategy.values()) {
            // Two balances are within the adaptive strategy's limit, so its plan is the minimum
            // one's, and the settlement names that strategy.
            SettlementStrategy computedBy =
                    SettlementStrategy.ADAPTIVE == strategy
                            ? SettlementStrategy.MINIMUM_TRANSFERS
                            : strategy;
            assertEquals(
                    new Settlement(
                            List.of(new Payment(flat.euros(500), flat.bob, flat.ana)), computedBy),
                    flat.group.settlement(strategy));
            assertEquals(strategy, SettlementStrategy.converted(strategy.generated()));
        }
    }
}
