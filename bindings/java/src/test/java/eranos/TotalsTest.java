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
                List.of(
                        new MemberSummary(
                                flat.ana,
                                flat.euros(3_000),
                                flat.euros(1_500),
                                flat.euros(0),
                                flat.euros(1_000),
                                flat.euros(500),
                                new Balance.GetsBack(flat.euros(500))),
                        new MemberSummary(
                                flat.bob,
                                flat.euros(0),
                                flat.euros(1_500),
                                flat.euros(1_000),
                                flat.euros(0),
                                flat.euros(-500),
                                new Balance.Owes(flat.euros(500)))),
                summary.members());
        assertEquals(
                List.of(flat.food),
                summary.categoryTotals().stream().map(CategoryTotal::category).toList());
        assertEquals(
                List.of(flat.euros(3_000)),
                summary.categoryTotals().stream().map(CategoryTotal::total).toList());
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
        Balance owes = new Balance.Owes(flat.euros(1_200));
        Balance getsBack = new Balance.GetsBack(flat.euros(1_200));
        assertEquals("owes 12.00 EUR", owes.toString());
        assertEquals("settled", new Balance.Settled().toString());
        assertEquals("gets back 12.00 EUR", getsBack.toString());
        for (Balance balance : List.of(owes, new Balance.Settled(), getsBack)) {
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
                                flat.euros(3_000),
                                flat.euros(0))),
                flat.group.expenseShares(flat.dinner));
        assertEquals(
                List.of(
                        SettlementStrategy.ADAPTIVE,
                        SettlementStrategy.MINIMUM_TRANSFERS,
                        SettlementStrategy.GREEDY,
                        SettlementStrategy.DIRECT),
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
                            List.of(new Payment(flat.bob, flat.ana, flat.euros(500))), computedBy),
                    flat.group.settlement(strategy));
            assertEquals(strategy, SettlementStrategy.converted(strategy.generated()));
        }
    }
}
