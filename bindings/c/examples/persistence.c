// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/// A row of the table of balances: a member's balance before saving and after loading.
typedef struct ComparisonRow {
    /// The member's first name.
    Line name;
    /// The balance before saving.
    Line before;
    /// The balance after loading.
    Line after;
} ComparisonRow;

/// A table of balances before and after, its columns from @p numbersFrom on aligned to the right.
[[nodiscard]] static Table *comparisonTable(size_t const numbersFrom) {
    return tableCreate(3, (char const *const[]){"Member", "Balance before saving", "Balance after loading"}, numbersFrom);
}

/// Adds @p row to @p table, a table of balances before and after, its cells in the order of the table's headers.
static void addComparisonRow(Table *const table, ComparisonRow const *const row) {
    tableAdd(table, (Line const[]){row->name, row->before, row->after});
}

/// The balance of @p member in @p group, as the library writes it, or ? for a member without totals.
[[nodiscard]] static Line balanceOf(EranosExpenseGroup const *const group, Id const member) {
    EranosSummary *const summary = eranosExpenseGroupSummary(group);
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    Line const line = nullptr == totals ? lineOf("%s", "?") : balanceLine(eranosMemberSummaryBalance(totals));
    eranosMemberSummaryDestroy(totals);
    eranosSummaryDestroy(summary);
    return line;
}

/// The lunch that Bob paid, with a tip of 10 % shared in proportion to what each consumed.
[[nodiscard]] static EranosExpense *lunchOf(Members const *const both, Id const bob, EranosCalendarDate const day) {
    EranosExpense *const expense = expenseOf("Lunch", localCost(amountOf("30.00", "EUR")), soleSplit(bob), equalSplit(both), day);
    int32_t basisPoints = 0;
    must(eranosPercentageParse("10", &basisPoints));
    EranosTipValue *const value = eranosTipValueCreatePercentage(basisPoints);
    EranosTipRule *const tip = eranosTipRuleCreate(value, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    eranosExpenseSetTip(expense, tip);
    eranosTipRuleDestroy(tip);
    eranosTipValueDestroy(value);
    return expense;
}

int main(void) {
    constexpr EranosCalendarDate DAY = {.year = 2026, .month = 9, .day = 23};
    EranosNewExpenseGroup *details = nullptr;
    must(eranosNewExpenseGroupCreate("Road trip", "EUR", &details));
    EranosExpenseGroup *group = nullptr;
    must(eranosExpenseGroupCreate(details, &group));
    eranosNewExpenseGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Members const both = {.count = 2, .ids = {ana, bob}};
    (void)addExpense(group, expenseOf("Fuel", localCost(amountOf("70.00", "EUR")), soleSplit(ana), equalSplit(&both), DAY));
    (void)addExpense(group, lunchOf(&both, bob, DAY));

    title("Road trip: saving and loading a group");
    printLine("With the `serde` feature a group serialises, and so does every type it stores.\n"
              "It saves only what was entered: its details, members, categories, expenses,\n"
              "and transfers. The totals, balances, and settlements are recomputed when it is\n"
              "loaded.");

    section("1. The saved form");
    // Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an
    // encrypted database).
    EranosText *const saved = eranosTextCreate();
    eranosExpenseGroupSave(group, saved);
    printLine(eranosTextData(saved));
    printLine("Amounts are minor units with their currency, dates are YYYY-MM-DD, and the\n"
              "`format` number lets a future release refuse a file it cannot read instead of\n"
              "dropping fields.");

    section("2. Loading it back");
    EranosExpenseGroup *loaded = nullptr;
    must(eranosExpenseGroupLoad(eranosTextData(saved), &loaded));
    printLine("Loading replays every entry through the same checks as adding it, then rebuilds\n"
              "the totals. The loaded group equals the saved one, and so do its balances:");
    Table *const table = comparisonTable(1);
    Id const members[] = {ana, bob};
    for (size_t index = 0; 2 > index; ++index) {
        Line const before = balanceOf(group, members[index]);
        Line const after = balanceOf(loaded, members[index]);
        check(0 == strcmp(before.text, after.text), "the loaded group has the saved one's balances");
        addComparisonRow(table, &(ComparisonRow){.name = firstName(group, members[index].text), .before = before, .after = after});
    }
    tablePrint(table);
    EranosSettlementPlan *plan = nullptr;
    must(eranosExpenseGroupSettlement(loaded, ERANOS_SETTLEMENT_STRATEGY_GREEDY, &plan));
    EranosPaymentList *const payments = eranosSettlementPlanPayments(plan);
    printFormatted("Payments needed to settle the loaded group: %zu.", eranosPaymentListLength(payments));

    section("3. A corrupted file is refused");
    printLine("Here Ana's identifier was edited in her member entry, so the fuel's payer is an\n"
              "identifier no member has. Reading the file as a SavedExpenseGroup and loading it\n"
              "with into_expense_group replays every entry through the same checks as adding it,\n"
              "and refuses the file instead of producing balances that do not add up:");
    // Ana's own entry is the first to name her: the members come before the expenses in the saved form, so the fuel she
    // paid is left naming an identifier no member has.
    char *const corrupted = replaceFirst(eranosTextData(saved), ana.text, "0199a000-0000-7000-8000-00000000ffff");
    EranosExpenseGroup *refusedGroup = nullptr;
    EranosError *const refused = eranosExpenseGroupLoad(corrupted, &refusedGroup);
    check(nullptr != refused && nullptr == refusedGroup, "the file is refused");
    Line const message = messageLine(refused);
    printFormatted("  error: %s", message.text);

    EranosText *const resaved = eranosTextCreate();
    eranosExpenseGroupSave(loaded, resaved);
    check(0 == strcmp(eranosTextData(resaved), eranosTextData(saved)), "the loaded group saves as the group did");
    check(1 == eranosPaymentListLength(payments), "one payment settles the loaded group");
    eranosTextDestroy(resaved);
    free(corrupted);
    eranosPaymentListDestroy(payments);
    eranosSettlementPlanDestroy(plan);
    eranosExpenseGroupDestroy(loaded);
    eranosTextDestroy(saved);
    eranosExpenseGroupDestroy(group);
    return EXIT_SUCCESS;
}
